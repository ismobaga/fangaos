//! Task Control Block (TCB)
//!
//! This module defines the Task Control Block structure which contains all the
//! information needed to manage a task/process in the operating system.

use super::context::TaskContext;
use crate::memory::{PhysAddr, VirtAddr};

/// Task ID - unique identifier for each task
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(pub usize);

impl TaskId {
    /// Create a new task ID
    pub const fn new(id: usize) -> Self {
        TaskId(id)
    }
    
    /// Get the raw ID value
    pub const fn as_usize(&self) -> usize {
        self.0
    }
}

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is ready to run
    Ready,
    /// Task is currently running
    Running,
    /// Task is blocked waiting for an event
    Blocked,
    /// Task has terminated
    Terminated,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Low priority (background tasks)
    Low = 0,
    /// Normal priority (default)
    Normal = 1,
    /// High priority (system tasks)
    High = 2,
    /// Critical priority (kernel tasks)
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// Task Control Block - contains all information about a task
#[derive(Debug)]
pub struct Task {
    /// Unique task identifier
    pub id: TaskId,
    
    /// Current task state
    pub state: TaskState,
    
    /// Task priority
    pub priority: TaskPriority,
    
    /// CPU context (saved registers)
    pub context: TaskContext,
    
    /// Kernel stack address
    pub kernel_stack: VirtAddr,
    
    /// Kernel stack size
    pub kernel_stack_size: usize,
    
    /// Page table physical address (CR3 value)
    pub page_table: PhysAddr,
    
    /// Task name (for debugging)
    pub name: [u8; 32],
}

impl Task {
    /// Create a new task
    pub fn new(
        id: TaskId,
        entry_point: VirtAddr,
        kernel_stack: VirtAddr,
        kernel_stack_size: usize,
        page_table: PhysAddr,
        priority: TaskPriority,
    ) -> Self {
        let mut task = Task {
            id,
            state: TaskState::Ready,
            priority,
            context: TaskContext::new(entry_point.as_u64(), kernel_stack.as_u64() + kernel_stack_size as u64),
            kernel_stack,
            kernel_stack_size,
            page_table,
            name: [0; 32],
        };
        
        // Set default name
        let default_name = b"task";
        task.name[..default_name.len()].copy_from_slice(default_name);
        
        task
    }
    
    /// Set the task name
    pub fn set_name(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let len = core::cmp::min(bytes.len(), self.name.len() - 1);
        self.name[..len].copy_from_slice(&bytes[..len]);
        self.name[len] = 0; // Null terminator
    }
    
    /// Get the task name as a string slice
    pub fn name(&self) -> &str {
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(self.name.len());
        core::str::from_utf8(&self.name[..len]).unwrap_or("<invalid>")
    }
    
    /// Check if the task is running
    pub fn is_running(&self) -> bool {
        self.state == TaskState::Running
    }
    
    /// Check if the task is ready
    pub fn is_ready(&self) -> bool {
        self.state == TaskState::Ready
    }
    
    /// Check if the task is terminated
    pub fn is_terminated(&self) -> bool {
        self.state == TaskState::Terminated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id() {
        let id1 = TaskId::new(1);
        let id2 = TaskId::new(2);
        
        assert_eq!(id1.as_usize(), 1);
        assert_eq!(id2.as_usize(), 2);
        assert!(id1 < id2);
    }

    #[test]
    fn test_task_state() {
        assert_eq!(TaskState::Ready, TaskState::Ready);
        assert_ne!(TaskState::Ready, TaskState::Running);
    }

    #[test]
    fn test_task_priority() {
        assert!(TaskPriority::Low < TaskPriority::Normal);
        assert!(TaskPriority::Normal < TaskPriority::High);
        assert!(TaskPriority::High < TaskPriority::Critical);
        assert_eq!(TaskPriority::default(), TaskPriority::Normal);
    }

    #[test]
    fn test_task_name() {
        let task = Task::new(
            TaskId::new(1),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            4096,
            PhysAddr::new(0x3000),
            TaskPriority::Normal,
        );
        
        assert_eq!(task.name(), "task");
    }

    #[test]
    fn test_task_set_name() {
        let mut task = Task::new(
            TaskId::new(1),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            4096,
            PhysAddr::new(0x3000),
            TaskPriority::Normal,
        );
        
        task.set_name("my_task");
        assert_eq!(task.name(), "my_task");
    }

    #[test]
    fn test_task_states() {
        let mut task = Task::new(
            TaskId::new(1),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            4096,
            PhysAddr::new(0x3000),
            TaskPriority::Normal,
        );
        
        assert!(task.is_ready());
        assert!(!task.is_running());
        
        task.state = TaskState::Running;
        assert!(task.is_running());
        assert!(!task.is_ready());
        
        task.state = TaskState::Terminated;
        assert!(task.is_terminated());
    }
}
