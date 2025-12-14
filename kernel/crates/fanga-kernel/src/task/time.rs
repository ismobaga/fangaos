//! Time Management
//!
//! This module provides time-related functions including:
//! - System uptime tracking
//! - Delay/sleep functions
//! - Time-based task blocking

use crate::task::{scheduler, TaskId};

/// Busy-wait delay for a specified number of milliseconds
///
/// This function spins in a loop until the specified time has elapsed.
/// It should be used sparingly as it wastes CPU cycles.
///
/// # Arguments
/// * `ms` - Number of milliseconds to delay
pub fn delay_ms(ms: u64) {
    let start = fanga_arch_x86_64::interrupts::idt::uptime_ms();
    let target = start + ms;
    
    while fanga_arch_x86_64::interrupts::idt::uptime_ms() < target {
        // Busy wait
        core::hint::spin_loop();
    }
}

/// Busy-wait delay for a specified number of microseconds
///
/// This is a very rough approximation and may not be accurate
/// for very short delays.
///
/// # Arguments
/// * `us` - Number of microseconds to delay
pub fn delay_us(us: u64) {
    // Convert microseconds to milliseconds (rough approximation)
    // For sub-millisecond delays, we'll do a busy loop
    if us < 1000 {
        // Very rough busy loop for microseconds
        for _ in 0..(us * 100) {
            core::hint::spin_loop();
        }
    } else {
        delay_ms(us / 1000);
    }
}

/// Sleep the current task for a specified number of milliseconds
///
/// This function blocks the current task and yields to other tasks.
/// The task will be woken up after approximately the specified time.
///
/// Note: This is a simplified implementation. In a full OS, this would
/// add the task to a sleep queue and wake it up via timer interrupt.
///
/// # Arguments
/// * `ms` - Number of milliseconds to sleep
pub fn sleep_ms(ms: u64) {
    // For now, use delay_ms as we don't have proper sleep queue implementation
    // In a full implementation, we would:
    // 1. Block the current task
    // 2. Set a wake time
    // 3. Yield to scheduler
    // 4. Timer interrupt would check wake times and unblock tasks
    delay_ms(ms);
}

/// Block the current task for a specified duration
///
/// # Arguments
/// * `task_id` - The task to block
/// * `duration_ms` - Duration in milliseconds
///
/// # Returns
/// Result indicating success or error
pub fn block_for_duration(task_id: TaskId, duration_ms: u64) -> Result<(), &'static str> {
    let mut scheduler_guard = scheduler::scheduler();
    
    // Block the task
    scheduler_guard.block_task(task_id)?;
    
    // In a real implementation, we would:
    // - Store the wake-up time
    // - Have a timer interrupt handler check wake times
    // - Automatically unblock tasks when their time expires
    
    // For now, we immediately unblock (simplified)
    drop(scheduler_guard);
    delay_ms(duration_ms);
    
    let mut scheduler_guard = scheduler::scheduler();
    scheduler_guard.unblock_task(task_id)?;
    
    Ok(())
}

/// Get system uptime in milliseconds
pub fn uptime_ms() -> u64 {
    fanga_arch_x86_64::interrupts::idt::uptime_ms()
}

/// Get system uptime in seconds
pub fn uptime_secs() -> u64 {
    fanga_arch_x86_64::interrupts::idt::uptime_secs()
}

/// Get timer ticks since boot
pub fn timer_ticks() -> u64 {
    fanga_arch_x86_64::interrupts::idt::timer_ticks()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_uptime_functions() {
        // These functions call into arch layer which is not available in tests
        // We just verify they compile and link correctly
        let _ticks = timer_ticks();
        let _ms = uptime_ms();
        let _secs = uptime_secs();
    }
    
    #[test]
    fn test_delay_ms() {
        // Test that delay_ms compiles
        // We can't actually test timing in unit tests
        delay_ms(0);
    }
    
    #[test]
    fn test_delay_us() {
        // Test that delay_us compiles
        delay_us(0);
    }
}
