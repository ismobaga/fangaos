//! ACPI Information
//!
//! This module provides basic ACPI parsing for SMP initialization.

use super::CpuId;

/// ACPI information structure
#[derive(Debug, Clone)]
pub struct AcpiInfo {
    /// Number of CPUs detected via ACPI
    pub cpu_count: usize,
    
    /// List of APIC IDs
    pub apic_ids: [u32; 256],
    
    /// Local APIC address
    pub lapic_addr: u64,
    
    /// I/O APIC address
    pub ioapic_addr: u64,
}

impl AcpiInfo {
    /// Create a new ACPI info structure
    pub const fn new() -> Self {
        Self {
            cpu_count: 0,
            apic_ids: [0; 256],
            lapic_addr: 0,
            ioapic_addr: 0,
        }
    }
    
    /// Parse ACPI tables to detect CPUs
    pub fn parse_madt(&mut self) -> Result<(), &'static str> {
        // TODO: Implement MADT (Multiple APIC Description Table) parsing
        // This would involve:
        // 1. Finding the RSDP (Root System Description Pointer)
        // 2. Following to RSDT/XSDT
        // 3. Locating MADT
        // 4. Parsing Local APIC and I/O APIC entries
        
        // For now, just set default values
        self.cpu_count = 1;
        self.apic_ids[0] = 0;
        self.lapic_addr = 0xFEE00000; // Default Local APIC address
        self.ioapic_addr = 0xFEC00000; // Default I/O APIC address
        
        Ok(())
    }
    
    /// Get APIC ID for a CPU
    pub fn get_apic_id(&self, cpu_id: CpuId) -> Option<u32> {
        if cpu_id.as_usize() < self.cpu_count {
            Some(self.apic_ids[cpu_id.as_usize()])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_acpi_info_creation() {
        let info = AcpiInfo::new();
        assert_eq!(info.cpu_count, 0);
    }
    
    #[test]
    fn test_acpi_parse_madt() {
        let mut info = AcpiInfo::new();
        info.parse_madt().unwrap();
        
        assert_eq!(info.cpu_count, 1);
        assert_eq!(info.lapic_addr, 0xFEE00000);
    }
    
    #[test]
    fn test_acpi_get_apic_id() {
        let mut info = AcpiInfo::new();
        info.parse_madt().unwrap();
        
        assert_eq!(info.get_apic_id(CpuId::new(0)), Some(0));
        assert_eq!(info.get_apic_id(CpuId::new(1)), None);
    }
}
