//! Memory Protection
//!
//! This module provides memory protection mechanisms including:
//! - Guard pages for stack overflow detection
//! - Memory region access control
//! - Per-process memory isolation

extern crate alloc;
use alloc::collections::BTreeSet;
use spin::Mutex;
use super::addr::{VirtAddr, PAGE_SIZE};
use super::paging::PageTableFlags;

/// Guard page manager
///
/// Tracks guard pages (non-accessible pages) used for detecting
/// stack overflows and buffer overruns
pub struct GuardPageManager {
    /// Set of virtual addresses marked as guard pages
    guard_pages: BTreeSet<u64>,
}

impl GuardPageManager {
    /// Create a new guard page manager
    pub const fn new() -> Self {
        Self {
            guard_pages: BTreeSet::new(),
        }
    }

    /// Mark a page as a guard page
    ///
    /// Guard pages should be mapped as not present to trigger page faults
    ///
    /// # Arguments
    /// * `addr` - Virtual address of the guard page (must be page-aligned)
    pub fn add_guard_page(&mut self, addr: VirtAddr) -> Result<(), &'static str> {
        if !addr.is_aligned(PAGE_SIZE as u64) {
            return Err("Address must be page-aligned");
        }

        self.guard_pages.insert(addr.as_u64());
        Ok(())
    }

    /// Remove a guard page
    ///
    /// # Arguments
    /// * `addr` - Virtual address of the guard page
    pub fn remove_guard_page(&mut self, addr: VirtAddr) {
        self.guard_pages.remove(&addr.as_u64());
    }

    /// Check if an address is a guard page
    ///
    /// # Arguments
    /// * `addr` - Virtual address to check
    pub fn is_guard_page(&self, addr: VirtAddr) -> bool {
        // Align down to page boundary and check
        let page_addr = addr.as_u64() & !(PAGE_SIZE as u64 - 1);
        self.guard_pages.contains(&page_addr)
    }

    /// Get the number of guard pages
    pub fn count(&self) -> usize {
        self.guard_pages.len()
    }
}

/// Memory protection flags for a region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryProtection {
    /// Region is readable
    pub read: bool,
    /// Region is writable
    pub write: bool,
    /// Region is executable
    pub exec: bool,
}

impl MemoryProtection {
    /// No access allowed
    pub const NONE: Self = Self {
        read: false,
        write: false,
        exec: false,
    };

    /// Read-only
    pub const READ: Self = Self {
        read: true,
        write: false,
        exec: false,
    };

    /// Read-write
    pub const READ_WRITE: Self = Self {
        read: true,
        write: true,
        exec: false,
    };

    /// Read-execute
    pub const READ_EXEC: Self = Self {
        read: true,
        write: false,
        exec: true,
    };

    /// Read-write-execute (dangerous, avoid if possible)
    pub const READ_WRITE_EXEC: Self = Self {
        read: true,
        write: true,
        exec: true,
    };

    /// Convert to page table flags
    pub fn to_page_flags(&self) -> PageTableFlags {
        let mut flags = PageTableFlags::PRESENT;

        if self.write {
            flags = flags.with(PageTableFlags::WRITABLE);
        }

        if !self.exec {
            flags = flags.with(PageTableFlags::NO_EXECUTE);
        }

        flags
    }

    /// Check if protection allows an access type
    pub fn allows_read(&self) -> bool {
        self.read
    }

    pub fn allows_write(&self) -> bool {
        self.write
    }

    pub fn allows_exec(&self) -> bool {
        self.exec
    }
}

/// Memory region with protection information
#[derive(Debug, Clone)]
pub struct ProtectedRegion {
    /// Start address
    pub start: VirtAddr,
    /// Size in bytes
    pub size: usize,
    /// Protection flags
    pub protection: MemoryProtection,
    /// Region name/description
    pub name: &'static str,
}

impl ProtectedRegion {
    /// Create a new protected region
    pub fn new(start: VirtAddr, size: usize, protection: MemoryProtection, name: &'static str) -> Self {
        Self {
            start,
            size,
            protection,
            name,
        }
    }

    /// Check if an address is within this region
    pub fn contains(&self, addr: VirtAddr) -> bool {
        let addr_val = addr.as_u64();
        let start_val = self.start.as_u64();
        let end_val = start_val + self.size as u64;
        addr_val >= start_val && addr_val < end_val
    }

    /// Get the end address of this region
    pub fn end(&self) -> VirtAddr {
        VirtAddr::new(self.start.as_u64() + self.size as u64)
    }
}

/// Process memory protection manager
pub struct MemoryProtectionManager {
    /// Guard pages
    guard_pages: GuardPageManager,
    /// Protected regions (not yet fully implemented)
    regions: alloc::vec::Vec<ProtectedRegion>,
}

impl MemoryProtectionManager {
    /// Create a new memory protection manager
    pub fn new() -> Self {
        Self {
            guard_pages: GuardPageManager::new(),
            regions: alloc::vec::Vec::new(),
        }
    }

