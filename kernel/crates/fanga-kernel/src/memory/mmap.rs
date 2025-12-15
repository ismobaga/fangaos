//! Memory Mapping (mmap/munmap) Support
//!
//! This module implements memory mapping functionality similar to POSIX mmap/munmap.
//! It allows processes to map virtual memory regions to physical memory or files.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use super::addr::{VirtAddr, PhysAddr, PAGE_SIZE, align_up, align_down};

/// Memory mapping flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MmapFlags(u32);

impl MmapFlags {
    /// No special flags
    pub const NONE: Self = Self(0);
    /// Changes are shared
    pub const SHARED: Self = Self(1 << 0);
    /// Changes are private (copy-on-write)
    pub const PRIVATE: Self = Self(1 << 1);
    /// Don't use a file descriptor (anonymous mapping)
    pub const ANONYMOUS: Self = Self(1 << 5);
    /// Place mapping at exact address
    pub const FIXED: Self = Self(1 << 4);

    /// Create empty flags
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Check if flags contain a specific flag
    pub const fn contains(&self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }

    /// Add a flag
    pub const fn with(self, flag: Self) -> Self {
        Self(self.0 | flag.0)
    }

    /// Get raw value
    pub const fn bits(&self) -> u32 {
        self.0
    }

    /// Create from raw bits
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
}

/// Memory protection flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MmapProt(u32);

impl MmapProt {
    /// Page cannot be accessed
    pub const NONE: Self = Self(0);
    /// Page can be read
    pub const READ: Self = Self(1 << 0);
    /// Page can be written
    pub const WRITE: Self = Self(1 << 1);
    /// Page can be executed
    pub const EXEC: Self = Self(1 << 2);

    /// Create empty protection
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Check if protection contains a specific flag
    pub const fn contains(&self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }

    /// Add a protection flag
    pub const fn with(self, flag: Self) -> Self {
        Self(self.0 | flag.0)
    }

    /// Get raw value
    pub const fn bits(&self) -> u32 {
        self.0
    }

    /// Create from raw bits
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
}

/// A memory mapped region
#[derive(Debug, Clone)]
pub struct MemoryMapping {
    /// Start virtual address
    pub start: VirtAddr,
    /// Size in bytes
    pub size: usize,
    /// Protection flags
    pub prot: MmapProt,
    /// Mapping flags
    pub flags: MmapFlags,
    /// Physical pages backing this mapping (for anonymous mappings)
    pub phys_pages: Vec<PhysAddr>,
}

impl MemoryMapping {
    /// Create a new memory mapping
    pub fn new(start: VirtAddr, size: usize, prot: MmapProt, flags: MmapFlags) -> Self {
        Self {
            start,
            size,
            prot,
            flags,
            phys_pages: Vec::new(),
        }
    }

    /// Get the end address of this mapping
    pub fn end(&self) -> VirtAddr {
        VirtAddr::new(self.start.as_u64() + self.size as u64)
    }

    /// Check if an address is within this mapping
    pub fn contains(&self, addr: VirtAddr) -> bool {
        let addr_val = addr.as_u64();
        let start_val = self.start.as_u64();
        let end_val = self.end().as_u64();
        addr_val >= start_val && addr_val < end_val
    }

    /// Add a physical page to this mapping
    pub fn add_phys_page(&mut self, phys: PhysAddr) {
        self.phys_pages.push(phys);
    }
}

/// Memory mapping manager for a process
pub struct MmapManager {
    /// All memory mappings for this process
    mappings: BTreeMap<u64, MemoryMapping>,
    /// Next available virtual address for automatic placement
    next_addr: u64,
}

impl MmapManager {
    /// Create a new mmap manager
    ///
    /// # Arguments
    /// * `start_addr` - Starting address for mmap allocations (typically user space start)
    pub fn new(start_addr: u64) -> Self {
        Self {
            mappings: BTreeMap::new(),
            next_addr: start_addr,
        }
    }

