//! Memory Statistics and Debugging
//!
//! This module provides utilities for tracking memory usage statistics
//! and debugging memory-related issues.

use core::fmt;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Memory statistics tracker
pub struct MemoryStats {
    /// Total physical memory (bytes)
    total_physical: AtomicUsize,
    /// Used physical memory (bytes)
    used_physical: AtomicUsize,
    /// Total heap memory (bytes)
    total_heap: AtomicUsize,
    /// Used heap memory (bytes)
    used_heap: AtomicUsize,
    /// Number of heap allocations
    heap_allocations: AtomicUsize,
    /// Number of heap deallocations
    heap_deallocations: AtomicUsize,
    /// Number of page allocations
    page_allocations: AtomicUsize,
    /// Number of page deallocations
    page_deallocations: AtomicUsize,
}

impl MemoryStats {
    /// Creates a new memory statistics tracker
    pub const fn new() -> Self {
        Self {
            total_physical: AtomicUsize::new(0),
            used_physical: AtomicUsize::new(0),
            total_heap: AtomicUsize::new(0),
            used_heap: AtomicUsize::new(0),
            heap_allocations: AtomicUsize::new(0),
            heap_deallocations: AtomicUsize::new(0),
            page_allocations: AtomicUsize::new(0),
            page_deallocations: AtomicUsize::new(0),
        }
    }

    /// Sets the total physical memory
    pub fn set_total_physical(&self, bytes: usize) {
        self.total_physical.store(bytes, Ordering::Relaxed);
    }

    /// Gets the total physical memory
    pub fn total_physical(&self) -> usize {
        self.total_physical.load(Ordering::Relaxed)
    }

    /// Sets the used physical memory
    pub fn set_used_physical(&self, bytes: usize) {
        self.used_physical.store(bytes, Ordering::Relaxed);
    }

    /// Gets the used physical memory
    pub fn used_physical(&self) -> usize {
        self.used_physical.load(Ordering::Relaxed)
    }

    /// Gets the free physical memory
    pub fn free_physical(&self) -> usize {
        self.total_physical()
            .saturating_sub(self.used_physical())
    }

    /// Sets the total heap memory
    pub fn set_total_heap(&self, bytes: usize) {
        self.total_heap.store(bytes, Ordering::Relaxed);
    }

    /// Gets the total heap memory
    pub fn total_heap(&self) -> usize {
        self.total_heap.load(Ordering::Relaxed)
    }

    /// Records a heap allocation
    pub fn record_heap_alloc(&self, bytes: usize) {
        self.used_heap.fetch_add(bytes, Ordering::Relaxed);
        self.heap_allocations.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a heap deallocation
    pub fn record_heap_dealloc(&self, bytes: usize) {
        self.used_heap.fetch_sub(bytes, Ordering::Relaxed);
        self.heap_deallocations.fetch_add(1, Ordering::Relaxed);
    }

    /// Gets the used heap memory
    pub fn used_heap(&self) -> usize {
        self.used_heap.load(Ordering::Relaxed)
    }

    /// Gets the free heap memory
    pub fn free_heap(&self) -> usize {
        self.total_heap()
            .saturating_sub(self.used_heap())
    }

    /// Gets the number of heap allocations
    pub fn heap_allocations(&self) -> usize {
        self.heap_allocations.load(Ordering::Relaxed)
    }

    /// Gets the number of heap deallocations
    pub fn heap_deallocations(&self) -> usize {
        self.heap_deallocations.load(Ordering::Relaxed)
    }

    /// Gets the number of active heap allocations
    pub fn active_heap_allocations(&self) -> usize {
        self.heap_allocations()
            .saturating_sub(self.heap_deallocations())
    }

    /// Records a page allocation
    pub fn record_page_alloc(&self) {
        self.page_allocations.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a page deallocation
    pub fn record_page_dealloc(&self) {
        self.page_deallocations.fetch_add(1, Ordering::Relaxed);
    }

    /// Gets the number of page allocations
    pub fn page_allocations(&self) -> usize {
        self.page_allocations.load(Ordering::Relaxed)
    }

    /// Gets the number of page deallocations
    pub fn page_deallocations(&self) -> usize {
        self.page_deallocations.load(Ordering::Relaxed)
    }

    /// Gets the number of active page allocations
    pub fn active_page_allocations(&self) -> usize {
        self.page_allocations()
            .saturating_sub(self.page_deallocations())
    }
}

impl fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Memory Statistics:")?;
        writeln!(f, "  Physical Memory:")?;
        writeln!(
            f,
            "    Total: {} KiB ({} MiB)",
            self.total_physical() / 1024,
            self.total_physical() / (1024 * 1024)
        )?;
        writeln!(
            f,
            "    Used:  {} KiB ({:.1}%)",
            self.used_physical() / 1024,
            (self.used_physical() as f32 / self.total_physical() as f32) * 100.0
        )?;
        writeln!(
            f,
            "    Free:  {} KiB ({:.1}%)",
            self.free_physical() / 1024,
            (self.free_physical() as f32 / self.total_physical() as f32) * 100.0
        )?;
        writeln!(f, "  Heap Memory:")?;
        writeln!(
            f,
            "    Total: {} KiB",
            self.total_heap() / 1024
        )?;
        writeln!(
            f,
            "    Used:  {} KiB ({:.1}%)",
            self.used_heap() / 1024,
            if self.total_heap() > 0 {
                (self.used_heap() as f32 / self.total_heap() as f32) * 100.0
            } else {
                0.0
            }
        )?;
        writeln!(
            f,
            "    Free:  {} KiB ({:.1}%)",
            self.free_heap() / 1024,
            if self.total_heap() > 0 {
                (self.free_heap() as f32 / self.total_heap() as f32) * 100.0
            } else {
                0.0
            }
        )?;
        writeln!(f, "  Allocations:")?;
        writeln!(
            f,
            "    Heap:  {} allocs, {} frees, {} active",
            self.heap_allocations(),
            self.heap_deallocations(),
            self.active_heap_allocations()
        )?;
        writeln!(
            f,
            "    Pages: {} allocs, {} frees, {} active",
            self.page_allocations(),
            self.page_deallocations(),
            self.active_page_allocations()
        )?;
        Ok(())
    }
}

