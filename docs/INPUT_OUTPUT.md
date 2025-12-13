# Input/Output System in FangaOS

This document describes the input/output subsystem implemented in FangaOS.

## Overview

The FangaOS kernel implements a comprehensive I/O system with the following components:

1. **PS/2 Keyboard Driver** - Hardware keyboard input with scancode translation
2. **Framebuffer Console** - Text output on screen with font rendering and scrolling
3. **Console Abstraction** - Unified interface for serial and screen output
4. **Logging Framework** - Structured logging with multiple log levels

## PS/2 Keyboard Driver

**Location**: `kernel/crates/fanga-arch-x86_64/src/keyboard.rs`

The PS/2 keyboard driver provides scancode translation for US keyboard layout (scancode set 1).

### Features

- **Scancode translation**: Converts PS/2 scancodes to ASCII characters
- **Modifier keys**: Tracks Shift, Ctrl, Alt, and Caps Lock states
- **Key events**: Distinguishes between key press and release events
- **Full ASCII support**: Handles letters, numbers, symbols, and control keys

### API

```rust
use fanga_arch_x86_64::keyboard;

// Get global keyboard state
let kbd = keyboard::keyboard();

// Read scancode from hardware
let scancode = kbd.read_scancode();

// Process scancode into key event
if let Some(event) = kbd.process_scancode(scancode) {
    match event {
        KeyEvent::Press(keycode) => {
            // Handle key press
            if let Some(ascii) = kbd.to_ascii(keycode) {
                // ASCII character available
            }
        }
        KeyEvent::Release(keycode) => {
            // Handle key release
        }
    }
}
```

### Supported Keys

- **Alphanumeric**: a-z, A-Z, 0-9
- **Symbols**: All standard US keyboard symbols
- **Modifiers**: Shift (left/right), Ctrl (left/right), Alt (left/right), Caps Lock
- **Control**: Enter, Backspace, Tab, Escape
- **Function**: F1-F12

### Keyboard State

The keyboard driver maintains state for:
- Shift pressed (left or right)
- Ctrl pressed (left or right)
- Alt pressed (left or right)
- Caps Lock toggle

## Framebuffer Console

**Location**: `kernel/crates/fanga-kernel/src/io/framebuffer.rs`

The framebuffer console provides text rendering on the screen using a bitmap font.

### Features

- **Font rendering**: 8x16 bitmap font for all printable ASCII characters
- **Automatic scrolling**: Screen scrolls up when reaching bottom
- **Color support**: Configurable foreground and background colors (ARGB format)
- **Cursor tracking**: Maintains current row and column position
- **Special characters**: Handles newline, carriage return, tab, and backspace

### API

```rust
use fanga_kernel::io::framebuffer;

// Initialize framebuffer (called once at boot)
framebuffer::init(addr, width, height, pitch, bpp);

// Print to framebuffer
fb_println!("Hello, FangaOS!");
fb_print!("Value: {}", 42);

// Direct access
let mut fb = framebuffer::framebuffer();
fb.set_fg_color(0xFFFF0000); // Red text
fb.write_string("Red text");
fb.clear(); // Clear screen
```

### Font

The embedded 8x16 bitmap font covers ASCII characters 32-126 (space through tilde). Each character is 8 pixels wide and 16 pixels tall, providing good readability on modern displays.

### Colors

Colors are specified in ARGB format (32-bit):
- `0xAARRGGBB` where AA=alpha, RR=red, GG=green, BB=blue
- Examples:
  - White: `0xFFFFFFFF`
  - Black: `0xFF000000`
  - Red: `0xFFFF0000`
  - Green: `0xFF00FF00`
  - Blue: `0xFF0000FF`

## Console Abstraction

**Location**: `kernel/crates/fanga-kernel/src/io/console.rs`

The console abstraction provides a unified interface that writes to both serial port and framebuffer simultaneously.

### Features

- **Multi-output**: Write to serial and framebuffer at once
- **Configurable**: Can enable/disable each output independently
- **Format support**: Full Rust formatting support via `core::fmt`

### API

```rust
use fanga_kernel::console_println;

// Print to both serial and framebuffer
console_println!("Hello, world!");
console_print!("Value: {}", 42);

// Configure outputs
let mut console = fanga_kernel::io::console::console();
console.set_serial_enabled(true);
console.set_framebuffer_enabled(true);
```

## Logging Framework

**Location**: `kernel/crates/fanga-kernel/src/io/logger.rs`

The logging framework provides structured logging with multiple log levels and color-coded output.

### Features

- **Log levels**: DEBUG, INFO, WARN, ERROR
- **Level filtering**: Set minimum log level to display
- **Color-coded**: Different colors for each log level (on framebuffer)
- **Structured output**: Consistent format with level prefix

