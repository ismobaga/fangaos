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
            if self.total_physical() > 0 {
                (self.used_physical() as f32 / self.total_physical() as f32) * 100.0
            } else {
                0.0
            }
        )?;
        writeln!(
            f,
            "    Free:  {} KiB ({:.1}%)",
            self.free_physical() / 1024,
            if self.total_physical() > 0 {
                (self.free_physical() as f32 / self.total_physical() as f32) * 100.0
            } else {
                0.0
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats_creation() {
        let stats = MemoryStats::new();
        assert_eq!(stats.total_physical(), 0);
        assert_eq!(stats.used_physical(), 0);
        assert_eq!(stats.total_heap(), 0);
        assert_eq!(stats.used_heap(), 0);
    }

    #[test]
    fn test_physical_memory_tracking() {
        let stats = MemoryStats::new();
        
        stats.set_total_physical(1024 * 1024); // 1 MiB
        assert_eq!(stats.total_physical(), 1024 * 1024);
        
        stats.set_used_physical(512 * 1024); // 512 KiB
        assert_eq!(stats.used_physical(), 512 * 1024);
        assert_eq!(stats.free_physical(), 512 * 1024);
    }

    #[test]
    fn test_heap_memory_tracking() {
        let stats = MemoryStats::new();
        
        stats.set_total_heap(64 * 1024); // 64 KiB
        assert_eq!(stats.total_heap(), 64 * 1024);
        
        stats.record_heap_alloc(1024);
        assert_eq!(stats.used_heap(), 1024);
        assert_eq!(stats.heap_allocations(), 1);
        assert_eq!(stats.active_heap_allocations(), 1);
        
        stats.record_heap_alloc(2048);
        assert_eq!(stats.used_heap(), 3072);
        assert_eq!(stats.heap_allocations(), 2);
        assert_eq!(stats.active_heap_allocations(), 2);
        
        stats.record_heap_dealloc(1024);
        assert_eq!(stats.used_heap(), 2048);
        assert_eq!(stats.heap_deallocations(), 1);
        assert_eq!(stats.active_heap_allocations(), 1);
    }

    #[test]
    fn test_page_tracking() {
        let stats = MemoryStats::new();
        
        stats.record_page_alloc();
        assert_eq!(stats.page_allocations(), 1);
        assert_eq!(stats.active_page_allocations(), 1);
        
        stats.record_page_alloc();
        stats.record_page_alloc();
        assert_eq!(stats.page_allocations(), 3);
        assert_eq!(stats.active_page_allocations(), 3);
        
        stats.record_page_dealloc();
        assert_eq!(stats.page_deallocations(), 1);
        assert_eq!(stats.active_page_allocations(), 2);
    }

    #[test]
    fn test_free_memory_calculations() {
        let stats = MemoryStats::new();
        
        stats.set_total_physical(1000);
        stats.set_used_physical(600);
        assert_eq!(stats.free_physical(), 400);
        
        stats.set_total_heap(500);
        stats.record_heap_alloc(200);
        assert_eq!(stats.free_heap(), 300);
    }

    #[test]
    fn test_saturating_operations() {
        let stats = MemoryStats::new();
        
        // Test free_physical with underflow protection
        stats.set_used_physical(100);
        stats.set_total_physical(50);
        assert_eq!(stats.free_physical(), 0); // Should saturate to 0
        
        // Test active allocations with underflow protection
        stats.record_page_dealloc();
        assert_eq!(stats.active_page_allocations(), 0); // Should saturate to 0
    }
}
