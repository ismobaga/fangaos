//! Preemption Points

use core::sync::atomic::{AtomicBool, Ordering};

/// Flag indicating whether a reschedule is needed
static NEED_RESCHED: AtomicBool = AtomicBool::new(false);

/// Set the "need reschedule" flag
pub fn set_need_resched() {
    NEED_RESCHED.store(true, Ordering::SeqCst);
}

/// Clear the "need reschedule" flag
pub fn clear_need_resched() {
    NEED_RESCHED.store(false, Ordering::SeqCst);
}

/// Check if a reschedule is needed
pub fn should_reschedule() -> bool {
    NEED_RESCHED.load(Ordering::SeqCst)
}

/// Check preemption and reschedule if needed
///
/// This is the main preemption point that should be called:
/// - When re-enabling preemption
/// - At strategic points in the kernel
/// - On return from interrupts
pub fn preempt_check() {
    // Only reschedule if:
    // 1. Preemption is enabled
    // 2. A reschedule is needed
    // 3. We're not in an interrupt context
    
    if !super::preemptible() {
        return;
    }
    
    if !should_reschedule() {
        return;
    }
    
    // TODO: Check if we're in an interrupt
    // For now, we'll assume we're not
    
    // Trigger reschedule
    do_preempt();
}

/// Perform the actual preemption
fn do_preempt() {
    // Clear the flag
    clear_need_resched();
    
    // TODO: Call into scheduler to pick next task
    // This would involve:
    // 1. Saving current context
    // 2. Calling scheduler
    // 3. Switching to new task
    
    // For now, this is a placeholder
}

/// Preemption point macro (for convenience)
/// This can be inserted at strategic points in kernel code
#[macro_export]
macro_rules! preempt_point {
    () => {
        $crate::preempt::preempt_check();
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_need_resched_flag() {
        clear_need_resched();
        assert!(!should_reschedule());
        
        set_need_resched();
        assert!(should_reschedule());
        
        clear_need_resched();
        assert!(!should_reschedule());
    }
    
    #[test]
    fn test_preempt_check_disabled() {
        use super::super::counter::{init_preempt_counters, preempt_disable, preempt_enable};
        
        init_preempt_counters();
        set_need_resched();
        
        preempt_disable();
        
        // Should not reschedule while disabled
        preempt_check();
        assert!(should_reschedule()); // Flag should still be set
        
        preempt_enable();
    }
}
