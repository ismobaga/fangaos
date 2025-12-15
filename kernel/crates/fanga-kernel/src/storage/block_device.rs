//! Block Device Abstraction
//!
//! This module defines the block device trait that all storage devices must implement.

extern crate alloc;
use alloc::vec::Vec;
use core::fmt;

/// Block device operations trait
///
/// This trait defines the interface for all block devices (ATA, AHCI, etc.)
pub trait BlockDevice: Send + Sync {
    /// Get the size of a single block in bytes
    fn block_size(&self) -> usize;
    
    /// Get the total number of blocks
    fn block_count(&self) -> u64;
    
    /// Read blocks from the device
    ///
    /// # Arguments
    /// * `start_block` - The starting block number
    /// * `buffer` - Buffer to read into (must be block_size * count bytes)
    fn read_blocks(&self, start_block: u64, buffer: &mut [u8]) -> Result<(), BlockDeviceError>;
    
    /// Write blocks to the device
    ///
    /// # Arguments
    /// * `start_block` - The starting block number
    /// * `buffer` - Buffer to write from (must be block_size * count bytes)
    fn write_blocks(&self, start_block: u64, buffer: &[u8]) -> Result<(), BlockDeviceError>;
    
    /// Flush any cached data to disk
    fn flush(&mut self) -> Result<(), BlockDeviceError>;
}

/// Block device error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockDeviceError {
    /// Device not found or not initialized
    NotFound,
    /// Invalid block number
    InvalidBlock,
    /// I/O error occurred
    IoError,
    /// Device is busy
    Busy,
    /// Timeout waiting for device
    Timeout,
    /// Invalid buffer size
    InvalidBufferSize,
    /// Device not ready
    NotReady,
}

impl fmt::Display for BlockDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockDeviceError::NotFound => write!(f, "Device not found"),
            BlockDeviceError::InvalidBlock => write!(f, "Invalid block number"),
            BlockDeviceError::IoError => write!(f, "I/O error"),
            BlockDeviceError::Busy => write!(f, "Device busy"),
            BlockDeviceError::Timeout => write!(f, "Device timeout"),
            BlockDeviceError::InvalidBufferSize => write!(f, "Invalid buffer size"),
            BlockDeviceError::NotReady => write!(f, "Device not ready"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockBlockDevice {
        block_size: usize,
        block_count: u64,
        data: Vec<u8>,
    }
    
    impl MockBlockDevice {
        fn new(block_size: usize, block_count: u64) -> Self {
            Self {
                block_size,
                block_count,
                data: vec![0; block_size * block_count as usize],
            }
        }
    }
    
    impl BlockDevice for MockBlockDevice {
        fn block_size(&self) -> usize {
            self.block_size
        }
        
        fn block_count(&self) -> u64 {
            self.block_count
        }
        
        fn read_blocks(&self, start_block: u64, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
            if start_block >= self.block_count {
                return Err(BlockDeviceError::InvalidBlock);
            }
            let start = (start_block as usize) * self.block_size;
            let len = buffer.len();
            if start + len > self.data.len() {
                return Err(BlockDeviceError::InvalidBlock);
            }
            buffer.copy_from_slice(&self.data[start..start + len]);
            Ok(())
        }
        
        fn write_blocks(&self, start_block: u64, _buffer: &[u8]) -> Result<(), BlockDeviceError> {
            if start_block >= self.block_count {
                return Err(BlockDeviceError::InvalidBlock);
            }
            // Mock write - would update self.data in real implementation
            Ok(())
        }
        
        fn flush(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }
    }
    
    #[test]
    fn test_mock_block_device() {
        let device = MockBlockDevice::new(512, 100);
        assert_eq!(device.block_size(), 512);
        assert_eq!(device.block_count(), 100);
    }
    
    #[test]
    fn test_read_blocks() {
        let device = MockBlockDevice::new(512, 100);
        let mut buffer = vec![0u8; 512];
        assert!(device.read_blocks(0, &mut buffer).is_ok());
    }
    
    #[test]
    fn test_invalid_block() {
        let device = MockBlockDevice::new(512, 100);
        let mut buffer = vec![0u8; 512];
        assert_eq!(device.read_blocks(100, &mut buffer), Err(BlockDeviceError::InvalidBlock));
    }
}
