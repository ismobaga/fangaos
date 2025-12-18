//! Task Management
//!
//! This module provides the kernel's task/process management infrastructure including:
//! - Task Control Block (TCB)
//! - Task states and lifecycle
//! - Task ID management
//! - Process creation and management
//! - Preemptive scheduling
//! - Time management and delays
//! - Multi-threading (kernel and user threads)
//! - Advanced synchronization (condition variables, RW locks, barriers)
//! - Process groups and sessions
//! - Advanced signal handling
//! - Core dumps for debugging

pub mod tcb;
pub mod scheduler;
pub mod context;
pub mod ipc;
pub mod process;
pub mod sched_timer;
pub mod timer_bridge;
pub mod time;

// Advanced process features
pub mod thread;
pub mod sync;
pub mod pgroup;
pub mod sigadv;
pub mod coredump;

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

// Re-export advanced features
pub use thread::{Thread, ThreadId, ThreadType, ThreadAttributes, ThreadManager, RtSchedulingPolicy};
pub use sync::{ConditionVariable, RwLock, Barrier};
pub use pgroup::{ProcessGroup, ProcessGroupId, Session, SessionId, ProcessGroupManager, TerminalId};
pub use sigadv::{SignalAction, SignalFlags, SignalInfo, SigAction, AdvancedSignalHandler, SignalTarget, SignalManager};
pub use coredump::{CoreDump, CoreDumpReason, CoreDumpManager, RegisterDump, MemoryDump, ThreadInfo};