/// Global memory statistics instance
static MEMORY_STATS: MemoryStats = MemoryStats::new();

/// Gets a reference to the global memory statistics
pub fn stats() -> &'static MemoryStats {
    &MEMORY_STATS
}

/// Memory debugging utilities
pub mod debug {
    use super::*;

    /// Prints a memory dump of a region
    ///
    /// # Safety
    /// The caller must ensure that the memory region is valid and readable
    pub unsafe fn dump_memory(addr: u64, size: usize, label: &str) {
        use fanga_arch_x86_64 as arch;

        arch::serial_println!("Memory Dump: {} @ 0x{:x} ({} bytes)", label, addr, size);

        let ptr = addr as *const u8;
        let mut offset = 0;

        while offset < size {
            // Print address
            arch::serial_print!("  0x{:08x}: ", addr + offset as u64);

            // Print hex bytes
            let line_size = core::cmp::min(16, size - offset);
            for i in 0..line_size {
                let byte = ptr.add(offset + i).read();
                arch::serial_print!("{:02x} ", byte);
            }

            // Padding for short lines
            for _ in line_size..16 {
                arch::serial_print!("   ");
            }

            // Print ASCII representation
            arch::serial_print!(" |");
            for i in 0..line_size {
                let byte = ptr.add(offset + i).read();
                if byte >= 0x20 && byte <= 0x7E {
                    arch::serial_print!("{}", byte as char);
                } else {
                    arch::serial_print!(".");
                }
            }
            arch::serial_println!("|");

            offset += line_size;
        }
    }

    /// Prints page table information for a virtual address
    pub fn dump_page_table_entry(virt_addr: u64, mapper: &crate::vmm::PageTableMapper) {
        use fanga_arch_x86_64 as arch;

        arch::serial_println!("Page Table Entry for 0x{:x}:", virt_addr);

        if let Some(phys_addr) = mapper.translate(virt_addr) {
            arch::serial_println!("  Physical: 0x{:x}", phys_addr);
            arch::serial_println!("  Mapped: Yes");
        } else {
            arch::serial_println!("  Mapped: No");
        }

        // Print indices
        arch::serial_println!("  PML4 index: {}", crate::vmm::pml4_index(virt_addr));
        arch::serial_println!("  PDPT index: {}", crate::vmm::pdpt_index(virt_addr));
        arch::serial_println!("  PD index:   {}", crate::vmm::pd_index(virt_addr));
        arch::serial_println!("  PT index:   {}", crate::vmm::pt_index(virt_addr));
        arch::serial_println!("  Offset:     0x{:x}", crate::vmm::page_offset(virt_addr));
    }
}
