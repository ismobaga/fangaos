//! Kernel Preemption
//!
//! This module provides fully preemptible kernel support including:
//! - Preemption disable/enable mechanisms
//! - Preemption points
//! - Preemption statistics
//! - Integration with scheduler

pub mod counter;
pub mod points;
pub mod stats;

pub use counter::{preempt_disable, preempt_enable, preempt_count, preemptible};
pub use points::{preempt_check, should_reschedule};
pub use stats::{PreemptionStats, get_preemption_stats};

/// Initialize preemption subsystem
pub fn init() {
    // Initialize per-CPU preemption counters
    counter::init_preempt_counters();
}

/// Check if we should yield to scheduler
pub fn need_resched() -> bool {
    points::should_reschedule()
}
