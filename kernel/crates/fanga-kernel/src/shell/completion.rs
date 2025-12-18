/// Tab completion for shell commands
///
/// Provides command name completion when Tab is pressed

use alloc::vec::Vec;
use alloc::string::String;

/// List of all available commands
const COMMANDS: &[&str] = &[
    "clear",
    "echo",
    "exit",
    "help",
    "memory",
    "ping",
    "power",
    "ps",
    "reboot",
    "shutdown",
    "suspend",
    "uname",
    "uptime",
];

/// Find completions for a partial command
///
/// Returns a list of commands that start with the given prefix
pub fn complete(prefix: &str) -> Vec<String> {
    if prefix.is_empty() {
        return Vec::new();
    }
    
    COMMANDS
        .iter()
        .filter(|cmd| cmd.starts_with(prefix))
        .map(|&cmd| String::from(cmd))
        .collect()
}

/// Get a single completion if there's only one match
///
/// Returns the completed command if there's exactly one match,
/// or None if there are zero or multiple matches
pub fn complete_single(prefix: &str) -> Option<String> {
    let matches = complete(prefix);
    
    if matches.len() == 1 {
        Some(matches[0].clone())
    } else {
        None
    }
}

/// Get the common prefix of all completions
///
/// If there are multiple matches, returns the longest common prefix
pub fn complete_common(prefix: &str) -> Option<String> {
    let matches = complete(prefix);
    
    if matches.is_empty() {
        return None;
    }
    
    if matches.len() == 1 {
        return Some(matches[0].clone());
    }
    
    // Find common prefix
    let first = &matches[0];
    let mut common_len = first.len();
    
    for cmd in matches.iter().skip(1) {
        let min_len = core::cmp::min(common_len, cmd.len());
        for i in 0..min_len {
            if first.as_bytes()[i] != cmd.as_bytes()[i] {
                common_len = i;
                break;
            }
        }
    }
    
    Some(String::from(&first[..common_len]))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_complete_single_match() {
        let matches = complete("hel");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], "help");
    }
    
    #[test]
    fn test_complete_multiple_matches() {
        let matches = complete("e");
        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&String::from("echo")));
        assert!(matches.contains(&String::from("exit")));
    }
    
    #[test]
    fn test_complete_no_matches() {
        let matches = complete("xyz");
        assert_eq!(matches.len(), 0);
    }
    
    #[test]
    fn test_complete_single() {
        assert_eq!(complete_single("hel"), Some(String::from("help")));
        assert_eq!(complete_single("e"), None); // Multiple matches
        assert_eq!(complete_single("xyz"), None); // No matches
    }
    
    #[test]
    fn test_complete_common() {
        assert_eq!(complete_common("e"), Some(String::from("e"))); // Common prefix is just "e"
        assert_eq!(complete_common("hel"), Some(String::from("help")));
        assert_eq!(complete_common("xyz"), None);
    }
}
