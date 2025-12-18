//! NUMA (Non-Uniform Memory Access) Support
//!
//! This module provides NUMA topology detection and memory allocation optimization.

pub mod topology;
pub mod allocator;
pub mod policy;

pub use topology::{NumaNode, NumaNodeId, NumaTopology, numa_topology};
pub use allocator::{NumaAllocator, AllocHint};
pub use policy::{NumaPolicy, NumaMemoryPolicy};

use spin::Once;

/// Global NUMA topology
static NUMA_TOPOLOGY: Once<spin::Mutex<NumaTopology>> = Once::new();

/// Initialize NUMA subsystem
pub fn init() -> Result<(), &'static str> {
    let topology = NumaTopology::new();
    NUMA_TOPOLOGY.call_once(|| spin::Mutex::new(topology));
    
    // Detect NUMA topology
    detect_numa()?;
    
    Ok(())
}

/// Detect NUMA topology
fn detect_numa() -> Result<(), &'static str> {
    let mut topology = NUMA_TOPOLOGY.get().ok_or("NUMA topology not initialized")?.lock();
    topology.detect()
}

/// Get reference to NUMA topology
pub fn get_numa_topology() -> &'static spin::Mutex<NumaTopology> {
    NUMA_TOPOLOGY.get().expect("NUMA topology not initialized")
}
