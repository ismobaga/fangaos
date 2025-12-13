//! Virtual Memory Manager (VMM)
//!
//! This module manages virtual memory through x86_64 page tables.
//! It provides functions to map/unmap virtual addresses to physical addresses
//! and manipulate page tables.
//!
//! # x86_64 Paging Structure
//!
//! x86_64 uses 4-level paging:
//! - PML4 (Page Map Level 4) - Top level
//! - PDPT (Page Directory Pointer Table)
//! - PD (Page Directory)
//! - PT (Page Table)
//!
//! Each table has 512 entries, and each page is 4 KiB.
//!
//! # Virtual Address Layout
//!
//! ```text
//! 63:48 - Sign extension (must match bit 47)
//! 47:39 - PML4 index (9 bits = 512 entries)
//! 38:30 - PDPT index (9 bits = 512 entries)
//! 29:21 - PD index (9 bits = 512 entries)
//! 20:12 - PT index (9 bits = 512 entries)
//! 11:0  - Page offset (12 bits = 4096 bytes)
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! // Create a new page table
//! let mut mapper = PageTableMapper::new(&mut PMM, hhdm_offset);
//!
//! // Map a virtual address to a physical address
//! mapper.map(virt_addr, phys_addr, flags);
//!
//! // Unmap a virtual address
//! mapper.unmap(virt_addr);
//! ```

use crate::pmm::{PhysicalMemoryManager, PAGE_SIZE};

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

/// Page table mapper - manages page tables using PMM for allocation
pub struct PageTableMapper {
    /// Pointer to the PML4 table (physical address)
    pml4_phys: u64,
    /// Higher Half Direct Map offset
    hhdm_offset: u64,
}

impl PageTableMapper {
    /// Creates a new page table mapper with a new PML4
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - The PMM is properly initialized and can allocate pages
    /// - The HHDM offset is valid and maps to accessible kernel memory
    /// - The allocated page is properly cleared and won't be accessed by other code
    pub unsafe fn new(pmm: &mut PhysicalMemoryManager, hhdm_offset: u64) -> Option<Self> {
        // Allocate a page for PML4
        let pml4_phys = pmm.alloc_page()?;
        
        // Clear the PML4
        // SAFETY: The physical address is valid (just allocated by PMM) and
        // the HHDM offset ensures it maps to a valid kernel virtual address.
        // The pointer is properly aligned (PMM returns page-aligned addresses)
        // and points to valid memory.
        let pml4_virt = (hhdm_offset + pml4_phys) as *mut PageTable;
        (*pml4_virt).clear();

        Some(Self {
            pml4_phys,
            hhdm_offset,
        })
    }

    /// Creates a mapper from an existing PML4 (e.g., the current CR3)
    pub fn from_pml4(pml4_phys: u64, hhdm_offset: u64) -> Self {
        Self {
            pml4_phys,
            hhdm_offset,
        }
    }

    /// Gets the physical address of the PML4
    pub fn pml4_addr(&self) -> u64 {
        self.pml4_phys
    }

    /// Converts a physical address to a virtual address using HHDM
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - The physical address is valid
    /// - The HHDM offset is correctly set
    /// - The resulting virtual address is within valid kernel space
    #[inline]
    fn phys_to_virt(&self, phys: u64) -> u64 {
        self.hhdm_offset + phys
    }

    /// Gets a mutable reference to the PML4
    unsafe fn pml4_mut(&mut self) -> &mut PageTable {
        &mut *(self.phys_to_virt(self.pml4_phys) as *mut PageTable)
    }

    /// Gets or creates a page table entry
    ///
    /// Returns a mutable pointer to the next level page table
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - The entry pointer is valid
    /// - No other mutable references to the same page table exist
    /// - The PMM is properly initialized and can allocate pages
    /// - The physical addresses returned by PMM are valid
    unsafe fn get_or_create_table(
        &mut self,
        entry: &mut PageTableEntry,
        pmm: &mut PhysicalMemoryManager,
    ) -> Option<*mut PageTable> {
        if !entry.is_present() {
            // Allocate a new page table
            let phys = pmm.alloc_page()?;
            
            // Convert physical to virtual address using HHDM
            // SAFETY: The physical address is valid (just allocated by PMM)
            // and the HHDM offset ensures it maps to a valid kernel virtual address
            let virt = self.phys_to_virt(phys) as *mut PageTable;
            (*virt).clear();

            // Set the entry with default flags
            entry.set(
                phys,
                PageTableFlags::PRESENT
                    .with(PageTableFlags::WRITABLE)
                    .with(PageTableFlags::USER),
            );
        }

        let table_phys = entry.addr();
        // SAFETY: The physical address comes from a present page table entry
        // which means it was previously validated when the entry was created
        let table_virt = self.phys_to_virt(table_phys) as *mut PageTable;
        Some(table_virt)
    }

