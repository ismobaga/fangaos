//! GPT (GUID Partition Table) Support
//!
//! This module implements GPT partition table parsing.

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use crate::storage::block_device::{BlockDevice, BlockDeviceError};
use crate::storage::partition::{Partition, PartitionTable, PartitionTableType, PartitionType};

/// GPT signature ("EFI PART")
const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";

/// GPT header location (LBA 1)
const GPT_HEADER_LBA: u64 = 1;

/// GPT partition entry size
const GPT_ENTRY_SIZE: usize = 128;

/// GPT partition table header
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GptHeader {
    /// Signature ("EFI PART")
    pub signature: [u8; 8],
    /// Revision (usually 0x00010000)
    pub revision: u32,
    /// Header size in bytes
    pub header_size: u32,
    /// CRC32 of header
    pub header_crc32: u32,
    /// Reserved (must be zero)
    pub reserved: u32,
    /// Current LBA (location of this header)
    pub current_lba: u64,
    /// Backup LBA (location of backup header)
    pub backup_lba: u64,
    /// First usable LBA for partitions
    pub first_usable_lba: u64,
    /// Last usable LBA for partitions
    pub last_usable_lba: u64,
    /// Disk GUID
    pub disk_guid: [u8; 16],
    /// Starting LBA of partition entries
    pub partition_entry_lba: u64,
    /// Number of partition entries
    pub num_partition_entries: u32,
    /// Size of a partition entry
    pub partition_entry_size: u32,
    /// CRC32 of partition array
    pub partition_array_crc32: u32,
}

/// GPT partition entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GptPartitionEntry {
    /// Partition type GUID
    pub partition_type_guid: [u8; 16],
    /// Unique partition GUID
    pub unique_guid: [u8; 16],
    /// Starting LBA
    pub start_lba: u64,
    /// Ending LBA (inclusive)
    pub end_lba: u64,
    /// Attribute flags
    pub attributes: u64,
    /// Partition name (36 UTF-16LE characters)
    pub name: [u16; 36],
}

impl GptPartitionEntry {
    /// Check if this partition entry is used
    pub fn is_used(&self) -> bool {
        // If partition type GUID is all zeros, entry is unused
        self.partition_type_guid.iter().any(|&b| b != 0)
    }
    
    /// Get partition type from GUID
    pub fn get_partition_type(&self) -> PartitionType {
        // Well-known partition type GUIDs
        const FAT32_GUID: [u8; 16] = [
            0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11,
            0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B,
        ];
        const LINUX_GUID: [u8; 16] = [
            0xAF, 0x3D, 0xC6, 0x0F, 0x83, 0x84, 0x72, 0x47,
            0x8E, 0x79, 0x3D, 0x69, 0xD8, 0x47, 0x7D, 0xE4,
        ];
        
        if self.partition_type_guid == FAT32_GUID {
            PartitionType::Fat32
        } else if self.partition_type_guid == LINUX_GUID {
            PartitionType::Ext
        } else {
            PartitionType::Unknown
        }
    }
    
    /// Get partition name as String
    pub fn get_name(&self) -> String {
        // Convert UTF-16LE to String
        // Copy name field to avoid alignment issues
        let name_copy = self.name;
        let mut name = String::new();
        for c in name_copy {
            if c == 0 {
                break;
            }
            if let Some(ch) = char::from_u32(c as u32) {
                name.push(ch);
            }
        }
        name
    }
    
    /// Get partition size in sectors
    pub fn size(&self) -> u64 {
        if self.end_lba >= self.start_lba {
            self.end_lba - self.start_lba + 1
        } else {
            0
        }
    }
}

/// GPT partition table
pub struct GptPartitionTable;

impl PartitionTable for GptPartitionTable {
    fn parse(device: &dyn BlockDevice) -> Result<Vec<Partition>, BlockDeviceError> {
        // Read GPT header (LBA 1)
        let mut header_buffer = [0u8; 512];
        device.read_blocks(GPT_HEADER_LBA, &mut header_buffer)?;
        
        // Parse header
        let header = unsafe {
            core::ptr::read_unaligned(header_buffer.as_ptr() as *const GptHeader)
        };
        
        // Verify signature
        if &header.signature != GPT_SIGNATURE {
            return Err(BlockDeviceError::IoError);
        }
        
        let mut partitions = Vec::new();
        let entries_per_block = device.block_size() / GPT_ENTRY_SIZE;
        
        // Read partition entries
        for i in 0..header.num_partition_entries as usize {
            let block_offset = i / entries_per_block;
            let entry_offset = (i % entries_per_block) * GPT_ENTRY_SIZE;
            
            // Read the block containing this entry if needed
            let mut buffer = [0u8; 512];
            device.read_blocks(header.partition_entry_lba + block_offset as u64, &mut buffer)?;
            
            // Parse entry
            let entry = unsafe {
                core::ptr::read_unaligned(
                    buffer[entry_offset..].as_ptr() as *const GptPartitionEntry
                )
            };
            
            // Only add used partitions
            if entry.is_used() {
                let name = entry.get_name();
                let partition = Partition::new(
                    i + 1,
                    entry.get_partition_type(),
                    entry.start_lba,
                    entry.size(),
                ).with_label(name);
                
                partitions.push(partition);
            }
        }
        
        Ok(partitions)
    }
    
    fn table_type() -> PartitionTableType {
        PartitionTableType::Gpt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpt_entry_used() {
        let mut entry = GptPartitionEntry {
            partition_type_guid: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            unique_guid: [0; 16],
            start_lba: 2048,
            end_lba: 104447,
            attributes: 0,
            name: [0; 36],
        };
        assert!(entry.is_used());
        
        entry.partition_type_guid = [0; 16];
        assert!(!entry.is_used());
    }
    
    #[test]
    fn test_gpt_entry_size() {
        let entry = GptPartitionEntry {
            partition_type_guid: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            unique_guid: [0; 16],
            start_lba: 2048,
            end_lba: 104447,
            attributes: 0,
            name: [0; 36],
        };
        assert_eq!(entry.size(), 102400);
    }
}
