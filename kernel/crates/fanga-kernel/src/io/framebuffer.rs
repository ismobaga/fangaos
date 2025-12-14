use super::font::{self, FONT_HEIGHT, FONT_WIDTH};
use core::fmt;
use spin::Mutex;

/// Framebuffer console writer with font rendering and scrolling
pub struct FramebufferWriter {
    addr: *mut u8,
    width: usize,
    height: usize,
    pitch: usize,
    bpp: usize,
    
    // Console state
    col: usize,
    row: usize,
    max_cols: usize,
    max_rows: usize,
    
    // Colors (ARGB format)
    pub fg_color: u32,
    pub bg_color: u32,
}

unsafe impl Send for FramebufferWriter {}

impl FramebufferWriter {
    pub const fn new() -> Self {
        Self {
            addr: core::ptr::null_mut(),
            width: 0,
            height: 0,
            pitch: 0,
            bpp: 0,
            col: 0,
            row: 0,
            max_cols: 0,
            max_rows: 0,
            fg_color: 0xFFFFFFFF, // White
            bg_color: 0xFF000000, // Black
        }
    }

    /// Initialize the framebuffer writer
    pub fn init(&mut self, addr: *mut u8, width: usize, height: usize, pitch: usize, bpp: usize) {
        self.addr = addr;
        self.width = width;
        self.height = height;
        self.pitch = pitch;
        self.bpp = bpp;
        
        self.max_cols = width / FONT_WIDTH;
        self.max_rows = height / FONT_HEIGHT;
        
        self.col = 0;
        self.row = 0;
        
        // Clear the screen
        self.clear();
    }

    /// Clear the entire screen
    pub fn clear(&mut self) {
        if self.addr.is_null() || self.bpp != 32 {
            return;
        }

        unsafe {
            for y in 0..self.height {
                let row = self.addr.add(y * self.pitch) as *mut u32;
                for x in 0..(self.pitch / 4) {
                    row.add(x).write_volatile(self.bg_color);
                }
            }
        }
        
        self.col = 0;
        self.row = 0;
    }

    /// Set foreground color (ARGB format)
    pub fn set_fg_color(&mut self, color: u32) {
        self.fg_color = color;
    }

    /// Set background color (ARGB format)
    pub fn set_bg_color(&mut self, color: u32) {
        self.bg_color = color;
    }

    /// Scroll the screen up by one line
    fn scroll_up(&mut self) {
        if self.addr.is_null() || self.bpp != 32 {
            return;
        }

        unsafe {
            // Copy all rows up by FONT_HEIGHT pixels
            for y in FONT_HEIGHT..self.height {
                let src_row = self.addr.add(y * self.pitch) as *const u32;
                let dst_row = self.addr.add((y - FONT_HEIGHT) * self.pitch) as *mut u32;
                
                for x in 0..(self.pitch / 4) {
                    dst_row.add(x).write_volatile(src_row.add(x).read_volatile());
                }
            }
            
            // Clear the last row
            let start_y = self.height - FONT_HEIGHT;
            for y in start_y..self.height {
                let row = self.addr.add(y * self.pitch) as *mut u32;
                for x in 0..(self.pitch / 4) {
                    row.add(x).write_volatile(self.bg_color);
                }
            }
        }
    }

    /// Draw a character at the current position
    fn draw_char(&mut self, ch: char) {
        if self.addr.is_null() || self.bpp != 32 {
            return;
        }

        let bitmap = font::get_char_bitmap(ch);
        let x_base = self.col * FONT_WIDTH;
        let y_base = self.row * FONT_HEIGHT;

        unsafe {
            for (row_idx, &byte) in bitmap.iter().enumerate() {
                let y = y_base + row_idx;
                if y >= self.height {
                    break;
                }
                
                let row_ptr = self.addr.add(y * self.pitch) as *mut u32;
                
                for bit_idx in 0..FONT_WIDTH {
                    let x = x_base + bit_idx;
                    if x >= self.width {
                        break;
                    }
                    
                    let pixel_on = (byte & (0x80 >> bit_idx)) != 0;
                    let color = if pixel_on { self.fg_color } else { self.bg_color };
                    row_ptr.add(x).write_volatile(color);
                }
            }
        }
    }

