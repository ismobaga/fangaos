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
pub const SYS_OPEN: u64 = 2;
pub const SYS_CLOSE: u64 = 3;
pub const SYS_LSEEK: u64 = 8;
pub const SYS_EXIT: u64 = 60;
pub const SYS_FORK: u64 = 57;
pub const SYS_EXEC: u64 = 59;

// Directory syscalls
pub const SYS_MKDIR: u64 = 83;
pub const SYS_RMDIR: u64 = 84;
pub const SYS_GETDENTS: u64 = 78;
pub const SYS_UNLINK: u64 = 87;

// IPC syscalls
pub const SYS_PIPE: u64 = 22;
pub const SYS_KILL: u64 = 62;
pub const SYS_SHMGET: u64 = 29;
pub const SYS_SHMAT: u64 = 30;
pub const SYS_SHMDT: u64 = 67;
pub const SYS_SHMCTL: u64 = 31;
pub const SYS_MSGGET: u64 = 68;
pub const SYS_MSGSND: u64 = 69;
pub const SYS_MSGRCV: u64 = 70;

// Memory management syscalls
pub const SYS_MMAP: u64 = 9;
pub const SYS_MUNMAP: u64 = 11;

/// Error codes (following POSIX conventions)
pub const EINVAL: i64 = -22;  // Invalid argument
pub const EBADF: i64 = -9;    // Bad file descriptor
pub const ENOMEM: i64 = -12;  // Out of memory
pub const ENOSYS: i64 = -38;  // Function not implemented
pub const EFAULT: i64 = -14;  // Bad address
pub const EACCES: i64 = -13;  // Permission denied
pub const EPERM: i64 = -1;    // Operation not permitted
pub const ESRCH: i64 = -3;    // No such process
pub const ENOENT: i64 = -2;   // No such file or directory
pub const EEXIST: i64 = -17;  // File exists
pub const ENOTDIR: i64 = -20; // Not a directory
pub const EISDIR: i64 = -21;  // Is a directory
pub const ENOTEMPTY: i64 = -39; // Directory not empty

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
        SYS_OPEN => sys_open(arg1 as *const u8, arg2 as i32, arg3 as i32),
        SYS_CLOSE => sys_close(arg1 as i32),
        SYS_LSEEK => sys_lseek(arg1 as i32, arg2 as i64, arg3 as i32),
        SYS_MMAP => sys_mmap(arg1 as u64, arg2 as usize, arg3 as i32, arg4 as i32, arg5 as i32, arg6 as i64),
        SYS_MUNMAP => sys_munmap(arg1 as u64, arg2 as usize),
        SYS_EXIT => sys_exit(arg1 as i32),
        SYS_FORK => sys_fork(),
        SYS_EXEC => sys_exec(arg1 as *const u8, arg2 as *const *const u8),
        SYS_MKDIR => sys_mkdir(arg1 as *const u8, arg2 as i32),
        SYS_RMDIR => sys_rmdir(arg1 as *const u8),
        SYS_GETDENTS => sys_getdents(arg1 as i32, arg2 as *mut u8, arg3 as usize),
        SYS_UNLINK => sys_unlink(arg1 as *const u8),
        SYS_PIPE => sys_pipe(arg1 as *mut i32),
        SYS_KILL => sys_kill(arg1 as i32, arg2 as i32),
        SYS_SHMGET => sys_shmget(arg1 as i32, arg2 as usize, arg3 as i32),
        SYS_SHMAT => sys_shmat(arg1 as i32, arg2 as *const u8, arg3 as i32),
        SYS_SHMDT => sys_shmdt(arg1 as *const u8),
        SYS_SHMCTL => sys_shmctl(arg1 as i32, arg2 as i32, arg3 as *mut u8),
        SYS_MSGGET => sys_msgget(arg1 as i32, arg2 as i32),
        SYS_MSGSND => sys_msgsnd(arg1 as i32, arg2 as *const u8, arg3 as usize, arg4 as i32),
        SYS_MSGRCV => sys_msgrcv(arg1 as i32, arg2 as *mut u8, arg3 as usize, arg4 as i64, arg5 as i32),
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

