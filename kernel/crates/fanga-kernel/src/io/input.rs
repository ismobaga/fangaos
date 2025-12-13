use spin::Mutex;

extern crate alloc;
use alloc::collections::VecDeque;

/// Keyboard input buffer
static INPUT_BUFFER: Mutex<VecDeque<char>> = Mutex::new(VecDeque::new());

/// Add a character to the input buffer
pub fn push_char(ch: char) {
    let mut buffer = INPUT_BUFFER.lock();
    
    // Limit buffer size to prevent memory exhaustion
    if buffer.len() < 1024 {
        buffer.push_back(ch);
    }
}

/// Get a character from the input buffer (non-blocking)
pub fn pop_char() -> Option<char> {
    INPUT_BUFFER.lock().pop_front()
}

/// Check if input buffer is empty
pub fn is_empty() -> bool {
    INPUT_BUFFER.lock().is_empty()
}

/// Clear the input buffer
pub fn clear() {
    INPUT_BUFFER.lock().clear()
}
