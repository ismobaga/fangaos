/// Command history for shell
///
/// Maintains a history of executed commands and allows
/// navigation with up/down arrows

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

/// Maximum number of commands to keep in history
const MAX_HISTORY: usize = 100;

/// Command history
pub struct History {
    /// List of commands (None until initialized)
    commands: Option<Vec<String>>,
    /// Current position in history (for navigation)
    position: Option<usize>,
}

impl History {
    pub const fn new() -> Self {
        Self {
            commands: None,
            position: None,
        }
    }

    /// Initialize the history
    pub fn init(&mut self) {
        if self.commands.is_none() {
            self.commands = Some(Vec::with_capacity(MAX_HISTORY));
            self.position = None;
        }
    }

    /// Ensure the history is initialized
    fn ensure_initialized(&mut self) {
        if self.commands.is_none() {
            self.init();
        }
    }

    /// Add a command to history
    pub fn add(&mut self, command: String) {
        self.ensure_initialized();
        let commands = self.commands.as_mut().unwrap();
        
        // Don't add empty commands
        if command.trim().is_empty() {
            return;
        }
        
        // Don't add duplicate consecutive commands
        if let Some(last) = commands.last() {
            if last == &command {
                return;
            }
        }
        
        // Add the command
        if commands.len() >= MAX_HISTORY {
            commands.remove(0);
        }
        commands.push(command);
        
        // Reset position
        self.position = None;
    }

    /// Get the previous command (up arrow)
    pub fn prev(&mut self) -> Option<String> {
        self.ensure_initialized();
        let commands = self.commands.as_ref().unwrap();
        
        if commands.is_empty() {
            return None;
        }
        
        match self.position {
            None => {
                // Start from the end
                self.position = Some(commands.len() - 1);
                Some(commands[commands.len() - 1].clone())
            }
            Some(pos) => {
                if pos > 0 {
                    self.position = Some(pos - 1);
                    Some(commands[pos - 1].clone())
                } else {
                    // Already at the beginning
                    Some(commands[0].clone())
                }
            }
        }
    }

    /// Get the next command (down arrow)
    pub fn next(&mut self) -> Option<String> {
        self.ensure_initialized();
        let commands = self.commands.as_ref().unwrap();
        
        if commands.is_empty() {
            return None;
        }
        
        match self.position {
            None => None, // Not navigating
            Some(pos) => {
                if pos < commands.len() - 1 {
                    self.position = Some(pos + 1);
                    Some(commands[pos + 1].clone())
                } else {
                    // At the end, return to empty
                    self.position = None;
                    Some(String::new())
                }
            }
        }
    }

    /// Reset navigation position
    pub fn reset_position(&mut self) {
        self.position = None;
    }

    /// Get the number of commands in history
    pub fn len(&self) -> usize {
        match &self.commands {
            Some(cmds) => cmds.len(),
            None => 0,
        }
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Global command history
static HISTORY: Mutex<Option<History>> = Mutex::new(None);

/// Initialize the global history
pub fn init() {
    let mut history = History::new();
    history.init();
    *HISTORY.lock() = Some(history);
}

/// Get access to the history
pub fn history() -> spin::MutexGuard<'static, Option<History>> {
    HISTORY.lock()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_command() {
        let mut history = History::new();
        history.init();
        
        history.add(String::from("help"));
        assert_eq!(history.len(), 1);
        
        history.add(String::from("clear"));
        assert_eq!(history.len(), 2);
    }
    
    #[test]
    fn test_skip_duplicates() {
        let mut history = History::new();
        history.init();
        
        history.add(String::from("help"));
        history.add(String::from("help"));
        assert_eq!(history.len(), 1);
    }
    
    #[test]
    fn test_navigation() {
        let mut history = History::new();
        history.init();
        
        history.add(String::from("help"));
        history.add(String::from("clear"));
        history.add(String::from("echo test"));
        
        // Navigate backwards
        assert_eq!(history.prev(), Some(String::from("echo test")));
        assert_eq!(history.prev(), Some(String::from("clear")));
        assert_eq!(history.prev(), Some(String::from("help")));
        
        // Navigate forward
        assert_eq!(history.next(), Some(String::from("clear")));
        assert_eq!(history.next(), Some(String::from("echo test")));
        assert_eq!(history.next(), Some(String::new())); // Back to empty
    }
}
