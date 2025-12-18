//! Preemption Counter Management

use core::sync::atomic::{AtomicUsize, Ordering};

/// Per-CPU preemption disable count
/// In a full SMP implementation, this would be in per-CPU data
static PREEMPT_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Initialize preemption counters
pub fn init_preempt_counters() {
    PREEMPT_COUNT.store(0, Ordering::SeqCst);
}

/// Disable preemption
///
/// This increments the preemption counter. Preemption is disabled when the
/// counter is greater than zero.
#[inline]
pub fn preempt_disable() {
    PREEMPT_COUNT.fetch_add(1, Ordering::SeqCst);
}

/// Enable preemption
///
/// This decrements the preemption counter. When the counter reaches zero,
/// preemption is re-enabled and a reschedule check is performed.
#[inline]
pub fn preempt_enable() {
    let old_count = PREEMPT_COUNT.fetch_sub(1, Ordering::SeqCst);
    
    // Check if we just re-enabled preemption
    if old_count == 1 {
        // Preemption is now enabled, check if we should reschedule
        super::preempt_check();
    }
}

/// Get current preemption count
#[inline]
pub fn preempt_count() -> usize {
    PREEMPT_COUNT.load(Ordering::SeqCst)
}

/// Check if preemption is enabled (count == 0)
#[inline]
pub fn preemptible() -> bool {
    preempt_count() == 0
}

/// RAII guard for temporarily disabling preemption
pub struct PreemptGuard {
    _private: (),
}

impl PreemptGuard {
    /// Create a new preemption guard (disables preemption)
    pub fn new() -> Self {
        preempt_disable();
        Self { _private: () }
    }
}

impl Drop for PreemptGuard {
    fn drop(&mut self) {
        preempt_enable();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preempt_counter() {
        init_preempt_counters();
        
        assert_eq!(preempt_count(), 0);
        assert!(preemptible());
        
        preempt_disable();
        assert_eq!(preempt_count(), 1);
        assert!(!preemptible());
        
        preempt_enable();
        assert_eq!(preempt_count(), 0);
        assert!(preemptible());
    }
    
    #[test]
    fn test_nested_preempt_disable() {
        init_preempt_counters();
        
        preempt_disable();
        preempt_disable();
        preempt_disable();
        assert_eq!(preempt_count(), 3);
        
        preempt_enable();
        assert_eq!(preempt_count(), 2);
        assert!(!preemptible());
        
        preempt_enable();
        preempt_enable();
        assert_eq!(preempt_count(), 0);
        assert!(preemptible());
    }
    
    #[test]
    fn test_preempt_guard() {
        init_preempt_counters();
        
        assert!(preemptible());
        
        {
            let _guard = PreemptGuard::new();
            assert!(!preemptible());
        }
        
        assert!(preemptible());
    }
}
