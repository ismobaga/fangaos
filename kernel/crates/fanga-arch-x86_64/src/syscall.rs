//! System Call Support for x86_64
//!
//! This module implements the low-level system call interface using the
//! SYSCALL/SYSRET instructions on x86_64. It handles MSR configuration,
//! the syscall entry point, and argument passing.

use core::arch::asm;
use crate::gdt::KERNEL_CODE_SELECTOR;

/// Model Specific Registers for SYSCALL/SYSRET
const IA32_STAR: u32 = 0xC0000081;
const IA32_LSTAR: u32 = 0xC0000082;
const IA32_FMASK: u32 = 0xC0000084;

/// RFLAGS bits to mask during syscall
const RFLAGS_IF: u64 = 1 << 9;  // Interrupt Flag
const RFLAGS_TF: u64 = 1 << 8;  // Trap Flag
const RFLAGS_DF: u64 = 1 << 10; // Direction Flag

/// Syscall numbers
pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT: u64 = 60;
pub const SYS_FORK: u64 = 57;
pub const SYS_EXEC: u64 = 59;

/// Error codes (following POSIX conventions)
pub const EINVAL: i64 = -22;  // Invalid argument
pub const EBADF: i64 = -9;    // Bad file descriptor
pub const ENOMEM: i64 = -12;  // Out of memory
pub const ENOSYS: i64 = -38;  // Function not implemented
pub const EFAULT: i64 = -14;  // Bad address
pub const EACCES: i64 = -13;  // Permission denied

/// Write a value to a Model Specific Register
#[inline]
unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nostack, preserves_flags)
    );
}

/// Read a value from a Model Specific Register
#[inline]
#[allow(dead_code)]
unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nostack, preserves_flags)
    );
    ((high as u64) << 32) | (low as u64)
}