### API

```rust
use fanga_kernel::{log_debug, log_info, log_warn, log_error};

// Log at different levels
log_debug!("Debug information: {}", value);
log_info!("System initialized");
log_warn!("Resource usage high: {}%", usage);
log_error!("Critical error: {}", error);

// Set log level filter
use fanga_kernel::io::logger::{set_log_level, LogLevel};
set_log_level(LogLevel::Info); // Only show INFO and above
```

### Log Levels

| Level | Priority | Color | Use Case |
|-------|----------|-------|----------|
| DEBUG | 0 | Gray | Detailed debugging information |
| INFO | 1 | White | General informational messages |
| WARN | 2 | Orange | Warning conditions |
| ERROR | 3 | Red | Error conditions |

### Log Format

Log messages are formatted as:
```
[LEVEL] message
```

For example:
```
[INFO] Heap allocator initialized
[WARN] Memory usage at 80%
[ERROR] Failed to allocate memory
```

## Interrupt Integration

The keyboard driver is integrated with the interrupt system:

**Location**: `kernel/crates/fanga-arch-x86_64/src/interrupts/idt.rs`

- **IRQ 1**: Keyboard interrupt handler
- **Scancode processing**: Reads from port 0x60
- **Event generation**: Converts scancodes to key events
- **Console output**: Logs keyboard events

## Usage Examples

### Basic Console Output

```rust
// Print to console (serial + framebuffer)
console_println!("FangaOS initialized");

// Use logging framework
log_info!("System started successfully");
log_warn!("Battery low: {}%", 15);
```

### Framebuffer Only

```rust
// Print only to framebuffer
fb_println!("This appears on screen only");
```

### Handling Keyboard Input

```rust
// In keyboard IRQ handler
let kbd = keyboard::keyboard();
let scancode = kbd.read_scancode();

if let Some(KeyEvent::Press(keycode)) = kbd.process_scancode(scancode) {
    if let Some(ch) = kbd.to_ascii(keycode) {
        console_print!("{}", ch); // Echo to console
    }
}
```

## Testing

The I/O system can be tested by running the kernel in QEMU:

```bash
make run
```

Expected output:
1. Framebuffer console initialized
2. Banner displayed on screen and serial
3. Log messages with different levels (color-coded on screen)
4. Keyboard input is logged to serial output

### Manual Testing

1. **Console output**: Check that text appears on both serial and framebuffer
2. **Scrolling**: Print many lines and verify screen scrolls correctly
3. **Colors**: Check that log levels have different colors on framebuffer
4. **Keyboard**: Type keys and verify scancodes are detected

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Print character to FB | O(1) | Direct pixel writing |
| Scroll screen | O(n) | n = screen pixels, optimized copy |
| Process scancode | O(1) | Lookup table translation |
| Console print | O(n) | n = string length, dual output |

## Memory Usage

- **Font data**: ~12 KB (95 characters × 16 bytes × 8 pixels)
- **Keyboard state**: < 100 bytes
- **Framebuffer**: Provided by bootloader (typically 1-8 MB)
- **Console state**: < 100 bytes

## Future Improvements

### Keyboard Driver
- [ ] Support for extended scancodes (E0 prefix)
- [ ] Multiple keyboard layouts (not just US)
- [ ] Keyboard repeat rate configuration
- [ ] NumLock, ScrollLock support
- [ ] USB keyboard support

### Framebuffer Console
- [ ] Multiple font sizes
- [ ] Unicode support (UTF-8)
- [ ] Hardware cursor
- [ ] Virtual terminals (multiple consoles)
- [ ] Double buffering for flicker-free updates

### Console Abstraction
- [ ] Console input buffering
- [ ] Line editing (backspace, cursor movement)
- [ ] Command history
- [ ] Tab completion

### Logging Framework
- [ ] Log to file/buffer
- [ ] Timestamp support
- [ ] Module-level filtering
- [ ] Performance profiling integration

## Integration with Boot Process

The I/O system is initialized early in the boot process:

1. **Serial port** (in `arch::init()`)
2. **Framebuffer** (after Limine provides framebuffer info)
3. **Interrupts** (IDT setup, keyboard IRQ enabled)
4. **Console** (automatically available after framebuffer init)
5. **Logging** (default level: DEBUG)

## References

- [OSDev Wiki - PS/2 Keyboard](https://wiki.osdev.org/PS/2_Keyboard)
- [OSDev Wiki - Text Mode Cursor](https://wiki.osdev.org/Text_Mode_Cursor)
- [PC Screen Font Specification](https://www.win.tue.nl/~aeb/linux/kbd/font-formats-1.html)
- [x86 I/O Ports](https://wiki.osdev.org/I/O_Ports)
