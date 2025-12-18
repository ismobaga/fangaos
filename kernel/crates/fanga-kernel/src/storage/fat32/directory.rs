//! FAT32 Directory Entries
//!
//! This module handles FAT32 directory entry parsing and manipulation.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// FAT32 directory entry (32 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct DirectoryEntry {
    /// Short file name (8.3 format)
    pub name: [u8; 11],
    /// File attributes
    pub attributes: u8,
    /// Reserved for Windows NT
    pub nt_reserved: u8,
    /// Creation time (tenths of second)
    pub creation_time_tenths: u8,
    /// Creation time
    pub creation_time: u16,
    /// Creation date
    pub creation_date: u16,
    /// Last access date
    pub last_access_date: u16,
    /// High word of first cluster
    pub first_cluster_high: u16,
    /// Last modification time
    pub modification_time: u16,
    /// Last modification date
    pub modification_date: u16,
    /// Low word of first cluster
    pub first_cluster_low: u16,
    /// File size in bytes
    pub file_size: u32,
}

/// Directory entry attributes
pub const ATTR_READ_ONLY: u8 = 0x01;
pub const ATTR_HIDDEN: u8 = 0x02;
pub const ATTR_SYSTEM: u8 = 0x04;
pub const ATTR_VOLUME_ID: u8 = 0x08;
pub const ATTR_DIRECTORY: u8 = 0x10;
pub const ATTR_ARCHIVE: u8 = 0x20;
pub const ATTR_LONG_NAME: u8 = ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM | ATTR_VOLUME_ID;

impl DirectoryEntry {
    /// Check if entry is empty (deleted or never used)
    pub fn is_empty(&self) -> bool {
        self.name[0] == 0x00 || self.name[0] == 0xE5
    }
    
    /// Check if this is a long file name entry
    pub fn is_long_name(&self) -> bool {
        self.attributes == ATTR_LONG_NAME
    }
    
    /// Check if this is a directory
    pub fn is_directory(&self) -> bool {
        (self.attributes & ATTR_DIRECTORY) != 0
    }
    
    /// Check if this is a regular file
    pub fn is_file(&self) -> bool {
        !self.is_directory() && !self.is_long_name()
    }
    
    /// Get the first cluster number
    pub fn first_cluster(&self) -> u32 {
        ((self.first_cluster_high as u32) << 16) | (self.first_cluster_low as u32)
    }
    
    /// Set the first cluster number
    pub fn set_first_cluster(&mut self, cluster: u32) {
        self.first_cluster_high = ((cluster >> 16) & 0xFFFF) as u16;
        self.first_cluster_low = (cluster & 0xFFFF) as u16;
    }
    
    /// Get the short file name as a string
    pub fn get_short_name(&self) -> String {
        let mut name = String::new();
        
        // Extract base name (first 8 bytes)
        for i in 0..8 {
            let c = self.name[i];
            if c == b' ' {
                break;
            }
            name.push(c as char);
        }
        
        // Check for extension (last 3 bytes)
        let has_ext = self.name[8] != b' ';
        if has_ext {
            name.push('.');
            for i in 8..11 {
                let c = self.name[i];
                if c == b' ' {
                    break;
                }
                name.push(c as char);
            }
        }
        
        name
    }
    
    /// Create a new directory entry
    pub fn new(name: &str, is_directory: bool) -> Self {
        let mut entry = Self {
            name: [b' '; 11],
            attributes: if is_directory { ATTR_DIRECTORY } else { ATTR_ARCHIVE },
            nt_reserved: 0,
            creation_time_tenths: 0,
            creation_time: 0,
            creation_date: 0,
            last_access_date: 0,
            first_cluster_high: 0,
            modification_time: 0,
            modification_date: 0,
            first_cluster_low: 0,
            file_size: 0,
        };
        
        // Set short name (simple 8.3 format)
        let parts: Vec<&str> = name.split('.').collect();
        let basename = parts[0].as_bytes();
        let ext = if parts.len() > 1 { parts[1].as_bytes() } else { &[] };
        
        for (i, &b) in basename.iter().take(8).enumerate() {
            entry.name[i] = b.to_ascii_uppercase();
        }
        
        for (i, &b) in ext.iter().take(3).enumerate() {
            entry.name[8 + i] = b.to_ascii_uppercase();
        }
        
        entry
    }
}

/// Iterator over directory entries
pub struct DirectoryIterator {
    entries: Vec<DirectoryEntry>,
    index: usize,
}

impl DirectoryIterator {
    /// Create a new directory iterator
    pub fn new(entries: Vec<DirectoryEntry>) -> Self {
        Self { entries, index: 0 }
    }
}

impl Iterator for DirectoryIterator {
    type Item = DirectoryEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.entries.len() {
            let entry = self.entries[self.index];
            self.index += 1;
            
            // Skip empty and long name entries
            if !entry.is_empty() && !entry.is_long_name() {
                return Some(entry);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_directory_entry_attributes() {
        let mut entry = DirectoryEntry::new("test.txt", false);
        assert!(entry.is_file());
        assert!(!entry.is_directory());
        
        entry.attributes = ATTR_DIRECTORY;
        assert!(entry.is_directory());
        assert!(!entry.is_file());
    }
    
    #[test]
    fn test_cluster_operations() {
        let mut entry = DirectoryEntry::new("test.txt", false);
        entry.set_first_cluster(0x12345678);
        assert_eq!(entry.first_cluster(), 0x12345678);
    }
    
    #[test]
    fn test_short_name() {
        let entry = DirectoryEntry::new("test.txt", false);
        let name = entry.get_short_name();
        assert!(name.starts_with("TEST"));
    }
    
    #[test]
    fn test_empty_entry() {
        let mut entry = DirectoryEntry::new("test.txt", false);
        assert!(!entry.is_empty());
        
        entry.name[0] = 0x00;
        assert!(entry.is_empty());
        
        entry.name[0] = 0xE5;
        assert!(entry.is_empty());
    }
}
