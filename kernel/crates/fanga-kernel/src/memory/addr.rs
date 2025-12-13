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
