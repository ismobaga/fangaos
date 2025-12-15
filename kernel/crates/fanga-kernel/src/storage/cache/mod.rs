//! Disk Cache
//!
//! This module implements a buffer cache for disk I/O operations.
//! It uses an LRU (Least Recently Used) eviction policy.

extern crate alloc;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use spin::Mutex;
use crate::storage::block_device::{BlockDevice, BlockDeviceError};

/// Cache entry
#[derive(Clone)]
struct CacheEntry {
    /// Block number
    block: u64,
    /// Block data
    data: Vec<u8>,
    /// Dirty flag (needs write back)
    dirty: bool,
    /// Access timestamp (for LRU)
    timestamp: u64,
}

/// Disk cache with LRU eviction
pub struct DiskCache {
    /// Underlying block device
    device: Arc<Mutex<dyn BlockDevice>>,
    /// Cache entries
    entries: Mutex<Vec<CacheEntry>>,
    /// LRU queue (block numbers)
    lru_queue: Mutex<VecDeque<u64>>,
    /// Maximum number of cached blocks
    max_entries: usize,
    /// Block size
    block_size: usize,
    /// Current timestamp counter
    timestamp: Mutex<u64>,
}

impl DiskCache {
    /// Create a new disk cache
    ///
    /// # Arguments
    /// * `device` - The underlying block device
    /// * `max_entries` - Maximum number of blocks to cache
    pub fn new(device: Arc<Mutex<dyn BlockDevice>>, max_entries: usize) -> Self {
        let block_size = device.lock().block_size();
        
        Self {
            device,
            entries: Mutex::new(Vec::new()),
            lru_queue: Mutex::new(VecDeque::new()),
            max_entries,
            block_size,
            timestamp: Mutex::new(0),
        }
    }
    
    /// Get next timestamp
    fn next_timestamp(&self) -> u64 {
        let mut ts = self.timestamp.lock();
        *ts += 1;
        *ts
    }
    
    /// Find a cached entry
    fn find_entry(&self, block: u64) -> Option<CacheEntry> {
        let entries = self.entries.lock();
        entries.iter().find(|e| e.block == block).cloned()
    }
    
    /// Update LRU queue
    fn update_lru(&self, block: u64) {
        let mut queue = self.lru_queue.lock();
        
        // Remove block if it's already in the queue
        if let Some(pos) = queue.iter().position(|&b| b == block) {
            queue.remove(pos);
        }
        
        // Add block to the front (most recently used)
        queue.push_front(block);
        
        // Limit queue size
        while queue.len() > self.max_entries {
            queue.pop_back();
        }
    }
    
    /// Evict least recently used entry
    fn evict_lru(&self) -> Result<(), BlockDeviceError> {
        let lru_block = {
            let queue = self.lru_queue.lock();
            queue.back().copied()
        };
        
        if let Some(block) = lru_block {
            self.flush_block(block)?;
            
            // Remove from cache
            let mut entries = self.entries.lock();
            if let Some(pos) = entries.iter().position(|e| e.block == block) {
                entries.remove(pos);
            }
            
            // Remove from LRU queue
            let mut queue = self.lru_queue.lock();
            if let Some(pos) = queue.iter().position(|&b| b == block) {
                queue.remove(pos);
            }
        }
        
        Ok(())
    }
    
    /// Read blocks with caching
    pub fn read_blocks(&self, start_block: u64, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let num_blocks = buffer.len() / self.block_size;
        
        for i in 0..num_blocks {
            let block = start_block + i as u64;
            let offset = i * self.block_size;
            
            // Check cache first
            if let Some(entry) = self.find_entry(block) {
                buffer[offset..offset + self.block_size].copy_from_slice(&entry.data);
                self.update_lru(block);
            } else {
                // Read from device
                let device = self.device.lock();
                device.read_blocks(block, &mut buffer[offset..offset + self.block_size])?;
                drop(device);
                
                // Add to cache
                let mut entries = self.entries.lock();
                
                // Evict if cache is full
                if entries.len() >= self.max_entries {
                    drop(entries);
                    self.evict_lru()?;
                    entries = self.entries.lock();
                }
                
                let entry = CacheEntry {
                    block,
                    data: buffer[offset..offset + self.block_size].to_vec(),
                    dirty: false,
                    timestamp: self.next_timestamp(),
                };
                entries.push(entry);
                self.update_lru(block);
            }
        }
        
        Ok(())
    }
    
