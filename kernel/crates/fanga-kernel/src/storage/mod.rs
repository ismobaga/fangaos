//! Storage and File System Support
//!
//! This module provides persistent storage capabilities including:
//! - ATA/AHCI disk drivers
//! - Partition table support (MBR and GPT)
//! - FAT32 file system
//! - Disk caching

pub mod drivers;
pub mod partition;
pub mod fat32;
pub mod cache;
pub mod block_device;

pub use block_device::{BlockDevice, BlockDeviceError};
pub use drivers::{ata::AtaDevice, ahci::AhciController};
pub use partition::{PartitionTable, Partition, PartitionType};
pub use fat32::Fat32FileSystem;
pub use cache::DiskCache;
