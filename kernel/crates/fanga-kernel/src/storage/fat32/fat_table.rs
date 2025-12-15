//! FAT Table Management
//!
//! This module handles reading and writing the File Allocation Table.

extern crate alloc;
use alloc::vec::Vec;
use crate::storage::fat32::boot_sector::Fat32BootSector;

/// FAT entry values
pub const FAT_FREE: u32 = 0x00000000;
pub const FAT_RESERVED_MIN: u32 = 0x0FFFFFF0;
pub const FAT_BAD_CLUSTER: u32 = 0x0FFFFFF7;
pub const FAT_EOC_MIN: u32 = 0x0FFFFFF8; // End of chain minimum
pub const FAT_EOC: u32 = 0x0FFFFFFF; // End of chain

/// FAT table structure
pub struct FatTable {
    /// Cached FAT entries
    entries: Vec<u32>,
    /// Total number of clusters
    total_clusters: u32,
}

impl FatTable {
    /// Create a new FAT table
    pub fn new(boot_sector: &Fat32BootSector) -> Self {
        let total_clusters = boot_sector.total_clusters();
        Self {
            entries: Vec::new(),
            total_clusters,
        }
    }
    
    /// Get the next cluster in a chain
    pub fn get_next_cluster(&self, cluster: u32) -> Option<u32> {
        if cluster as usize >= self.entries.len() {
            return None;
        }
        
        let next = self.entries[cluster as usize] & 0x0FFFFFFF;
        
        if next == FAT_FREE {
            None
        } else if next >= FAT_EOC_MIN {
            None // End of chain
        } else if next == FAT_BAD_CLUSTER {
            None
        } else {
            Some(next)
        }
    }
    
    /// Set the next cluster in a chain
    pub fn set_next_cluster(&mut self, cluster: u32, next: u32) -> Result<(), &'static str> {
        if cluster as usize >= self.entries.len() {
            return Err("Cluster out of range");
        }
        
        // Preserve upper 4 bits, set lower 28 bits
        let masked_next = next & 0x0FFFFFFF;
        self.entries[cluster as usize] = masked_next;
        Ok(())
    }
    
    /// Mark a cluster as free
    pub fn free_cluster(&mut self, cluster: u32) -> Result<(), &'static str> {
        self.set_next_cluster(cluster, FAT_FREE)
    }
    
    /// Mark a cluster as end of chain
    pub fn mark_eoc(&mut self, cluster: u32) -> Result<(), &'static str> {
        self.set_next_cluster(cluster, FAT_EOC)
    }
    
    /// Find a free cluster
    pub fn find_free_cluster(&self) -> Option<u32> {
        for (i, &entry) in self.entries.iter().enumerate() {
            if i >= 2 && (entry & 0x0FFFFFFF) == FAT_FREE {
                return Some(i as u32);
            }
        }
        None
    }
    
    /// Allocate a new cluster
    pub fn allocate_cluster(&mut self) -> Option<u32> {
        if let Some(cluster) = self.find_free_cluster() {
            let _ = self.mark_eoc(cluster);
            Some(cluster)
        } else {
            None
        }
    }
    
    /// Get cluster chain starting from a given cluster
    pub fn get_chain(&self, start_cluster: u32) -> Vec<u32> {
        let mut chain = Vec::new();
        let mut current = start_cluster;
        
        while let Some(next) = self.get_next_cluster(current) {
            chain.push(current);
            if chain.len() > self.total_clusters as usize {
                break; // Prevent infinite loops on corrupt FAT
            }
            current = next;
        }
        
        // Add the last cluster (EOC)
        if current >= 2 && current < self.total_clusters {
            chain.push(current);
        }
        
        chain
    }
    
    /// Free an entire cluster chain
    pub fn free_chain(&mut self, start_cluster: u32) -> Result<(), &'static str> {
        let chain = self.get_chain(start_cluster);
        
        for cluster in chain {
            self.free_cluster(cluster)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fat_table_creation() {
        let bs = Fat32BootSector::default();
        let fat = FatTable::new(&bs);
        assert!(fat.entries.is_empty());
    }
    
    #[test]
    fn test_cluster_chain_operations() {
        let bs = Fat32BootSector {
            total_sectors_32: 204800,
            sectors_per_cluster: 8,
            reserved_sectors: 32,
            num_fats: 2,
            sectors_per_fat_32: 1024,
            ..Default::default()
        };
        
        let mut fat = FatTable::new(&bs);
        
        // Initialize with some entries
        fat.entries = vec![0; 100];
        fat.entries[0] = 0x0FFFFFF8; // Reserved
        fat.entries[1] = 0x0FFFFFFF; // Reserved
        
        // Create a simple chain: 2 -> 3 -> EOC
        fat.entries[2] = 3;
        fat.entries[3] = FAT_EOC;
        
        assert_eq!(fat.get_next_cluster(2), Some(3));
        assert_eq!(fat.get_next_cluster(3), None); // EOC
        
        let chain = fat.get_chain(2);
        assert_eq!(chain, vec![2, 3]);
    }
    
    #[test]
    fn test_free_cluster_allocation() {
        let bs = Fat32BootSector::default();
        let mut fat = FatTable::new(&bs);
        
        // Initialize with some entries
        fat.entries = vec![0; 10];
        fat.entries[0] = FAT_EOC; // Reserved
        fat.entries[1] = FAT_EOC; // Reserved
        fat.entries[2] = FAT_EOC; // Used
        fat.entries[3] = FAT_FREE; // Free
        
        assert_eq!(fat.find_free_cluster(), Some(3));
        
        let allocated = fat.allocate_cluster();
        assert_eq!(allocated, Some(3));
        assert_eq!(fat.entries[3], FAT_EOC);
    }
}
