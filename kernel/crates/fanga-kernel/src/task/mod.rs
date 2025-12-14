//! Task Management
//!
//! This module provides the kernel's task/process management infrastructure including:
//! - Task Control Block (TCB)
//! - Task states and lifecycle
//! - Task ID management
//! - Process creation and management
//! - Preemptive scheduling
//! - Time management and delays

pub mod tcb;
pub mod scheduler;
pub mod context;
pub mod ipc;
pub mod process;
pub mod sched_timer;
pub mod timer_bridge;
pub mod time;

// Example tasks only available in no_std builds
#[cfg(not(test))]
pub mod examples;

// Re-export commonly used types
pub use tcb::{Task, TaskId, TaskState, TaskPriority};
pub use scheduler::Scheduler;
pub use context::TaskContext;
pub use ipc::{
    MessageQueue, Message, 
    Pipe, SharedMemory, 
    Signal, SignalHandler,
    Semaphore, TaskMutex,
};
pub use process::{ProcessManager, create_process, fork, exit};
pub use time::{delay_ms, delay_us, sleep_ms, uptime_ms, uptime_secs, timer_ticks};
