//! Copy-on-Write (CoW) Memory Management
//!
//! This module implements Copy-on-Write functionality for efficient memory sharing
//! between processes. When a page is marked as CoW, it's shared read-only between
//! processes until one attempts to write to it, at which point a private copy is made.

extern crate alloc;
use alloc::collections::BTreeMap;
use spin::Mutex;
use super::addr::PhysAddr;

/// Reference counter for CoW pages
/// 
/// Tracks how many processes share a physical page. When the count reaches 1,
/// the page can be made writable again since it's no longer shared.
#[derive(Debug)]
struct RefCount {
    count: usize,
}

impl RefCount {
    fn new() -> Self {
        Self { count: 1 }
    }

    fn increment(&mut self) {
        self.count += 1;
    }

    fn decrement(&mut self) -> usize {
        self.count = self.count.saturating_sub(1);
        self.count
    }

    fn get(&self) -> usize {
        self.count
    }
}

/// Global CoW page manager
pub struct CowPageManager {
    /// Reference counts for physical pages
    /// Maps physical address -> reference count
    ref_counts: BTreeMap<u64, RefCount>,
}

impl CowPageManager {
    /// Create a new CoW page manager
    pub const fn new() -> Self {
        Self {
            ref_counts: BTreeMap::new(),
        }
    }

    /// Mark a page as CoW (increment reference count)
    ///
    /// # Arguments
    /// * `phys_addr` - Physical address of the page
    ///
    /// # Returns
    /// The new reference count
    pub fn mark_cow(&mut self, phys_addr: PhysAddr) -> usize {
        let addr = phys_addr.as_u64();
        self.ref_counts
            .entry(addr)
            .or_insert_with(RefCount::new)
            .increment();
        self.ref_counts[&addr].get()
    }

    /// Get the reference count for a page
    ///
    /// # Arguments
    /// * `phys_addr` - Physical address of the page
    ///
    /// # Returns
    /// The reference count, or 0 if the page is not tracked
    pub fn get_ref_count(&self, phys_addr: PhysAddr) -> usize {
        let addr = phys_addr.as_u64();
        self.ref_counts.get(&addr).map(|rc| rc.get()).unwrap_or(0)
    }

    /// Release a CoW page reference
    ///
    /// Decrements the reference count. If the count reaches zero, the page
    /// is no longer tracked and can be freed.
    ///
    /// # Arguments
    /// * `phys_addr` - Physical address of the page
    ///
    /// # Returns
    /// The new reference count
    pub fn release_cow(&mut self, phys_addr: PhysAddr) -> usize {
        let addr = phys_addr.as_u64();
        if let Some(rc) = self.ref_counts.get_mut(&addr) {
            let new_count = rc.decrement();
            if new_count == 0 {
                self.ref_counts.remove(&addr);
            }
            new_count
        } else {
            0
        }
    }

    /// Check if a page is shared (reference count > 1)
    ///
    /// # Arguments
    /// * `phys_addr` - Physical address of the page
    ///
    /// # Returns
    /// true if the page is shared among multiple processes
    pub fn is_shared(&self, phys_addr: PhysAddr) -> bool {
        self.get_ref_count(phys_addr) > 1
    }

    /// Add a new CoW page with an initial reference count
    ///
    /// # Arguments
    /// * `phys_addr` - Physical address of the page
    pub fn add_page(&mut self, phys_addr: PhysAddr) {
        let addr = phys_addr.as_u64();
        self.ref_counts.insert(addr, RefCount::new());
    }
}

/// Global CoW page manager instance
static COW_MANAGER: Mutex<CowPageManager> = Mutex::new(CowPageManager::new());

/// Mark a page as Copy-on-Write
///
/// # Arguments
/// * `phys_addr` - Physical address of the page to mark as CoW
pub fn mark_cow_page(phys_addr: PhysAddr) {
    COW_MANAGER.lock().mark_cow(phys_addr);
}

/// Release a CoW page reference
///
/// # Arguments
/// * `phys_addr` - Physical address of the page
///
/// # Returns
/// The new reference count (0 if page can be freed)
pub fn release_cow_page(phys_addr: PhysAddr) -> usize {
    COW_MANAGER.lock().release_cow(phys_addr)
}

/// Check if a page is shared (CoW)
///
/// # Arguments
/// * `phys_addr` - Physical address of the page
///
/// # Returns
/// true if the page has multiple references
pub fn is_cow_page(phys_addr: PhysAddr) -> bool {
    COW_MANAGER.lock().is_shared(phys_addr)
}

/// Get the reference count for a CoW page
///
/// # Arguments
/// * `phys_addr` - Physical address of the page
///
/// # Returns
/// The reference count
pub fn get_cow_ref_count(phys_addr: PhysAddr) -> usize {
    COW_MANAGER.lock().get_ref_count(phys_addr)
}

/// Add a page to CoW tracking
///
/// # Arguments
/// * `phys_addr` - Physical address of the page
pub fn add_cow_page(phys_addr: PhysAddr) {
    COW_MANAGER.lock().add_page(phys_addr);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_page_tracking() {
        let phys_addr = PhysAddr::new(0x1000);
        
        // Initially not tracked
        assert_eq!(get_cow_ref_count(phys_addr), 0);
        assert!(!is_cow_page(phys_addr));

        // Add page
        add_cow_page(phys_addr);
        assert_eq!(get_cow_ref_count(phys_addr), 1);
        assert!(!is_cow_page(phys_addr)); // Not shared yet

        // Mark as CoW (increment ref count)
        mark_cow_page(phys_addr);
        assert_eq!(get_cow_ref_count(phys_addr), 2);
        assert!(is_cow_page(phys_addr)); // Now shared

        // Release one reference
        let count = release_cow_page(phys_addr);
        assert_eq!(count, 1);
        assert!(!is_cow_page(phys_addr)); // No longer shared

        // Release final reference
        let count = release_cow_page(phys_addr);
        assert_eq!(count, 0);
        assert_eq!(get_cow_ref_count(phys_addr), 0); // No longer tracked
    }

    #[test]
    fn test_multiple_cow_pages() {
        let page1 = PhysAddr::new(0x1000);
        let page2 = PhysAddr::new(0x2000);

        add_cow_page(page1);
        add_cow_page(page2);

        assert_eq!(get_cow_ref_count(page1), 1);
        assert_eq!(get_cow_ref_count(page2), 1);

        mark_cow_page(page1);
        mark_cow_page(page1);

        assert_eq!(get_cow_ref_count(page1), 3);
        assert_eq!(get_cow_ref_count(page2), 1);

        release_cow_page(page1);
        release_cow_page(page1);
        
        assert_eq!(get_cow_ref_count(page1), 1);
        assert_eq!(get_cow_ref_count(page2), 1);
    }
}
