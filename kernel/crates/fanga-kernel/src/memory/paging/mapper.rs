//! Page Table Mapper
//!
//! This module provides high-level page table mapping operations.

use crate::memory::pmm::PhysicalMemoryManager;
use crate::memory::addr::PAGE_SIZE;
use super::arch_x86_64::{
    PageTable, PageTableEntry, PageTableFlags,
    pml4_index, pdpt_index, pd_index, pt_index,
};

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
    pub unsafe fn new(pmm: &PhysicalMemoryManager, hhdm_offset: u64) -> Option<Self> {
        // Allocate a page for PML4
        let pml4_phys = pmm.alloc_page()?;
        
        // Clear the PML4
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
    #[inline]
    fn phys_to_virt(&self, phys: u64) -> u64 {
        self.hhdm_offset + phys
    }

    /// Gets a mutable reference to the PML4
    unsafe fn pml4_mut(&mut self) -> &mut PageTable {
        &mut *(self.phys_to_virt(self.pml4_phys) as *mut PageTable)
    }

    /// Gets or creates a page table entry
    unsafe fn get_or_create_table(
        &mut self,
        entry: &mut PageTableEntry,
        pmm: &PhysicalMemoryManager,
    ) -> Option<*mut PageTable> {
        if !entry.is_present() {
            let phys = pmm.alloc_page()?;
            let virt = self.phys_to_virt(phys) as *mut PageTable;
            (*virt).clear();

            entry.set(
                phys,
                PageTableFlags::PRESENT
                    .with(PageTableFlags::WRITABLE)
                    .with(PageTableFlags::USER),
            );
        }

        let table_phys = entry.addr();
        let table_virt = self.phys_to_virt(table_phys) as *mut PageTable;
        Some(table_virt)
    }

    /// Maps a virtual address to a physical address
    pub unsafe fn map(
        &mut self,
        virt_addr: u64,
        phys_addr: u64,
        flags: PageTableFlags,
        pmm: &PhysicalMemoryManager,
    ) -> Result<(), &'static str> {
        if virt_addr % PAGE_SIZE as u64 != 0 || phys_addr % PAGE_SIZE as u64 != 0 {
            return Err("Addresses must be page-aligned");
        }

        let pml4_idx = pml4_index(virt_addr);
        let pdpt_idx = pdpt_index(virt_addr);
        let pd_idx = pd_index(virt_addr);
        let pt_idx = pt_index(virt_addr);

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

        let entry = (*pt).entry_mut(pt_idx);
        if entry.is_present() {
            return Err("Virtual address already mapped");
        }

        entry.set(phys_addr, flags.with(PageTableFlags::PRESENT));

        Self::flush_tlb(virt_addr);

        Ok(())
    }

    /// Unmaps a virtual address
    pub unsafe fn unmap(&mut self, virt_addr: u64) -> Result<u64, &'static str> {
        if virt_addr % PAGE_SIZE as u64 != 0 {
            return Err("Address must be page-aligned");
        }

        let pml4_idx = pml4_index(virt_addr);
        let pdpt_idx = pdpt_index(virt_addr);
        let pd_idx = pd_index(virt_addr);
        let pt_idx = pt_index(virt_addr);

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

        Self::flush_tlb(virt_addr);

        Ok(phys_addr)
    }

    /// Translates a virtual address to a physical address
    pub fn translate(&self, virt_addr: u64) -> Option<u64> {
        let pml4_idx = pml4_index(virt_addr);
        let pdpt_idx = pdpt_index(virt_addr);
        let pd_idx = pd_index(virt_addr);
        let pt_idx = pt_index(virt_addr);
        let offset = virt_addr & 0xFFF;

        unsafe {
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
    #[inline]
    fn flush_tlb(virt_addr: u64) {
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) virt_addr, options(nostack, preserves_flags));
        }
    }

    /// Loads this page table into CR3
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
