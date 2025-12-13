# System Calls

This document describes the system call implementation in FangaOS.

## Overview

FangaOS implements system calls using the x86_64 SYSCALL/SYSRET instruction pair, which provides a fast mechanism for transitioning between user mode and kernel mode.

## Architecture

### Low-Level (fanga-arch-x86_64)

The architecture-specific crate (`fanga-arch-x86_64`) handles the low-level details:

- **MSR Configuration**: Sets up the Model Specific Registers (IA32_STAR, IA32_LSTAR, IA32_FMASK) needed for SYSCALL/SYSRET
- **Entry Point**: Provides the `syscall_entry` naked function that the CPU jumps to on SYSCALL
- **Register Management**: Saves/restores registers according to the System V ABI
- **Syscall Handler**: Dispatches to the appropriate syscall implementation

### High-Level (fanga-kernel)

The kernel crate provides a higher-level interface:

- **Constants**: Re-exports syscall numbers and error codes
- **Result Type**: `SyscallResult` for Rust-style error handling
- **Dispatcher Trait**: Extensible interface for syscall dispatching

## Calling Convention

System calls follow the Linux x86_64 calling convention:

| Register | Purpose |
|----------|---------|
| rax      | Syscall number (input), return value (output) |
| rdi      | Argument 1 |
| rsi      | Argument 2 |
| rdx      | Argument 3 |
| r10      | Argument 4 (not rcx, as SYSCALL uses it) |
| r8       | Argument 5 |
| r9       | Argument 6 |

Return values:
- **Success**: Non-negative value in rax
- **Error**: Negative error code in rax

## Implemented System Calls

### SYS_READ (0)
Read data from a file descriptor.

**Arguments:**
- fd (i32): File descriptor to read from
- buf (*mut u8): Buffer to read into
- count (usize): Number of bytes to read

**Returns:** Number of bytes read on success, error code on failure

**Status:** Currently only supports stdin (fd=0), returns EOF (0)

### SYS_WRITE (1)
Write data to a file descriptor.

**Arguments:**
- fd (i32): File descriptor to write to
- buf (*const u8): Buffer containing data to write
- count (usize): Number of bytes to write

**Returns:** Number of bytes written on success, error code on failure

**Status:** Supports stdout (fd=1) and stderr (fd=2), writes to serial port

### SYS_EXIT (60)
Terminate the current process.

**Arguments:**
- status (i32): Exit status code

**Returns:** Does not return (enters infinite halt loop)

**Status:** Halts the system (will integrate with process management when available)

### SYS_FORK (57)
Create a child process (copy of current process).

**Arguments:** None

**Returns:** 0 in child process, child PID in parent process, or error code

**Status:** Not yet implemented (returns ENOSYS)

### SYS_EXEC (59)
Replace current process with a new program.

**Arguments:**
- path (*const u8): Path to executable
- argv (*const *const u8): Array of argument strings

**Returns:** Does not return on success, error code on failure

**Status:** Not yet implemented (returns ENOSYS)

## Error Codes

Following POSIX conventions, all error codes are negative:

| Code | Value | Description |
|------|-------|-------------|
| EINVAL | -22 | Invalid argument |
| EBADF | -9 | Bad file descriptor |
| ENOMEM | -12 | Out of memory |
| ENOSYS | -38 | Function not implemented |
| EFAULT | -14 | Bad address (invalid pointer) |
| EACCES | -13 | Permission denied |

## Argument Validation

All system calls perform argument validation:

1. **Pointer Validation**: Check for null pointers
2. **Bounds Checking**: Verify buffer sizes are reasonable
3. **Descriptor Validation**: Check file descriptors are valid
4. **Permission Checks**: (Future) Verify process has required permissions

## Security Considerations

### Current Implementation

The current implementation assumes kernel-mode operation and does not yet implement:
- User/kernel space separation
- Memory access validation (checking if user pointers point to valid user memory)
- Capability-based security
- Process isolation

### Future Enhancements

When user mode support is added, the following security features will be required:

1. **Address Space Validation**: Verify all user-provided pointers are in user space
2. **Capability Checks**: Ensure process has permission for the requested operation
3. **Resource Limits**: Enforce limits on memory, CPU time, etc.
4. **Secure Stack Switching**: Use per-process kernel stacks to prevent information leakage

## Testing

The syscall implementation includes:

### Unit Tests (fanga-arch-x86_64/src/syscall.rs)
- Syscall number constants
- Error code values
- Argument validation (null pointers, invalid fds)

### Unit Tests (fanga-kernel/src/syscall.rs)
- Syscall result conversion
- Error code propagation
- Constant definitions

### Integration Tests (fanga-kernel/tests/syscall_integration.rs)
- Syscall constant uniqueness
- Error code uniqueness
- Result type behavior
- Boundary conditions

## Usage Example

```rust
// From user space (when implemented):
// rax = syscall number
// rdi = fd, rsi = buffer, rdx = count
// syscall
// result in rax

// From kernel (current):
use fanga_kernel::syscall::*;

// Check return value
let ret = syscall_result(some_syscall_return);
match ret {
    Ok(n) => println!("Success: {} bytes", n),
    Err(EBADF) => println!("Bad file descriptor"),
    Err(e) => println!("Error: {}", e),
}
```

## Future Work

1. **Process Management Integration**
   - Implement SYS_FORK
   - Implement SYS_EXEC
   - Integrate with task scheduler

2. **File System Support**
   - Implement actual file I/O
   - Support for open, close, seek
   - Directory operations

3. **Memory Management**
   - SYS_MMAP for memory mapping
   - SYS_MUNMAP for unmapping
   - SYS_BRK for heap management

4. **Networking**
   - Socket system calls
   - Network I/O

5. **User Mode Support**
   - Transition to user mode
   - User space validation
   - Proper privilege separation

## References

- [Linux System Call Table](https://filippo.io/linux-syscall-table/)
- [System V ABI x86_64](https://gitlab.com/x86-psABIs/x86-64-ABI)
- [Intel Software Developer Manual, Volume 2B](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html) (SYSCALL/SYSRET instructions)
