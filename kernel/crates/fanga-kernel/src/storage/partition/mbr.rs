//! MBR (Master Boot Record) Partition Table
//!
//! This module implements MBR partition table parsing.

extern crate alloc;
use alloc::vec::Vec;
use crate::storage::block_device::{BlockDevice, BlockDeviceError};
use crate::storage::partition::{Partition, PartitionTable, PartitionTableType, PartitionType};

/// MBR signature (last 2 bytes of sector 0)
const MBR_SIGNATURE: u16 = 0xAA55;

/// MBR partition entry offset
const MBR_PARTITION_TABLE_OFFSET: usize = 0x1BE;

/// Number of partition entries in MBR
const MBR_PARTITION_COUNT: usize = 4;

/// MBR partition entry (16 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MbrPartitionEntry {
    /// Boot indicator (0x80 = bootable, 0x00 = non-bootable)
    pub boot_indicator: u8,
    /// Starting CHS address (3 bytes)
    pub start_chs: [u8; 3],
    /// Partition type
    pub partition_type: u8,
    /// Ending CHS address (3 bytes)
    pub end_chs: [u8; 3],
    /// Starting LBA
    pub start_lba: u32,
    /// Size in sectors
    pub size_sectors: u32,
}

impl MbrPartitionEntry {
    /// Check if this partition entry is valid
    pub fn is_valid(&self) -> bool {
        self.partition_type != 0 && self.size_sectors > 0
    }
    
    /// Convert partition type byte to PartitionType enum
    pub fn get_partition_type(&self) -> PartitionType {
        match self.partition_type {
            0x0B | 0x0C => PartitionType::Fat32,
            0x07 => PartitionType::Ntfs,
            0x83 => PartitionType::Ext,
            0x82 => PartitionType::Swap,
            _ => PartitionType::Unknown,
        }
    }
}

/// MBR partition table
pub struct MbrPartitionTable;

impl PartitionTable for MbrPartitionTable {
    fn parse(device: &dyn BlockDevice) -> Result<Vec<Partition>, BlockDeviceError> {
        // Read the first sector (MBR)
        let mut buffer = [0u8; 512];
        device.read_blocks(0, &mut buffer)?;
        
        // Verify MBR signature
        let signature = u16::from_le_bytes([buffer[510], buffer[511]]);
        if signature != MBR_SIGNATURE {
            return Err(BlockDeviceError::IoError);
        }
        
        let mut partitions = Vec::new();
        
        // Parse partition entries
        for i in 0..MBR_PARTITION_COUNT {
            let offset = MBR_PARTITION_TABLE_OFFSET + (i * 16);
            let entry_bytes = &buffer[offset..offset + 16];
            
            // Parse partition entry
            let entry = MbrPartitionEntry {
                boot_indicator: entry_bytes[0],
                start_chs: [entry_bytes[1], entry_bytes[2], entry_bytes[3]],
                partition_type: entry_bytes[4],
                end_chs: [entry_bytes[5], entry_bytes[6], entry_bytes[7]],
                start_lba: u32::from_le_bytes([
                    entry_bytes[8],
                    entry_bytes[9],
                    entry_bytes[10],
                    entry_bytes[11],
                ]),
                size_sectors: u32::from_le_bytes([
                    entry_bytes[12],
                    entry_bytes[13],
                    entry_bytes[14],
                    entry_bytes[15],
                ]),
            };
            
            // Only add valid partitions
            if entry.is_valid() {
                let partition = Partition::new(
                    i + 1,
                    entry.get_partition_type(),
                    entry.start_lba as u64,
                    entry.size_sectors as u64,
                );
                partitions.push(partition);
            }
        }
        
        Ok(partitions)
    }
    
    fn table_type() -> PartitionTableType {
        PartitionTableType::Mbr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mbr_entry_validity() {
        let valid_entry = MbrPartitionEntry {
            boot_indicator: 0x00,
            start_chs: [0, 0, 0],
            partition_type: 0x0B, // FAT32
            end_chs: [0, 0, 0],
            start_lba: 2048,
            size_sectors: 102400,
        };
        assert!(valid_entry.is_valid());
        
        let invalid_entry = MbrPartitionEntry {
            boot_indicator: 0x00,
            start_chs: [0, 0, 0],
            partition_type: 0x00, // Empty
            end_chs: [0, 0, 0],
            start_lba: 0,
            size_sectors: 0,
        };
        assert!(!invalid_entry.is_valid());
    }
    
    #[test]
    fn test_partition_type_detection() {
        let fat32_entry = MbrPartitionEntry {
            boot_indicator: 0x00,
            start_chs: [0, 0, 0],
            partition_type: 0x0B,
            end_chs: [0, 0, 0],
            start_lba: 2048,
            size_sectors: 102400,
        };
        assert_eq!(fat32_entry.get_partition_type(), PartitionType::Fat32);
        
        let ntfs_entry = MbrPartitionEntry {
            boot_indicator: 0x00,
            start_chs: [0, 0, 0],
            partition_type: 0x07,
            end_chs: [0, 0, 0],
            start_lba: 2048,
            size_sectors: 102400,
        };
        assert_eq!(ntfs_entry.get_partition_type(), PartitionType::Ntfs);
    }
}
