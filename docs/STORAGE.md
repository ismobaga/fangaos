# Storage and File Systems

This document describes the storage subsystem in FangaOS, including disk drivers, partition support, file systems, and disk caching.

## Overview

The storage subsystem provides persistent storage capabilities for FangaOS, enabling the kernel to:
- Access hard disks and SSDs through ATA/AHCI interfaces
- Parse partition tables (MBR and GPT)
- Read and write FAT32 file systems
- Cache disk I/O for improved performance

## Architecture

The storage subsystem is organized into several layers:

```
┌─────────────────────────────────────┐
│       VFS Integration Layer         │
├─────────────────────────────────────┤
│      File System Layer (FAT32)      │
├─────────────────────────────────────┤
│        Partition Layer (MBR/GPT)    │
├─────────────────────────────────────┤
│      Disk Cache (Buffer Cache)      │
├─────────────────────────────────────┤
│    Block Device Abstraction Layer   │
├─────────────────────────────────────┤
│   Hardware Drivers (ATA/AHCI)       │
└─────────────────────────────────────┘
```

## Block Device Abstraction

The `BlockDevice` trait provides a uniform interface for all storage devices:

```rust
pub trait BlockDevice: Send + Sync {
    fn block_size(&self) -> usize;
    fn block_count(&self) -> u64;
    fn read_blocks(&self, start_block: u64, buffer: &mut [u8]) -> Result<(), BlockDeviceError>;
    fn write_blocks(&self, start_block: u64, buffer: &[u8]) -> Result<(), BlockDeviceError>;
    fn flush(&mut self) -> Result<(), BlockDeviceError>;
}
```

This abstraction allows higher layers to work with any storage device without knowing the specific hardware details.

## Hardware Drivers

### ATA/IDE Driver

The ATA (Advanced Technology Attachment) driver supports older IDE hard drives using PIO (Programmed I/O) mode:

- **Supported**: Primary and secondary buses, master and slave drives
- **Mode**: PIO (Programmed I/O) for simplicity and compatibility
- **Addressing**: LBA28 (up to 128 GB per drive)
- **Operations**: Read, write, flush, and identify

Example usage:
```rust
use fanga_kernel::storage::drivers::ata::{AtaDevice, AtaBus, AtaDrive};

// Initialize primary master drive
let mut device = AtaDevice::new(AtaBus::Primary, AtaDrive::Master);
device.init()?;

// Read sector
let mut buffer = vec![0u8; 512];
device.read_blocks(0, &mut buffer)?;
```

### AHCI/SATA Driver

The AHCI (Advanced Host Controller Interface) driver provides a modern interface for SATA devices:

- **Status**: Basic structure implemented, full driver planned for future
- **Features**: DMA transfers, command queuing, hot-plug support
- **Future work**: PCI enumeration, memory-mapped I/O, interrupt handling

## Partition Support

### MBR (Master Boot Record)

The MBR partition table parser supports legacy DOS partition tables:

- **Partitions**: Up to 4 primary partitions
- **Size limit**: 2 TB per partition
- **Detection**: Automatic detection via 0xAA55 signature
- **Supported types**: FAT32, NTFS, EXT, Swap

Example:
```rust
use fanga_kernel::storage::partition::{PartitionTable, MbrPartitionTable};

let partitions = MbrPartitionTable::parse(&device)?;
for partition in partitions {
    println!("Partition {}: {} sectors at LBA {}",
        partition.number, partition.size, partition.start_lba);
}
```

### GPT (GUID Partition Table)

The GPT partition table parser supports modern EFI partition tables:

- **Partitions**: Up to 128 partitions (typical)
- **Size limit**: 9.4 ZB (zettabytes) theoretical
- **Features**: Partition names (UTF-16), UUIDs, backup table
- **Detection**: Automatic detection via "EFI PART" signature

Example:
```rust
use fanga_kernel::storage::partition::{PartitionTable, GptPartitionTable};

let partitions = GptPartitionTable::parse(&device)?;
for partition in partitions {
    if let Some(label) = &partition.label {
        println!("Partition {}: {}", partition.number, label);
    }
}
```

## FAT32 File System

The FAT32 implementation provides read and write support for FAT32 volumes:

### Features

- **Boot sector parsing**: BPB (BIOS Parameter Block) parsing
- **FAT operations**: Cluster chain traversal and allocation
- **Directory entries**: Short file name (8.3) support
- **File operations**: Read and write through VFS interface
- **Cluster size**: 512 bytes to 32 KB per cluster

### Components

#### Boot Sector
Parses the FAT32 boot sector to extract file system parameters:
```rust
use fanga_kernel::storage::fat32::Fat32BootSector;

let boot_sector = Fat32BootSector::read(&device, partition_start)?;
println!("Cluster size: {} bytes", 
    boot_sector.bytes_per_sector as usize * 
    boot_sector.sectors_per_cluster as usize);
```

#### FAT Table
Manages the File Allocation Table for cluster chain operations:
```rust
use fanga_kernel::storage::fat32::FatTable;

let mut fat = FatTable::new(&boot_sector);
let chain = fat.get_chain(start_cluster);
```

