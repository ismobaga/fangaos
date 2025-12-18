//! NUMA Memory Policies

use super::topology::NumaNodeId;

/// NUMA memory policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumaPolicy {
    /// Default policy - allocate from any node
    Default,
    
    /// Bind to specific nodes
    Bind,
    
    /// Interleave across nodes
    Interleave,
    
    /// Prefer specific node but allow fallback
    Preferred,
}

/// NUMA memory policy configuration
#[derive(Debug, Clone)]
pub struct NumaMemoryPolicy {
    /// Policy type
    pub policy: NumaPolicy,
    
    /// Allowed nodes (bitmask)
    pub allowed_nodes: u64,
    
    /// Preferred node (for Preferred policy)
    pub preferred_node: Option<NumaNodeId>,
}

impl NumaMemoryPolicy {
    /// Create a default policy
    pub fn default() -> Self {
        Self {
            policy: NumaPolicy::Default,
            allowed_nodes: u64::MAX, // All nodes allowed
            preferred_node: None,
        }
    }
    
    /// Create a bind policy
    pub fn bind(nodes: &[NumaNodeId]) -> Self {
        let mut allowed_nodes = 0u64;
        for node in nodes {
            allowed_nodes |= 1 << node.as_usize();
        }
        
        Self {
            policy: NumaPolicy::Bind,
            allowed_nodes,
            preferred_node: None,
        }
    }
    
    /// Create an interleave policy
    pub fn interleave(nodes: &[NumaNodeId]) -> Self {
        let mut allowed_nodes = 0u64;
        for node in nodes {
            allowed_nodes |= 1 << node.as_usize();
        }
        
        Self {
            policy: NumaPolicy::Interleave,
            allowed_nodes,
            preferred_node: None,
        }
    }
    
    /// Create a preferred policy
    pub fn preferred(node: NumaNodeId) -> Self {
        Self {
            policy: NumaPolicy::Preferred,
            allowed_nodes: u64::MAX,
            preferred_node: Some(node),
        }
    }
    
    /// Check if a node is allowed by this policy
    pub fn is_node_allowed(&self, node: NumaNodeId) -> bool {
        (self.allowed_nodes & (1 << node.as_usize())) != 0
    }
    
    /// Get the next node to allocate from (for interleave)
    pub fn next_node(&self, current: NumaNodeId) -> Option<NumaNodeId> {
        if self.policy != NumaPolicy::Interleave {
            return None;
        }
        
        // Find next allowed node after current
        let start = current.as_usize() + 1;
        for i in start..64 {
            if self.is_node_allowed(NumaNodeId::new(i)) {
                return Some(NumaNodeId::new(i));
            }
        }
        
        // Wrap around
        for i in 0..start {
            if self.is_node_allowed(NumaNodeId::new(i)) {
                return Some(NumaNodeId::new(i));
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_numa_policy_default() {
        let policy = NumaMemoryPolicy::default();
        assert_eq!(policy.policy, NumaPolicy::Default);
        assert!(policy.is_node_allowed(NumaNodeId::new(0)));
        assert!(policy.is_node_allowed(NumaNodeId::new(63)));
    }
    
    #[test]
    fn test_numa_policy_bind() {
        let policy = NumaMemoryPolicy::bind(&[NumaNodeId::new(0), NumaNodeId::new(2)]);
        assert_eq!(policy.policy, NumaPolicy::Bind);
        assert!(policy.is_node_allowed(NumaNodeId::new(0)));
        assert!(!policy.is_node_allowed(NumaNodeId::new(1)));
        assert!(policy.is_node_allowed(NumaNodeId::new(2)));
    }
    
    #[test]
    fn test_numa_policy_preferred() {
        let policy = NumaMemoryPolicy::preferred(NumaNodeId::new(1));
        assert_eq!(policy.policy, NumaPolicy::Preferred);
        assert_eq!(policy.preferred_node, Some(NumaNodeId::new(1)));
    }
    
    #[test]
    fn test_numa_policy_interleave() {
        let policy = NumaMemoryPolicy::interleave(&[NumaNodeId::new(0), NumaNodeId::new(2)]);
        assert_eq!(policy.policy, NumaPolicy::Interleave);
        
        let next = policy.next_node(NumaNodeId::new(0));
        assert_eq!(next, Some(NumaNodeId::new(2)));
        
        let next = policy.next_node(NumaNodeId::new(2));
        assert_eq!(next, Some(NumaNodeId::new(0))); // Should wrap around
    }
}
