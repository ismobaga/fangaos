//! NUMA-aware Memory Allocator

use super::topology::NumaNodeId;
use crate::memory::{PhysAddr, VirtAddr};

/// Allocation hint for NUMA allocator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocHint {
    /// Allocate from any node
    Any,
    
    /// Prefer allocation from specific node
    PreferNode(NumaNodeId),
    
    /// Strict allocation from specific node only
    StrictNode(NumaNodeId),
    
    /// Allocate from local node (CPU's NUMA node)
    Local,
}

/// NUMA-aware memory allocator
pub struct NumaAllocator {
    /// Enable NUMA-aware allocation
    enabled: bool,
}

impl NumaAllocator {
    /// Create a new NUMA allocator
    pub const fn new() -> Self {
        Self { enabled: false }
    }
    
    /// Enable NUMA-aware allocation
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// Check if NUMA allocation is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Allocate memory with NUMA hint
    pub fn alloc(&self, size: usize, hint: AllocHint) -> Result<PhysAddr, &'static str> {
        if !self.enabled {
            // Fall back to regular allocation
            return self.alloc_regular(size);
        }
        
        match hint {
            AllocHint::Any => self.alloc_regular(size),
            AllocHint::PreferNode(node_id) => self.alloc_from_node(size, node_id, false),
            AllocHint::StrictNode(node_id) => self.alloc_from_node(size, node_id, true),
            AllocHint::Local => self.alloc_local(size),
        }
    }
    
    /// Allocate memory from a specific node
    fn alloc_from_node(&self, size: usize, node_id: NumaNodeId, strict: bool) -> Result<PhysAddr, &'static str> {
        // TODO: Implement actual NUMA-aware allocation
        // This would integrate with the physical memory manager
        
        // For now, fall back to regular allocation
        if strict {
            // In strict mode, we should only allocate from the specific node
            // Return error if node has no memory
            Err("NUMA node has no available memory")
        } else {
            // In prefer mode, fall back to any node
            self.alloc_regular(size)
        }
    }
    
    /// Allocate memory from local node (current CPU's node)
    fn alloc_local(&self, size: usize) -> Result<PhysAddr, &'static str> {
        // TODO: Get current CPU's NUMA node and allocate from it
        self.alloc_regular(size)
    }
    
    /// Regular allocation (no NUMA awareness)
    fn alloc_regular(&self, _size: usize) -> Result<PhysAddr, &'static str> {
        // TODO: Call into physical memory manager
        Err("Not implemented")
    }
    
    /// Free memory allocated with NUMA awareness
    pub fn free(&self, addr: PhysAddr, size: usize) -> Result<(), &'static str> {
        // TODO: Implement deallocation
        // Track which node the memory came from
        let _ = (addr, size); // Silence unused warnings
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_numa_allocator_creation() {
        let allocator = NumaAllocator::new();
        assert!(!allocator.is_enabled());
    }
    
    #[test]
    fn test_numa_allocator_enable() {
        let mut allocator = NumaAllocator::new();
        allocator.enable();
        assert!(allocator.is_enabled());
    }
    
    #[test]
    fn test_alloc_hints() {
        let hint1 = AllocHint::Any;
        let hint2 = AllocHint::PreferNode(NumaNodeId::new(0));
        let hint3 = AllocHint::StrictNode(NumaNodeId::new(1));
        let hint4 = AllocHint::Local;
        
        assert_eq!(hint1, AllocHint::Any);
        assert!(matches!(hint2, AllocHint::PreferNode(_)));
        assert!(matches!(hint3, AllocHint::StrictNode(_)));
        assert_eq!(hint4, AllocHint::Local);
    }
}
