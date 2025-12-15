//! Demand Paging and Page Replacement
//!
//! This module implements demand paging (allocating pages only when accessed)
//! and page replacement using the LRU (Least Recently Used) algorithm.

extern crate alloc;
use alloc::collections::{VecDeque, BTreeMap};
use spin::Mutex;
use super::addr::{VirtAddr, PhysAddr};

/// Page state for demand paging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageState {
    /// Page is not allocated yet
    NotAllocated,
    /// Page is allocated and in memory
    InMemory,
    /// Page has been swapped out
    SwappedOut,
}

/// LRU page tracking entry
#[derive(Debug, Clone)]
struct LruPage {
    /// Virtual address of the page
    virt_addr: u64,
    /// Physical address of the page
    phys_addr: u64,
    /// Access count (for statistics)
    access_count: usize,
}

/// LRU Page Replacement Manager
///
/// Uses a queue-based LRU implementation. Recently accessed pages are moved
/// to the back of the queue, and pages are evicted from the front.
pub struct LruPageManager {
    /// Queue of pages in LRU order (front = least recently used)
    page_queue: VecDeque<LruPage>,
    /// Map from virtual address to position in queue for fast lookup
    page_map: BTreeMap<u64, usize>,
    /// Maximum number of pages to track
    max_pages: usize,
}

impl LruPageManager {
    /// Create a new LRU page manager
    ///
    /// # Arguments
    /// * `max_pages` - Maximum number of pages to track
    pub fn new(max_pages: usize) -> Self {
        Self {
            page_queue: VecDeque::with_capacity(max_pages),
            page_map: BTreeMap::new(),
            max_pages,
        }
    }

    /// Record a page access (moves page to back of queue)
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address of the accessed page
    /// * `phys_addr` - Physical address of the page
    pub fn access_page(&mut self, virt_addr: VirtAddr, phys_addr: PhysAddr) {
        let virt = virt_addr.as_u64();
        let phys = phys_addr.as_u64();

        if let Some(&index) = self.page_map.get(&virt) {
            // Page already tracked - move to back
            if let Some(mut page) = self.page_queue.remove(index) {
                page.access_count += 1;
                self.page_queue.push_back(page);
                self.rebuild_map();
            }
        } else {
            // New page - add to back
            if self.page_queue.len() >= self.max_pages {
                // Remove least recently used page
                if let Some(removed) = self.page_queue.pop_front() {
                    self.page_map.remove(&removed.virt_addr);
                }
            }

            let page = LruPage {
                virt_addr: virt,
                phys_addr: phys,
                access_count: 1,
            };
            self.page_queue.push_back(page);
            self.rebuild_map();
        }
    }

    /// Get the least recently used page (for eviction)
    ///
    /// # Returns
    /// Virtual and physical addresses of the LRU page, or None if no pages tracked
    pub fn get_lru_page(&self) -> Option<(VirtAddr, PhysAddr)> {
        self.page_queue.front().map(|page| {
            (VirtAddr::new(page.virt_addr), PhysAddr::new(page.phys_addr))
        })
    }

    /// Remove a page from tracking
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address of the page to remove
    pub fn remove_page(&mut self, virt_addr: VirtAddr) {
        let virt = virt_addr.as_u64();
        if let Some(&index) = self.page_map.get(&virt) {
            self.page_queue.remove(index);
            self.page_map.remove(&virt);
            self.rebuild_map();
        }
    }

    /// Get the number of tracked pages
    pub fn page_count(&self) -> usize {
        self.page_queue.len()
    }

    /// Check if a page is being tracked
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address to check
    pub fn contains(&self, virt_addr: VirtAddr) -> bool {
        self.page_map.contains_key(&virt_addr.as_u64())
    }

    /// Rebuild the index map after queue modification
    fn rebuild_map(&mut self) {
        self.page_map.clear();
        for (index, page) in self.page_queue.iter().enumerate() {
            self.page_map.insert(page.virt_addr, index);
        }
    }

    /// Get statistics about page accesses
    pub fn get_stats(&self) -> LruStats {
        let total_accesses: usize = self.page_queue.iter().map(|p| p.access_count).sum();
        LruStats {
            tracked_pages: self.page_queue.len(),
            max_pages: self.max_pages,
            total_accesses,
        }
    }
}

/// LRU statistics
#[derive(Debug, Clone, Copy)]
pub struct LruStats {
    /// Number of pages currently tracked
    pub tracked_pages: usize,
    /// Maximum pages that can be tracked
    pub max_pages: usize,
    /// Total page accesses
    pub total_accesses: usize,
}

