//! FAT32 File System Implementation
//!
//! This module implements read and write support for the FAT32 file system.

pub mod boot_sector;
pub mod fat_table;
pub mod directory;

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::sync::Arc;
use spin::Mutex;
use crate::storage::block_device::{BlockDevice, BlockDeviceError};
use crate::fs::vfs::{FileSystem, VNode, VNodeType, VNodeAttr, DirEntry, FsError};

pub use boot_sector::Fat32BootSector;
pub use fat_table::FatTable;
pub use directory::{DirectoryEntry, DirectoryIterator};

/// FAT32 filesystem implementation
pub struct Fat32FileSystem {
    device: Arc<Mutex<dyn BlockDevice>>,
    boot_sector: Fat32BootSector,
    fat_table: Mutex<FatTable>,
    partition_start: u64,
}

impl Fat32FileSystem {
    /// Create a new FAT32 filesystem from a block device
    pub fn new(device: Arc<Mutex<dyn BlockDevice>>, partition_start: u64) -> Result<Self, BlockDeviceError> {
        // Read boot sector
        let boot_sector = {
            let dev = device.lock();
            Fat32BootSector::read(&*dev, partition_start)?
        };
        
        // Initialize FAT table
        let fat_table = FatTable::new(&boot_sector);
        
        Ok(Self {
            device,
            boot_sector,
            fat_table: Mutex::new(fat_table),
            partition_start,
        })
    }
    
    /// Get the cluster size in bytes
    pub fn cluster_size(&self) -> usize {
        self.boot_sector.bytes_per_sector as usize * self.boot_sector.sectors_per_cluster as usize
    }
    
    /// Convert cluster number to LBA
    fn cluster_to_lba(&self, cluster: u32) -> u64 {
        let first_data_sector = self.boot_sector.reserved_sectors as u64
            + (self.boot_sector.num_fats as u64 * self.boot_sector.sectors_per_fat_32 as u64);
        
        self.partition_start + first_data_sector + ((cluster - 2) as u64 * self.boot_sector.sectors_per_cluster as u64)
    }
    
    /// Read a cluster from disk
    fn read_cluster(&self, cluster: u32, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let lba = self.cluster_to_lba(cluster);
        let sectors = self.boot_sector.sectors_per_cluster as usize;
        let sector_size = self.boot_sector.bytes_per_sector as usize;
        
        if buffer.len() < sectors * sector_size {
            return Err(BlockDeviceError::InvalidBufferSize);
        }
        
        let device = self.device.lock();
        device.read_blocks(lba, &mut buffer[0..sectors * sector_size])
    }
    
    /// Write a cluster to disk
    fn write_cluster(&self, cluster: u32, buffer: &[u8]) -> Result<(), BlockDeviceError> {
        let lba = self.cluster_to_lba(cluster);
        let sectors = self.boot_sector.sectors_per_cluster as usize;
        let sector_size = self.boot_sector.bytes_per_sector as usize;
        
        if buffer.len() < sectors * sector_size {
            return Err(BlockDeviceError::InvalidBufferSize);
        }
        
        let device = self.device.lock();
        device.write_blocks(lba, &buffer[0..sectors * sector_size])
    }
}

// Stub implementation of FileSystem trait for FAT32
// Full implementation would require complete directory and file handling
impl FileSystem for Fat32FileSystem {
    fn root(&self) -> Result<VNode, FsError> {
        Ok(VNode::new(
            self.boot_sector.root_cluster as u64,
            VNodeType::Directory,
            String::from("/"),
        ))
    }
    
    fn lookup(&self, path: &str) -> Result<VNode, FsError> {
        if path == "/" {
            return self.root();
        }
        Err(FsError::NotFound)
    }
    
    fn create(&mut self, _path: &str, _vtype: VNodeType) -> Result<VNode, FsError> {
        Err(FsError::IoError)
    }
    
    fn remove(&mut self, _path: &str) -> Result<(), FsError> {
        Err(FsError::IoError)
    }
    
    fn read(&self, _vnode: &VNode, _offset: usize, _buffer: &mut [u8]) -> Result<usize, FsError> {
        // Stub - would read file data from clusters
        Ok(0)
    }
    
    fn write(&mut self, _vnode: &VNode, _offset: usize, _buffer: &[u8]) -> Result<usize, FsError> {
        // Stub - would write file data to clusters
        Ok(0)
    }
    
    fn stat(&self, vnode: &VNode) -> Result<VNodeAttr, FsError> {
        Ok(VNodeAttr {
            size: 0,
            vtype: vnode.vtype,
        })
    }
    
    fn readdir(&self, _vnode: &VNode) -> Result<Vec<DirEntry>, FsError> {
        // Stub - would read directory entries
        Ok(Vec::new())
    }
    
    fn truncate(&mut self, _vnode: &VNode, _size: usize) -> Result<(), FsError> {
        Err(FsError::IoError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cluster_calculation() {
        // Test with typical FAT32 values
        let bs = Fat32BootSector {
            bytes_per_sector: 512,
            sectors_per_cluster: 8,
            reserved_sectors: 32,
            num_fats: 2,
            sectors_per_fat_32: 1024,
            root_cluster: 2,
            ..Default::default()
        };
        
        let cluster_size = bs.bytes_per_sector as usize * bs.sectors_per_cluster as usize;
        assert_eq!(cluster_size, 4096);
    }
}
