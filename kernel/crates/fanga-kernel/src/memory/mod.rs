//! Memory Management
//!
//! This module provides the kernel's memory management infrastructure including:
//! - Physical memory allocation (PMM)
//! - Virtual memory management (paging/VMM)
//! - Heap allocation
//! - Memory regions tracking
//! - Statistics and debugging
//! - Copy-on-Write (CoW)
//! - Memory mapping (mmap/munmap)
//! - Demand paging
//! - Page replacement (LRU)
//! - Swap support

pub mod addr;
pub mod pmm;
pub mod paging;
pub mod heap;
pub mod regions;
pub mod stats;
pub mod debug;
pub mod cow;
pub mod mmap;
pub mod demand_paging;
pub mod swap;

// Re-export commonly used types and functions
pub use addr::{PhysAddr, VirtAddr, PAGE_SIZE, align_up, align_down};
pub use pmm::PhysicalMemoryManager;
pub use paging::{PageTableMapper, PageTableFlags};
pub use heap::GlobalHeapAllocator;
pub use regions::{MemoryRegion, MemoryRegionType, MemoryRegionManager};
pub use cow::{mark_cow_page, release_cow_page, is_cow_page, get_cow_ref_count, add_cow_page};
pub use mmap::{MmapFlags, MmapProt, MemoryMapping, MmapManager};
pub use demand_paging::{PageState, record_page_access, get_lru_page, get_lru_stats,
                         reserve_demand_pages, allocate_demand_page, get_page_state,
                         should_allocate_on_fault, get_demand_paging_stats};
pub use swap::{init_swap, swap_out_page, swap_in_page, is_page_swapped, get_swap_stats, has_swap_space};

/// Initialize memory management
///
/// This should be called early in kernel initialization after the bootloader
/// has provided the memory map.
pub fn init_pmm(
    pmm: &mut PhysicalMemoryManager,
    memmap: &limine::response::MemoryMapResponse,
    hhdm_offset: u64,
) {
    unsafe {
        pmm.init(memmap, hhdm_offset);
    }
}

/// Allocate a physical frame (4 KiB page)
///
/// Returns the physical address of the allocated frame, or None if out of memory.
pub fn alloc_frame(pmm: &mut PhysicalMemoryManager) -> Option<u64> {
    pmm.alloc_page()
}

/// Free a physical frame
///
/// # Safety
/// The caller must ensure that the frame is no longer in use.
pub unsafe fn free_frame(pmm: &mut PhysicalMemoryManager, addr: u64) {
    pmm.free_page(addr);
}

/// Get memory statistics
pub fn memory_stats() -> &'static stats::MemoryStats {
    stats::stats()
}