/// Demand paging manager
///
/// Tracks which pages are allocated on-demand vs. pre-allocated
pub struct DemandPagingManager {
    /// Map from virtual address to page state
    page_states: BTreeMap<u64, PageState>,
}

impl DemandPagingManager {
    /// Create a new demand paging manager
    pub fn new() -> Self {
        Self {
            page_states: BTreeMap::new(),
        }
    }

    /// Reserve a virtual address range without allocating physical pages
    ///
    /// # Arguments
    /// * `start` - Starting virtual address
    /// * `num_pages` - Number of pages to reserve
    pub fn reserve_pages(&mut self, start: VirtAddr, num_pages: usize) {
        use crate::memory::addr::PAGE_SIZE;
        for i in 0..num_pages {
            let virt = start.as_u64() + (i as u64 * PAGE_SIZE as u64);
            self.page_states.insert(virt, PageState::NotAllocated);
        }
    }

    /// Mark a page as allocated and in memory
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address of the page
    pub fn allocate_page(&mut self, virt_addr: VirtAddr) {
        self.page_states.insert(virt_addr.as_u64(), PageState::InMemory);
    }

    /// Get the state of a page
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address to check
    pub fn get_page_state(&self, virt_addr: VirtAddr) -> PageState {
        self.page_states
            .get(&virt_addr.as_u64())
            .copied()
            .unwrap_or(PageState::NotAllocated)
    }

    /// Mark a page as swapped out
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address of the page
    pub fn mark_swapped_out(&mut self, virt_addr: VirtAddr) {
        self.page_states.insert(virt_addr.as_u64(), PageState::SwappedOut);
    }

    /// Check if a page needs to be allocated on access (page fault)
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address being accessed
    ///
    /// # Returns
    /// true if the page should be allocated on demand
    pub fn should_allocate_on_fault(&self, virt_addr: VirtAddr) -> bool {
        matches!(
            self.get_page_state(virt_addr),
            PageState::NotAllocated | PageState::SwappedOut
        )
    }

    /// Remove a page from tracking
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address of the page
    pub fn remove_page(&mut self, virt_addr: VirtAddr) {
        self.page_states.remove(&virt_addr.as_u64());
    }

    /// Get the number of pages in each state
    pub fn get_stats(&self) -> DemandPagingStats {
        let mut stats = DemandPagingStats::default();
        for state in self.page_states.values() {
            match state {
                PageState::NotAllocated => stats.not_allocated += 1,
                PageState::InMemory => stats.in_memory += 1,
                PageState::SwappedOut => stats.swapped_out += 1,
            }
        }
        stats
    }
}

/// Demand paging statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct DemandPagingStats {
    /// Pages reserved but not allocated
    pub not_allocated: usize,
    /// Pages allocated and in memory
    pub in_memory: usize,
    /// Pages swapped out to disk
    pub swapped_out: usize,
}

/// Global LRU page manager
static LRU_MANAGER: Mutex<LruPageManager> = Mutex::new(LruPageManager {
    page_queue: VecDeque::new(),
    page_map: BTreeMap::new(),
    max_pages: 1024, // Track up to 1024 pages (4 MiB)
});

/// Global demand paging manager
static DEMAND_PAGING_MANAGER: Mutex<DemandPagingManager> = Mutex::new(DemandPagingManager {
    page_states: BTreeMap::new(),
});

/// Record a page access for LRU tracking
pub fn record_page_access(virt_addr: VirtAddr, phys_addr: PhysAddr) {
    LRU_MANAGER.lock().access_page(virt_addr, phys_addr);
}

/// Get the least recently used page
pub fn get_lru_page() -> Option<(VirtAddr, PhysAddr)> {
    LRU_MANAGER.lock().get_lru_page()
}

/// Remove a page from LRU tracking
pub fn remove_lru_page(virt_addr: VirtAddr) {
    LRU_MANAGER.lock().remove_page(virt_addr);
}

/// Get LRU statistics
pub fn get_lru_stats() -> LruStats {
    LRU_MANAGER.lock().get_stats()
}

/// Reserve pages for demand paging
pub fn reserve_demand_pages(start: VirtAddr, num_pages: usize) {
    DEMAND_PAGING_MANAGER.lock().reserve_pages(start, num_pages);
}

/// Allocate a demand-paged page
pub fn allocate_demand_page(virt_addr: VirtAddr) {
    DEMAND_PAGING_MANAGER.lock().allocate_page(virt_addr);
}

