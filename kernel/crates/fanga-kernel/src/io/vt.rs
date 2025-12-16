/// Virtual Terminal (VT) system for multiple console support
///
/// This module implements virtual terminals that allow switching between
/// multiple independent console sessions using Alt+F1 through Alt+F12.

use super::font::{self, FONT_HEIGHT, FONT_WIDTH};
use alloc::vec::Vec;
use alloc::string::String;
use core::fmt;
use spin::Mutex;

/// Maximum number of virtual terminals
pub const MAX_TERMINALS: usize = 12;

/// Virtual terminal history size (lines)
const HISTORY_SIZE: usize = 1000;

/// A single virtual terminal
pub struct VirtualTerminal {
    // Terminal ID (0-11 for F1-F12)
    id: usize,
    
    // Current cursor position
    col: usize,
    row: usize,
    
    // Display dimensions
    max_cols: usize,
    max_rows: usize,
    
    // Colors
    fg_color: u32,
    bg_color: u32,
    
    // Screen buffer (character + color)
    screen_buffer: Vec<(char, u32, u32)>, // (char, fg, bg)
    
    // History buffer for scrollback
    history: Vec<String>,
    history_offset: usize, // For scrollback
    
    // Active state
    active: bool,
}

impl VirtualTerminal {
    pub fn new(id: usize, max_cols: usize, max_rows: usize) -> Self {
        let buffer_size = max_cols * max_rows;
        let mut screen_buffer = Vec::with_capacity(buffer_size);
        screen_buffer.resize(buffer_size, (' ', 0xFFFFFFFF, 0xFF000000));
        
        Self {
            id,
            col: 0,
            row: 0,
            max_cols,
            max_rows,
            fg_color: 0xFFFFFFFF,
            bg_color: 0xFF000000,
            screen_buffer,
            history: Vec::with_capacity(HISTORY_SIZE),
            history_offset: 0,
            active: false,
        }
    }
    
    /// Get terminal ID
    pub fn id(&self) -> usize {
        self.id
    }
    
    /// Check if terminal is active
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    /// Set active state
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
    
    /// Write a character at the current position
    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.newline();
            }
            '\r' => {
                self.col = 0;
            }
            '\t' => {
                for _ in 0..4 {
                    self.write_char(' ');
                }
            }
            '\x08' => {
                // Backspace
                if self.col > 0 {
                    self.col -= 1;
                    self.put_char(' ');
                    self.col -= 1;
                }
            }
            ch if ch >= ' ' && ch <= '~' => {
                if self.col >= self.max_cols {
                    self.newline();
                }
                self.put_char(ch);
                self.col += 1;
            }
            _ => {
                // Ignore other control characters
            }
        }
    }
    
    /// Write a string
    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }
    
    /// Put a character at the current position without moving cursor
    fn put_char(&mut self, ch: char) {
        if self.row >= self.max_rows {
            self.scroll_up();
            self.row = self.max_rows - 1;
        }
        
        let idx = self.row * self.max_cols + self.col;
        if idx < self.screen_buffer.len() {
            self.screen_buffer[idx] = (ch, self.fg_color, self.bg_color);
        }
    }
    
    /// Move to next line
    fn newline(&mut self) {
        self.col = 0;
        self.row += 1;
        
        if self.row >= self.max_rows {
            self.scroll_up();
            self.row = self.max_rows - 1;
        }
    }
    
    /// Scroll screen content up by one line
    fn scroll_up(&mut self) {
        // Save first line to history (preserve full content)
        let mut line = String::new();
        for col in 0..self.max_cols {
            let (ch, _, _) = self.screen_buffer[col];
            line.push(ch);
        }
        // Trim trailing spaces but keep internal spacing
        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            if self.history.len() >= HISTORY_SIZE {
                self.history.remove(0);
            }
            self.history.push(String::from(trimmed));
        }
        
        // Shift all rows up
        for row in 1..self.max_rows {
            for col in 0..self.max_cols {
                let src_idx = row * self.max_cols + col;
                let dst_idx = (row - 1) * self.max_cols + col;
                self.screen_buffer[dst_idx] = self.screen_buffer[src_idx];
            }
        }
        
        // Clear last row
        let last_row_start = (self.max_rows - 1) * self.max_cols;
        for i in 0..self.max_cols {
            self.screen_buffer[last_row_start + i] = (' ', self.fg_color, self.bg_color);
        }
    }
    
    /// Clear the terminal
    pub fn clear(&mut self) {
        for i in 0..self.screen_buffer.len() {
            self.screen_buffer[i] = (' ', self.fg_color, self.bg_color);
        }
        self.col = 0;
        self.row = 0;
    }
    
    /// Get the screen buffer for rendering
    pub fn screen_buffer(&self) -> &[(char, u32, u32)] {
        &self.screen_buffer
    }
    
    /// Set cursor position
    pub fn set_position(&mut self, col: usize, row: usize) {
        self.col = col.min(self.max_cols - 1);
        self.row = row.min(self.max_rows - 1);
    }
    
    /// Get cursor position
    pub fn position(&self) -> (usize, usize) {
        (self.col, self.row)
    }
    
    /// Set colors
    pub fn set_colors(&mut self, fg: u32, bg: u32) {
        self.fg_color = fg;
        self.bg_color = bg;
    }
}