    /// Maps a virtual address to a physical address
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address to map (must be page-aligned)
    /// * `phys_addr` - Physical address to map to (must be page-aligned)
    /// * `flags` - Page flags
    /// * `pmm` - Physical memory manager for allocating page tables
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - Both addresses are page-aligned
    /// - The physical address is valid and accessible
    /// - The PMM is properly initialized
    /// - The mapping won't conflict with existing kernel mappings
    /// - No other references to the page tables exist during this operation
    ///
    /// # Aliasing and Borrowing
    /// This function uses raw pointers to traverse the page table hierarchy.
    /// While this creates temporary aliasing, it's safe because:
    /// 1. We only hold each pointer briefly during traversal
    /// 2. Each pointer points to a different page table level
    /// 3. The page tables are not accessed concurrently
    /// 4. The function is marked unsafe and documented
    pub unsafe fn map(
        &mut self,
        virt_addr: u64,
        phys_addr: u64,
        flags: PageTableFlags,
        pmm: &mut PhysicalMemoryManager,
    ) -> Result<(), &'static str> {
        // Ensure addresses are page-aligned
        if virt_addr % PAGE_SIZE as u64 != 0 || phys_addr % PAGE_SIZE as u64 != 0 {
            return Err("Addresses must be page-aligned");
        }

        // Get indices
        let pml4_idx = pml4_index(virt_addr);
        let pdpt_idx = pdpt_index(virt_addr);
        let pd_idx = pd_index(virt_addr);
        let pt_idx = pt_index(virt_addr);

        // Walk the page table hierarchy, creating tables as needed
        // SAFETY: We create raw pointers to avoid overlapping mutable borrows,
        // but we ensure that each pointer is only dereferenced when no other
        // mutable references to the same memory exist.
        let pml4 = self.pml4_mut();
        let pml4_entry_ptr = pml4.entry_mut(pml4_idx) as *mut PageTableEntry;
        
        let pdpt = self
            .get_or_create_table(&mut *pml4_entry_ptr, pmm)
            .ok_or("Failed to allocate PDPT")?;

        let pdpt_entry_ptr = (*pdpt).entry_mut(pdpt_idx) as *mut PageTableEntry;
        let pd = self
            .get_or_create_table(&mut *pdpt_entry_ptr, pmm)
            .ok_or("Failed to allocate PD")?;

        let pd_entry_ptr = (*pd).entry_mut(pd_idx) as *mut PageTableEntry;
        let pt = self
            .get_or_create_table(&mut *pd_entry_ptr, pmm)
            .ok_or("Failed to allocate PT")?;

        // Set the final page table entry
        let entry = (*pt).entry_mut(pt_idx);
        if entry.is_present() {
            return Err("Virtual address already mapped");
        }

        entry.set(phys_addr, flags.with(PageTableFlags::PRESENT));

        // Flush TLB for this address
        Self::flush_tlb(virt_addr);