    /// Write a single byte (character) to the framebuffer
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                // Newline
                self.col = 0;
                self.row += 1;
            }
            b'\r' => {
                // Carriage return
                self.col = 0;
            }
            b'\t' => {
                // Tab (4 spaces)
                for _ in 0..4 {
                    self.write_byte(b' ');
                }
            }
            0x08 => {
                // Backspace
                if self.col > 0 {
                    self.col -= 1;
                    self.draw_char(' ');
                }
            }
            byte => {
                // Regular character
                if self.col >= self.max_cols {
                    self.col = 0;
                    self.row += 1;
                }
                
                if byte >= 32 && byte < 127 {
                    self.draw_char(byte as char);
                    self.col += 1;
                }
            }
        }

        // Check if we need to scroll
        if self.row >= self.max_rows {
            self.scroll_up();
            self.row = self.max_rows - 1;
        }
    }

    /// Write a string to the framebuffer
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    /// Get current column position
    pub fn get_col(&self) -> usize {
        self.col
    }

    /// Get current row position
    pub fn get_row(&self) -> usize {
        self.row
    }

    /// Set cursor position
    pub fn set_position(&mut self, col: usize, row: usize) {
        self.col = col.min(self.max_cols);
        self.row = row.min(self.max_rows - 1);
    }

    /// Clear from current position to end of line
    pub fn clear_to_eol(&mut self) {
        let saved_col = self.col;
        let saved_row = self.row;
        
        // Clear from current position to end of line
        while self.col < self.max_cols {
            self.draw_char(' ');
            self.col += 1;
        }
        
        // Restore position
        self.col = saved_col;
        self.row = saved_row;
    }

    /// Redraw the current line with the given text, starting from the given column
    pub fn redraw_line(&mut self, start_col: usize, text: &[char]) {
        let saved_col = self.col;
        let saved_row = self.row;
        
        self.col = start_col;
        
        // Draw the text
        for &ch in text {
            if self.col >= self.max_cols {
                break;
            }
            self.draw_char(ch);
            self.col += 1;
        }
        
        // Clear remaining characters on the line
        while self.col < self.max_cols {
            self.draw_char(' ');
            self.col += 1;
        }
        
        self.col = saved_col;
        self.row = saved_row;
    }

    /// Draw a cursor at the current position
    pub fn draw_cursor(&mut self) {
        // Save current colors
        let saved_fg = self.fg_color;
        let saved_bg = self.bg_color;
        
        // Swap colors for cursor effect
        self.fg_color = saved_bg;
        self.bg_color = saved_fg;
        
        self.draw_char(' ');
        
        // Restore colors
        self.fg_color = saved_fg;
        self.bg_color = saved_bg;
    }

    pub fn is_initialized(&self) -> bool {
        !self.addr.is_null()
    }
}

impl fmt::Write for FramebufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Global framebuffer writer
static FRAMEBUFFER: Mutex<FramebufferWriter> = Mutex::new(FramebufferWriter::new());

/// Initialize the global framebuffer
pub fn init(addr: *mut u8, width: usize, height: usize, pitch: usize, bpp: usize) {
    FRAMEBUFFER.lock().init(addr, width, height, pitch, bpp);
}

/// Get access to the framebuffer
pub fn framebuffer() -> spin::MutexGuard<'static, FramebufferWriter> {
    FRAMEBUFFER.lock()
}

/// Print to the framebuffer
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    FRAMEBUFFER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! fb_print {
    ($($arg:tt)*) => {{
        $crate::io::framebuffer::_print(core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! fb_println {
    () => {$crate::fb_print!("\n")};
    ($fmt:expr) => {$crate::fb_print!(concat!($fmt, "\n"))};
    ($fmt:expr, $($arg:tt)*) => {$crate::fb_print!(concat!($fmt, "\n"), $($arg)*)};
}