impl fmt::Write for VirtualTerminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

/// Virtual terminal manager
pub struct VtManager {
    terminals: Vec<VirtualTerminal>,
    current: usize,
    max_cols: usize,
    max_rows: usize,
    
    // Framebuffer info for rendering
    fb_addr: *mut u8,
    fb_width: usize,
    fb_height: usize,
    fb_pitch: usize,
}

unsafe impl Send for VtManager {}

impl VtManager {
    pub const fn new() -> Self {
        Self {
            terminals: Vec::new(),
            current: 0,
            max_cols: 0,
            max_rows: 0,
            fb_addr: core::ptr::null_mut(),
            fb_width: 0,
            fb_height: 0,
            fb_pitch: 0,
        }
    }
    
    /// Initialize the virtual terminal manager
    pub fn init(&mut self, fb_addr: *mut u8, width: usize, height: usize, pitch: usize) {
        self.fb_addr = fb_addr;
        self.fb_width = width;
        self.fb_height = height;
        self.fb_pitch = pitch;
        
        self.max_cols = width / FONT_WIDTH;
        self.max_rows = height / FONT_HEIGHT;
        
        // Create terminals
        for id in 0..MAX_TERMINALS {
            let mut vt = VirtualTerminal::new(id, self.max_cols, self.max_rows);
            if id == 0 {
                vt.set_active(true);
            }
            self.terminals.push(vt);
        }
    }
    
    /// Get current terminal
    pub fn current_terminal(&mut self) -> Option<&mut VirtualTerminal> {
        self.terminals.get_mut(self.current)
    }
    
    /// Get a specific terminal
    pub fn terminal(&mut self, id: usize) -> Option<&mut VirtualTerminal> {
        self.terminals.get_mut(id)
    }
    
    /// Switch to a terminal
    pub fn switch_to(&mut self, id: usize) -> bool {
        if id >= MAX_TERMINALS {
            return false;
        }
        
        if id == self.current {
            return true;
        }
        
        // Deactivate current
        if let Some(vt) = self.terminals.get_mut(self.current) {
            vt.set_active(false);
        }
        
        // Activate new terminal
        self.current = id;
        if let Some(vt) = self.terminals.get_mut(id) {
            vt.set_active(true);
        }
        
        // Redraw the new terminal
        self.render_current();
        
        true
    }
    
    /// Get current terminal ID
    pub fn current_id(&self) -> usize {
        self.current
    }
    