    /// Map a memory region
    ///
    /// # Arguments
    /// * `addr` - Requested virtual address (or 0 for automatic)
    /// * `length` - Size in bytes
    /// * `prot` - Protection flags
    /// * `flags` - Mapping flags
    ///
    /// # Returns
    /// Virtual address of the mapping, or None on error
    pub fn mmap(
        &mut self,
        addr: u64,
        length: usize,
        prot: MmapProt,
        flags: MmapFlags,
    ) -> Option<VirtAddr> {
        if length == 0 {
            return None;
        }

        // Align length to page boundary
        let aligned_length = align_up(length as u64, PAGE_SIZE as u64) as usize;

        // Determine the virtual address
        let virt_addr = if addr == 0 || !flags.contains(MmapFlags::FIXED) {
            // Automatic placement
            let va = self.next_addr;
            self.next_addr = align_up(self.next_addr + aligned_length as u64, PAGE_SIZE as u64);
            va
        } else {
            // Fixed address - align down to page boundary
            align_down(addr, PAGE_SIZE as u64)
        };

        // Check for overlaps
        if self.check_overlap(virt_addr, aligned_length) {
            return None;
        }

        // Create the mapping
        let mapping = MemoryMapping::new(
            VirtAddr::new(virt_addr),
            aligned_length,
            prot,
            flags,
        );

        self.mappings.insert(virt_addr, mapping);

        Some(VirtAddr::new(virt_addr))
    }

    /// Unmap a memory region
    ///
    /// # Arguments
    /// * `addr` - Virtual address of the mapping
    /// * `length` - Size in bytes
    ///
    /// # Returns
    /// true on success
    pub fn munmap(&mut self, addr: u64, length: usize) -> bool {
        if length == 0 {
            return false;
        }

        let aligned_addr = align_down(addr, PAGE_SIZE as u64);
        let aligned_length = align_up(length as u64, PAGE_SIZE as u64) as usize;

        // Find mappings that overlap with the region to unmap
        let mut to_remove = Vec::new();
        for (&start, mapping) in &self.mappings {
            let mapping_start = start;
            let mapping_end = start + mapping.size as u64;
            let unmap_end = aligned_addr + aligned_length as u64;

            // Check for overlap
            if aligned_addr < mapping_end && unmap_end > mapping_start {
                to_remove.push(start);
            }
        }

        // Remove the mappings
        for addr in to_remove.iter() {
            self.mappings.remove(addr);
        }

        // Return true if we successfully removed any mappings
        !to_remove.is_empty()
    }

    /// Find a mapping containing the given address
    ///
    /// # Arguments
    /// * `addr` - Virtual address to look up
    ///
    /// # Returns
    /// Reference to the mapping, or None if not found
    pub fn find_mapping(&self, addr: VirtAddr) -> Option<&MemoryMapping> {
        let addr_val = addr.as_u64();
        for mapping in self.mappings.values() {
            if mapping.contains(addr) {
                return Some(mapping);
            }
        }
        None
    }

    /// Get a mutable reference to a mapping
    ///
    /// # Arguments
    /// * `addr` - Virtual address within the mapping
    ///
    /// # Returns
    /// Mutable reference to the mapping, or None if not found
    pub fn find_mapping_mut(&mut self, addr: VirtAddr) -> Option<&mut MemoryMapping> {
        let addr_val = addr.as_u64();
        for mapping in self.mappings.values_mut() {
            if mapping.contains(addr) {
                return Some(mapping);
            }
        }
        None
    }

    /// Check if a region overlaps with existing mappings
    fn check_overlap(&self, addr: u64, length: usize) -> bool {
        let end = addr + length as u64;
        for mapping in self.mappings.values() {
            let mapping_start = mapping.start.as_u64();
            let mapping_end = mapping.end().as_u64();
            if addr < mapping_end && end > mapping_start {
                return true;
            }
        }
        false
    }

    /// Get all mappings
    pub fn mappings(&self) -> impl Iterator<Item = &MemoryMapping> {
        self.mappings.values()
    }

