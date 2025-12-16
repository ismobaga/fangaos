//! Bitmap-based Physical Memory Allocator
//!
//! This module implements a bitmap allocator for physical memory pages.
//! Each bit in the bitmap represents one page (4 KiB) of physical memory.
//!
//! # Thread Safety
//!
//! This allocator is thread-safe and can be used from multiple CPUs
//! concurrently. All operations are protected by an internal spinlock.

use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;
use crate::memory::addr::{PAGE_SIZE, align_up, align_down};

/// Bitmap entry type - each u64 covers 64 pages
type BitmapEntry = u64;
const BITS_PER_ENTRY: usize = 64;

/// Inner state of the Physical Memory Manager
struct PhysicalMemoryManagerInner {
    /// Pointer to the bitmap stored in higher-half direct map
    bitmap: *mut BitmapEntry,
    /// Number of bitmap entries
    bitmap_entries: usize,
    /// Total number of pages managed
    total_pages: usize,
    /// Number of free pages available
    free_pages: usize,
    /// Highest physical address managed
    highest_addr: u64,
}

/// Physical Memory Manager using bitmap allocation
///
/// This structure is thread-safe and can be used from multiple CPUs.
/// All operations acquire an internal lock to ensure consistency.
pub struct PhysicalMemoryManager {
    inner: Mutex<PhysicalMemoryManagerInner>,
}

