//! FAT32 Boot Sector
//!
//! This module handles parsing of FAT32 boot sector (BPB - BIOS Parameter Block).

use crate::storage::block_device::{BlockDevice, BlockDeviceError};

/// FAT32 Boot Sector structure
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Fat32BootSector {
    /// Jump instruction (3 bytes)
    pub jump: [u8; 3],
    /// OEM name (8 bytes)
    pub oem_name: [u8; 8],
    /// Bytes per sector (usually 512)
    pub bytes_per_sector: u16,
    /// Sectors per cluster
    pub sectors_per_cluster: u8,
    /// Reserved sectors (including boot sector)
    pub reserved_sectors: u16,
    /// Number of FATs (usually 2)
    pub num_fats: u8,
    /// Root directory entries (0 for FAT32)
    pub root_entries: u16,
    /// Total sectors (16-bit, 0 for FAT32)
    pub total_sectors_16: u16,
    /// Media descriptor
    pub media_descriptor: u8,
    /// Sectors per FAT (16-bit, 0 for FAT32)
    pub sectors_per_fat_16: u16,
    /// Sectors per track
    pub sectors_per_track: u16,
    /// Number of heads
    pub num_heads: u16,
    /// Hidden sectors
    pub hidden_sectors: u32,
    /// Total sectors (32-bit)
    pub total_sectors_32: u32,
    /// Sectors per FAT (32-bit, for FAT32)
    pub sectors_per_fat_32: u32,
    /// Extended flags
    pub extended_flags: u16,
    /// Filesystem version
    pub fs_version: u16,
    /// Root directory cluster
    pub root_cluster: u32,
    /// FSInfo sector
    pub fsinfo_sector: u16,
    /// Backup boot sector
    pub backup_boot_sector: u16,
    /// Reserved
    pub reserved: [u8; 12],
    /// Drive number
    pub drive_number: u8,
    /// Reserved
    pub reserved1: u8,
    /// Boot signature
    pub boot_signature: u8,
    /// Volume ID
    pub volume_id: u32,
    /// Volume label (11 bytes)
    pub volume_label: [u8; 11],
    /// Filesystem type (8 bytes, should be "FAT32   ")
    pub fs_type: [u8; 8],
}

impl Default for Fat32BootSector {
    fn default() -> Self {
        Self {
            jump: [0; 3],
            oem_name: [0; 8],
            bytes_per_sector: 512,
            sectors_per_cluster: 8,
            reserved_sectors: 32,
            num_fats: 2,
            root_entries: 0,
            total_sectors_16: 0,
            media_descriptor: 0xF8,
            sectors_per_fat_16: 0,
            sectors_per_track: 0,
            num_heads: 0,
            hidden_sectors: 0,
            total_sectors_32: 0,
            sectors_per_fat_32: 1024, // Non-zero for FAT32
            extended_flags: 0,
            fs_version: 0,
            root_cluster: 2,
            fsinfo_sector: 1,
            backup_boot_sector: 6,
            reserved: [0; 12],
            drive_number: 0,
            reserved1: 0,
            boot_signature: 0x29,
            volume_id: 0,
            volume_label: [0; 11],
            fs_type: [0; 8],
        }
    }
}

impl Fat32BootSector {
    /// Read and parse boot sector from device
    pub fn read(device: &dyn BlockDevice, partition_start: u64) -> Result<Self, BlockDeviceError> {
        let mut buffer = [0u8; 512];
        device.read_blocks(partition_start, &mut buffer)?;
        
        // Parse boot sector
        let boot_sector = unsafe {
            core::ptr::read_unaligned(buffer.as_ptr() as *const Fat32BootSector)
        };
        
        // Verify it's FAT32
        if boot_sector.bytes_per_sector != 512 && boot_sector.bytes_per_sector != 1024 
           && boot_sector.bytes_per_sector != 2048 && boot_sector.bytes_per_sector != 4096 {
            return Err(BlockDeviceError::IoError);
        }
        
        if boot_sector.sectors_per_fat_32 == 0 {
            return Err(BlockDeviceError::IoError);
        }
        
        Ok(boot_sector)
    }
    
    /// Check if this is a valid FAT32 boot sector
    pub fn is_valid(&self) -> bool {
        // Copy bytes_per_sector to avoid alignment issues
        let bps = self.bytes_per_sector;
        
        // Check for valid bytes per sector
        if ![512, 1024, 2048, 4096].contains(&bps) {
            return false;
        }
        
        // Check for FAT32 (sectors_per_fat_32 must be non-zero)
        if self.sectors_per_fat_32 == 0 {
            return false;
        }
        
        // Check for valid cluster size
        if ![1, 2, 4, 8, 16, 32, 64, 128].contains(&self.sectors_per_cluster) {
            return false;
        }
        
        true
    }
    
    /// Get the first FAT sector
    pub fn fat_start_sector(&self) -> u32 {
        self.reserved_sectors as u32
    }
    
    /// Get the first data sector
    pub fn data_start_sector(&self) -> u32 {
        self.fat_start_sector() + (self.num_fats as u32 * self.sectors_per_fat_32)
    }
    
    /// Get total data clusters
    pub fn total_clusters(&self) -> u32 {
        let data_start = self.data_start_sector();
        if self.total_sectors_32 <= data_start {
            return 0;
        }
        let data_sectors = self.total_sectors_32 - data_start;
        data_sectors / self.sectors_per_cluster as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_boot_sector_validity() {
        let mut bs = Fat32BootSector::default();
        assert!(bs.is_valid());
        
        bs.bytes_per_sector = 256; // Invalid
        assert!(!bs.is_valid());
        
        bs.bytes_per_sector = 512;
        bs.sectors_per_fat_32 = 0; // Invalid for FAT32
        assert!(!bs.is_valid());
    }
    
    #[test]
    fn test_sector_calculations() {
        let bs = Fat32BootSector {
            reserved_sectors: 32,
            num_fats: 2,
            sectors_per_fat_32: 1024,
            sectors_per_cluster: 8,
            total_sectors_32: 204800,
            ..Default::default()
        };
        
        assert_eq!(bs.fat_start_sector(), 32);
        assert_eq!(bs.data_start_sector(), 32 + 2 * 1024);
        
        let data_sectors = bs.total_sectors_32 - bs.data_start_sector();
        let clusters = data_sectors / 8;
        assert_eq!(bs.total_clusters(), clusters);
    }
}
