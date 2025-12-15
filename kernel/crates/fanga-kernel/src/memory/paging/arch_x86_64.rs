//! x86_64 Page Table Structures
//!
//! This module provides x86_64-specific page table structures and operations.

use crate::memory::addr::PAGE_SIZE;

/// Page table entry flags
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageTableFlags(u64);

impl PageTableFlags {
    /// Entry is present in memory
    pub const PRESENT: Self = Self(1 << 0);
    /// Page is writable (if not set, page is read-only)
    pub const WRITABLE: Self = Self(1 << 1);
    /// Page is accessible from user mode
    pub const USER: Self = Self(1 << 2);
    /// Write-through caching
    pub const WRITE_THROUGH: Self = Self(1 << 3);
    /// Disable cache for this page
    pub const NO_CACHE: Self = Self(1 << 4);
    /// Page has been accessed
    pub const ACCESSED: Self = Self(1 << 5);
    /// Page has been written to (dirty)
    pub const DIRTY: Self = Self(1 << 6);
    /// Huge page (2 MiB or 1 GiB depending on level)
    pub const HUGE_PAGE: Self = Self(1 << 7);
    /// Page won't be flushed from TLB on CR3 reload
    pub const GLOBAL: Self = Self(1 << 8);
    /// Copy-on-Write (uses available bit 9)
    pub const COPY_ON_WRITE: Self = Self(1 << 9);
    /// Disable execution (NX bit, requires EFER.NXE)
    pub const NO_EXECUTE: Self = Self(1 << 63);

    /// Creates an empty flags set
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Checks if the flags contain the given flag
    pub const fn contains(&self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }

    /// Adds a flag to the set
    pub const fn with(self, flag: Self) -> Self {
        Self(self.0 | flag.0)
    }

    /// Removes a flag from the set
    pub const fn without(self, flag: Self) -> Self {
        Self(self.0 & !flag.0)
    }

    /// Gets the raw flags value
    pub const fn bits(&self) -> u64 {
        self.0
    }
}

/// A page table entry (8 bytes)
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Physical address mask (bits 12-51)
    const ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

    /// Creates a new null entry
    pub const fn new() -> Self {
        Self(0)
    }

    /// Checks if the entry is present
    pub fn is_present(&self) -> bool {
        self.flags().contains(PageTableFlags::PRESENT)
    }

    /// Gets the physical address this entry points to
    pub fn addr(&self) -> u64 {
        self.0 & Self::ADDR_MASK
    }

    /// Gets the flags for this entry
    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags(self.0 & !Self::ADDR_MASK)
    }

    /// Sets the entry to map to a physical address with flags
    pub fn set(&mut self, addr: u64, flags: PageTableFlags) {
        // Ensure address is page-aligned
        debug_assert!(addr % PAGE_SIZE as u64 == 0);
        self.0 = (addr & Self::ADDR_MASK) | flags.bits();
    }

    /// Clears the entry
    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

/// A page table (512 entries = 4 KiB)
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    /// Creates a new empty page table
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); 512],
        }
    }

    /// Gets a reference to an entry
    pub fn entry(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }

    /// Gets a mutable reference to an entry
    pub fn entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }

    /// Clears all entries
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            entry.clear();
        }
    }
}

/// Extracts the PML4 index from a virtual address
#[inline]
pub fn pml4_index(virt_addr: u64) -> usize {
    ((virt_addr >> 39) & 0x1FF) as usize
}

/// Extracts the PDPT index from a virtual address
#[inline]
pub fn pdpt_index(virt_addr: u64) -> usize {
    ((virt_addr >> 30) & 0x1FF) as usize
}

/// Extracts the PD index from a virtual address
#[inline]
pub fn pd_index(virt_addr: u64) -> usize {
    ((virt_addr >> 21) & 0x1FF) as usize
}

/// Extracts the PT index from a virtual address
#[inline]
pub fn pt_index(virt_addr: u64) -> usize {
    ((virt_addr >> 12) & 0x1FF) as usize
}

/// Extracts the page offset from a virtual address
#[inline]
pub fn page_offset(virt_addr: u64) -> u64 {
    virt_addr & 0xFFF
}

