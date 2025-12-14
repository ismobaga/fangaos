//! Integration tests for the shell module

#![cfg(test)]

use fanga_kernel::shell::{parser, completion, history};

#[test]
fn test_command_parsing() {
    // Test empty command
    let (cmd, args) = parser::parse_command("");
    assert_eq!(cmd, "");
    assert_eq!(args.len(), 0);
    
    // Test single command
    let (cmd, args) = parser::parse_command("help");
    assert_eq!(cmd, "help");
    assert_eq!(args.len(), 0);
    
    // Test command with arguments
    let (cmd, args) = parser::parse_command("echo hello world");
    assert_eq!(cmd, "echo");
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], "hello");
    assert_eq!(args[1], "world");
    
    // Test with extra whitespace
    let (cmd, args) = parser::parse_command("  clear  ");
    assert_eq!(cmd, "clear");
    assert_eq!(args.len(), 0);
}

#[test]
fn test_tab_completion() {
    // Test single match
    let matches = completion::complete("hel");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0], "help");
    
    // Test multiple matches
    let matches = completion::complete("e");
    assert!(matches.len() >= 2);
    assert!(matches.contains(&"echo".to_string()));
    assert!(matches.contains(&"exit".to_string()));
    
    // Test no matches
    let matches = completion::complete("xyz");
    assert_eq!(matches.len(), 0);
    
    // Test complete_single
    assert_eq!(completion::complete_single("hel"), Some("help".to_string()));
    assert_eq!(completion::complete_single("e"), None); // Multiple matches
    assert_eq!(completion::complete_single("xyz"), None); // No matches
}

#[test]
fn test_command_history() {
    let mut hist = history::History::new();
    hist.init();
    
    // Test adding commands
    hist.add("help".to_string());
    hist.add("clear".to_string());
    hist.add("echo test".to_string());
    assert_eq!(hist.len(), 3);
    
    // Test duplicate prevention
    hist.add("echo test".to_string());
    assert_eq!(hist.len(), 3); // Should still be 3
    
    // Test navigation backwards
    assert_eq!(hist.prev(), Some("echo test".to_string()));
    assert_eq!(hist.prev(), Some("clear".to_string()));
    assert_eq!(hist.prev(), Some("help".to_string()));
    assert_eq!(hist.prev(), Some("help".to_string())); // At beginning, stays there
    
    // Test navigation forward
    assert_eq!(hist.next(), Some("clear".to_string()));
    assert_eq!(hist.next(), Some("echo test".to_string()));
    assert_eq!(hist.next(), Some("".to_string())); // Back to empty
}

#[test]
fn test_history_empty_commands() {
    let mut hist = history::History::new();
    hist.init();
    
    // Empty commands should not be added
    hist.add("".to_string());
    hist.add("   ".to_string());
    assert_eq!(hist.len(), 0);
    
    // But non-empty ones should
    hist.add("help".to_string());
    assert_eq!(hist.len(), 1);
}

#[test]
fn test_all_commands_have_completions() {
    // Make sure all commands are available for tab completion
    let commands = ["help", "clear", "echo", "memory", "ps", "exit"];
    
    for cmd in commands {
        let matches = completion::complete(cmd);
        assert!(
            !matches.is_empty() && matches.contains(&cmd.to_string()),
            "Command '{}' should be available for completion",
            cmd
        );
    }
}
