# FangaOS Interactive Shell/REPL

## Overview

FangaOS now includes a fully interactive command-line shell (REPL - Read-Eval-Print Loop) that provides a user-friendly interface for interacting with the operating system.

## Features

### 1. Command Parser
The shell includes a robust command parser that:
- Parses user input into commands and arguments
- Handles multiple arguments with whitespace separation
- Supports empty commands gracefully
- Trims leading and trailing whitespace

### 2. Built-in Commands

The shell provides the following built-in commands:

#### `help`
Displays a list of available commands with brief descriptions.

```
fangaos> help
FangaOS Shell - Available Commands:
  help     - Display this help message
  clear    - Clear the screen
  echo     - Echo arguments to screen
  memory   - Display memory statistics
  ps       - Display process/task list
  power    - Display/control power management
  uptime   - Show system uptime
  uname    - Display system information
  ping     - Send ICMP echo request (network)
  reboot   - Reboot the system
  shutdown - Power off the system
  suspend  - Suspend system to low power state
  exit     - Exit the shell
```

#### `clear`
Clears the screen/framebuffer, providing a fresh view.

```
fangaos> clear
```

#### `echo`
Echoes the provided arguments back to the screen.

```
fangaos> echo Hello, FangaOS!
Hello, FangaOS!
```

#### `memory`
Displays comprehensive memory statistics including:
- Total, used, and free physical memory
- Total and used heap memory
- Memory allocation/deallocation counts

```
fangaos> memory
Memory Statistics:
  Total Physical: 2048 MiB
  Used Physical:  64 MiB
  Free Physical:  1984 MiB
  Total Heap:     12 KiB
  Used Heap:      8 KiB
  Allocations:    142
  Deallocations:  38
```

#### `ps`
Displays information about running tasks/processes.

```
fangaos> ps
Task List:
  ID    NAME                STATE       PRIORITY
  ----  ------------------  ----------  --------
  Total tasks: 3
  Ready tasks: 2
```

#### `power`
Displays power management status and controls power settings. See the Power Management documentation for details.

```
fangaos> power
Power Management Status:
  ...
  
fangaos> power policy performance
CPU scaling policy set to: Performance
```

#### `uptime`
Shows system uptime in a human-readable format including days, hours, minutes, and seconds.

```
fangaos> uptime
System Uptime:
  0:15:42
  Total: 942000 ms
```

#### `uname`
Displays system information including kernel name, version, and architecture.

```
fangaos> uname
FangaOS System Information:
  Kernel Name:    FangaOS
  Kernel Version: 0.1.0
  Architecture:   x86_64
  Machine:        PC (UEFI)
```

#### `ping`
Sends ICMP echo requests to test network connectivity. Currently shows a stub message as ICMP is not fully implemented.

```
fangaos> ping 8.8.8.8
PING 8.8.8.8...
Error: Network stack not fully initialized
ICMP protocol implementation pending
```

#### `reboot`
Reboots the system using the keyboard controller reset method.

```
fangaos> reboot
Rebooting system...
```

#### `shutdown`
Powers off the system. Attempts QEMU/Bochs shutdown port for graceful shutdown in virtual environments.

```
fangaos> shutdown
Shutting down system...
```

#### `suspend`
Suspends the system to a low power state (S3 - Suspend to RAM).

```
fangaos> suspend
Suspending system to S3 (Suspend to RAM)...
System resumed from suspend.
```

#### `exit`
Exits the shell (stops accepting commands).

```
fangaos> exit
Exiting shell...
```

### 3. Command History

The shell maintains a history of executed commands (up to 100 entries) with navigation support:

- **Up Arrow (↑)**: Navigate to the previous command in history
- **Down Arrow (↓)**: Navigate to the next command in history

Features:
- Prevents duplicate consecutive commands
- Empty commands are not added to history
- Allows cycling through command history
- Returns to empty line when navigating past the last command

### 4. Tab Completion

The shell provides intelligent tab completion for command names:

- Press **Tab** to auto-complete a partial command
- If there's only one match, it completes automatically
- If there are multiple matches, it displays all possibilities
- If there are no matches, nothing happens

Example:
```
fangaos> hel<Tab>
fangaos> help

fangaos> e<Tab>
echo  exit
fangaos> e
```

### 5. Line Editing

The shell supports comprehensive line editing features (inherited from the line editor):

- **Left Arrow (←)**: Move cursor left
- **Right Arrow (→)**: Move cursor right
- **Home**: Move cursor to beginning of line
- **End**: Move cursor to end of line
- **Backspace**: Delete character before cursor
- **Delete**: Delete character at cursor
- **Ctrl+C**: Interrupt/clear current line
- **Ctrl+D**: EOF signal (or delete at cursor if line is not empty)

### 6. Customizable Prompt

The shell features a customizable prompt (default: `fangaos> `). The prompt can be changed programmatically:

```rust
let mut shell_guard = shell::shell();
if let Some(shell) = shell_guard.as_mut() {
    shell.set_prompt(String::from("my_prompt> "));
}
```

## Architecture

### Module Structure

```
kernel/src/shell/
├── mod.rs          # Main shell module with state management
├── parser.rs       # Command line parser
├── commands.rs     # Built-in command implementations
├── history.rs      # Command history management
└── completion.rs   # Tab completion logic
```

