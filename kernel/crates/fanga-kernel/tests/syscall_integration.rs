//! Integration tests for system call functionality
//!
//! These tests verify that the syscall interface works correctly
//! at the kernel level.

use fanga_kernel::syscall::*;

#[test]
fn test_syscall_constants() {
    // Verify standard Linux syscall numbers
    assert_eq!(SYS_READ, 0);
    assert_eq!(SYS_WRITE, 1);
    assert_eq!(SYS_EXIT, 60);
    assert_eq!(SYS_FORK, 57);
    assert_eq!(SYS_EXEC, 59);
}

#[test]
fn test_error_codes() {
    // Verify error codes are negative (Linux convention)
    assert!(EINVAL < 0);
    assert!(EBADF < 0);
    assert!(ENOMEM < 0);
    assert!(ENOSYS < 0);
    assert!(EFAULT < 0);
    assert!(EACCES < 0);
}

#[test]
fn test_syscall_result_conversion_success() {
    let result = syscall_result(0);
    assert_eq!(result, Ok(0));
    
    let result = syscall_result(42);
    assert_eq!(result, Ok(42));
    
    let result = syscall_result(1024);
    assert_eq!(result, Ok(1024));
}

#[test]
fn test_syscall_result_conversion_error() {
    let result = syscall_result(EINVAL);
    assert_eq!(result, Err(EINVAL));
    
    let result = syscall_result(EBADF);
    assert_eq!(result, Err(EBADF));
    
    let result = syscall_result(ENOMEM);
    assert_eq!(result, Err(ENOMEM));
}

#[test]
fn test_syscall_result_boundary() {
    // Test at the boundary between success and error
    let result = syscall_result(-1);
    assert_eq!(result, Err(-1));
    
    let result = syscall_result(0);
    assert_eq!(result, Ok(0));
    
    let result = syscall_result(1);
    assert_eq!(result, Ok(1));
}

#[test]
fn test_error_code_uniqueness() {
    // Ensure error codes are unique
    let errors = vec![EINVAL, EBADF, ENOMEM, ENOSYS, EFAULT, EACCES];
    for i in 0..errors.len() {
        for j in (i + 1)..errors.len() {
            assert_ne!(errors[i], errors[j], 
                "Error codes at positions {} and {} are not unique", i, j);
        }
    }
}

#[test]
fn test_syscall_numbers_uniqueness() {
    // Ensure syscall numbers are unique (where they should be)
    assert_ne!(SYS_READ, SYS_WRITE);
    assert_ne!(SYS_READ, SYS_EXIT);
    assert_ne!(SYS_READ, SYS_FORK);
    assert_ne!(SYS_READ, SYS_EXEC);
    assert_ne!(SYS_WRITE, SYS_EXIT);
    assert_ne!(SYS_WRITE, SYS_FORK);
    assert_ne!(SYS_WRITE, SYS_EXEC);
    assert_ne!(SYS_EXIT, SYS_FORK);
    assert_ne!(SYS_EXIT, SYS_EXEC);
    assert_ne!(SYS_FORK, SYS_EXEC);
}
