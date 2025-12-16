/// Keyboard layout support for multiple languages and regions
///
/// This module provides an abstraction for keyboard layouts, allowing support
/// for different languages and keyboard configurations beyond the US layout.

use crate::keyboard::KeyCode;

/// Keyboard layout trait
pub trait KeyboardLayout: Send + Sync {
    /// Get the name of this layout
    fn name(&self) -> &'static str;
    
    /// Convert a keycode to a character with modifiers applied
    fn to_char(&self, keycode: KeyCode, shift: bool, caps_lock: bool) -> Option<char>;
}

/// US QWERTY keyboard layout
pub struct UsLayout;

impl KeyboardLayout for UsLayout {
    fn name(&self) -> &'static str {
        "US"
    }
    
    fn to_char(&self, keycode: KeyCode, shift: bool, caps_lock: bool) -> Option<char> {
        match keycode {
            KeyCode::Char(c) => {
                let should_uppercase = (shift && !caps_lock)
                    || (!shift && caps_lock && c.is_ascii_alphabetic());

                if should_uppercase && c.is_ascii_alphabetic() {
                    Some(c.to_ascii_uppercase())
                } else if shift {
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
}

/// UK QWERTY keyboard layout
pub struct UkLayout;

impl KeyboardLayout for UkLayout {
    fn name(&self) -> &'static str {
        "UK"
    }
    
    fn to_char(&self, keycode: KeyCode, shift: bool, caps_lock: bool) -> Option<char> {
        match keycode {
            KeyCode::Char(c) => {
                let should_uppercase = (shift && !caps_lock)
                    || (!shift && caps_lock && c.is_ascii_alphabetic());

                if should_uppercase && c.is_ascii_alphabetic() {
                    Some(c.to_ascii_uppercase())
                } else if shift {
                    // UK-specific symbol mappings
                    match c {
                        '1' => Some('!'),
                        '2' => Some('"'), // Different from US
                        '3' => Some('£'), // UK pound sign
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
                        '\'' => Some('@'), // Different from US
                        '`' => Some('¬'), // UK negation sign
                        '\\' => Some('|'),
                        ',' => Some('<'),
                        '.' => Some('>'),
                        '/' => Some('?'),
                        '#' => Some('~'), // UK # key shifted
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
}

/// German QWERTZ keyboard layout
pub struct DeLayout;

impl KeyboardLayout for DeLayout {
    fn name(&self) -> &'static str {
        "DE"
    }
    
    fn to_char(&self, keycode: KeyCode, shift: bool, caps_lock: bool) -> Option<char> {
        match keycode {
            KeyCode::Char(c) => {
                // German layout has different key positions (QWERTZ instead of QWERTY)
                let mapped_char = match c {
                    'y' => 'z', // Y and Z are swapped
                    'z' => 'y',
                    _ => c,
                };
                
                let should_uppercase = (shift && !caps_lock)
                    || (!shift && caps_lock && mapped_char.is_ascii_alphabetic());

                if should_uppercase && mapped_char.is_ascii_alphabetic() {
                    Some(mapped_char.to_ascii_uppercase())
                } else if shift {
                    // German-specific symbol mappings
                    match mapped_char {
                        '1' => Some('!'),
                        '2' => Some('"'),
                        '3' => Some('§'), // German section sign
                        '4' => Some('$'),
                        '5' => Some('%'),
                        '6' => Some('&'),
                        '7' => Some('/'),
                        '8' => Some('('),
                        '9' => Some(')'),
                        '0' => Some('='),
                        '-' => Some('_'), // ß key
                        '=' => Some('?'), // ´ key shifted
                        '[' => Some('Ü'),
                        ']' => Some('*'),
                        ';' => Some('Ö'),
                        '\'' => Some('Ä'),
                        '`' => Some('°'),
                        '\\' => Some('\''),
                        ',' => Some(';'),
                        '.' => Some(':'),
                        '/' => Some('_'),
                        _ => Some(mapped_char),
                    }
                } else {
                    Some(mapped_char)
                }
            }
            KeyCode::Backspace => Some('\x08'),
            KeyCode::Enter => Some('\n'),
            KeyCode::Tab => Some('\t'),
            _ => None,
        }
    }
}

/// French AZERTY keyboard layout
pub struct FrLayout;

impl KeyboardLayout for FrLayout {
    fn name(&self) -> &'static str {
        "FR"
    }
    
    fn to_char(&self, keycode: KeyCode, shift: bool, caps_lock: bool) -> Option<char> {
        match keycode {
            KeyCode::Char(c) => {
                // French layout (AZERTY) has significantly different layout
                // Physical key mappings (not character transformations)
                let mapped_char = match c {
                    'q' => 'a', // AZERTY: physical Q key produces 'a'
                    'w' => 'z', // AZERTY: physical W key produces 'z'
                    'a' => 'q', // AZERTY: physical A key produces 'q'
                    'z' => 'w', // AZERTY: physical Z key produces 'w'
                    'm' => ',', // AZERTY: physical M key produces ','
                    ',' => ';', // AZERTY: physical comma key produces ';'
                    ';' => 'm', // AZERTY: physical semicolon key produces 'm'
                    _ => c,
                };
                
                let should_uppercase = (shift && !caps_lock)
                    || (!shift && caps_lock && mapped_char.is_ascii_alphabetic());

                if should_uppercase && mapped_char.is_ascii_alphabetic() {
                    Some(mapped_char.to_ascii_uppercase())
                } else if shift {
                    // French-specific symbol mappings
                    match mapped_char {
                        '1' => Some('&'), // Numbers are shifted in French
                        '2' => Some('é'),
                        '3' => Some('"'),
                        '4' => Some('\''),
                        '5' => Some('('),
                        '6' => Some('-'),
                        '7' => Some('è'),
                        '8' => Some('_'),
                        '9' => Some('ç'),
                        '0' => Some('à'),
                        '-' => Some('°'),
                        '=' => Some('+'),
                        _ => Some(mapped_char),
                    }
                } else {
                    // Unshifted numbers in French layout
                    match mapped_char {
                        '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '0' => {
                            // Keep as-is, these produce symbols when unshifted
                            Some(mapped_char)
                        }
                        _ => Some(mapped_char),
                    }
                }
            }
            KeyCode::Backspace => Some('\x08'),
            KeyCode::Enter => Some('\n'),
            KeyCode::Tab => Some('\t'),
            _ => None,
        }
    }
}

/// Global keyboard layout manager
pub struct LayoutManager {
    current_layout: LayoutType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutType {
    Us,
    Uk,
    De,
    Fr,
}

impl LayoutManager {
    pub const fn new() -> Self {
        Self {
            current_layout: LayoutType::Us,
        }
    }
    
    /// Get the current layout
    pub fn current_layout(&self) -> LayoutType {
        self.current_layout
    }
    
    /// Set the current layout
    pub fn set_layout(&mut self, layout: LayoutType) {
        self.current_layout = layout;
    }
    
    /// Convert a keycode to character using the current layout
    pub fn to_char(&self, keycode: KeyCode, shift: bool, caps_lock: bool) -> Option<char> {
        match self.current_layout {
            LayoutType::Us => UsLayout.to_char(keycode, shift, caps_lock),
            LayoutType::Uk => UkLayout.to_char(keycode, shift, caps_lock),
            LayoutType::De => DeLayout.to_char(keycode, shift, caps_lock),
            LayoutType::Fr => FrLayout.to_char(keycode, shift, caps_lock),
        }
    }
    
    /// Get the name of the current layout
    pub fn current_layout_name(&self) -> &'static str {
        match self.current_layout {
            LayoutType::Us => UsLayout.name(),
            LayoutType::Uk => UkLayout.name(),
            LayoutType::De => DeLayout.name(),
            LayoutType::Fr => FrLayout.name(),
        }
    }
}

/// Global layout manager
/// Note: This is accessed from interrupt context, so we use a simple static.
/// The layout is typically set once during initialization and then only read,
/// making this safe. Future improvements could use a lock-free atomic pointer.
static mut LAYOUT_MANAGER: LayoutManager = LayoutManager::new();

/// Get a mutable reference to the global layout manager
pub fn layout_manager() -> &'static mut LayoutManager {
    unsafe { &mut LAYOUT_MANAGER }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard::KeyCode;
    
    #[test]
    fn test_us_layout() {
        let layout = UsLayout;
        assert_eq!(layout.name(), "US");
        // Test basic character without shift
        assert_eq!(layout.to_char(KeyCode::Char('a'), false, false), Some('a'));
        // Test with shift
        assert_eq!(layout.to_char(KeyCode::Char('a'), true, false), Some('A'));
        // Test number key
        assert_eq!(layout.to_char(KeyCode::Char('1'), false, false), Some('1'));
        // Test shifted number
        assert_eq!(layout.to_char(KeyCode::Char('1'), true, false), Some('!'));
    }
    
    #[test]
    fn test_uk_layout() {
        let layout = UkLayout;
        assert_eq!(layout.name(), "UK");
        // UK has different shift mappings
        assert_eq!(layout.to_char(KeyCode::Char('a'), false, false), Some('a'));
        assert_eq!(layout.to_char(KeyCode::Char('a'), true, false), Some('A'));
    }
    
    #[test]
    fn test_layout_manager() {
        let mut mgr = LayoutManager::new();
        assert_eq!(mgr.current_layout(), LayoutType::Us);
        
        mgr.set_layout(LayoutType::Uk);
        assert_eq!(mgr.current_layout(), LayoutType::Uk);
        assert_eq!(mgr.current_layout_name(), "UK");
        
        // Test conversion through manager
        let result = mgr.to_char(KeyCode::Char('a'), false, false);
        assert_eq!(result, Some('a'));
    }
}
