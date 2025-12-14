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
/// for very short delays. The timing is not calibrated to actual CPU speed
/// and should only be used when precise timing is not critical.
///
/// # Arguments
/// * `us` - Number of microseconds to delay
/// 
/// # Note
/// For delays < 1ms, this uses an uncalibrated busy loop and timing
/// will vary significantly across different hardware.
pub fn delay_us(us: u64) {
    // Convert microseconds to milliseconds (rough approximation)
    // For sub-millisecond delays, we'll do a busy loop
    if us < 1000 {
        // Very rough busy loop for microseconds
        // Note: This is uncalibrated and will vary by CPU speed
        for _ in 0..(us * 100) {
            core::hint::spin_loop();
        }
    } else {
        delay_ms(us / 1000);
    }
}

/// Sleep the current task for a specified number of milliseconds
///
/// **Note**: This is a simplified implementation that uses busy-wait.
/// In a full OS implementation, this would block the task, add it to
/// a sleep queue, and yield to the scheduler. Other tasks would run
/// while this task sleeps, and a timer interrupt would wake it up.
///
/// Current implementation uses `delay_ms` which wastes CPU cycles.
/// This is acceptable for initial implementation but should be improved.
///
/// # Arguments
/// * `ms` - Number of milliseconds to sleep
/// 
/// # TODO
/// - Implement proper sleep queue
/// - Yield to scheduler instead of busy-wait
/// - Wake tasks from timer interrupt when sleep expires
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
/// **Note**: This is a simplified placeholder implementation.
/// It blocks the task in the scheduler but then busy-waits in the
/// calling context, which defeats the purpose and wastes CPU.
///
/// In a proper implementation:
/// - The task would be blocked and added to a sleep queue
/// - The calling context would return immediately
/// - Timer interrupt would check wake times and unblock tasks
/// - Scheduler would automatically resume the task when ready
///
/// # Arguments
/// * `task_id` - The task to block
/// * `duration_ms` - Duration in milliseconds
///
/// # Returns
/// Result indicating success or error
/// 
/// # TODO
/// - Implement proper sleep queue with wake times
/// - Remove busy-wait delay
/// - Handle wake-up from timer interrupt
pub fn block_for_duration(task_id: TaskId, duration_ms: u64) -> Result<(), &'static str> {
    let mut scheduler_guard = scheduler::scheduler();
    
    // Block the task
    scheduler_guard.block_task(task_id)?;
    
    // In a real implementation, we would:
    // - Store the wake-up time in a sleep queue
    // - Return immediately (don't wait here)
    // - Have a timer interrupt handler check wake times
    // - Automatically unblock tasks when their time expires
    
    // Current simplified approach (not ideal):
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