impl PhysicalMemoryManager {
    /// Creates a new, uninitialized PMM
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(PhysicalMemoryManagerInner {
                bitmap: core::ptr::null_mut(),
                bitmap_entries: 0,
                total_pages: 0,
                free_pages: 0,
                highest_addr: 0,
            }),
        }
    }

    /// Initializes the PMM using the memory map from Limine
    ///
    /// # Arguments
    /// * `memmap` - Memory map response from Limine
    /// * `hhdm_offset` - Higher Half Direct Map offset for virtual addressing
    ///
    /// # Safety
    /// This function must be called exactly once during kernel initialization
    pub unsafe fn init(&self, memmap: &limine::response::MemoryMapResponse, hhdm_offset: u64) {
        let mut inner = self.inner.lock();
        
        // Find the highest physical address
        let mut highest = 0u64;
        for entry in memmap.entries() {
            let end = entry.base + entry.length;
            if end > highest {
                highest = end;
            }
        }
        inner.highest_addr = highest;

        // Calculate how many pages we need to manage
        inner.total_pages = (highest as usize + PAGE_SIZE - 1) / PAGE_SIZE;

        // Calculate bitmap size (in entries)
        inner.bitmap_entries = (inner.total_pages + BITS_PER_ENTRY - 1) / BITS_PER_ENTRY;
        let bitmap_size = inner.bitmap_entries * core::mem::size_of::<BitmapEntry>();

        // Find a usable memory region large enough for the bitmap
        let mut bitmap_phys: u64 = 0;
        for entry in memmap.entries() {
            if entry.entry_type == limine::memory_map::EntryType::USABLE
                && entry.length >= bitmap_size as u64
            {
                bitmap_phys = entry.base;
                break;
            }
        }

        if bitmap_phys == 0 {
            panic!("PMM: No memory region large enough for bitmap!");
        }

        // Map bitmap to virtual address using HHDM
        inner.bitmap = (hhdm_offset + bitmap_phys) as *mut BitmapEntry;

        // Initialize bitmap - mark all as used (1 = used, 0 = free)
        for i in 0..inner.bitmap_entries {
            inner.bitmap.add(i).write_volatile(!0);
        }

        // Mark usable regions as free
        let mut free_count = 0usize;
        for entry in memmap.entries() {
            if entry.entry_type == limine::memory_map::EntryType::USABLE {
                let start = align_up(entry.base, PAGE_SIZE as u64);
                let end = align_down(entry.base + entry.length, PAGE_SIZE as u64);

                let start_page = (start as usize) / PAGE_SIZE;
                let page_count = ((end - start) as usize) / PAGE_SIZE;

                for page in start_page..(start_page + page_count) {
                    Self::mark_page_free_inner(&mut inner, page);
                    free_count += 1;
                }
            }
        }

        // Mark the bitmap pages themselves as used
        let bitmap_pages = (bitmap_size + PAGE_SIZE - 1) / PAGE_SIZE;
        let bitmap_start_page = (bitmap_phys as usize) / PAGE_SIZE;
        for page in bitmap_start_page..(bitmap_start_page + bitmap_pages) {
            if page < inner.total_pages {
                Self::mark_page_used_inner(&mut inner, page);
                free_count = free_count.saturating_sub(1);
            }
        }

        inner.free_pages = free_count;
    }

    /// Internal helper: Marks a page as free in the bitmap (no locking)
    unsafe fn mark_page_free_inner(inner: &mut PhysicalMemoryManagerInner, page: usize) {
        if page >= inner.total_pages {
            return;
        }

        let entry_idx = page / BITS_PER_ENTRY;
        let bit_idx = page % BITS_PER_ENTRY;

        let entry_ptr = inner.bitmap.add(entry_idx);
        let mut entry = entry_ptr.read_volatile();
        entry &= !(1u64 << bit_idx); // Clear bit = free
        entry_ptr.write_volatile(entry);
    }

    /// Internal helper: Marks a page as used in the bitmap (no locking)
    unsafe fn mark_page_used_inner(inner: &mut PhysicalMemoryManagerInner, page: usize) {
        if page >= inner.total_pages {
            return;
        }

        let entry_idx = page / BITS_PER_ENTRY;
        let bit_idx = page % BITS_PER_ENTRY;

        let entry_ptr = inner.bitmap.add(entry_idx);
        let mut entry = entry_ptr.read_volatile();
        entry |= 1u64 << bit_idx; // Set bit = used
        entry_ptr.write_volatile(entry);
    }

    /// Allocates a single physical page
    ///
    /// Returns the physical address of the allocated page, or None if no pages are available.
    /// This method is thread-safe and can be called from multiple CPUs concurrently.
    pub fn alloc_page(&self) -> Option<u64> {
        let mut inner = self.inner.lock();
        
        // Check if we have free pages
        if inner.free_pages == 0 {
            return None;
        }

        // Search for a free page
        for entry_idx in 0..inner.bitmap_entries {
            unsafe {
                let entry_ptr = inner.bitmap.add(entry_idx);
                let entry = entry_ptr.read_volatile();

                // If entry is not all 1s, there's at least one free page
                if entry != !0 {
                    // Find the first free bit
                    for bit_idx in 0..BITS_PER_ENTRY {
                        if (entry & (1u64 << bit_idx)) == 0 {
                            // Found a free page
                            let page = entry_idx * BITS_PER_ENTRY + bit_idx;
                            if page < inner.total_pages {
                                // Mark as used
                                let new_entry = entry | (1u64 << bit_idx);
                                entry_ptr.write_volatile(new_entry);

                                // Update free count
                                inner.free_pages -= 1;

                                // Return physical address
                                return Some((page * PAGE_SIZE) as u64);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Frees a physical page
    ///
    /// # Arguments
    /// * `addr` - Physical address of the page to free (must be page-aligned)
    ///
    /// This method is thread-safe and can be called from multiple CPUs concurrently.
    /// Freeing an already-free page is safely ignored (no-op).
    pub fn free_page(&self, addr: u64) {
        // Ensure page-aligned
        if addr % PAGE_SIZE as u64 != 0 {
            return;
        }

        let page = (addr as usize) / PAGE_SIZE;
        
        let mut inner = self.inner.lock();
        
        if page >= inner.total_pages {
            return;
        }

        // Check if already free
        let entry_idx = page / BITS_PER_ENTRY;
        let bit_idx = page % BITS_PER_ENTRY;
        
        unsafe {
            let entry_ptr = inner.bitmap.add(entry_idx);
            let entry = entry_ptr.read_volatile();
            
            // If bit is already 0 (free), it's a double-free - ignore
            if (entry & (1u64 << bit_idx)) == 0 {
                return;
            }
            
            // Mark as free
            let new_entry = entry & !(1u64 << bit_idx);
            entry_ptr.write_volatile(new_entry);
            
            inner.free_pages += 1;
        }
    }

    /// Returns the number of free pages
    pub fn free_pages(&self) -> usize {
        self.inner.lock().free_pages
    }

    /// Returns the total number of pages managed
    pub fn total_pages(&self) -> usize {
        self.inner.lock().total_pages
    }

    /// Returns the number of used pages
    pub fn used_pages(&self) -> usize {
        let inner = self.inner.lock();
        inner.total_pages.saturating_sub(inner.free_pages)
    }
}
