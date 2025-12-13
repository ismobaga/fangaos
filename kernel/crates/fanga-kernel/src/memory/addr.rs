//! Physical and Virtual Address Types
//!
//! This module provides newtype wrappers for physical and virtual addresses
//! along with alignment utilities.

use core::fmt;

/// Page size constant (4 KiB)
pub const PAGE_SIZE: usize = 4096;

/// Physical address newtype
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysAddr(u64);

impl PhysAddr {
    /// Creates a new physical address
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Creates a physical address, truncating to canonical form if needed
    #[inline]
    pub const fn new_truncate(addr: u64) -> Self {
        Self(addr)
    }

    /// Returns the raw address value
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Checks if the address is page-aligned
    #[inline]
    pub const fn is_aligned(self, align: u64) -> bool {
        self.0 % align == 0
    }

    /// Aligns the address upwards
    #[inline]
    pub const fn align_up(self, align: u64) -> Self {
        Self(align_up(self.0, align))
    }

    /// Aligns the address downwards
    #[inline]
    pub const fn align_down(self, align: u64) -> Self {
        Self(align_down(self.0, align))
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysAddr(0x{:x})", self.0)
    }
}

impl From<u64> for PhysAddr {
    #[inline]
    fn from(addr: u64) -> Self {
        Self::new(addr)
    }
}

impl From<PhysAddr> for u64 {
    #[inline]
    fn from(addr: PhysAddr) -> Self {
        addr.as_u64()
    }
}

/// Virtual address newtype
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtAddr(u64);

impl VirtAddr {
    /// Creates a new virtual address
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Creates a virtual address, extending to canonical form
    #[inline]
    pub const fn new_truncate(addr: u64) -> Self {
        // Sign extend from bit 47
        let sign_bit = addr & (1 << 47);
        if sign_bit != 0 {
            Self(addr | 0xFFFF_0000_0000_0000)
        } else {
            Self(addr)
        }
    }

    /// Returns the raw address value
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Checks if the address is canonical
    #[inline]
    pub const fn is_canonical(self) -> bool {
        // Bits 48-63 must be same as bit 47
        let bit_47 = (self.0 >> 47) & 1;
        let upper_bits = (self.0 >> 48) & 0xFFFF;
        (bit_47 == 0 && upper_bits == 0) || (bit_47 == 1 && upper_bits == 0xFFFF)
    }

    /// Checks if the address is page-aligned
    #[inline]
    pub const fn is_aligned(self, align: u64) -> bool {
        self.0 % align == 0
    }

    /// Aligns the address upwards
    #[inline]
    pub const fn align_up(self, align: u64) -> Self {
        Self(align_up(self.0, align))
    }

    /// Aligns the address downwards
    #[inline]
    pub const fn align_down(self, align: u64) -> Self {
        Self(align_down(self.0, align))
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtAddr(0x{:x})", self.0)
    }
}

impl From<u64> for VirtAddr {
    #[inline]
    fn from(addr: u64) -> Self {
        Self::new(addr)
    }
}

impl From<VirtAddr> for u64 {
    #[inline]
    fn from(addr: VirtAddr) -> Self {
        addr.as_u64()
    }
}

/// Aligns a value upwards to the given alignment
#[inline]
pub const fn align_up(val: u64, align: u64) -> u64 {
    (val + (align - 1)) & !(align - 1)
}

/// Aligns a value downwards to the given alignment
#[inline]
pub const fn align_down(val: u64, align: u64) -> u64 {
    val & !(align - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phys_addr_creation() {
        let addr = PhysAddr::new(0x1000);
        assert_eq!(addr.as_u64(), 0x1000);
    }

    #[test]
    fn test_phys_addr_alignment() {
        let addr = PhysAddr::new(0x1234);
        assert!(!addr.is_aligned(PAGE_SIZE as u64));
        
        let aligned_up = addr.align_up(PAGE_SIZE as u64);
        assert!(aligned_up.is_aligned(PAGE_SIZE as u64));
        assert_eq!(aligned_up.as_u64(), 0x2000);
        
        let aligned_down = addr.align_down(PAGE_SIZE as u64);
        assert!(aligned_down.is_aligned(PAGE_SIZE as u64));
        assert_eq!(aligned_down.as_u64(), 0x1000);
    }

    #[test]
    fn test_virt_addr_creation() {
        let addr = VirtAddr::new(0x1000);
        assert_eq!(addr.as_u64(), 0x1000);
    }

    #[test]
    fn test_virt_addr_canonical() {
        // Lower half addresses (canonical)
        let addr1 = VirtAddr::new(0x0000_7FFF_FFFF_FFFF);
        assert!(addr1.is_canonical());
        
        // Higher half addresses (canonical)
        let addr2 = VirtAddr::new(0xFFFF_8000_0000_0000);
        assert!(addr2.is_canonical());
        
        // Non-canonical address
        let addr3 = VirtAddr::new(0x0000_8000_0000_0000);
        assert!(!addr3.is_canonical());
    }

    #[test]
    fn test_virt_addr_truncate() {
        // Test sign extension from bit 47
        let addr1 = VirtAddr::new_truncate(0x0000_0000_0000_1000);
        assert_eq!(addr1.as_u64(), 0x0000_0000_0000_1000);
        
        let addr2 = VirtAddr::new_truncate(0x0000_8000_0000_0000);
        assert_eq!(addr2.as_u64(), 0xFFFF_8000_0000_0000);
    }

    #[test]
    fn test_virt_addr_alignment() {
        let addr = VirtAddr::new(0x5678);
        assert!(!addr.is_aligned(PAGE_SIZE as u64));
        
        let aligned_up = addr.align_up(PAGE_SIZE as u64);
        assert!(aligned_up.is_aligned(PAGE_SIZE as u64));
        assert_eq!(aligned_up.as_u64(), 0x6000);
        
        let aligned_down = addr.align_down(PAGE_SIZE as u64);
        assert!(aligned_down.is_aligned(PAGE_SIZE as u64));
        assert_eq!(aligned_down.as_u64(), 0x5000);
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4096), 0);
        assert_eq!(align_up(1, 4096), 4096);
        assert_eq!(align_up(4095, 4096), 4096);
        assert_eq!(align_up(4096, 4096), 4096);
        assert_eq!(align_up(4097, 4096), 8192);
    }

    #[test]
    fn test_align_down() {
        assert_eq!(align_down(0, 4096), 0);
        assert_eq!(align_down(1, 4096), 0);
        assert_eq!(align_down(4095, 4096), 0);
        assert_eq!(align_down(4096, 4096), 4096);
        assert_eq!(align_down(4097, 4096), 4096);
        assert_eq!(align_down(8192, 4096), 8192);
    }

    #[test]
    fn test_addr_conversions() {
        let phys: PhysAddr = 0x1000u64.into();
        assert_eq!(phys.as_u64(), 0x1000);
        
        let val: u64 = phys.into();
        assert_eq!(val, 0x1000);
        
        let virt: VirtAddr = 0x2000u64.into();
        assert_eq!(virt.as_u64(), 0x2000);
        
        let val: u64 = virt.into();
        assert_eq!(val, 0x2000);
    }
}
