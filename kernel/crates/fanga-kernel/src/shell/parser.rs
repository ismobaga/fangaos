/// Command parser for the shell
///
/// Parses user input into commands and arguments

use alloc::vec::Vec;

/// Parse a command line into command and arguments
///
/// Returns (command, args) where command is the first word
/// and args is a vector of the remaining words
pub fn parse_command(line: &str) -> (&str, Vec<&str>) {
    let line = line.trim();
    
    if line.is_empty() {
        return ("", Vec::new());
    }
    
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.is_empty() {
        return ("", Vec::new());
    }
    
    let command = parts[0];
    let args = if parts.len() > 1 {
        parts[1..].to_vec()
    } else {
        Vec::new()
    };
    
    (command, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_empty() {
        let (cmd, args) = parse_command("");
        assert_eq!(cmd, "");
        assert_eq!(args.len(), 0);
    }
    
    #[test]
    fn test_parse_single_command() {
        let (cmd, args) = parse_command("help");
        assert_eq!(cmd, "help");
        assert_eq!(args.len(), 0);
    }
    
    #[test]
    fn test_parse_command_with_args() {
        let (cmd, args) = parse_command("echo hello world");
        assert_eq!(cmd, "echo");
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "hello");
        assert_eq!(args[1], "world");
    }
    
    #[test]
    fn test_parse_with_extra_whitespace() {
        let (cmd, args) = parse_command("  clear  ");
        assert_eq!(cmd, "clear");
        assert_eq!(args.len(), 0);
    }
}
