//! Task Management
//!
//! This module provides the kernel's task/process management infrastructure including:
//! - Task Control Block (TCB)
//! - Task states and lifecycle
//! - Task ID management
//! - Process creation and management
//! - Preemptive scheduling

pub mod tcb;
pub mod scheduler;
pub mod context;
pub mod ipc;
pub mod process;
pub mod sched_timer;

// Example tasks only available in no_std builds
#[cfg(not(test))]
pub mod examples;

// Re-export commonly used types
pub use tcb::{Task, TaskId, TaskState, TaskPriority};
pub use scheduler::Scheduler;
pub use context::TaskContext;
pub use ipc::{MessageQueue, Message};
pub use process::{ProcessManager, create_process, fork, exit};
