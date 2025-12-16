/// Enhanced framebuffer with double buffering and resolution management
///
/// This module extends the basic framebuffer with advanced features like
/// double buffering for smooth rendering and resolution detection.

use alloc::vec::Vec;
use spin::Mutex;

/// Framebuffer resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

impl Resolution {
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

/// Common resolutions
impl Resolution {
    pub const VGA: Self = Self::new(640, 480);
    pub const SVGA: Self = Self::new(800, 600);
    pub const XGA: Self = Self::new(1024, 768);
    pub const HD: Self = Self::new(1280, 720);
    pub const FULL_HD: Self = Self::new(1920, 1080);
}

/// Framebuffer configuration
pub struct FramebufferConfig {
    pub addr: *mut u8,
    pub resolution: Resolution,
    pub pitch: usize,
    pub bpp: usize,
    pub double_buffering: bool,
}

/// Enhanced framebuffer with double buffering support
pub struct EnhancedFramebuffer {
    // Hardware framebuffer
    hw_addr: *mut u8,
    
    // Back buffer for double buffering (if enabled)
    back_buffer: Option<Vec<u8>>,
    
    // Configuration
    resolution: Resolution,
    pitch: usize,
    bpp: usize,
    double_buffering: bool,
    
    // Statistics
    flip_count: usize,
}

unsafe impl Send for EnhancedFramebuffer {}

impl EnhancedFramebuffer {
    pub const fn new() -> Self {
        Self {
            hw_addr: core::ptr::null_mut(),
            back_buffer: None,
            resolution: Resolution::new(0, 0),
            pitch: 0,
            bpp: 0,
            double_buffering: false,
            flip_count: 0,
        }
    }
    
    /// Initialize the enhanced framebuffer
    pub fn init(&mut self, config: FramebufferConfig) {
        self.hw_addr = config.addr;
        self.resolution = config.resolution;
        self.pitch = config.pitch;
        self.bpp = config.bpp;
        self.double_buffering = config.double_buffering;
        
        // Allocate back buffer if double buffering is enabled
        if self.double_buffering && self.bpp == 32 {
            let buffer_size = config.resolution.height * config.pitch;
            self.back_buffer = Some(Vec::with_capacity(buffer_size));
            if let Some(ref mut buf) = self.back_buffer {
                buf.resize(buffer_size, 0);
            }
        }
        
        self.clear();
    }
    
    /// Get the current resolution
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }
    
    /// Get the drawing surface address (back buffer if enabled, otherwise hardware buffer)
    pub fn draw_addr(&mut self) -> *mut u8 {
        if let Some(ref mut buf) = self.back_buffer {
            buf.as_mut_ptr()
        } else {
            self.hw_addr
        }
    }
    
    /// Flip buffers (copy back buffer to hardware buffer)
    pub fn flip(&mut self) {
        if !self.double_buffering {
            return;
        }
        
        if let Some(ref buf) = self.back_buffer {
            if !self.hw_addr.is_null() && self.bpp == 32 {
                unsafe {
                    // Fast memcpy from back buffer to hardware buffer
                    core::ptr::copy_nonoverlapping(
                        buf.as_ptr(),
                        self.hw_addr,
                        buf.len(),
                    );
                }
                self.flip_count += 1;
            }
        }
    }
    
    /// Clear the framebuffer
    pub fn clear(&mut self) {
        let addr = self.draw_addr();
        if addr.is_null() || self.bpp != 32 {
            return;
        }
        
        unsafe {
            for y in 0..self.resolution.height {
                let row = addr.add(y * self.pitch) as *mut u32;
                for x in 0..(self.pitch / 4) {
                    row.add(x).write_volatile(0xFF000000); // Black
                }
            }
        }
    }
    
    /// Fill a rectangle
    pub fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: u32) {
        let addr = self.draw_addr();
        if addr.is_null() || self.bpp != 32 {
            return;
        }
        
        let x_end = (x + width).min(self.resolution.width);
        let y_end = (y + height).min(self.resolution.height);
        
        unsafe {
            for row_y in y..y_end {
                let row = addr.add(row_y * self.pitch) as *mut u32;
                for col_x in x..x_end {
                    row.add(col_x).write_volatile(color);
                }
            }
        }
    }
    
    /// Draw a pixel
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.resolution.width || y >= self.resolution.height {
            return;
        }
        
        let addr = self.draw_addr();
        if addr.is_null() || self.bpp != 32 {
            return;
        }
        
        unsafe {
            let row = addr.add(y * self.pitch) as *mut u32;
            row.add(x).write_volatile(color);
        }
    }
    
    /// Get a pixel color
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<u32> {
        if x >= self.resolution.width || y >= self.resolution.height {
            return None;
        }
        
        let addr = if let Some(ref buf) = self.back_buffer {
            buf.as_ptr() as *const u8
        } else {
            self.hw_addr
        };
        
        if addr.is_null() || self.bpp != 32 {
            return None;
        }
        
        unsafe {
            let row = addr.add(y * self.pitch) as *const u32;
            Some(row.add(x).read_volatile())
        }
    }
    
    /// Get flip count (for debugging/stats)
    pub fn flip_count(&self) -> usize {
        self.flip_count
    }
    
    /// Check if double buffering is enabled
    pub fn is_double_buffered(&self) -> bool {
        self.double_buffering
    }
    
    /// Copy a region from source coordinates to destination coordinates
    pub fn copy_region(
        &mut self,
        src_x: usize,
        src_y: usize,
        dst_x: usize,
        dst_y: usize,
        width: usize,
        height: usize,
    ) {
        // Bounds checking
        if src_x + width > self.resolution.width
            || src_y + height > self.resolution.height
            || dst_x + width > self.resolution.width
            || dst_y + height > self.resolution.height
        {
            return;
        }
        
        let addr = self.draw_addr();
        if addr.is_null() || self.bpp != 32 {
            return;
        }
        
        // Copy row by row to handle overlapping regions correctly
        if src_y < dst_y {
            // Copy from bottom to top
            for dy in (0..height).rev() {
                unsafe {
                    let src_row = addr.add((src_y + dy) * self.pitch) as *const u32;
                    let dst_row = addr.add((dst_y + dy) * self.pitch) as *mut u32;
                    core::ptr::copy(
                        src_row.add(src_x),
                        dst_row.add(dst_x),
                        width,
                    );
                }
            }
        } else {
            // Copy from top to bottom
            for dy in 0..height {
                unsafe {
                    let src_row = addr.add((src_y + dy) * self.pitch) as *const u32;
                    let dst_row = addr.add((dst_y + dy) * self.pitch) as *mut u32;
                    core::ptr::copy(
                        src_row.add(src_x),
                        dst_row.add(dst_x),
                        width,
                    );
                }
            }
        }
    }
}

/// Global enhanced framebuffer
static ENHANCED_FB: Mutex<EnhancedFramebuffer> = Mutex::new(EnhancedFramebuffer::new());

/// Initialize the enhanced framebuffer
pub fn init(config: FramebufferConfig) {
    ENHANCED_FB.lock().init(config);
}

/// Get access to the enhanced framebuffer
pub fn enhanced_framebuffer() -> spin::MutexGuard<'static, EnhancedFramebuffer> {
    ENHANCED_FB.lock()
}
