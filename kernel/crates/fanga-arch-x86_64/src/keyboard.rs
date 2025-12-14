use crate::port::inb;

/// PS/2 Keyboard driver for x86_64
///
/// This module provides a simple PS/2 keyboard driver that translates
/// scancodes to ASCII characters using US keyboard layout (scancode set 1).

const PS2_DATA_PORT: u16 = 0x60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Backspace,
    Delete,
    Enter,
    Tab,
    Escape,
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    LeftAlt,
    RightAlt,
    CapsLock,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Up, Down, Left, Right,
    Home, End,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    Press(KeyCode),
    Release(KeyCode),
}

/// Keyboard state tracker
pub struct Keyboard {
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    caps_lock: bool,
    extended_scancode: bool,
}

impl Keyboard {
    pub const fn new() -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            caps_lock: false,
            extended_scancode: false,
        }
    }

    /// Read scancode from PS/2 data port
    pub fn read_scancode(&self) -> u8 {
        unsafe { inb(PS2_DATA_PORT) }
    }

    /// Process a scancode and return the corresponding key event
    pub fn process_scancode(&mut self, scancode: u8) -> Option<KeyEvent> {
        // Handle extended scancode prefix (0xE0)
        if scancode == 0xE0 {
            self.extended_scancode = true;
            return None;
        }
        
        // Check if it's a key release (bit 7 set)
        let is_release = (scancode & 0x80) != 0;
        let key_code = scancode & 0x7F;

        let keycode = if self.extended_scancode {
            // Extended scancodes (arrow keys, etc.)
            self.extended_scancode = false;
            match key_code {
                0x48 => KeyCode::Up,
                0x50 => KeyCode::Down,
                0x4B => KeyCode::Left,
                0x4D => KeyCode::Right,
                0x47 => KeyCode::Home,
                0x4F => KeyCode::End,
                0x53 => KeyCode::Delete,
                0x1D => KeyCode::RightCtrl,
                _ => KeyCode::Unknown,
            }
        } else {
            // Normal scancodes
            match key_code {
                0x01 => KeyCode::Escape,
                0x02 => KeyCode::Char('1'),
                0x03 => KeyCode::Char('2'),
                0x04 => KeyCode::Char('3'),
                0x05 => KeyCode::Char('4'),
                0x06 => KeyCode::Char('5'),
                0x07 => KeyCode::Char('6'),
                0x08 => KeyCode::Char('7'),
                0x09 => KeyCode::Char('8'),
                0x0A => KeyCode::Char('9'),
                0x0B => KeyCode::Char('0'),
                0x0C => KeyCode::Char('-'),
                0x0D => KeyCode::Char('='),
                0x0E => KeyCode::Backspace,
                0x0F => KeyCode::Tab,
                0x10 => KeyCode::Char('q'),
                0x11 => KeyCode::Char('w'),
                0x12 => KeyCode::Char('e'),
                0x13 => KeyCode::Char('r'),
                0x14 => KeyCode::Char('t'),
                0x15 => KeyCode::Char('y'),
                0x16 => KeyCode::Char('u'),
                0x17 => KeyCode::Char('i'),
                0x18 => KeyCode::Char('o'),
                0x19 => KeyCode::Char('p'),
                0x1A => KeyCode::Char('['),
                0x1B => KeyCode::Char(']'),
                0x1C => KeyCode::Enter,
                0x1D => KeyCode::LeftCtrl,
                0x1E => KeyCode::Char('a'),
                0x1F => KeyCode::Char('s'),
                0x20 => KeyCode::Char('d'),
                0x21 => KeyCode::Char('f'),
                0x22 => KeyCode::Char('g'),
                0x23 => KeyCode::Char('h'),
                0x24 => KeyCode::Char('j'),
                0x25 => KeyCode::Char('k'),
                0x26 => KeyCode::Char('l'),
                0x27 => KeyCode::Char(';'),
                0x28 => KeyCode::Char('\''),
                0x29 => KeyCode::Char('`'),
                0x2A => KeyCode::LeftShift,
                0x2B => KeyCode::Char('\\'),
                0x2C => KeyCode::Char('z'),
                0x2D => KeyCode::Char('x'),
                0x2E => KeyCode::Char('c'),
                0x2F => KeyCode::Char('v'),
                0x30 => KeyCode::Char('b'),
                0x31 => KeyCode::Char('n'),
                0x32 => KeyCode::Char('m'),
                0x33 => KeyCode::Char(','),
                0x34 => KeyCode::Char('.'),
                0x35 => KeyCode::Char('/'),
                0x36 => KeyCode::RightShift,
                0x38 => KeyCode::LeftAlt,
                0x39 => KeyCode::Char(' '),
                0x3A => KeyCode::CapsLock,
                0x3B => KeyCode::F1,
                0x3C => KeyCode::F2,
                0x3D => KeyCode::F3,
                0x3E => KeyCode::F4,
                0x3F => KeyCode::F5,
                0x40 => KeyCode::F6,
                0x41 => KeyCode::F7,
                0x42 => KeyCode::F8,
                0x43 => KeyCode::F9,
                0x44 => KeyCode::F10,
                0x57 => KeyCode::F11,
                0x58 => KeyCode::F12,
                _ => KeyCode::Unknown,
            }
        };

        // Update modifier state
        if !is_release {
            match keycode {
                KeyCode::LeftShift | KeyCode::RightShift => {
                    self.shift_pressed = true;
                }
                KeyCode::LeftCtrl | KeyCode::RightCtrl => {
                    self.ctrl_pressed = true;
                }
                KeyCode::LeftAlt | KeyCode::RightAlt => {
                    self.alt_pressed = true;
                }
                KeyCode::CapsLock => {
                    self.caps_lock = !self.caps_lock;
                }
                _ => {}
            }
        } else {
            match keycode {
                KeyCode::LeftShift | KeyCode::RightShift => {
                    self.shift_pressed = false;
                }
                KeyCode::LeftCtrl | KeyCode::RightCtrl => {
                    self.ctrl_pressed = false;
                }
                KeyCode::LeftAlt | KeyCode::RightAlt => {
                    self.alt_pressed = false;
                }
                _ => {}
            }
        }

        // Return key event
        if is_release {
            Some(KeyEvent::Release(keycode))
        } else {
            Some(KeyEvent::Press(keycode))
        }
    }

    /// Convert a keycode to ASCII character, applying shift/caps lock
    pub fn to_ascii(&self, keycode: KeyCode) -> Option<char> {
        match keycode {
            KeyCode::Char(c) => {
                let should_uppercase = (self.shift_pressed && !self.caps_lock)
                    || (!self.shift_pressed && self.caps_lock && c.is_ascii_alphabetic());

                if should_uppercase {
                    Some(c.to_ascii_uppercase())
                } else if self.shift_pressed {
                    // Apply shift to symbols
                    match c {
                        '1' => Some('!'),
                        '2' => Some('@'),
                        '3' => Some('#'),
                        '4' => Some('$'),
                        '5' => Some('%'),
                        '6' => Some('^'),
                        '7' => Some('&'),
                        '8' => Some('*'),
                        '9' => Some('('),
                        '0' => Some(')'),
                        '-' => Some('_'),
                        '=' => Some('+'),
                        '[' => Some('{'),
                        ']' => Some('}'),
                        ';' => Some(':'),
                        '\'' => Some('"'),
                        '`' => Some('~'),
                        '\\' => Some('|'),
                        ',' => Some('<'),
                        '.' => Some('>'),
                        '/' => Some('?'),
                        _ => Some(c),
                    }
                } else {
                    Some(c)
                }
            }
            KeyCode::Backspace => Some('\x08'),
            KeyCode::Enter => Some('\n'),
            KeyCode::Tab => Some('\t'),
            _ => None,
        }
    }

    pub fn is_shift_pressed(&self) -> bool {
        self.shift_pressed
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        self.ctrl_pressed
    }

    pub fn is_alt_pressed(&self) -> bool {
        self.alt_pressed
    }

    pub fn is_caps_lock(&self) -> bool {
        self.caps_lock
    }
}

/// Global keyboard state
static mut KEYBOARD: Keyboard = Keyboard::new();

/// Type for keyboard event callback
pub type KeyboardCallback = fn(KeyEvent, &Keyboard);

/// Global keyboard event callback
static mut KEYBOARD_CALLBACK: Option<KeyboardCallback> = None;

/// Set the keyboard event callback
///
/// # Safety
/// Must be called before enabling keyboard interrupts
pub unsafe fn set_keyboard_callback(callback: KeyboardCallback) {
    KEYBOARD_CALLBACK = Some(callback);
}

/// Get a mutable reference to the global keyboard
pub fn keyboard() -> &'static mut Keyboard {
    unsafe { &mut KEYBOARD }
}

/// Dispatch a keyboard event to the registered callback
pub(crate) fn dispatch_event(event: KeyEvent, kbd: &Keyboard) {
    unsafe {
        if let Some(callback) = KEYBOARD_CALLBACK {
            callback(event, kbd);
        }
    }
}