/// sys_open - Open a file
fn sys_open(pathname: *const u8, _flags: i32, _mode: i32) -> i64 {
    if pathname.is_null() {
        return EFAULT;
    }
    crate::serial_println!("[SYSCALL] sys_open() - stub");
    // TODO: Implement with VFS
    ENOSYS
}

/// sys_close - Close a file descriptor
fn sys_close(fd: i32) -> i64 {
    if fd < 0 {
        return EBADF;
    }
    crate::serial_println!("[SYSCALL] sys_close() - stub");
    // TODO: Implement with VFS
    ENOSYS
}

/// sys_lseek - Seek in a file
fn sys_lseek(fd: i32, _offset: i64, whence: i32) -> i64 {
    if fd < 0 {
        return EBADF;
    }
    if whence < 0 || whence > 2 {
        return EINVAL;
    }
    crate::serial_println!("[SYSCALL] sys_lseek() - stub");
    // TODO: Implement with VFS
    ENOSYS
}

/// sys_mkdir - Create a directory
fn sys_mkdir(pathname: *const u8, _mode: i32) -> i64 {
    if pathname.is_null() {
        return EFAULT;
    }
    crate::serial_println!("[SYSCALL] sys_mkdir() - stub");
    // TODO: Implement with VFS
    ENOSYS
}

/// sys_rmdir - Remove a directory
fn sys_rmdir(pathname: *const u8) -> i64 {
    if pathname.is_null() {
        return EFAULT;
    }
    crate::serial_println!("[SYSCALL] sys_rmdir() - stub");
    // TODO: Implement with VFS
    ENOSYS
}

/// sys_getdents - Get directory entries
fn sys_getdents(fd: i32, dirp: *mut u8, _count: usize) -> i64 {
    if fd < 0 {
        return EBADF;
    }
    if dirp.is_null() {
        return EFAULT;
    }
    crate::serial_println!("[SYSCALL] sys_getdents() - stub");
    // TODO: Implement with VFS
    ENOSYS
}

/// sys_unlink - Remove a file
fn sys_unlink(pathname: *const u8) -> i64 {
    if pathname.is_null() {
        return EFAULT;
    }
    crate::serial_println!("[SYSCALL] sys_unlink() - stub");
    // TODO: Implement with VFS
    ENOSYS
}

/// sys_mmap - Map memory region
/// 
/// # Arguments
/// * `addr` - Requested address (or 0 for automatic)
/// * `length` - Size of mapping in bytes
/// * `prot` - Protection flags (PROT_READ, PROT_WRITE, PROT_EXEC)
/// * `flags` - Mapping flags (MAP_SHARED, MAP_PRIVATE, MAP_ANONYMOUS, MAP_FIXED)
/// * `fd` - File descriptor (ignored for MAP_ANONYMOUS)
/// * `offset` - File offset (ignored for MAP_ANONYMOUS)
///
/// # Returns
/// Virtual address of mapping on success, or negative error code
fn sys_mmap(addr: u64, length: usize, prot: i32, flags: i32, _fd: i32, _offset: i64) -> i64 {
    #[cfg(not(test))]
    crate::serial_println!(
        "[SYSCALL] sys_mmap(addr={:#x}, len={}, prot={:#x}, flags={:#x})",
        addr, length, prot, flags
    );

    // Validate arguments
    if length == 0 {
        return EINVAL;
    }

    // For now, only support anonymous mappings
    const MAP_ANONYMOUS: i32 = 0x20;
    if (flags & MAP_ANONYMOUS) == 0 {
        return ENOSYS; // File-backed mappings not yet implemented
    }

    // TODO: Implement actual mmap functionality
    // This would involve:
    // 1. Allocating physical pages
    // 2. Setting up page table mappings
    // 3. Recording the mapping in the process's mmap manager
    // 4. Handling MAP_FIXED, MAP_SHARED, etc.
    
    // For now, return a dummy address in user space
    // In a real implementation, this would be a proper allocated region
    if addr == 0 {
        // Automatic placement - return an address in user space
        0x4000_0000 // Dummy address
    } else {
        // Fixed placement
        addr as i64
    }
}

