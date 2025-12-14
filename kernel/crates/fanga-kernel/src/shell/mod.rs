/// Interactive command-line shell/REPL
///
/// This module provides an interactive shell with:
/// - Command parsing
/// - Built-in commands (help, clear, echo, memory, ps, exit)
/// - Command history navigation
/// - Tab completion
/// - Customizable prompt

pub mod parser;
pub mod commands;
pub mod history;
pub mod completion;

use alloc::string::String;
use spin::Mutex;

/// Shell state
pub struct Shell {
    /// Command prompt
    prompt: String,
    /// Whether the shell is running
    running: bool,
}

impl Shell {
    pub const fn new() -> Self {
        Self {
            prompt: String::new(),
            running: false,
        }
    }

    /// Initialize the shell with a default prompt
    pub fn init(&mut self) {
        self.prompt = String::from("fangaos> ");
        self.running = true;
    }

    /// Get the current prompt
    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    /// Set a custom prompt
    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt = prompt;
    }

    /// Check if the shell is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the shell
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Process a command line
    pub fn execute(&mut self, line: &str) -> Result<(), &'static str> {
        // Parse the command
        let (command, args) = parser::parse_command(line);

        // Execute the command
        commands::execute(command, args, self)
    }
}

/// Global shell instance
static SHELL: Mutex<Option<Shell>> = Mutex::new(None);

/// Initialize the global shell
pub fn init() {
    let mut shell = Shell::new();
    shell.init();
    *SHELL.lock() = Some(shell);
}

/// Get access to the shell
pub fn shell() -> spin::MutexGuard<'static, Option<Shell>> {
    SHELL.lock()
}

/// Check if the shell is initialized
pub fn is_initialized() -> bool {
    SHELL.lock().is_some()
}