#### Directory Entries
Handles FAT32 directory entry parsing:
```rust
use fanga_kernel::storage::fat32::DirectoryEntry;

let entry = DirectoryEntry::new("file.txt", false);
println!("Name: {}", entry.get_short_name());
```

### VFS Integration

FAT32 implements the `FileSystem` trait for VFS integration:
```rust
use fanga_kernel::storage::fat32::Fat32FileSystem;
use fanga_kernel::fs::vfs::FileSystem;

let fs = Fat32FileSystem::new(device, partition_start)?;
let root = fs.root()?;
```

## Disk Caching

The disk cache implements a buffer cache with LRU (Least Recently Used) eviction:

### Features

- **Cache size**: Configurable number of cached blocks
- **Eviction policy**: LRU (Least Recently Used)
- **Write policy**: Write-through (dirty blocks flushed on eviction)
- **Thread-safe**: Uses Mutex for concurrent access

### Usage

```rust
use fanga_kernel::storage::cache::DiskCache;
use std::sync::Arc;
use spin::Mutex;

// Create cache with 100 block capacity
let device = Arc::new(Mutex::new(ata_device));
let cache = DiskCache::new(device, 100);

// Cached read
let mut buffer = vec![0u8; 512];
cache.read_blocks(0, &mut buffer)?;

// Cached write
cache.write_blocks(0, &buffer)?;

// Flush dirty blocks
cache.flush()?;
```

### Performance

The cache improves performance by:
- **Read caching**: Avoiding repeated disk reads
- **Write buffering**: Batching writes to reduce I/O operations
- **Sequential optimization**: Keeping recently accessed blocks in memory

## Error Handling

All storage operations use the `BlockDeviceError` enum for error reporting:

```rust
pub enum BlockDeviceError {
    NotFound,        // Device not found
    InvalidBlock,    // Invalid block number
    IoError,         // I/O error
    Busy,            // Device busy
    Timeout,         // Operation timeout
    InvalidBufferSize, // Wrong buffer size
    NotReady,        // Device not ready
}
```

## Testing

The storage subsystem includes comprehensive tests:

### Unit Tests
- Block device operations
- Partition table parsing (MBR/GPT)
- FAT32 boot sector and FAT table operations
- Directory entry handling
- Cache operations

### Integration Tests
- End-to-end storage operations
- Partition discovery
- File system mounting
- Cached I/O operations

Run tests:
```bash
# Unit tests
cargo test --lib storage

# Integration tests
cargo test --test storage_integration
```

## Future Enhancements

### Planned Features

1. **AHCI Driver**: Complete AHCI implementation for modern SATA drives
2. **PCI Enumeration**: Automatic discovery of storage controllers
3. **DMA Support**: Direct Memory Access for faster transfers
4. **Long File Names**: LFN (Long File Name) support for FAT32
5. **Write Optimization**: Write-back caching with sync policies
6. **Ext2/3/4 Support**: Linux file system support
7. **RAID Support**: Software RAID 0/1/5/10

### Performance Optimizations

1. **Read-ahead**: Predictive prefetching of sequential blocks
2. **Write coalescing**: Combining multiple small writes
3. **Async I/O**: Non-blocking disk operations
4. **Multi-queue**: Parallel I/O operations on multiple drives

## Examples

### Complete Example: Reading a File from FAT32

```rust
use fanga_kernel::storage::{
    drivers::ata::{AtaDevice, AtaBus, AtaDrive},
    partition::{PartitionTable, MbrPartitionTable, PartitionType},
    fat32::Fat32FileSystem,
    cache::DiskCache,
};
use std::sync::Arc;
use spin::Mutex;

// Initialize ATA device
let mut ata = AtaDevice::new(AtaBus::Primary, AtaDrive::Master);
ata.init()?;

// Parse partition table
let partitions = MbrPartitionTable::parse(&ata)?;
let fat32_partition = partitions.iter()
    .find(|p| p.ptype == PartitionType::Fat32)
    .expect("No FAT32 partition");

// Create cached device
let device = Arc::new(Mutex::new(ata));
let cache = Arc::new(DiskCache::new(device.clone(), 100));

// Mount FAT32 filesystem
let fs = Fat32FileSystem::new(
    Arc::new(Mutex::new(cache)), 
    fat32_partition.start_lba
)?;

// Access files through VFS
let root = fs.root()?;
let entries = fs.readdir(&root)?;
for entry in entries {
    println!("File: {}", entry.name);
}
```

## References

- [ATA/ATAPI Specification](https://www.t13.org/)
- [AHCI Specification](https://www.intel.com/content/www/us/en/io/serial-ata/ahci.html)
- [FAT32 File System Specification](https://www.win.tue.nl/~aeb/linux/fs/fat/fat-1.html)
- [GPT Specification](https://uefi.org/specifications)
- [OSDev Wiki - Storage](https://wiki.osdev.org/Category:Storage)