    /// Get the number of mappings
    pub fn count(&self) -> usize {
        self.mappings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mmap_flags() {
        let flags = MmapFlags::PRIVATE.with(MmapFlags::ANONYMOUS);
        assert!(flags.contains(MmapFlags::PRIVATE));
        assert!(flags.contains(MmapFlags::ANONYMOUS));
        assert!(!flags.contains(MmapFlags::SHARED));
    }

    #[test]
    fn test_mmap_prot() {
        let prot = MmapProt::READ.with(MmapProt::WRITE);
        assert!(prot.contains(MmapProt::READ));
        assert!(prot.contains(MmapProt::WRITE));
        assert!(!prot.contains(MmapProt::EXEC));
    }

    #[test]
    fn test_memory_mapping() {
        let start = VirtAddr::new(0x10000);
        let size = 0x2000;
        let mapping = MemoryMapping::new(
            start,
            size,
            MmapProt::READ.with(MmapProt::WRITE),
            MmapFlags::PRIVATE.with(MmapFlags::ANONYMOUS),
        );

        assert_eq!(mapping.start, start);
        assert_eq!(mapping.size, size);
        assert!(mapping.contains(VirtAddr::new(0x10000)));
        assert!(mapping.contains(VirtAddr::new(0x11000)));
        assert!(!mapping.contains(VirtAddr::new(0x12000)));
    }

    #[test]
    fn test_mmap_manager_automatic() {
        let mut manager = MmapManager::new(0x1000_0000);

        // Automatic placement
        let addr1 = manager.mmap(
            0,
            0x1000,
            MmapProt::READ.with(MmapProt::WRITE),
            MmapFlags::PRIVATE.with(MmapFlags::ANONYMOUS),
        );
        assert!(addr1.is_some());

        let addr2 = manager.mmap(
            0,
            0x1000,
            MmapProt::READ,
            MmapFlags::PRIVATE.with(MmapFlags::ANONYMOUS),
        );
        assert!(addr2.is_some());

        // Should be different addresses
        assert_ne!(addr1.unwrap().as_u64(), addr2.unwrap().as_u64());
    }

    #[test]
    fn test_mmap_manager_fixed() {
        let mut manager = MmapManager::new(0x1000_0000);

        // Fixed address
        let addr = manager.mmap(
            0x2000_0000,
            0x1000,
            MmapProt::READ.with(MmapProt::WRITE),
            MmapFlags::PRIVATE.with(MmapFlags::ANONYMOUS).with(MmapFlags::FIXED),
        );
        assert_eq!(addr.unwrap().as_u64(), 0x2000_0000);

        // Overlapping fixed address should fail
        let addr2 = manager.mmap(
            0x2000_0000,
            0x1000,
            MmapProt::READ,
            MmapFlags::PRIVATE.with(MmapFlags::ANONYMOUS).with(MmapFlags::FIXED),
        );
        assert!(addr2.is_none());
    }

    #[test]
    fn test_munmap() {
        let mut manager = MmapManager::new(0x1000_0000);

        let addr = manager.mmap(
            0,
            0x1000,
            MmapProt::READ.with(MmapProt::WRITE),
            MmapFlags::PRIVATE.with(MmapFlags::ANONYMOUS),
        ).unwrap();

        assert_eq!(manager.count(), 1);

        // Unmap
        let result = manager.munmap(addr.as_u64(), 0x1000);
        assert!(result);
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_find_mapping() {
        let mut manager = MmapManager::new(0x1000_0000);

        let addr = manager.mmap(
            0,
            0x2000,
            MmapProt::READ.with(MmapProt::WRITE),
            MmapFlags::PRIVATE.with(MmapFlags::ANONYMOUS),
        ).unwrap();

        // Should find mapping
        assert!(manager.find_mapping(addr).is_some());
        assert!(manager.find_mapping(VirtAddr::new(addr.as_u64() + 0x1000)).is_some());
        
        // Should not find mapping outside range
        assert!(manager.find_mapping(VirtAddr::new(addr.as_u64() + 0x3000)).is_none());
    }
}
