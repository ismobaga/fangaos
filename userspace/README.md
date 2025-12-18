# User Space Applications

This directory contains user-mode applications for FangaOS.

## Structure

- `libc.rs` - Minimal C library with syscall wrappers
- `hello.rs` - Simple "Hello World" user application
- `user.ld` - Linker script for user programs
- `build.sh` - Build script to compile user applications
- `build/` - Output directory for compiled binaries

## Building

To build all user applications:

```bash
./build.sh
```

This will create ELF64 executables in the `build/` directory.

## Requirements

- Rust nightly toolchain
- `x86_64-unknown-none` target (installed automatically)

## Creating New Applications

1. Create a new `.rs` file in this directory
2. Use `#![no_std]` and `#![no_main]`
3. Include the `libc` module for syscalls
4. Implement `_start` and `main` functions
5. Add panic handler
6. Update `build.sh` to compile the new application

Example:

```rust
#![no_std]
#![no_main]

mod libc;
use libc::{println, exit};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(1);
}

#[no_mangle]
pub extern "C" fn main() -> i32 {
    println("My application");
    0
}

#[no_mangle]
#[link_section = ".text.start"]
pub extern "C" fn _start() -> ! {
    let exit_code = main();
    exit(exit_code);
}
```

## Available System Calls

The minimal libc provides the following syscalls:

- `read(fd, buf)` - Read from file descriptor
- `write(fd, buf)` - Write to file descriptor
- `open(path, flags, mode)` - Open file
- `close(fd)` - Close file descriptor
- `exit(code)` - Exit program
- `print(s)` - Print string to stdout
- `println(s)` - Print string with newline

## Memory Layout

User programs are loaded at:

- Code: `0x400000` (4MB)
- Stack: Grows down from `0x7fffffffffff`

## See Also

- [User Space Documentation](../docs/USER_SPACE.md) - Detailed documentation
- [System Calls Documentation](../docs/SYSTEM_CALLS.md) - System call interface
