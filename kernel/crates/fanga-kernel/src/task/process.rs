//! Process Management
//!
//! This module provides high-level process management functionality including:
//! - Process creation and termination
//! - fork() and exit() implementations
//! - Integration with scheduler and context switching

extern crate alloc;
use alloc::vec::Vec;

use super::tcb::{Task, TaskId, TaskState, TaskPriority};
use super::scheduler;
use crate::memory::{PhysAddr, VirtAddr};

/// Process Manager
pub struct ProcessManager {
    /// Next available process ID
    next_pid: usize,
}

impl ProcessManager {
    /// Create a new process manager
    pub const fn new() -> Self {
        Self { next_pid: 1 }
    }
    
    /// Create a new process
    ///
    /// # Arguments
    /// * `entry_point` - Virtual address of the process entry point
    /// * `stack_size` - Size of the kernel stack in bytes
    /// * `page_table` - Physical address of the process's page table
    /// * `priority` - Process priority
    ///
    /// # Returns
    /// The TaskId of the created process, or an error
    pub fn create_process(
        &mut self,
        entry_point: VirtAddr,
        stack_size: usize,
        page_table: PhysAddr,
        priority: TaskPriority,
    ) -> Result<TaskId, &'static str> {
        // Allocate kernel stack (simplified - in real OS we'd allocate from VMM)
        let kernel_stack = VirtAddr::new(0x10000 + (self.next_pid as u64 * 0x10000));
        
        // Create task
        let task = Task::new(
            TaskId::new(0), // Will be set by scheduler
            entry_point,
            kernel_stack,
            stack_size,
            page_table,
            priority,
        );
        
        self.next_pid += 1;
        
        // Add to scheduler
        let mut scheduler_guard = scheduler::scheduler();
        scheduler_guard.add_task(task)
    }
    
    /// Fork the current process (create a copy)
    ///
    /// # Returns
    /// - In parent: child's TaskId
    /// - In child: TaskId(0)
    /// - On error: Err
    pub fn fork_process(&mut self, parent_id: TaskId) -> Result<TaskId, &'static str> {
        let mut scheduler_guard = scheduler::scheduler();
        
        // Get parent task
        let parent = scheduler_guard.get_task(parent_id).ok_or("Parent task not found")?;
        
        // Duplicate the parent task
        let mut child = Task::new(
            TaskId::new(0), // Will be set by scheduler
            VirtAddr::new(parent.context.rip),
            VirtAddr::new(0x10000 + (self.next_pid as u64 * 0x10000)),
            parent.kernel_stack_size,
            parent.page_table, // In real OS, we'd duplicate the page table
            parent.priority,
        );
        
        // Copy parent's context
        child.context = parent.context;
        
        // Set return value to 0 for child (will be returned when child is scheduled)
        child.context.rax = 0;
        
        // Copy parent's name with "_child" suffix
        let parent_name = parent.name();
        let mut child_name = [0u8; 32];
        let name_len = core::cmp::min(parent_name.len(), 24);
        child_name[..name_len].copy_from_slice(&parent_name.as_bytes()[..name_len]);
        let suffix = b"_child";
        child_name[name_len..name_len + suffix.len()].copy_from_slice(suffix);
        child.name = child_name;
        
        self.next_pid += 1;
        
        // Add child to scheduler
        let child_id = scheduler_guard.add_task(child)?;
        
        Ok(child_id)
    }
    
    /// Terminate a process
    ///
    /// # Arguments
    /// * `task_id` - The task to terminate
    /// * `exit_code` - The exit code
    pub fn exit_process(&mut self, task_id: TaskId, _exit_code: i32) -> Result<(), &'static str> {
        let mut scheduler_guard = scheduler::scheduler();
        
        // Mark task as terminated
        scheduler_guard.terminate_task(task_id)?;
        
        // In a real OS, we would:
        // - Notify parent process
        // - Clean up resources (memory, file descriptors, etc.)
        // - Wake up any waiting processes
        
        // Log the exit
        #[cfg(not(test))]
        fanga_arch_x86_64::serial_println!(
            "[PROCESS] Task {:?} exited with code {}",
            task_id,
            _exit_code
        );
        
        Ok(())
    }
    
    /// Get the next available PID
    pub fn next_pid(&self) -> usize {
        self.next_pid
    }
}

/// Global process manager instance
use spin::Mutex;
static PROCESS_MANAGER: Mutex<ProcessManager> = Mutex::new(ProcessManager::new());

/// Initialize the process manager
pub fn init() {
    // Process manager is already initialized via static initialization
    // This function is provided for consistency with other subsystems
}

/// Get a reference to the global process manager
pub fn process_manager() -> spin::MutexGuard<'static, ProcessManager> {
    PROCESS_MANAGER.lock()
}

/// Create a new process
pub fn create_process(
    entry_point: VirtAddr,
    stack_size: usize,
    page_table: PhysAddr,
    priority: TaskPriority,
) -> Result<TaskId, &'static str> {
    process_manager().create_process(entry_point, stack_size, page_table, priority)
}

/// Fork the current process
pub fn fork(parent_id: TaskId) -> Result<TaskId, &'static str> {
    process_manager().fork_process(parent_id)
}

/// Exit the current process
pub fn exit(task_id: TaskId, exit_code: i32) -> Result<(), &'static str> {
    process_manager().exit_process(task_id, exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_manager_creation() {
        let pm = ProcessManager::new();
        assert_eq!(pm.next_pid(), 1);
    }
    
    #[test]
    fn test_process_creation() {
        // Initialize scheduler first
        scheduler::init();
        
        let mut pm = ProcessManager::new();
        let result = pm.create_process(
            VirtAddr::new(0x1000),
            4096,
            PhysAddr::new(0x0),
            TaskPriority::Normal,
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_fork_process() {
        // Initialize scheduler
        scheduler::init();
        
        let mut pm = ProcessManager::new();
        
        // Create parent process
        let parent_id = pm.create_process(
            VirtAddr::new(0x1000),
            4096,
            PhysAddr::new(0x0),
            TaskPriority::Normal,
        ).unwrap();
        
        // Fork it
        let result = pm.fork_process(parent_id);
        assert!(result.is_ok());
        
        let child_id = result.unwrap();
        assert_ne!(parent_id, child_id);
    }
}