    /// Render the current terminal to the framebuffer
    pub fn render_current(&mut self) {
        if self.fb_addr.is_null() {
            return;
        }
        
        if let Some(vt) = self.terminals.get(self.current) {
            self.render_terminal(vt);
        }
    }
    
    /// Render a specific terminal
    fn render_terminal(&self, vt: &VirtualTerminal) {
        if self.fb_addr.is_null() {
            return;
        }
        
        let buffer = vt.screen_buffer();
        
        // Clear screen first
        unsafe {
            for y in 0..self.fb_height {
                let row = self.fb_addr.add(y * self.fb_pitch) as *mut u32;
                for x in 0..(self.fb_pitch / 4) {
                    row.add(x).write_volatile(0xFF000000);
                }
            }
        }
        
        // Render each character
        for row in 0..self.max_rows {
            for col in 0..self.max_cols {
                let idx = row * self.max_cols + col;
                if idx >= buffer.len() {
                    continue;
                }
                
                let (ch, fg, bg) = buffer[idx];
                if ch != ' ' || bg != 0xFF000000 {
                    self.draw_char(ch, col, row, fg, bg);
                }
            }
        }
        
        // Draw cursor at current position
        let (cur_col, cur_row) = vt.position();
        self.draw_cursor(cur_col, cur_row);
    }
    
    /// Draw a character at a specific position
    fn draw_char(&self, ch: char, col: usize, row: usize, fg: u32, bg: u32) {
        if self.fb_addr.is_null() {
            return;
        }
        
        let bitmap = font::get_char_bitmap(ch);
        let x_base = col * FONT_WIDTH;
        let y_base = row * FONT_HEIGHT;
        
        unsafe {
            for (row_idx, &byte) in bitmap.iter().enumerate() {
                let y = y_base + row_idx;
                if y >= self.fb_height {
                    break;
                }
                
                let row_ptr = self.fb_addr.add(y * self.fb_pitch) as *mut u32;
                
                for bit_idx in 0..FONT_WIDTH {
                    let x = x_base + bit_idx;
                    if x >= self.fb_width {
                        break;
                    }
                    
                    let pixel_on = (byte & (0x80 >> bit_idx)) != 0;
                    let color = if pixel_on { fg } else { bg };
                    row_ptr.add(x).write_volatile(color);
                }
            }
        }
    }
    
    /// Draw cursor at position
    fn draw_cursor(&self, col: usize, row: usize) {
        if self.fb_addr.is_null() {
            return;
        }
        
        let x_base = col * FONT_WIDTH;
        let y_base = row * FONT_HEIGHT;
        
        // Draw inverted space character as cursor
        unsafe {
            for row_idx in 0..FONT_HEIGHT {
                let y = y_base + row_idx;
                if y >= self.fb_height {
                    break;
                }
                
                let row_ptr = self.fb_addr.add(y * self.fb_pitch) as *mut u32;
                
                for bit_idx in 0..FONT_WIDTH {
                    let x = x_base + bit_idx;
                    if x >= self.fb_width {
                        break;
                    }
                    
                    row_ptr.add(x).write_volatile(0xFFFFFFFF);
                }
            }
        }
    }
}

/// Global VT manager
static VT_MANAGER: Mutex<VtManager> = Mutex::new(VtManager::new());

/// Initialize the virtual terminal system
pub fn init(fb_addr: *mut u8, width: usize, height: usize, pitch: usize) {
    VT_MANAGER.lock().init(fb_addr, width, height, pitch);
}

/// Get access to the VT manager
pub fn vt_manager() -> spin::MutexGuard<'static, VtManager> {
    VT_MANAGER.lock()
}

/// Switch to a specific virtual terminal (0-11 for F1-F12)
pub fn switch_terminal(id: usize) -> bool {
    VT_MANAGER.lock().switch_to(id)
}

/// Get the current terminal ID
pub fn current_terminal_id() -> usize {
    VT_MANAGER.lock().current_id()
}
