/// Line editor for interactive keyboard input
///
/// Provides line-based input handling with features like:
/// - Backspace and delete support
/// - Left/right arrow cursor movement
/// - Line buffering
/// - Echo to framebuffer console

use spin::Mutex;
extern crate alloc;
use alloc::vec::Vec;

/// Maximum line length
const MAX_LINE_LENGTH: usize = 256;

/// Line editor state
pub struct LineEditor {
    /// Current line buffer
    buffer: Vec<char>,
    /// Cursor position (index in buffer)
    cursor: usize,
}

impl LineEditor {
    pub const fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: 0,
        }
    }

    /// Initialize the line editor (needed because const fn can't create Vec)
    pub fn init(&mut self) {
        self.buffer = Vec::with_capacity(MAX_LINE_LENGTH);
        self.cursor = 0;
    }

    /// Insert a character at cursor position
    pub fn insert_char(&mut self, ch: char) -> bool {
        if self.buffer.len() >= MAX_LINE_LENGTH {
            return false;
        }

        if self.cursor >= self.buffer.len() {
            self.buffer.push(ch);
            self.cursor = self.buffer.len();
        } else {
            self.buffer.insert(self.cursor, ch);
            self.cursor += 1;
        }
        true
    }

    /// Delete character at cursor (like Delete key)
    pub fn delete_char(&mut self) -> bool {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
            true
        } else {
            false
        }
    }

    /// Delete character before cursor (like Backspace key)
    pub fn backspace(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer.remove(self.cursor);
            true
        } else {
            false
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor -= 1;
            true
        } else {
            false
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) -> bool {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    /// Move cursor to beginning of line
    pub fn move_home(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor = 0;
            true
        } else {
            false
        }
    }

    /// Move cursor to end of line
    pub fn move_end(&mut self) -> bool {
        if self.cursor < self.buffer.len() {
            self.cursor = self.buffer.len();
            true
        } else {
            false
        }
    }

    /// Get current line as a string
    pub fn get_line(&self) -> alloc::string::String {
        self.buffer.iter().collect()
    }

    /// Get current line length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if line is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get cursor position
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Clear the line
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    /// Get the buffer slice
    pub fn buffer(&self) -> &[char] {
        &self.buffer
    }
}

/// Global line editor
static LINE_EDITOR: Mutex<Option<LineEditor>> = Mutex::new(None);

/// Initialize the global line editor
pub fn init() {
    let mut editor = LineEditor::new();
    editor.init();
    *LINE_EDITOR.lock() = Some(editor);
}

/// Get access to the line editor
pub fn editor() -> spin::MutexGuard<'static, Option<LineEditor>> {
    LINE_EDITOR.lock()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_char() {
        let mut editor = LineEditor::new();
        editor.init();
        
        assert!(editor.insert_char('a'));
        assert_eq!(editor.len(), 1);
        assert_eq!(editor.cursor(), 1);
        assert_eq!(editor.get_line(), "a");
    }

    #[test]
    fn test_backspace() {
        let mut editor = LineEditor::new();
        editor.init();
        
        editor.insert_char('a');
        editor.insert_char('b');
        assert!(editor.backspace());
        assert_eq!(editor.len(), 1);
        assert_eq!(editor.get_line(), "a");
    }

    #[test]
    fn test_cursor_movement() {
        let mut editor = LineEditor::new();
        editor.init();
        
        editor.insert_char('a');
        editor.insert_char('b');
        editor.insert_char('c');
        
        assert!(editor.move_left());
        assert_eq!(editor.cursor(), 2);
        
        assert!(editor.move_left());
        assert_eq!(editor.cursor(), 1);
        
        assert!(editor.move_right());
        assert_eq!(editor.cursor(), 2);
    }

    #[test]
    fn test_home_end() {
        let mut editor = LineEditor::new();
        editor.init();
        
        editor.insert_char('a');
        editor.insert_char('b');
        editor.insert_char('c');
        
        assert!(editor.move_home());
        assert_eq!(editor.cursor(), 0);
        
        assert!(editor.move_end());
        assert_eq!(editor.cursor(), 3);
    }
}