/// Get page state
pub fn get_page_state(virt_addr: VirtAddr) -> PageState {
    DEMAND_PAGING_MANAGER.lock().get_page_state(virt_addr)
}

/// Check if a page should be allocated on fault
pub fn should_allocate_on_fault(virt_addr: VirtAddr) -> bool {
    DEMAND_PAGING_MANAGER.lock().should_allocate_on_fault(virt_addr)
}

/// Get demand paging statistics
pub fn get_demand_paging_stats() -> DemandPagingStats {
    DEMAND_PAGING_MANAGER.lock().get_stats()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_page_manager() {
        let mut lru = LruPageManager::new(3);

        // Add pages
        lru.access_page(VirtAddr::new(0x1000), PhysAddr::new(0x1000));
        lru.access_page(VirtAddr::new(0x2000), PhysAddr::new(0x2000));
        lru.access_page(VirtAddr::new(0x3000), PhysAddr::new(0x3000));

        assert_eq!(lru.page_count(), 3);

        // LRU should be the first page added
        let (virt, _) = lru.get_lru_page().unwrap();
        assert_eq!(virt.as_u64(), 0x1000);

        // Access the first page again
        lru.access_page(VirtAddr::new(0x1000), PhysAddr::new(0x1000));

        // Now LRU should be the second page
        let (virt, _) = lru.get_lru_page().unwrap();
        assert_eq!(virt.as_u64(), 0x2000);

        // Add a fourth page - should evict LRU
        lru.access_page(VirtAddr::new(0x4000), PhysAddr::new(0x4000));
        assert_eq!(lru.page_count(), 3);
        assert!(!lru.contains(VirtAddr::new(0x2000)));
    }

    #[test]
    fn test_demand_paging_manager() {
        let mut manager = DemandPagingManager::new();

        // Reserve pages
        manager.reserve_pages(VirtAddr::new(0x1000), 3);

        assert_eq!(manager.get_page_state(VirtAddr::new(0x1000)), PageState::NotAllocated);
        assert_eq!(manager.get_page_state(VirtAddr::new(0x2000)), PageState::NotAllocated);
        assert_eq!(manager.get_page_state(VirtAddr::new(0x3000)), PageState::NotAllocated);

        // Allocate one page
        manager.allocate_page(VirtAddr::new(0x1000));
        assert_eq!(manager.get_page_state(VirtAddr::new(0x1000)), PageState::InMemory);

        // Check should_allocate_on_fault
        assert!(!manager.should_allocate_on_fault(VirtAddr::new(0x1000)));
        assert!(manager.should_allocate_on_fault(VirtAddr::new(0x2000)));
    }

    #[test]
    fn test_page_state_transitions() {
        let mut manager = DemandPagingManager::new();
        let addr = VirtAddr::new(0x1000);

        // Start as not allocated
        assert_eq!(manager.get_page_state(addr), PageState::NotAllocated);

        // Allocate
        manager.allocate_page(addr);
        assert_eq!(manager.get_page_state(addr), PageState::InMemory);

        // Swap out
        manager.mark_swapped_out(addr);
        assert_eq!(manager.get_page_state(addr), PageState::SwappedOut);

        // Swap back in
        manager.allocate_page(addr);
        assert_eq!(manager.get_page_state(addr), PageState::InMemory);
    }

    #[test]
    fn test_lru_stats() {
        let mut lru = LruPageManager::new(10);

        lru.access_page(VirtAddr::new(0x1000), PhysAddr::new(0x1000));
        lru.access_page(VirtAddr::new(0x1000), PhysAddr::new(0x1000));
        lru.access_page(VirtAddr::new(0x2000), PhysAddr::new(0x2000));

        let stats = lru.get_stats();
        assert_eq!(stats.tracked_pages, 2);
        assert_eq!(stats.max_pages, 10);
        assert_eq!(stats.total_accesses, 3);
    }

    #[test]
    fn test_demand_paging_stats() {
        let mut manager = DemandPagingManager::new();

        manager.reserve_pages(VirtAddr::new(0x1000), 5);
        manager.allocate_page(VirtAddr::new(0x1000));
        manager.allocate_page(VirtAddr::new(0x2000));
        manager.mark_swapped_out(VirtAddr::new(0x3000));

        let stats = manager.get_stats();
        assert_eq!(stats.not_allocated, 2);
        assert_eq!(stats.in_memory, 2);
        assert_eq!(stats.swapped_out, 1);
    }
}
