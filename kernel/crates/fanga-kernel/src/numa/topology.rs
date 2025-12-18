//! NUMA Topology Detection and Management

extern crate alloc;
use alloc::vec::Vec;
use crate::smp::CpuId;

/// Maximum number of NUMA nodes
pub const MAX_NUMA_NODES: usize = 64;

/// NUMA node ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NumaNodeId(pub usize);

impl NumaNodeId {
    /// Create a new NUMA node ID
    pub const fn new(id: usize) -> Self {
        Self(id)
    }
    
    /// Get the raw ID value
    pub const fn as_usize(&self) -> usize {
        self.0
    }
}

/// NUMA node information
#[derive(Debug, Clone)]
pub struct NumaNode {
    /// Node ID
    pub id: NumaNodeId,
    
    /// CPUs in this node
    pub cpus: Vec<CpuId>,
    
    /// Base address of memory in this node
    pub mem_base: u64,
    
    /// Size of memory in this node
    pub mem_size: u64,
    
    /// Distance to other nodes (latency metric)
    pub distances: [u8; MAX_NUMA_NODES],
}

impl NumaNode {
    /// Create a new NUMA node
    pub fn new(id: NumaNodeId) -> Self {
        Self {
            id,
            cpus: Vec::new(),
            mem_base: 0,
            mem_size: 0,
            distances: [255; MAX_NUMA_NODES], // 255 = unreachable
        }
    }
    
    /// Add a CPU to this node
    pub fn add_cpu(&mut self, cpu_id: CpuId) {
        if !self.cpus.contains(&cpu_id) {
            self.cpus.push(cpu_id);
        }
    }
    
    /// Get distance to another node
    pub fn distance_to(&self, other: NumaNodeId) -> u8 {
        self.distances[other.as_usize()]
    }
    
    /// Set distance to another node
    pub fn set_distance(&mut self, other: NumaNodeId, distance: u8) {
        self.distances[other.as_usize()] = distance;
    }
}

/// NUMA topology
pub struct NumaTopology {
    /// List of NUMA nodes
    nodes: Vec<NumaNode>,
    
    /// Is NUMA supported?
    enabled: bool,
}

impl NumaTopology {
    /// Create a new NUMA topology
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            enabled: false,
        }
    }
    
    /// Detect NUMA topology
    pub fn detect(&mut self) -> Result<(), &'static str> {
        // TODO: Parse ACPI SRAT (System Resource Affinity Table)
        // and SLIT (System Locality Information Table)
        
        // For now, create a single node (UMA system)
        let mut node = NumaNode::new(NumaNodeId::new(0));
        node.mem_base = 0;
        node.mem_size = 0x100000000; // 4 GB
        node.add_cpu(CpuId::new(0));
        node.set_distance(NumaNodeId::new(0), 10); // Local distance
        
        self.nodes.push(node);
        self.enabled = false; // Not true NUMA, just single node
        
        Ok(())
    }
    
    /// Get the number of NUMA nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    /// Check if NUMA is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Get a NUMA node
    pub fn get_node(&self, id: NumaNodeId) -> Option<&NumaNode> {
        self.nodes.get(id.as_usize())
    }
    
    /// Get a mutable NUMA node
    pub fn get_node_mut(&mut self, id: NumaNodeId) -> Option<&mut NumaNode> {
        self.nodes.get_mut(id.as_usize())
    }
    
    /// Find the NUMA node for a CPU
    pub fn node_for_cpu(&self, cpu_id: CpuId) -> Option<NumaNodeId> {
        for node in &self.nodes {
            if node.cpus.contains(&cpu_id) {
                return Some(node.id);
            }
        }
        None
    }
    
    /// Find the closest NUMA node to a given node
    pub fn closest_node(&self, from: NumaNodeId) -> Option<NumaNodeId> {
        let node = self.get_node(from)?;
        let mut closest = from;
        let mut min_distance = u8::MAX;
        
        for other_node in &self.nodes {
            if other_node.id == from {
                continue;
            }
            
            let distance = node.distance_to(other_node.id);
            if distance < min_distance {
                min_distance = distance;
                closest = other_node.id;
            }
        }
        
        Some(closest)
    }
}

/// Get the NUMA topology (convenience function)
pub fn numa_topology() -> &'static spin::Mutex<NumaTopology> {
    super::get_numa_topology()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_numa_node_id() {
        let id = NumaNodeId::new(5);
        assert_eq!(id.as_usize(), 5);
    }
    
    #[test]
    fn test_numa_node_creation() {
        let node = NumaNode::new(NumaNodeId::new(0));
        assert_eq!(node.id.as_usize(), 0);
        assert_eq!(node.cpus.len(), 0);
    }
    
    #[test]
    fn test_numa_node_add_cpu() {
        let mut node = NumaNode::new(NumaNodeId::new(0));
        node.add_cpu(CpuId::new(0));
        node.add_cpu(CpuId::new(1));
        
        assert_eq!(node.cpus.len(), 2);
        assert!(node.cpus.contains(&CpuId::new(0)));
    }
    
    #[test]
    fn test_numa_topology_detect() {
        let mut topology = NumaTopology::new();
        topology.detect().unwrap();
        
        assert_eq!(topology.node_count(), 1);
        assert!(!topology.is_enabled()); // Single node = UMA
    }
    
    #[test]
    fn test_numa_node_for_cpu() {
        let mut topology = NumaTopology::new();
        topology.detect().unwrap();
        
        let node_id = topology.node_for_cpu(CpuId::new(0));
        assert_eq!(node_id, Some(NumaNodeId::new(0)));
    }
}