    /// Add a guard page
    pub fn add_guard_page(&mut self, addr: VirtAddr) -> Result<(), &'static str> {
        self.guard_pages.add_guard_page(addr)
    }

    /// Check if an address is a guard page
    pub fn is_guard_page(&self, addr: VirtAddr) -> bool {
        self.guard_pages.is_guard_page(addr)
    }

    /// Add a protected region
    pub fn add_region(&mut self, region: ProtectedRegion) {
        self.regions.push(region);
    }

    /// Find the region containing an address
    pub fn find_region(&self, addr: VirtAddr) -> Option<&ProtectedRegion> {
        self.regions.iter().find(|r| r.contains(addr))
    }

    /// Check if an access to an address is allowed
    pub fn check_access(&self, addr: VirtAddr, write: bool, exec: bool) -> bool {
        // Check if it's a guard page (always deny)
        if self.is_guard_page(addr) {
            return false;
        }

        // Check region protections
        if let Some(region) = self.find_region(addr) {
            if write && !region.protection.allows_write() {
                return false;
            }
            if exec && !region.protection.allows_exec() {
                return false;
            }
            if !region.protection.allows_read() {
                return false;
            }
            true
        } else {
            // No region found - deny by default for safety
            false
        }
    }
}

/// Global memory protection manager
static PROTECTION_MANAGER: Mutex<MemoryProtectionManager> = Mutex::new(MemoryProtectionManager {
    guard_pages: GuardPageManager::new(),
    regions: alloc::vec::Vec::new(),
});

/// Add a guard page
pub fn add_guard_page(addr: VirtAddr) -> Result<(), &'static str> {
    PROTECTION_MANAGER.lock().add_guard_page(addr)
}

/// Check if an address is a guard page
pub fn is_guard_page(addr: VirtAddr) -> bool {
    PROTECTION_MANAGER.lock().is_guard_page(addr)
}

/// Add a protected region
pub fn add_protected_region(region: ProtectedRegion) {
    PROTECTION_MANAGER.lock().add_region(region);
}

/// Check if an access is allowed
pub fn check_memory_access(addr: VirtAddr, write: bool, exec: bool) -> bool {
    PROTECTION_MANAGER.lock().check_access(addr, write, exec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_protection_flags() {
        assert!(MemoryProtection::READ.allows_read());
        assert!(!MemoryProtection::READ.allows_write());
        assert!(!MemoryProtection::READ.allows_exec());

        assert!(MemoryProtection::READ_WRITE.allows_read());
        assert!(MemoryProtection::READ_WRITE.allows_write());
        assert!(!MemoryProtection::READ_WRITE.allows_exec());

        assert!(MemoryProtection::READ_EXEC.allows_read());
        assert!(!MemoryProtection::READ_EXEC.allows_write());
        assert!(MemoryProtection::READ_EXEC.allows_exec());
    }

    #[test]
    fn test_protection_to_page_flags() {
        let prot = MemoryProtection::READ;
        let flags = prot.to_page_flags();
        assert!(flags.contains(PageTableFlags::PRESENT));
        assert!(!flags.contains(PageTableFlags::WRITABLE));
        assert!(flags.contains(PageTableFlags::NO_EXECUTE));

        let prot = MemoryProtection::READ_WRITE;
        let flags = prot.to_page_flags();
        assert!(flags.contains(PageTableFlags::PRESENT));
        assert!(flags.contains(PageTableFlags::WRITABLE));
    }

    #[test]
    fn test_guard_page_manager() {
        let mut manager = GuardPageManager::new();
        let addr = VirtAddr::new(0x1000);

        assert!(!manager.is_guard_page(addr));
        assert_eq!(manager.count(), 0);

        manager.add_guard_page(addr).unwrap();
        assert!(manager.is_guard_page(addr));
        assert_eq!(manager.count(), 1);

        // Check address within the page
        assert!(manager.is_guard_page(VirtAddr::new(0x1500)));

        manager.remove_guard_page(addr);
        assert!(!manager.is_guard_page(addr));
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_guard_page_alignment() {
        let mut manager = GuardPageManager::new();
        let unaligned = VirtAddr::new(0x1001);

        let result = manager.add_guard_page(unaligned);
        assert!(result.is_err());
    }

    #[test]
    fn test_protected_region() {
        let region = ProtectedRegion::new(
            VirtAddr::new(0x1000),
            0x2000,
            MemoryProtection::READ,
            "test"
        );

        assert_eq!(region.start.as_u64(), 0x1000);
        assert_eq!(region.size, 0x2000);
        assert_eq!(region.end().as_u64(), 0x3000);

        assert!(region.contains(VirtAddr::new(0x1000)));
        assert!(region.contains(VirtAddr::new(0x2000)));
        assert!(!region.contains(VirtAddr::new(0x3000)));
    }

    #[test]
    fn test_memory_protection_manager() {
        let mut manager = MemoryProtectionManager::new();

        // Add a guard page
        let guard_addr = VirtAddr::new(0x1000);
        manager.add_guard_page(guard_addr).unwrap();

        // Should deny all access to guard page
        assert!(!manager.check_access(guard_addr, false, false));
        assert!(!manager.check_access(guard_addr, true, false));

        // Add a protected region
        let region = ProtectedRegion::new(
            VirtAddr::new(0x2000),
            0x1000,
            MemoryProtection::READ,
            "test"
        );
        manager.add_region(region);

        // Should allow read, deny write
        assert!(manager.check_access(VirtAddr::new(0x2000), false, false));
        assert!(!manager.check_access(VirtAddr::new(0x2000), true, false));
    }
}
