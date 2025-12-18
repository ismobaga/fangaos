/// Keyboard interrupt bridge
///
/// This module provides the bridge between the arch-specific keyboard driver
/// and the kernel's keyboard input handler.

use fanga_arch_x86_64::keyboard::{KeyEvent, Keyboard};

/// Keyboard event callback that will be called from the interrupt handler
pub fn keyboard_callback(event: KeyEvent, kbd: &Keyboard) {
    crate::io::keyboard_handler::handle_key_event(event, kbd);
}

/// Initialize the keyboard input system
pub fn init() {
    // Initialize the line editor
    crate::io::line_editor::init();
    
    // Register our keyboard callback with the arch layer
    unsafe {
        fanga_arch_x86_64::keyboard::set_keyboard_callback(keyboard_callback);
    }
}