/// sys_munmap - Unmap memory region
///
/// # Arguments
/// * `addr` - Virtual address of mapping to unmap
/// * `length` - Size of region to unmap
///
/// # Returns
/// 0 on success, or negative error code
fn sys_munmap(addr: u64, length: usize) -> i64 {
    #[cfg(not(test))]
    crate::serial_println!(
        "[SYSCALL] sys_munmap(addr={:#x}, len={})",
        addr, length
    );

    // Validate arguments
    if length == 0 {
        return EINVAL;
    }

    // Check alignment (use constant from memory module when available)
    const PAGE_SIZE: u64 = 4096;
    if addr % PAGE_SIZE != 0 {
        return EINVAL;
    }

    // TODO: Implement actual munmap functionality
    // This would involve:
    // 1. Finding the mapping in the process's mmap manager
    // 2. Unmapping the pages from the page table
    // 3. Freeing the physical pages (if not shared)
    // 4. Removing the mapping record

    // For now, just return success
    0
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

/// sys_pipe - Create a pipe
fn sys_pipe(pipefd: *mut i32) -> i64 {
    // Validate arguments
    if pipefd.is_null() {
        return EFAULT;
    }
    
    crate::serial_println!("[SYSCALL] sys_pipe() - basic implementation");
    
    // TODO: Implement full pipe when we have file descriptor table:
    // - Create a new Pipe object
    // - Allocate two file descriptors
    // - pipefd[0] = read end, pipefd[1] = write end
    // - Return 0 on success
    
    // Dummy file descriptors (placeholder until full implementation)
    const DUMMY_READ_FD: i32 = 3;
    const DUMMY_WRITE_FD: i32 = 4;
    
    // For now, just return success with dummy file descriptors
    unsafe {
        *pipefd.offset(0) = DUMMY_READ_FD;
        *pipefd.offset(1) = DUMMY_WRITE_FD;
    }
    
    0
}

/// sys_kill - Send a signal to a process
fn sys_kill(pid: i32, sig: i32) -> i64 {
    crate::serial_println!("[SYSCALL] sys_kill(pid={}, sig={})", pid, sig);
    
    if pid <= 0 {
        return EINVAL;
    }
    
    if sig < 0 || sig > 31 {
        return EINVAL;
    }
    
    // TODO: Implement signal sending when we have full process management:
    // - Find the process by PID
    // - Check permissions
    // - Send the signal to the process's signal handler
    // - Wake up the process if necessary
    
    // For now, return ESRCH (no such process)
    ESRCH
}

/// sys_shmget - Get shared memory segment
fn sys_shmget(key: i32, size: usize, shmflg: i32) -> i64 {
    crate::serial_println!("[SYSCALL] sys_shmget(key={}, size={}, flags={})", key, size, shmflg);
    
    if size == 0 {
        return EINVAL;
    }
    
    // TODO: Implement shared memory when we have full IPC system:
    // - Allocate physical memory for the segment
    // - Register the segment in a global shared memory table
    // - Return a shared memory ID
    
    // For now, return a dummy ID
    1
}

/// sys_shmat - Attach shared memory segment
fn sys_shmat(shmid: i32, shmaddr: *const u8, shmflg: i32) -> i64 {
    crate::serial_println!("[SYSCALL] sys_shmat(id={}, addr={:p}, flags={})", shmid, shmaddr, shmflg);
    
    if shmid < 0 {
        return EINVAL;
    }
    
    // TODO: Implement shared memory attachment:
    // - Find the shared memory segment by ID
    // - Map it into the current process's address space
    // - Return the virtual address where it's mapped
    
    // For now, return EINVAL
    EINVAL
}

/// sys_shmdt - Detach shared memory segment
fn sys_shmdt(shmaddr: *const u8) -> i64 {
    crate::serial_println!("[SYSCALL] sys_shmdt(addr={:p})", shmaddr);
    
    if shmaddr.is_null() {
        return EINVAL;
    }
    
    // TODO: Implement shared memory detachment:
    // - Find the shared memory segment by address
    // - Unmap it from the current process's address space
    // - Decrement reference count
    
    // For now, return success
    0
}

/// sys_shmctl - Control shared memory segment
fn sys_shmctl(shmid: i32, cmd: i32, _buf: *mut u8) -> i64 {
    crate::serial_println!("[SYSCALL] sys_shmctl(id={}, cmd={})", shmid, cmd);
    
    if shmid < 0 {
        return EINVAL;
    }
    
    // TODO: Implement shared memory control operations:
    // - IPC_STAT: Get segment info
    // - IPC_SET: Set segment info
    // - IPC_RMID: Remove segment
    
    // For now, return ENOSYS
    ENOSYS
}

/// sys_msgget - Get message queue
fn sys_msgget(key: i32, msgflg: i32) -> i64 {
    crate::serial_println!("[SYSCALL] sys_msgget(key={}, flags={})", key, msgflg);
    
    // TODO: Implement message queue creation:
    // - Create or get existing message queue by key
    // - Return message queue ID
    
    // For now, return a dummy ID
    1
}

/// sys_msgsnd - Send message to queue
fn sys_msgsnd(msqid: i32, msgp: *const u8, msgsz: usize, msgflg: i32) -> i64 {
    crate::serial_println!("[SYSCALL] sys_msgsnd(id={}, size={}, flags={})", msqid, msgsz, msgflg);
    
    if msgp.is_null() || msqid < 0 {
        return EINVAL;
    }
    
    // TODO: Implement message sending:
    // - Find the message queue by ID
    // - Copy message from user space
    // - Add to queue
    // - Wake up waiting receivers
    
    // For now, return success
    0
}

/// sys_msgrcv - Receive message from queue
fn sys_msgrcv(msqid: i32, msgp: *mut u8, msgsz: usize, msgtyp: i64, msgflg: i32) -> i64 {
    crate::serial_println!("[SYSCALL] sys_msgrcv(id={}, size={}, type={}, flags={})", msqid, msgsz, msgtyp, msgflg);
    
    if msgp.is_null() || msqid < 0 {
        return EINVAL;
    }
    
    // TODO: Implement message receiving:
    // - Find the message queue by ID
    // - Wait for message matching type
    // - Copy message to user space
    // - Return message size
    
    // For now, return 0 (no message)
    0
}

/// The actual syscall entry point (naked function in assembly)
///
/// This is called directly by the CPU when a SYSCALL instruction is executed.
/// It must:
/// 1. Save all registers
/// 2. Call syscall_handler
/// 3. Restore registers
/// 4. Return via SYSRET
///
/// NOTE: This is a minimal implementation for kernel-mode syscall testing.
/// A full implementation would need to handle user/kernel stack switching.
#[unsafe(naked)]
#[no_mangle]
unsafe extern "C" fn syscall_entry() -> ! {
    core::arch::naked_asm!(
        // rcx = return RIP (saved by SYSCALL instruction)
        // r11 = RFLAGS (saved by SYSCALL instruction)
        // rax = syscall number
        // rdi, rsi, rdx, r10, r8, r9 = arguments
        
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
        // C calling convention: rdi, rsi, rdx, rcx, r8, r9, [stack]
        // syscall_handler(syscall_number, arg1, arg2, arg3, arg4, arg5, arg6)
        
        // Current state:
        // rax = syscall number
        // rdi = arg1
        // rsi = arg2
        // rdx = arg3
        // r10 = arg4 (not rcx, as SYSCALL uses it)
        // r8 = arg5
        // r9 = arg6
        
        // Save arguments that will be overwritten
        "push r9",                    // arg6
        "push r8",                    // arg5
        "push r10",                   // arg4
        "push rdx",                   // arg3
        "push rsi",                   // arg2
        "push rdi",                   // arg1
        "push rax",                   // syscall number
        
        // Now rearrange for C calling convention
        "pop rdi",                    // syscall_number
        "pop rsi",                    // arg1
        "pop rdx",                    // arg2
        "pop rcx",                    // arg3
        "pop r8",                     // arg4
        "pop r9",                     // arg5
        // arg6 is still on stack, which is correct for 7th C argument
        
        // Call the handler
        "call syscall_handler",
        
        // Return value is in rax - leave it there
        
        // Clean up arg6 from stack
        "add rsp, 8",
        
        // Restore user registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "pop r11",                    // RFLAGS
        "pop rcx",                    // Return RIP
        
        // Return to user space
        // NOTE: sysretq requires:
        // - rcx = return RIP
        // - r11 = RFLAGS
        // - rax = return value (already set)
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
        
        // IPC syscalls
        assert_eq!(SYS_PIPE, 22);
        assert_eq!(SYS_KILL, 62);
        assert_eq!(SYS_SHMGET, 29);
        assert_eq!(SYS_SHMAT, 30);
        assert_eq!(SYS_SHMDT, 67);
        assert_eq!(SYS_SHMCTL, 31);
        assert_eq!(SYS_MSGGET, 68);
        assert_eq!(SYS_MSGSND, 69);
        assert_eq!(SYS_MSGRCV, 70);
    }

    #[test]
    fn test_error_codes() {
        assert!(EINVAL < 0);
        assert!(EBADF < 0);
        assert!(ENOMEM < 0);
        assert!(ENOSYS < 0);
        assert!(EFAULT < 0);
        assert!(EPERM < 0);
        assert!(ESRCH < 0);
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
    
    #[test]
    fn test_sys_pipe_null_ptr() {
        let result = sys_pipe(core::ptr::null_mut());
        assert_eq!(result, EFAULT);
    }
    
    #[test]
    fn test_sys_pipe_basic() {
        let mut pipefd = [0i32; 2];
        let result = sys_pipe(pipefd.as_mut_ptr());
        assert_eq!(result, 0);
        assert!(pipefd[0] > 0);
        assert!(pipefd[1] > 0);
        assert_ne!(pipefd[0], pipefd[1]);
    }
    
    #[test]
    fn test_sys_kill_invalid_pid() {
        let result = sys_kill(-1, 9);
        assert_eq!(result, EINVAL);
    }
    
    #[test]
    fn test_sys_kill_invalid_signal() {
        let result = sys_kill(1, -1);
        assert_eq!(result, EINVAL);
    }
    
    #[test]
    fn test_sys_shmget_zero_size() {
        let result = sys_shmget(1, 0, 0);
        assert_eq!(result, EINVAL);
    }
    
    #[test]
    fn test_sys_shmget_basic() {
        let result = sys_shmget(1, 4096, 0);
        assert!(result > 0);
    }
    
    #[test]
    fn test_sys_mmap_anonymous() {
        const MAP_ANONYMOUS: i32 = 0x20;
        const PROT_READ: i32 = 0x1;
        const PROT_WRITE: i32 = 0x2;
        
        let result = sys_mmap(0, 4096, PROT_READ | PROT_WRITE, MAP_ANONYMOUS, -1, 0);
        assert!(result > 0); // Should return a valid address
    }
    
    #[test]
    fn test_sys_mmap_zero_length() {
        const MAP_ANONYMOUS: i32 = 0x20;
        const PROT_READ: i32 = 0x1;
        
        let result = sys_mmap(0, 0, PROT_READ, MAP_ANONYMOUS, -1, 0);
        assert_eq!(result, EINVAL);
    }
    
    #[test]
    fn test_sys_mmap_fixed_address() {
        const MAP_ANONYMOUS: i32 = 0x20;
        const MAP_FIXED: i32 = 0x10;
        const PROT_READ: i32 = 0x1;
        
        let addr = 0x4000_0000u64;
        let result = sys_mmap(addr, 4096, PROT_READ, MAP_ANONYMOUS | MAP_FIXED, -1, 0);
        assert_eq!(result, addr as i64);
    }
    
    #[test]
    fn test_sys_munmap_valid() {
        let addr = 0x4000_0000u64;
        let result = sys_munmap(addr, 4096);
        assert_eq!(result, 0);
    }
    
    #[test]
    fn test_sys_munmap_zero_length() {
        let result = sys_munmap(0x4000_0000, 0);
        assert_eq!(result, EINVAL);
    }
    
    #[test]
    fn test_sys_munmap_unaligned() {
        let result = sys_munmap(0x4000_0001, 4096);
        assert_eq!(result, EINVAL);
    }
}
