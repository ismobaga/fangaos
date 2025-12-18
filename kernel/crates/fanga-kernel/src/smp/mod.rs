//! Symmetric Multi-Processing (SMP) Support
//!
//! This module provides multi-core CPU support including:
//! - CPU detection and enumeration
//! - Per-CPU data structures
//! - Application Processor (AP) startup
//! - Inter-Processor Interrupts (IPI)
//! - CPU-local storage
//! - SMP-safe synchronization primitives

pub mod cpu;
pub mod percpu;
pub mod ipi;
pub mod spinlock;
pub mod acpi;

pub use cpu::{CpuId, CpuInfo, CpuState, CpuManager, current_cpu_id, cpu_count};
pub use percpu::{PerCpu, PerCpuData};
pub use ipi::{Ipi, IpiType, IpiTarget, send_ipi};
pub use spinlock::SpinLock;
pub use acpi::AcpiInfo;

use spin::Once;

/// Global CPU manager instance
static CPU_MANAGER: Once<spin::Mutex<CpuManager>> = Once::new();

/// Initialize the SMP subsystem
pub fn init() -> Result<(), &'static str> {
    // Initialize CPU manager
    let manager = CpuManager::new();
    CPU_MANAGER.call_once(|| spin::Mutex::new(manager));
    
    // Detect and enumerate CPUs
    detect_cpus()?;
    
    Ok(())
}

/// Detect and enumerate all CPUs in the system
fn detect_cpus() -> Result<(), &'static str> {
    let mut manager = CPU_MANAGER.get().ok_or("CPU manager not initialized")?.lock();
    manager.detect_cpus()
}

/// Get reference to CPU manager
pub fn cpu_manager() -> &'static spin::Mutex<CpuManager> {
    CPU_MANAGER.get().expect("CPU manager not initialized")
}

/// Initialize Application Processors (APs)
pub fn start_application_processors() -> Result<(), &'static str> {
    let mut manager = cpu_manager().lock();
    manager.start_aps()
}