        Ok(())
    }

    /// Unmaps a virtual address
    ///
    /// Returns the physical address that was unmapped
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - Unmapping this address is safe and won't break kernel functionality
    /// - No references to the unmapped memory exist
    /// - The virtual address is page-aligned
    pub unsafe fn unmap(&mut self, virt_addr: u64) -> Result<u64, &'static str> {
        // Ensure address is page-aligned
        if virt_addr % PAGE_SIZE as u64 != 0 {
            return Err("Address must be page-aligned");
        }

        // Get indices
        let pml4_idx = pml4_index(virt_addr);
        let pdpt_idx = pdpt_index(virt_addr);
        let pd_idx = pd_index(virt_addr);
        let pt_idx = pt_index(virt_addr);

        // Walk the page table hierarchy
        // SAFETY: We convert physical addresses to virtual using HHDM,
        // which is safe because the page tables are in the higher half direct map
        let pml4 = self.pml4_mut();
        if !pml4.entry(pml4_idx).is_present() {
            return Err("PML4 entry not present");
        }

        let pdpt_phys = pml4.entry(pml4_idx).addr();
        let pdpt = &mut *(self.phys_to_virt(pdpt_phys) as *mut PageTable);
        if !pdpt.entry(pdpt_idx).is_present() {
            return Err("PDPT entry not present");
        }

        let pd_phys = pdpt.entry(pdpt_idx).addr();
        let pd = &mut *(self.phys_to_virt(pd_phys) as *mut PageTable);
        if !pd.entry(pd_idx).is_present() {
            return Err("PD entry not present");
        }

        let pt_phys = pd.entry(pd_idx).addr();
        let pt = &mut *(self.phys_to_virt(pt_phys) as *mut PageTable);
        if !pt.entry(pt_idx).is_present() {
            return Err("PT entry not present");
        }

        let entry = pt.entry_mut(pt_idx);
        let phys_addr = entry.addr();
        entry.clear();

        // Flush TLB for this address
        Self::flush_tlb(virt_addr);

        Ok(phys_addr)
    }

    /// Translates a virtual address to a physical address
    ///
    /// Returns the physical address if the virtual address is mapped, or None otherwise
    ///
    /// # Safety
    /// This function performs unsafe operations:
    /// - Dereferences raw pointers to page tables
    /// - The physical addresses are converted to virtual using HHDM
    /// - Each pointer is only dereferenced after checking the present bit
    /// - The page table hierarchy must be valid and not concurrently modified
    pub fn translate(&self, virt_addr: u64) -> Option<u64> {
        // Get indices
        let pml4_idx = pml4_index(virt_addr);
        let pdpt_idx = pdpt_index(virt_addr);
        let pd_idx = pd_index(virt_addr);
        let pt_idx = pt_index(virt_addr);
        let offset = page_offset(virt_addr);

        unsafe {
            // Walk the page table hierarchy
            // SAFETY: Each pointer dereference is safe because:
            // 1. The physical address comes from a valid page table entry
            // 2. We check is_present() before dereferencing
            // 3. The HHDM mapping ensures valid virtual addresses
            // 4. Page tables are properly aligned (4KiB)
            let pml4 = &*(self.phys_to_virt(self.pml4_phys) as *const PageTable);
            if !pml4.entry(pml4_idx).is_present() {
                return None;
            }

            let pdpt_phys = pml4.entry(pml4_idx).addr();
            let pdpt = &*(self.phys_to_virt(pdpt_phys) as *const PageTable);
            if !pdpt.entry(pdpt_idx).is_present() {
                return None;
            }

            let pd_phys = pdpt.entry(pdpt_idx).addr();
            let pd = &*(self.phys_to_virt(pd_phys) as *const PageTable);
            if !pd.entry(pd_idx).is_present() {
                return None;
            }

            let pt_phys = pd.entry(pd_idx).addr();
            let pt = &*(self.phys_to_virt(pt_phys) as *const PageTable);
            if !pt.entry(pt_idx).is_present() {
                return None;
            }

            let page_phys = pt.entry(pt_idx).addr();
            Some(page_phys + offset)
        }
    }

    /// Flushes the TLB entry for a virtual address
    ///
    /// Uses the INVLPG instruction to invalidate the TLB entry for the given address.
    /// This ensures that subsequent accesses use the updated page table entry.
    ///
    /// # Safety
    /// This inline assembly is safe because:
    /// - INVLPG is a privileged instruction that only affects TLB entries
    /// - It doesn't modify any registers or flags
    /// - The instruction only takes the virtual address as input
    /// - The kernel is running in ring 0 with proper privileges
    #[inline]
    fn flush_tlb(virt_addr: u64) {
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) virt_addr, options(nostack, preserves_flags));
        }
    }

    /// Loads this page table into CR3
    ///
    /// # Safety
    /// The caller must ensure that the page tables are properly set up
    pub unsafe fn load(&self) {
        core::arch::asm!("mov cr3, {}", in(reg) self.pml4_phys, options(nostack, preserves_flags));
    }

    /// Gets the current CR3 value
    pub fn current_cr3() -> u64 {
        let cr3: u64;
        unsafe {
            core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
        }
        cr3
    }
}
