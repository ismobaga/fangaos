//! Inter-Processor Interrupts (IPI)
//!
//! This module provides IPI support for inter-CPU communication.

use super::CpuId;

/// IPI type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpiType {
    /// Generic IPI (for testing)
    Generic,
    
    /// Reschedule IPI (trigger scheduler on target CPU)
    Reschedule,
    
    /// TLB flush IPI (invalidate TLB on target CPU)
    TlbFlush,
    
    /// Function call IPI (execute function on target CPU)
    FunctionCall,
    
    /// Halt IPI (stop target CPU)
    Halt,
}

/// IPI target specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpiTarget {
    /// Send to specific CPU
    Cpu(CpuId),
    
    /// Send to all CPUs except self
    AllExceptSelf,
    
    /// Send to all CPUs including self
    All,
    
    /// Send to specific set of CPUs (bitmask)
    Mask(u64),
}

/// IPI structure
#[derive(Debug, Clone, Copy)]
pub struct Ipi {
    /// IPI type
    pub ipi_type: IpiType,
    
    /// Target CPUs
    pub target: IpiTarget,
    
    /// Optional data parameter
    pub data: u64,
}

impl Ipi {
    /// Create a new IPI
    pub const fn new(ipi_type: IpiType, target: IpiTarget) -> Self {
        Self {
            ipi_type,
            target,
            data: 0,
        }
    }
    
    /// Create a new IPI with data
    pub const fn with_data(ipi_type: IpiType, target: IpiTarget, data: u64) -> Self {
        Self {
            ipi_type,
            target,
            data,
        }
    }
}

/// Send an IPI
pub fn send_ipi(ipi: Ipi) -> Result<(), &'static str> {
    // TODO: Implement actual IPI sending via APIC
    // This would use the Local APIC's ICR (Interrupt Command Register)
    
    match ipi.target {
        IpiTarget::Cpu(cpu_id) => {
            // Send to specific CPU
            send_ipi_to_cpu(cpu_id, ipi.ipi_type, ipi.data)
        }
        IpiTarget::AllExceptSelf => {
            // Send to all CPUs except current
            send_ipi_broadcast(ipi.ipi_type, ipi.data, true)
        }
        IpiTarget::All => {
            // Send to all CPUs including current
            send_ipi_broadcast(ipi.ipi_type, ipi.data, false)
        }
        IpiTarget::Mask(mask) => {
            // Send to CPUs specified in mask
            send_ipi_to_mask(mask, ipi.ipi_type, ipi.data)
        }
    }
}

/// Send IPI to a specific CPU
fn send_ipi_to_cpu(_cpu_id: CpuId, _ipi_type: IpiType, _data: u64) -> Result<(), &'static str> {
    // TODO: Use APIC to send IPI
    Ok(())
}

/// Send IPI broadcast
fn send_ipi_broadcast(_ipi_type: IpiType, _data: u64, _exclude_self: bool) -> Result<(), &'static str> {
    // TODO: Use APIC to broadcast IPI
    Ok(())
}

/// Send IPI to CPUs specified in mask
fn send_ipi_to_mask(_mask: u64, _ipi_type: IpiType, _data: u64) -> Result<(), &'static str> {
    // TODO: Use APIC to send IPI to mask
    Ok(())
}

/// TLB shootdown - invalidate TLB entries on all CPUs
pub fn tlb_shootdown(addr: u64) -> Result<(), &'static str> {
    let ipi = Ipi::with_data(IpiType::TlbFlush, IpiTarget::AllExceptSelf, addr);
    send_ipi(ipi)
}

/// Trigger reschedule on a specific CPU
pub fn reschedule_cpu(cpu_id: CpuId) -> Result<(), &'static str> {
    let ipi = Ipi::new(IpiType::Reschedule, IpiTarget::Cpu(cpu_id));
    send_ipi(ipi)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ipi_creation() {
        let ipi = Ipi::new(IpiType::Reschedule, IpiTarget::Cpu(CpuId::new(1)));
        assert_eq!(ipi.ipi_type, IpiType::Reschedule);
        assert_eq!(ipi.data, 0);
    }
    
    #[test]
    fn test_ipi_with_data() {
        let ipi = Ipi::with_data(IpiType::TlbFlush, IpiTarget::AllExceptSelf, 0x1000);
        assert_eq!(ipi.ipi_type, IpiType::TlbFlush);
        assert_eq!(ipi.data, 0x1000);
    }
    
    #[test]
    fn test_ipi_targets() {
        let ipi1 = Ipi::new(IpiType::Generic, IpiTarget::Cpu(CpuId::new(0)));
        assert!(matches!(ipi1.target, IpiTarget::Cpu(_)));
        
        let ipi2 = Ipi::new(IpiType::Generic, IpiTarget::AllExceptSelf);
        assert_eq!(ipi2.target, IpiTarget::AllExceptSelf);
    }
}
