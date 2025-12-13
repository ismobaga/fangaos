//! Task Management
//!
//! This module provides the kernel's task/process management infrastructure including:
//! - Task Control Block (TCB)
//! - Task states and lifecycle
//! - Task ID management

pub mod tcb;
pub mod scheduler;
pub mod context;
pub mod ipc;

// Re-export commonly used types
pub use tcb::{Task, TaskId, TaskState, TaskPriority};
pub use scheduler::Scheduler;
pub use context::TaskContext;
pub use ipc::{MessageQueue, Message};
