//! System Call Handlers
//!
//! This module provides the kernel-side implementation of system calls,
//! integrating with task management, memory management, and I/O subsystems.

use crate::task::{self, TaskId};

/// Handle fork() system call
///
/// Creates a copy of the current process.
///
/// # Returns
/// - Parent process: child's TaskId
/// - Child process: 0
/// - On error: negative error code
pub fn handle_fork(current_task_id: TaskId) -> i64 {
    match task::fork(current_task_id) {
        Ok(child_id) => {
            // Return child's PID to parent
            child_id.as_usize() as i64
        }
        Err(_) => {
            // Return error code
            fanga_arch_x86_64::syscall::ENOMEM
        }
    }
}

/// Handle exit() system call
///
/// Terminates the current process with the given exit code.
///
/// # Arguments
/// * `task_id` - Current task ID
/// * `exit_code` - Exit status code
///
/// # Returns
/// This function does not return (process is terminated)
pub fn handle_exit(task_id: TaskId, exit_code: i32) -> ! {
    // Terminate the process
    let _ = task::exit(task_id, exit_code);
    
    // Schedule next task
    let mut scheduler_guard = task::scheduler::scheduler();
    let (_, next, _) = scheduler_guard.schedule();
    
    if let Some(next_task_id) = next {
        // In a real OS, we would context switch here
        // For now, we'll just log and halt
        drop(scheduler_guard);
        
        fanga_arch_x86_64::serial_println!(
            "[SYSCALL] Would switch to task {:?} after exit",
            next_task_id
        );
    }
    
    // No more tasks to run, halt
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nostack, nomem));
        }
    }
}

/// Get the current task ID from scheduler
///
/// # Returns
/// The current task ID, or None if no task is running
pub fn get_current_task() -> Option<TaskId> {
    let scheduler_guard = task::scheduler::scheduler();
    scheduler_guard.current_task()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{PhysAddr, VirtAddr};
    use crate::task::{TaskPriority, scheduler};
    
    #[test]
    fn test_handle_fork() {
        // Initialize scheduler
        scheduler::init();
        
        // Create a parent task
        let parent_id = task::create_process(
            VirtAddr::new(0x1000),
            4096,
            PhysAddr::new(0x0),
            TaskPriority::Normal,
        ).unwrap();
        
        // Fork it
        let result = handle_fork(parent_id);
        assert!(result > 0); // Should return child PID
    }
}
