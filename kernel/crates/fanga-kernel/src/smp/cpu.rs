//! CPU Management
//!
//! This module provides CPU detection, enumeration, and management.

extern crate alloc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// Maximum number of CPUs supported
pub const MAX_CPUS: usize = 256;

/// CPU ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CpuId(pub usize);

impl CpuId {
    /// Create a new CPU ID
    pub const fn new(id: usize) -> Self {
        Self(id)
    }
    
    /// Get the raw ID value
    pub const fn as_usize(&self) -> usize {
        self.0
    }
}

/// CPU state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuState {
    /// CPU is offline/not detected
    Offline,
    /// CPU is online and running
    Online,
    /// CPU is being initialized
    Initializing,
    /// CPU has failed
    Failed,
}

/// CPU information
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// CPU ID
    pub id: CpuId,
    
    /// APIC ID (from hardware)
    pub apic_id: u32,
    
    /// CPU state
    pub state: CpuState,
    
    /// Is this the bootstrap processor (BSP)?
    pub is_bsp: bool,
    
    /// NUMA node ID (if NUMA is supported)
    pub numa_node: Option<usize>,
}

impl CpuInfo {
    /// Create a new CPU info structure
    pub fn new(id: CpuId, apic_id: u32, is_bsp: bool) -> Self {
        Self {
            id,
            apic_id,
            state: CpuState::Offline,
            is_bsp,
            numa_node: None,
        }
    }
}

/// CPU Manager
pub struct CpuManager {
    /// List of all CPUs
    cpus: Vec<CpuInfo>,
    
    /// Number of online CPUs
    online_count: AtomicUsize,
    
    /// Bootstrap processor ID
    bsp_id: CpuId,
}

impl CpuManager {
    /// Create a new CPU manager
    pub fn new() -> Self {
        Self {
            cpus: Vec::new(),
            online_count: AtomicUsize::new(0),
            bsp_id: CpuId::new(0),
        }
    }
    
    /// Detect and enumerate all CPUs
    pub fn detect_cpus(&mut self) -> Result<(), &'static str> {
        // Start with BSP (Bootstrap Processor)
        let bsp = CpuInfo::new(CpuId::new(0), 0, true);
        self.cpus.push(bsp);
        
        // TODO: Parse ACPI MADT to find additional CPUs
        // For now, we'll detect via CPUID if available
        
        // Mark BSP as online
        self.cpus[0].state = CpuState::Online;
        self.online_count.store(1, Ordering::SeqCst);
        
        Ok(())
    }
    
    /// Get the number of CPUs
    pub fn cpu_count(&self) -> usize {
        self.cpus.len()
    }
    
    /// Get the number of online CPUs
    pub fn online_count(&self) -> usize {
        self.online_count.load(Ordering::SeqCst)
    }
    
    /// Get CPU information
    pub fn get_cpu(&self, id: CpuId) -> Option<&CpuInfo> {
        self.cpus.get(id.as_usize())
    }
    
    /// Get mutable CPU information
    pub fn get_cpu_mut(&mut self, id: CpuId) -> Option<&mut CpuInfo> {
        self.cpus.get_mut(id.as_usize())
    }
    
    /// Add a CPU to the manager
    pub fn add_cpu(&mut self, apic_id: u32) -> Result<CpuId, &'static str> {
        if self.cpus.len() >= MAX_CPUS {
            return Err("Maximum CPU count reached");
        }
        
        let id = CpuId::new(self.cpus.len());
        let cpu = CpuInfo::new(id, apic_id, false);
        self.cpus.push(cpu);
        
        Ok(id)
    }
    
    /// Start Application Processors
    pub fn start_aps(&mut self) -> Result<(), &'static str> {
        // TODO: Implement AP startup sequence
        // This involves:
        // 1. Sending INIT IPI
        // 2. Waiting
        // 3. Sending SIPI (Startup IPI) with trampoline code address
        // 4. Waiting for AP to initialize
        
        // For now, this is a placeholder
        Ok(())
    }
    
    /// Bring a CPU online
    pub fn bring_cpu_online(&mut self, id: CpuId) -> Result<(), &'static str> {
        let cpu = self.get_cpu_mut(id).ok_or("Invalid CPU ID")?;
        
        if cpu.state == CpuState::Online {
            return Ok(());
        }
        
        cpu.state = CpuState::Initializing;
        
        // TODO: Actual initialization sequence
        
        cpu.state = CpuState::Online;
        self.online_count.fetch_add(1, Ordering::SeqCst);
        
        Ok(())
    }
}

/// Current CPU ID (will be set via CPU-local storage)
static CURRENT_CPU: AtomicU32 = AtomicU32::new(0);

/// Get the current CPU ID
pub fn current_cpu_id() -> CpuId {
    CpuId::new(CURRENT_CPU.load(Ordering::Relaxed) as usize)
}

/// Set the current CPU ID (called during CPU initialization)
pub fn set_current_cpu_id(id: CpuId) {
    CURRENT_CPU.store(id.as_usize() as u32, Ordering::Relaxed);
}

/// Get the number of CPUs
pub fn cpu_count() -> usize {
    // Access via global manager
    1 // Placeholder - will be updated when manager is accessible
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_id() {
        let id = CpuId::new(5);
        assert_eq!(id.as_usize(), 5);
    }
    
    #[test]
    fn test_cpu_manager_creation() {
        let manager = CpuManager::new();
        assert_eq!(manager.cpu_count(), 0);
        assert_eq!(manager.online_count(), 0);
    }
    
    #[test]
    fn test_cpu_detection() {
        let mut manager = CpuManager::new();
        manager.detect_cpus().unwrap();
        
        assert_eq!(manager.cpu_count(), 1); // Should detect BSP
        assert_eq!(manager.online_count(), 1);
        
        let bsp = manager.get_cpu(CpuId::new(0)).unwrap();
        assert!(bsp.is_bsp);
        assert_eq!(bsp.state, CpuState::Online);
    }
    
    #[test]
    fn test_add_cpu() {
        let mut manager = CpuManager::new();
        manager.detect_cpus().unwrap();
        
        let cpu_id = manager.add_cpu(1).unwrap();
        assert_eq!(cpu_id.as_usize(), 1);
        assert_eq!(manager.cpu_count(), 2);
        
        let cpu = manager.get_cpu(cpu_id).unwrap();
        assert_eq!(cpu.apic_id, 1);
        assert_eq!(cpu.state, CpuState::Offline);
        assert!(!cpu.is_bsp);
    }
}