    /// Write blocks with caching
    pub fn write_blocks(&self, start_block: u64, buffer: &[u8]) -> Result<(), BlockDeviceError> {
        let num_blocks = buffer.len() / self.block_size;
        
        for i in 0..num_blocks {
            let block = start_block + i as u64;
            let offset = i * self.block_size;
            let block_data = &buffer[offset..offset + self.block_size];
            
            let mut entries = self.entries.lock();
            
            // Update existing entry or create new one
            if let Some(entry) = entries.iter_mut().find(|e| e.block == block) {
                entry.data.copy_from_slice(block_data);
                entry.dirty = true;
                entry.timestamp = self.next_timestamp();
            } else {
                // Evict if cache is full
                if entries.len() >= self.max_entries {
                    drop(entries);
                    self.evict_lru()?;
                    entries = self.entries.lock();
                }
                
                let entry = CacheEntry {
                    block,
                    data: block_data.to_vec(),
                    dirty: true,
                    timestamp: self.next_timestamp(),
                };
                entries.push(entry);
            }
            
            drop(entries);
            self.update_lru(block);
        }
        
        Ok(())
    }
    
    /// Flush a specific block to disk
    fn flush_block(&self, block: u64) -> Result<(), BlockDeviceError> {
        let mut entries = self.entries.lock();
        
        if let Some(entry) = entries.iter_mut().find(|e| e.block == block) {
            if entry.dirty {
                let device = self.device.lock();
                device.write_blocks(entry.block, &entry.data)?;
                entry.dirty = false;
            }
        }
        
        Ok(())
    }
    
    /// Flush all dirty blocks to disk
    pub fn flush(&self) -> Result<(), BlockDeviceError> {
        let blocks: Vec<u64> = {
            let entries = self.entries.lock();
            entries.iter()
                .filter(|e| e.dirty)
                .map(|e| e.block)
                .collect()
        };
        
        for block in blocks {
            self.flush_block(block)?;
        }
        
        Ok(())
    }
    
    /// Clear the cache
    pub fn clear(&self) {
        let mut entries = self.entries.lock();
        entries.clear();
        let mut queue = self.lru_queue.lock();
        queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockBlockDevice {
        block_size: usize,
        block_count: u64,
    }
    
    impl BlockDevice for MockBlockDevice {
        fn block_size(&self) -> usize {
            self.block_size
        }
        
        fn block_count(&self) -> u64 {
            self.block_count
        }
        
        fn read_blocks(&self, _start_block: u64, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
            buffer.fill(0);
            Ok(())
        }
        
        fn write_blocks(&self, _start_block: u64, _buffer: &[u8]) -> Result<(), BlockDeviceError> {
            Ok(())
        }
        
        fn flush(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }
    }
    
    #[test]
    fn test_cache_creation() {
        let device = Arc::new(Mutex::new(MockBlockDevice {
            block_size: 512,
            block_count: 100,
        }));
        
        let cache = DiskCache::new(device, 10);
        assert_eq!(cache.block_size, 512);
        assert_eq!(cache.max_entries, 10);
    }
    
    #[test]
    fn test_cache_operations() {
        let device = Arc::new(Mutex::new(MockBlockDevice {
            block_size: 512,
            block_count: 100,
        }));
        
        let cache = DiskCache::new(device, 10);
        
        let mut buffer = vec![0u8; 512];
        assert!(cache.read_blocks(0, &mut buffer).is_ok());
        assert!(cache.write_blocks(0, &buffer).is_ok());
        assert!(cache.flush().is_ok());
    }
}
