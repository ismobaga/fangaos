//! Per-CPU Data Structures
//!
//! This module provides CPU-local storage for per-CPU data.

use super::CpuId;

/// Per-CPU data structure
///
/// This structure holds data that is unique to each CPU.
/// In a real implementation, this would use GS segment register
/// or similar mechanism for fast access.
#[derive(Debug)]
pub struct PerCpuData {
    /// CPU ID
    pub cpu_id: CpuId,
    
    /// Current task running on this CPU
    pub current_task: Option<usize>,
    
    /// Interrupt nesting level
    pub interrupt_depth: usize,
    
    /// Preemption disable count
    pub preempt_count: usize,
    
    /// Idle time counter (in ticks)
    pub idle_ticks: u64,
    
    /// Total ticks this CPU has been running
    pub total_ticks: u64,
}

impl PerCpuData {
    /// Create a new per-CPU data structure
    pub const fn new(cpu_id: CpuId) -> Self {
        Self {
            cpu_id,
            current_task: None,
            interrupt_depth: 0,
            preempt_count: 0,
            idle_ticks: 0,
            total_ticks: 0,
        }
    }
    
    /// Check if interrupts are nested
    pub fn in_interrupt(&self) -> bool {
        self.interrupt_depth > 0
    }
    
    /// Check if preemption is enabled
    pub fn preemptible(&self) -> bool {
        self.preempt_count == 0 && !self.in_interrupt()
    }
}

/// Per-CPU data accessor
///
/// This trait provides a uniform interface for accessing per-CPU data.
pub trait PerCpu {
    /// Get the current CPU's data
    fn current() -> &'static mut PerCpuData;
    
    /// Get a specific CPU's data
    fn get(cpu_id: CpuId) -> Option<&'static mut PerCpuData>;
}

// TODO: Implement actual per-CPU data storage
// For now, we use a simple array indexed by CPU ID

use core::cell::UnsafeCell;
use super::cpu::MAX_CPUS;

/// Static array of per-CPU data
static mut PER_CPU_DATA: [UnsafeCell<PerCpuData>; MAX_CPUS] = {
    const INIT: UnsafeCell<PerCpuData> = UnsafeCell::new(PerCpuData::new(CpuId(0)));
    [INIT; MAX_CPUS]
};

/// Initialize per-CPU data for a specific CPU
pub fn init_percpu_data(cpu_id: CpuId) {
    unsafe {
        let data = &mut *PER_CPU_DATA[cpu_id.as_usize()].get();
        data.cpu_id = cpu_id;
        data.current_task = None;
        data.interrupt_depth = 0;
        data.preempt_count = 0;
        data.idle_ticks = 0;
        data.total_ticks = 0;
    }
}

/// Get the current CPU's per-CPU data
pub fn current_cpu_data() -> &'static mut PerCpuData {
    let cpu_id = super::current_cpu_id();
    unsafe { &mut *PER_CPU_DATA[cpu_id.as_usize()].get() }
}

/// Get a specific CPU's per-CPU data
pub fn get_cpu_data(cpu_id: CpuId) -> Option<&'static mut PerCpuData> {
    if cpu_id.as_usize() >= MAX_CPUS {
        return None;
    }
    unsafe { Some(&mut *PER_CPU_DATA[cpu_id.as_usize()].get()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_percpu_data_creation() {
        let data = PerCpuData::new(CpuId::new(0));
        assert_eq!(data.cpu_id.as_usize(), 0);
        assert_eq!(data.interrupt_depth, 0);
        assert_eq!(data.preempt_count, 0);
        assert!(data.preemptible());
    }
    
    #[test]
    fn test_interrupt_nesting() {
        let mut data = PerCpuData::new(CpuId::new(0));
        assert!(!data.in_interrupt());
        
        data.interrupt_depth = 1;
        assert!(data.in_interrupt());
        assert!(!data.preemptible());
    }
    
    #[test]
    fn test_preemption_control() {
        let mut data = PerCpuData::new(CpuId::new(0));
        assert!(data.preemptible());
        
        data.preempt_count = 1;
        assert!(!data.preemptible());
        
        data.preempt_count = 0;
        assert!(data.preemptible());
    }
}