/// System call handler - called from syscall entry
///
/// Arguments are passed in registers according to the System V ABI:
/// - syscall number: rax
/// - arg1: rdi
/// - arg2: rsi
/// - arg3: rdx
/// - arg4: r10 (not rcx, as rcx is used by syscall instruction)
/// - arg5: r8
/// - arg6: r9
///
/// Return value in rax
#[no_mangle]
extern "C" fn syscall_handler(
    syscall_number: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> i64 {
    crate::serial_println!(
        "[SYSCALL] num={} args=({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
        syscall_number, arg1, arg2, arg3, arg4, arg5, arg6
    );

    match syscall_number {
        SYS_READ => sys_read(arg1 as i32, arg2 as *mut u8, arg3 as usize),
        SYS_WRITE => sys_write(arg1 as i32, arg2 as *const u8, arg3 as usize),
        SYS_EXIT => sys_exit(arg1 as i32),
        SYS_FORK => sys_fork(),
        SYS_EXEC => sys_exec(arg1 as *const u8, arg2 as *const *const u8),
        _ => ENOSYS,
    }
}

/// sys_read - Read from a file descriptor
fn sys_read(fd: i32, buf: *mut u8, count: usize) -> i64 {
    // Validate arguments
    if buf.is_null() {
        return EFAULT;
    }
    
    // For now, only support stdin (fd=0)
    if fd != 0 {
        return EBADF;
    }

    // TODO: Implement actual reading from stdin
    crate::serial_println!("[SYSCALL] sys_read(fd={}, buf={:p}, count={})", fd, buf, count);
    
    // Return 0 for now (EOF)
    0
}

/// sys_write - Write to a file descriptor
fn sys_write(fd: i32, buf: *const u8, count: usize) -> i64 {
    // Validate arguments
    if buf.is_null() {
        return EFAULT;
    }

    // Only support stdout (1) and stderr (2) for now
    if fd != 1 && fd != 2 {
        return EBADF;
    }

    // Validate buffer is readable
    // In a real OS, we'd check if the buffer is in user space and accessible
    // For now, we'll just trust it (since we don't have user space yet)
    
    unsafe {
        let slice = core::slice::from_raw_parts(buf, count);
        if let Ok(s) = core::str::from_utf8(slice) {
            crate::serial_print!("{}", s);
        } else {
            // If not valid UTF-8, write as hex
            for &byte in slice {
                crate::serial_print!("{:02x}", byte);
            }
        }
    }

    count as i64
}

/// sys_exit - Terminate the current process
fn sys_exit(status: i32) -> i64 {
    crate::serial_println!("[SYSCALL] sys_exit(status={})", status);
    
    // TODO: When we have proper process management, this should:
    // - Mark the process as terminated
    // - Store the exit status
    // - Trigger scheduler to switch to another process
    // - Clean up process resources
    
    // For now, just halt (in a real OS, we'd never return from this)
    loop {
        unsafe {
            asm!("hlt", options(nostack, nomem));
        }
    }
}

/// sys_fork - Create a child process
fn sys_fork() -> i64 {
    crate::serial_println!("[SYSCALL] sys_fork() - not implemented yet");
    
    // TODO: Implement fork when we have proper process management:
    // - Duplicate the current process's address space
    // - Create new TCB (Task Control Block)
    // - Copy registers and stack
    // - Return 0 in child, child PID in parent
    
    ENOSYS
}

/// sys_exec - Execute a new program
fn sys_exec(_path: *const u8, _argv: *const *const u8) -> i64 {
    crate::serial_println!("[SYSCALL] sys_exec() - not implemented yet");
    
    // TODO: Implement exec when we have filesystem and ELF loader:
    // - Load the program from filesystem
    // - Parse ELF binary
    // - Replace current process's address space
    // - Set up new stack with arguments
    // - Jump to entry point
    
    ENOSYS
}

/// The actual syscall entry point (naked function in assembly)
///
/// This is called directly by the CPU when a SYSCALL instruction is executed.
/// It must:
/// 1. Save the user-space stack pointer
/// 2. Switch to kernel stack
/// 3. Save all registers
/// 4. Call syscall_handler
/// 5. Restore registers
/// 6. Switch back to user stack
/// 7. Return via SYSRET
#[unsafe(naked)]
#[no_mangle]
unsafe extern "C" fn syscall_entry() -> ! {
    core::arch::naked_asm!(
        // rcx = return RIP (saved by SYSCALL instruction)
        // r11 = RFLAGS (saved by SYSCALL instruction)
        // rax = syscall number
        // rdi, rsi, rdx, r10, r8, r9 = arguments
        
        // Save user stack pointer
        "mov gs:0x10, rsp",          // Save user RSP to TSS.rsp0 equivalent
        
        // Switch to kernel stack (for now, use current stack)
        // TODO: Load kernel stack from TSS when we have per-process kernel stacks
        
        // Save user registers on stack
        "push rcx",                   // Return RIP
        "push r11",                   // RFLAGS
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        
        // Prepare arguments for syscall_handler
        // syscall_handler(syscall_number, arg1, arg2, arg3, arg4, arg5, arg6)
        // Already in correct registers: rax=num, rdi=arg1, rsi=arg2, rdx=arg3
        // Move r10 to rcx (arg4), r8 and r9 are already correct
        "mov rcx, r10",               // arg4
        // r8 = arg5 (already correct)
        // r9 = arg6 (already correct)
        
        // rax already contains syscall number
        "mov rdi, rax",               // syscall_number
        "mov rsi, rdi",               // Save original rdi
        "mov rdx, rsi",               // Save original rsi
        "mov rcx, rdx",               // Save original rdx
        // We need to rearrange...
        
        // Let me redo this properly:
        // Current state:
        // rax = syscall number
        // rdi = arg1
        // rsi = arg2  
        // rdx = arg3
        // r10 = arg4
        // r8 = arg5
        // r9 = arg6
        
        // Need for C function:
        // rdi = syscall_number
        // rsi = arg1
        // rdx = arg2
        // rcx = arg3
        // r8 = arg4
        // r9 = arg5
        // stack = arg6
        
        // Save arguments
        "push r9",                    // arg6
        "mov r9, r8",                 // arg5
        "mov r8, r10",                // arg4
        "mov rcx, rdx",               // arg3
        "mov rdx, rsi",               // arg2
        "mov rsi, rdi",               // arg1
        "mov rdi, rax",               // syscall_number
        
        // Call the handler
        "call syscall_handler",
        
        // Return value is in rax
        
        // Restore arg6 from stack
        "pop r9",
        
        // Restore user registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "pop r11",                    // RFLAGS
        "pop rcx",                    // Return RIP
        
        // Restore user stack
        "mov rsp, gs:0x10",
        
        // Return to user space
        "sysretq",
    )
}

/// Initialize the syscall mechanism
///
/// This configures the MSRs needed for SYSCALL/SYSRET:
/// - IA32_STAR: Segment selectors for kernel and user code/data
/// - IA32_LSTAR: Address of syscall entry point
/// - IA32_FMASK: RFLAGS bits to mask during syscall
pub fn init() {
    unsafe {
        // IA32_STAR layout:
        // Bits 63:48 = User CS (SYSRET) and SS (+8) base selector
        // Bits 47:32 = Kernel CS (SYSCALL) and SS (+8) base selector  
        // Bits 31:0 = Reserved (must be 0)
        
        // For SYSCALL: CS = STAR[47:32], SS = STAR[47:32] + 8
        // For SYSRET: CS = STAR[63:48] + 16, SS = STAR[63:48] + 8
        
        // Kernel CS = 0x08, User CS base = 0x18 (will use 0x18+16=0x28 for CS)
        let star = ((0x18u64) << 48) | ((KERNEL_CODE_SELECTOR as u64) << 32);
        wrmsr(IA32_STAR, star);
        
        // Set the syscall entry point
        let lstar = syscall_entry as *const () as u64;
        wrmsr(IA32_LSTAR, lstar);
        
        // Mask interrupts (IF), trap flag (TF), and direction flag (DF) during syscall
        let fmask = RFLAGS_IF | RFLAGS_TF | RFLAGS_DF;
        wrmsr(IA32_FMASK, fmask);
        
        crate::serial_println!("[SYSCALL] initialized ✅");
        crate::serial_println!("  Entry point: 0x{:x}", lstar);
        crate::serial_println!("  STAR: 0x{:x}", star);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_numbers() {
        assert_eq!(SYS_READ, 0);
        assert_eq!(SYS_WRITE, 1);
        assert_eq!(SYS_EXIT, 60);
        assert_eq!(SYS_FORK, 57);
        assert_eq!(SYS_EXEC, 59);
    }

    #[test]
    fn test_error_codes() {
        assert!(EINVAL < 0);
        assert!(EBADF < 0);
        assert!(ENOMEM < 0);
        assert!(ENOSYS < 0);
        assert!(EFAULT < 0);
    }

    #[test]
    fn test_sys_write_null_buffer() {
        let result = sys_write(1, core::ptr::null(), 10);
        assert_eq!(result, EFAULT);
    }

    #[test]
    fn test_sys_write_invalid_fd() {
        let buf = b"test";
        let result = sys_write(99, buf.as_ptr(), buf.len());
        assert_eq!(result, EBADF);
    }

    #[test]
    fn test_sys_read_null_buffer() {
        let result = sys_read(0, core::ptr::null_mut(), 10);
        assert_eq!(result, EFAULT);
    }

    #[test]
    fn test_sys_read_invalid_fd() {
        let mut buf = [0u8; 10];
        let result = sys_read(99, buf.as_mut_ptr(), buf.len());
        assert_eq!(result, EBADF);
    }
}
