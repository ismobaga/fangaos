//! System Call Interface
//!
//! This module provides the kernel's system call interface, including
//! re-exports from the architecture-specific syscall implementation and
//! higher-level functionality for managing system calls.

// Re-export syscall constants and error codes from architecture layer
pub use fanga_arch_x86_64::syscall::{
    SYS_READ, SYS_WRITE, SYS_EXIT, SYS_FORK, SYS_EXEC,
    SYS_PIPE, SYS_KILL, 
    SYS_SHMGET, SYS_SHMAT, SYS_SHMDT, SYS_SHMCTL,
    SYS_MSGGET, SYS_MSGSND, SYS_MSGRCV,
    EINVAL, EBADF, ENOMEM, ENOSYS, EFAULT, EACCES, EPERM, ESRCH,
};

/// Result type for system calls
pub type SyscallResult = Result<usize, i64>;

/// Convert a raw syscall return value to a SyscallResult
pub fn syscall_result(ret: i64) -> SyscallResult {
    if ret < 0 {
        Err(ret)
    } else {
        Ok(ret as usize)
    }
}

/// System call dispatcher trait
/// 
/// This trait can be implemented by different syscall handlers
/// to provide custom behavior for syscall dispatching.
pub trait SyscallDispatcher {
    /// Dispatch a system call
    fn dispatch(&self, num: u64, args: &[u64; 6]) -> i64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_result_success() {
        let result = syscall_result(42);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_syscall_result_error() {
        let result = syscall_result(EINVAL);
        assert_eq!(result, Err(EINVAL));
    }

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
    fn test_error_codes_negative() {
        assert!(EINVAL < 0);
        assert!(EBADF < 0);
        assert!(ENOMEM < 0);
        assert!(ENOSYS < 0);
        assert!(EFAULT < 0);
        assert!(EACCES < 0);
        assert!(EPERM < 0);
        assert!(ESRCH < 0);
    }
}
