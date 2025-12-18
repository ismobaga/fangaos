//! Preemption Statistics

use core::sync::atomic::{AtomicU64, Ordering};

/// Preemption statistics
#[derive(Debug, Clone, Copy)]
pub struct PreemptionStats {
    /// Total number of preemptions
    pub total_preemptions: u64,
    
    /// Number of voluntary preemptions (yield)
    pub voluntary_preemptions: u64,
    
    /// Number of involuntary preemptions (timer)
    pub involuntary_preemptions: u64,
    
    /// Maximum preemption latency (in microseconds)
    pub max_latency_us: u64,
}

impl PreemptionStats {
    /// Create empty statistics
    pub const fn new() -> Self {
        Self {
            total_preemptions: 0,
            voluntary_preemptions: 0,
            involuntary_preemptions: 0,
            max_latency_us: 0,
        }
    }
}

/// Global preemption statistics
static TOTAL_PREEMPTIONS: AtomicU64 = AtomicU64::new(0);
static VOLUNTARY_PREEMPTIONS: AtomicU64 = AtomicU64::new(0);
static INVOLUNTARY_PREEMPTIONS: AtomicU64 = AtomicU64::new(0);
static MAX_LATENCY_US: AtomicU64 = AtomicU64::new(0);

/// Record a voluntary preemption
pub fn record_voluntary_preempt() {
    TOTAL_PREEMPTIONS.fetch_add(1, Ordering::Relaxed);
    VOLUNTARY_PREEMPTIONS.fetch_add(1, Ordering::Relaxed);
}

/// Record an involuntary preemption
pub fn record_involuntary_preempt() {
    TOTAL_PREEMPTIONS.fetch_add(1, Ordering::Relaxed);
    INVOLUNTARY_PREEMPTIONS.fetch_add(1, Ordering::Relaxed);
}

/// Record preemption latency
pub fn record_latency(latency_us: u64) {
    // Update max latency if this is higher
    let mut current_max = MAX_LATENCY_US.load(Ordering::Relaxed);
    while latency_us > current_max {
        match MAX_LATENCY_US.compare_exchange_weak(
            current_max,
            latency_us,
            Ordering::Relaxed,
            Ordering::Relaxed
        ) {
            Ok(_) => break,
            Err(x) => current_max = x,
        }
    }
}

/// Get current preemption statistics
pub fn get_preemption_stats() -> PreemptionStats {
    PreemptionStats {
        total_preemptions: TOTAL_PREEMPTIONS.load(Ordering::Relaxed),
        voluntary_preemptions: VOLUNTARY_PREEMPTIONS.load(Ordering::Relaxed),
        involuntary_preemptions: INVOLUNTARY_PREEMPTIONS.load(Ordering::Relaxed),
        max_latency_us: MAX_LATENCY_US.load(Ordering::Relaxed),
    }
}

/// Reset preemption statistics
pub fn reset_stats() {
    TOTAL_PREEMPTIONS.store(0, Ordering::Relaxed);
    VOLUNTARY_PREEMPTIONS.store(0, Ordering::Relaxed);
    INVOLUNTARY_PREEMPTIONS.store(0, Ordering::Relaxed);
    MAX_LATENCY_US.store(0, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preemption_stats() {
        reset_stats();
        
        let stats = get_preemption_stats();
        assert_eq!(stats.total_preemptions, 0);
        
        record_voluntary_preempt();
        let stats = get_preemption_stats();
        assert_eq!(stats.total_preemptions, 1);
        assert_eq!(stats.voluntary_preemptions, 1);
        
        record_involuntary_preempt();
        let stats = get_preemption_stats();
        assert_eq!(stats.total_preemptions, 2);
        assert_eq!(stats.involuntary_preemptions, 1);
    }
    
    #[test]
    fn test_latency_tracking() {
        reset_stats();
        
        record_latency(100);
        let stats = get_preemption_stats();
        assert_eq!(stats.max_latency_us, 100);
        
        record_latency(50); // Should not update max
        let stats = get_preemption_stats();
        assert_eq!(stats.max_latency_us, 100);
        
        record_latency(200); // Should update max
        let stats = get_preemption_stats();
        assert_eq!(stats.max_latency_us, 200);
    }
}
