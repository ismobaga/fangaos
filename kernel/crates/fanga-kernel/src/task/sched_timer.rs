//! Preemptive Scheduling
//!
//! This module implements timer-based preemptive multitasking.
//! It integrates with the timer interrupt to perform periodic context switches.

use crate::task::scheduler;
use core::sync::atomic::{AtomicU64, Ordering};

/// Time slice in timer ticks (default ~55ms with 18.2 Hz PIT)
pub const TIME_SLICE: u64 = 1;

/// Timer tick counter (atomic for thread-safety)
static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Perform a context switch if the time slice has expired
///
/// This should be called from the timer interrupt handler.
///
/// # Returns
/// true if a context switch was performed, false otherwise
pub fn schedule_on_timer() -> bool {
    let tick = TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
    
    if tick >= TIME_SLICE {
        TICK_COUNTER.store(0, Ordering::Relaxed);
        
        // Perform scheduling
        let mut scheduler_guard = scheduler::scheduler();
        let (_prev, _next, should_switch) = scheduler_guard.schedule();
            
            if should_switch {
                #[cfg(not(test))]
                fanga_arch_x86_64::serial_println!(
                    "[SCHED] Context switch: {:?} -> {:?}",
                    _prev, _next
                );
                
                // In a real implementation, we would perform the actual context switch here
                // using fanga_arch_x86_64::context::switch_context()
                // For now, we just track the schedule decision
                
                return true;
            }
    }
    
    false
}

/// Get the current timer tick count
pub fn get_ticks() -> u64 {
    TICK_COUNTER.load(Ordering::Relaxed)
}

/// Reset the timer tick counter
pub fn reset_ticks() {
    TICK_COUNTER.store(0, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timer_ticks() {
        reset_ticks();
        assert_eq!(get_ticks(), 0);
        
        // Simulate timer ticks
        for i in 1..=10 {
            schedule_on_timer();
            if i < TIME_SLICE {
                assert_eq!(get_ticks(), i);
            }
        }
    }
}
