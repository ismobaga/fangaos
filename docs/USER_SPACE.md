# User Space Support

## Overview

FangaOS now supports user-mode applications with proper privilege separation between kernel and user space. This document describes the user space implementation and how to develop user applications.

## Architecture

### Privilege Separation

The operating system uses x86_64 privilege levels to separate kernel and user code:

- **Ring 0 (Kernel Mode)**: The kernel runs with full privileges (DPL=0)
- **Ring 3 (User Mode)**: User applications run with limited privileges (DPL=3)

The Global Descriptor Table (GDT) is configured with separate code and data segments for kernel and user space:

```
Segment      | Selector | DPL | Type
-------------|----------|-----|------
Kernel Code  | 0x08     | 0   | Code
Kernel Data  | 0x10     | 0   | Data
User Code    | 0x18     | 3   | Code
User Data    | 0x20     | 3   | Data
```

### Memory Layout

User applications are loaded at standard x86_64 addresses:

- **Code/Data**: Starts at `0x400000` (4MB)
- **Stack**: Grows down from `0x7fffffffffff` (high memory)

## Components

### ELF Loader (`kernel/crates/fanga-kernel/src/elf/`)

The ELF loader parses and loads ELF64 binaries:

- **`parser.rs`**: Parses ELF headers and program headers
- **`loader.rs`**: Loads PT_LOAD segments into memory

Supported features:
- ELF64 format validation
- PT_LOAD segment loading
- PT_INTERP (interpreter path) detection
- Entry point extraction

### User Space Loader (`kernel/crates/fanga-kernel/src/userspace/`)

Prepares and transitions to user mode:

- **`loader.rs`**: Loads user binaries and prepares execution context
- **`transition.rs`**: Handles privilege level transitions using IRET

### System Calls

User applications interact with the kernel through system calls:

```rust
// Available syscalls
SYS_READ    = 0   // Read from file descriptor
SYS_WRITE   = 1   // Write to file descriptor
SYS_OPEN    = 2   // Open file
SYS_CLOSE   = 3   // Close file descriptor
SYS_EXIT    = 60  // Terminate process
```

## Developing User Applications

### Minimal User Application

A minimal user application requires:

1. **No standard library** (`#![no_std]`)
2. **Custom entry point** (`_start` function)
3. **Syscall interface** (minimal libc)
4. **Panic handler**

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
    println("Hello from user space!");
    0
}

#[no_mangle]
#[link_section = ".text.start"]
pub extern "C" fn _start() -> ! {
    let exit_code = main();
    exit(exit_code);
}
```

### Minimal LibC

The minimal libc provides syscall wrappers:

```rust
// Write to stdout
pub fn write(fd: i32, buf: &[u8]) -> i64;

// Read from stdin
pub fn read(fd: i32, buf: &mut [u8]) -> i64;

// Open file
pub fn open(path: &str, flags: i32, mode: i32) -> i64;

// Close file descriptor
pub fn close(fd: i32) -> i64;

// Exit program
pub fn exit(code: i32) -> !;

// Print string
pub fn print(s: &str);
pub fn println(s: &str);
```

### Building User Applications

User applications are compiled as static ELF binaries:

```bash
rustc \
    --target x86_64-unknown-none \
    --crate-type bin \
    --edition 2021 \
    -C panic=abort \
    -C relocation-model=static \
    -C code-model=small \
    -C link-arg=-T./user.ld \
    -C link-arg=--no-dynamic-linker \
    -C link-arg=-nostdlib \
    -O \
    -o hello \
    hello.rs
```

Or use the provided build script:

```bash
cd userspace
./build.sh
```

### Linker Script

User applications use a custom linker script (`user.ld`) that:

- Sets the entry point to `_start`
- Places code at `0x400000`
- Aligns sections to page boundaries (4KB)
- Discards unnecessary sections

## Loading and Executing User Programs

### From Kernel Code

```rust
use fanga_kernel::syscall_handlers::handle_exec;

// Load the ELF binary data
let binary_data: &[u8] = include_bytes!("../userspace/build/hello");

// Execute the program
unsafe {
    handle_exec(binary_data, 0, &[]).unwrap();
}
// Does not return - program is now running in user mode
```

### Via exec() System Call

```rust
// From another user program (when filesystem is available)
let path = "/bin/hello";
let argv = [path.as_ptr(), core::ptr::null()];
syscall2(SYS_EXEC, path.as_ptr() as u64, argv.as_ptr() as u64);
```

## Technical Details

### Mode Transition

Transitioning from kernel mode to user mode uses the IRET instruction:

1. Set up user mode stack frame:
   - SS (Stack Segment, DPL=3)
   - RSP (User Stack Pointer)
   - RFLAGS (with IF set)
   - CS (Code Segment, DPL=3)
   - RIP (Entry Point)

2. Execute `IRETQ` instruction

3. CPU switches to user mode and jumps to entry point

### System Call Interface

User mode makes syscalls using the `SYSCALL` instruction:

1. User program loads syscall number in RAX and arguments in registers
2. Executes `SYSCALL` instruction
3. CPU switches to kernel mode via MSR configuration
4. Kernel handler processes the syscall
5. Kernel returns to user mode via `SYSRET`

Register usage (System V ABI):
- `RAX`: Syscall number / Return value
- `RDI`: Argument 1
- `RSI`: Argument 2
- `RDX`: Argument 3
- `R10`: Argument 4 (not RCX, which is used by SYSCALL)
- `R8`: Argument 5
- `R9`: Argument 6

## Testing

Test the ELF loader and user space support:

```bash
cd kernel/crates/fanga-kernel
cargo test --target x86_64-unknown-linux-gnu elf::
cargo test --target x86_64-unknown-linux-gnu userspace::
```

## Current Limitations

1. **Memory Management**: User programs share the kernel page table. Per-process page tables not yet implemented.
2. **Process Isolation**: No memory protection between processes.
3. **Dynamic Linking**: Only static linking is supported.
4. **Filesystem Integration**: exec() loads from memory, not filesystem.
5. **Standard Library**: Minimal libc only. No full standard library port.

## Future Enhancements

1. Per-process page tables with copy-on-write fork()
2. Dynamic linker for shared libraries
3. Full POSIX-compatible libc
4. Process signal handling
5. Memory-mapped files
6. Thread support
7. Improved security and sandboxing

## References

- [Intel® 64 and IA-32 Architectures Software Developer's Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
- [System V ABI](https://refspecs.linuxfoundation.org/elf/x86_64-abi-0.99.pdf)
- [ELF Specification](https://refspecs.linuxbase.org/elf/elf.pdf)
- [OSDev Wiki - User Mode](https://wiki.osdev.org/Getting_to_Ring_3)