### Integration

The shell is integrated with the existing kernel infrastructure:

1. **Keyboard Handler** (`io/keyboard_handler.rs`): Processes keyboard events and forwards them to the shell
2. **Line Editor** (`io/line_editor.rs`): Provides line editing capabilities
3. **Framebuffer** (`io/framebuffer.rs`): Handles output to the screen
4. **Scheduler** (`task/scheduler.rs`): Accessed by the `ps` command
5. **Memory Stats** (`memory/stats.rs`): Accessed by the `memory` command
6. **Power Management** (`power/`): Accessed by the `power`, `shutdown`, `reboot`, and `suspend` commands
7. **Timer** (`fanga-arch-x86_64::interrupts::idt`): Accessed by the `uptime` command

### Initialization

The shell is initialized during kernel startup after the heap and keyboard subsystems:

```rust
// In main.rs
shell::init();
shell::history::init();
io::line_editor::init();
```

## Testing

The shell implementation includes comprehensive tests:

### Unit Tests

Each shell module includes its own unit tests:

```bash
cd kernel/crates/fanga-kernel && cargo test --lib
```

Tests cover:
- Command parsing with various inputs
- Tab completion logic
- Command history navigation
- Edge cases and error handling

### Integration Tests

The `tests/shell_integration.rs` file provides end-to-end testing:

```bash
cd kernel/crates/fanga-kernel && cargo test --tests
```

Integration tests verify:
- Command parsing workflow
- Tab completion with all available commands
- Command history with realistic usage patterns
- Empty command handling

### Running All Tests

```bash
make test
```

All tests pass successfully, including:
- 62 unit tests in the main kernel
- 5 shell integration tests
- Additional tests in architecture-specific modules

## Future Enhancements

Potential improvements for the shell:

1. **Pipeline Support**: Command chaining with `|` operator
2. **Redirection**: Output redirection with `>` and `>>`
3. **Variables**: Environment variable support
4. **Scripting**: Ability to run shell scripts
5. **More Commands**: Additional built-in commands (ls, cat, etc.)
6. **Command Aliases**: User-defined command shortcuts
7. **Job Control**: Background processes with `&`
8. **History Search**: Reverse search with Ctrl+R
9. **Auto-suggestions**: Fish-like command suggestions
10. **Syntax Highlighting**: Color-coded command syntax

## Usage Example

Here's a typical interactive session:

```
Welcome to FangaOS Interactive Shell!
Type 'help' for available commands.

fangaos> help
FangaOS Shell - Available Commands:
  help     - Display this help message
  clear    - Clear the screen
  echo     - Echo arguments to screen
  memory   - Display memory statistics
  ps       - Display process/task list
  power    - Display/control power management
  uptime   - Show system uptime
  uname    - Display system information
  ping     - Send ICMP echo request (network)
  reboot   - Reboot the system
  shutdown - Power off the system
  suspend  - Suspend system to low power state
  exit     - Exit the shell

fangaos> echo Hello from FangaOS!
Hello from FangaOS!

fangaos> uptime
System Uptime:
  0:15:42
  Total: 942000 ms

fangaos> uname
FangaOS System Information:
  Kernel Name:    FangaOS
  Kernel Version: 0.1.0
  Architecture:   x86_64
  Machine:        PC (UEFI)

fangaos> memory
Memory Statistics:
  Total Physical: 2048 MiB
  Used Physical:  64 MiB
  Free Physical:  1984 MiB
  Total Heap:     12 KiB
  Used Heap:      8 KiB
  Allocations:    142
  Deallocations:  38

fangaos> ps
Task List:
  ID    NAME                STATE       PRIORITY
  ----  ------------------  ----------  --------
  Total tasks: 3
  Ready tasks: 2

fangaos> <Up Arrow shows previous command>
fangaos> ps

fangaos> exit
Exiting shell...
```

## Implementation Details

### Command Execution Flow

1. User types characters → Keyboard interrupt → Keyboard handler
2. Characters are added to line editor buffer
3. User presses Enter → Line is submitted
4. Command is added to history (if not empty/duplicate)
5. Command is parsed into command name and arguments
6. Shell executes the command
7. Command output is displayed on framebuffer
8. Shell displays prompt for next command

### Memory Safety

The shell implementation follows Rust's memory safety principles:
- All data structures use safe abstractions (`Vec`, `String`, `Mutex`)
- No unsafe code in shell modules
- Proper lifetime management for references
- Thread-safe access to global state via `Mutex`

### Performance

The shell is designed for efficiency:
- Command parsing is O(n) where n is input length
- Tab completion is O(m) where m is number of commands
- History navigation is O(1)
- Minimal heap allocations during normal operation

## Contributing

When adding new commands:

1. Add the command name to `completion.rs::COMMANDS`
2. Implement the command in `commands.rs`
3. Add a case in `commands.rs::execute()`
4. Add tests in the respective test modules
5. Update this documentation

## Conclusion

The FangaOS shell provides a powerful and user-friendly interface for interacting with the operating system. It combines modern shell features like command history and tab completion with a clean, minimal design suitable for a kernel environment.
