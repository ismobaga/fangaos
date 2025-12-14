//! Partition Table Support
//!
//! This module provides support for MBR and GPT partition tables.

pub mod mbr;
pub mod gpt;

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use crate::storage::block_device::{BlockDevice, BlockDeviceError};

pub use mbr::{MbrPartitionTable, MbrPartitionEntry};
pub use gpt::{GptPartitionTable, GptPartitionEntry};

/// Partition table types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionTableType {
    Mbr,
    Gpt,
}

/// Partition type identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionType {
    /// FAT32 partition
    Fat32,
    /// NTFS partition
    Ntfs,
    /// EXT partition (Linux)
    Ext,
    /// Swap partition
    Swap,
    /// Unknown/Other type
    Unknown,
}

/// Generic partition entry
#[derive(Debug, Clone)]
pub struct Partition {
    /// Partition number (1-based)
    pub number: usize,
    /// Partition type
    pub ptype: PartitionType,
    /// Starting LBA
    pub start_lba: u64,
    /// Size in sectors
    pub size: u64,
    /// Partition label (if available)
    pub label: Option<String>,
}

impl Partition {
    /// Create a new partition entry
    pub fn new(number: usize, ptype: PartitionType, start_lba: u64, size: u64) -> Self {
        Self {
            number,
            ptype,
            start_lba,
            size,
            label: None,
        }
    }
    
    /// Set partition label
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
}

/// Trait for partition table implementations
pub trait PartitionTable {
    /// Parse partition table from a block device
    fn parse(device: &dyn BlockDevice) -> Result<Vec<Partition>, BlockDeviceError>;
    
    /// Get partition table type
    fn table_type() -> PartitionTableType;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_partition_creation() {
        let partition = Partition::new(1, PartitionType::Fat32, 2048, 1024000);
        assert_eq!(partition.number, 1);
        assert_eq!(partition.ptype, PartitionType::Fat32);
        assert_eq!(partition.start_lba, 2048);
        assert_eq!(partition.size, 1024000);
        assert!(partition.label.is_none());
    }
    
    #[test]
    fn test_partition_with_label() {
        let partition = Partition::new(1, PartitionType::Fat32, 2048, 1024000)
            .with_label(String::from("DATA"));
        assert_eq!(partition.label, Some(String::from("DATA")));
    }
}
