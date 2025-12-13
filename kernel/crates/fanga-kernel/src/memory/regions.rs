//! Memory Regions Management
//!
//! This module manages different types of memory regions in the kernel:
//! - Kernel code and data
//! - User space
//! - Memory-Mapped I/O (MMIO)
//! - Available physical memory
//!
//! It provides utilities to classify addresses and ensure proper
//! access control and permissions for different memory regions.

use core::fmt;

/// Memory region type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    /// Kernel code (read-only, executable)
    KernelCode,
    /// Kernel data (read-write, non-executable)
    KernelData,
    /// Kernel stack (read-write, non-executable)
    KernelStack,
    /// Kernel heap (read-write, non-executable)
    KernelHeap,
    /// User space (varies by page)
    UserSpace,
    /// Memory-Mapped I/O (read-write, non-cacheable)
    MMIO,
    /// Available physical memory (not mapped)
    Available,
    /// Reserved memory (bootloader, firmware, etc.)
    Reserved,
}

impl fmt::Display for MemoryRegionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KernelCode => write!(f, "Kernel Code"),
            Self::KernelData => write!(f, "Kernel Data"),
            Self::KernelStack => write!(f, "Kernel Stack"),
            Self::KernelHeap => write!(f, "Kernel Heap"),
            Self::UserSpace => write!(f, "User Space"),
            Self::MMIO => write!(f, "MMIO"),
            Self::Available => write!(f, "Available"),
            Self::Reserved => write!(f, "Reserved"),
        }
    }
}

/// A memory region with start address, size, and type
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
    pub region_type: MemoryRegionType,
}

impl MemoryRegion {
    /// Creates a new memory region
    pub const fn new(start: u64, size: u64, region_type: MemoryRegionType) -> Self {
        Self {
            start,
            size,
            region_type,
        }
    }

    /// Returns the end address of the region (exclusive)
    pub const fn end(&self) -> u64 {
        self.start + self.size
    }

    /// Checks if the region contains the given address
    pub const fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr < self.end()
    }

    /// Checks if the region overlaps with another region
    pub const fn overlaps(&self, other: &MemoryRegion) -> bool {
        self.start < other.end() && other.start < self.end()
    }
}

impl fmt::Display for MemoryRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[0x{:016x} - 0x{:016x}] {} ({} KiB)",
            self.start,
            self.end(),
            self.region_type,
            self.size / 1024
        )
    }
}

/// Memory region manager
pub struct MemoryRegionManager {
    regions: [Option<MemoryRegion>; 32],
    count: usize,
}

impl MemoryRegionManager {
    /// Creates a new memory region manager
    pub const fn new() -> Self {
        Self {
            regions: [None; 32],
            count: 0,
        }
    }

    /// Adds a memory region to the manager
    ///
    /// Returns true if the region was added successfully, false if the
    /// region array is full.
    pub fn add_region(&mut self, region: MemoryRegion) -> bool {
        if self.count >= self.regions.len() {
            return false;
        }

        self.regions[self.count] = Some(region);
        self.count += 1;
        true
    }

    /// Finds the memory region containing the given address
    pub fn find_region(&self, addr: u64) -> Option<&MemoryRegion> {
        for i in 0..self.count {
            if let Some(ref region) = self.regions[i] {
                if region.contains(addr) {
                    return Some(region);
                }
            }
        }
        None
    }

    /// Returns an iterator over all memory regions
    pub fn iter(&self) -> impl Iterator<Item = &MemoryRegion> {
        self.regions[..self.count]
            .iter()
            .filter_map(|r| r.as_ref())
    }

    /// Returns the total size of all regions of a given type
    pub fn total_size_by_type(&self, region_type: MemoryRegionType) -> u64 {
        self.iter()
            .filter(|r| r.region_type == region_type)
            .map(|r| r.size)
            .sum()
    }

    /// Returns the number of regions of a given type
    pub fn count_by_type(&self, region_type: MemoryRegionType) -> usize {
        self.iter()
            .filter(|r| r.region_type == region_type)
            .count()
    }
}

/// Virtual memory address spaces
pub mod address_space {
    /// Higher Half Direct Map (HHDM) start address
    /// This is typically set by the bootloader (Limine)
    pub const HHDM_START: u64 = 0xFFFF_8000_0000_0000;

    /// Kernel space start (higher half)
    pub const KERNEL_SPACE_START: u64 = 0xFFFF_FFFF_8000_0000;

    /// User space end (lower half)
    pub const USER_SPACE_END: u64 = 0x0000_8000_0000_0000;

    /// Checks if an address is in kernel space
    pub const fn is_kernel_space(addr: u64) -> bool {
        addr >= KERNEL_SPACE_START
    }

    /// Checks if an address is in user space
    pub const fn is_user_space(addr: u64) -> bool {
        addr < USER_SPACE_END
    }

    /// Checks if an address is in the HHDM region
    pub const fn is_hhdm(addr: u64, hhdm_offset: u64) -> bool {
        addr >= hhdm_offset && addr < KERNEL_SPACE_START
    }

    /// Converts a physical address to HHDM virtual address
    pub const fn phys_to_hhdm(phys: u64, hhdm_offset: u64) -> u64 {
        hhdm_offset + phys
    }

    /// Converts an HHDM virtual address to physical address
    pub const fn hhdm_to_phys(virt: u64, hhdm_offset: u64) -> u64 {
        virt - hhdm_offset
    }
}

/// Memory access permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl MemoryPermissions {
    /// Read-only, non-executable
    pub const RO: Self = Self {
        read: true,
        write: false,
        execute: false,
    };

    /// Read-write, non-executable
    pub const RW: Self = Self {
        read: true,
        write: true,
        execute: false,
    };

    /// Read-execute, non-writable
    pub const RX: Self = Self {
        read: true,
        write: false,
        execute: true,
    };

    /// Read-write-execute (generally unsafe)
    pub const RWX: Self = Self {
        read: true,
        write: true,
        execute: true,
    };

    /// Returns the recommended permissions for a region type
    pub const fn for_region_type(region_type: MemoryRegionType) -> Self {
        match region_type {
            MemoryRegionType::KernelCode => Self::RX,
            MemoryRegionType::KernelData
            | MemoryRegionType::KernelStack
            | MemoryRegionType::KernelHeap => Self::RW,
            MemoryRegionType::MMIO => Self::RW,
            MemoryRegionType::UserSpace => Self::RW,
            MemoryRegionType::Available | MemoryRegionType::Reserved => Self::RO,
        }
    }
}

impl fmt::Display for MemoryPermissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            if self.read { "R" } else { "-" },
            if self.write { "W" } else { "-" },
            if self.execute { "X" } else { "-" }
        )
    }
}
