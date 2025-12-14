//! File Descriptor Management
//!
//! This module provides per-process file descriptor tables and file descriptor operations.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::Mutex;

use super::vfs::{VNode, OpenFlags, SeekWhence};

/// File descriptor
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    /// The vnode this descriptor refers to
    pub vnode: VNode,
    /// Open flags
    pub flags: OpenFlags,
    /// Current file offset
    pub offset: usize,
}

impl FileDescriptor {
    /// Create a new file descriptor
    pub fn new(vnode: VNode, flags: OpenFlags) -> Self {
        Self {
            vnode,
            flags,
            offset: 0,
        }
    }
    
    /// Seek to a new position in the file
    pub fn seek(&mut self, offset: i64, whence: SeekWhence, file_size: usize) -> Result<usize, &'static str> {
        let new_offset = match whence {
            SeekWhence::Start => {
                if offset < 0 {
                    return Err("Negative offset from start");
                }
                offset as usize
            }
            SeekWhence::Current => {
                let current = self.offset as i64;
                let new = current + offset;
                if new < 0 {
                    return Err("Offset before start of file");
                }
                new as usize
            }
            SeekWhence::End => {
                let end = file_size as i64;
                let new = end + offset;
                if new < 0 {
                    return Err("Offset before start of file");
                }
                new as usize
            }
        };
        
        self.offset = new_offset;
        Ok(new_offset)
    }
}

/// File descriptor table for a process
pub struct FileDescriptorTable {
    /// Map of file descriptor numbers to file descriptors
    descriptors: BTreeMap<i32, FileDescriptor>,
    /// Next available file descriptor number
    next_fd: i32,
}

impl FileDescriptorTable {
    /// Create a new file descriptor table
    pub fn new() -> Self {
        Self {
            descriptors: BTreeMap::new(),
            next_fd: 0,
        }
    }
    
    /// Allocate a new file descriptor
    pub fn alloc(&mut self, fd: FileDescriptor) -> Result<i32, &'static str> {
        // Find next available FD number
        while self.descriptors.contains_key(&self.next_fd) {
            self.next_fd += 1;
            if self.next_fd < 0 {
                return Err("Too many open files");
            }
        }
        
        let fd_num = self.next_fd;
        self.descriptors.insert(fd_num, fd);
        self.next_fd += 1;
        
        Ok(fd_num)
    }
    
    /// Get a file descriptor
    pub fn get(&self, fd_num: i32) -> Option<&FileDescriptor> {
        self.descriptors.get(&fd_num)
    }
    
    /// Get a mutable file descriptor
    pub fn get_mut(&mut self, fd_num: i32) -> Option<&mut FileDescriptor> {
        self.descriptors.get_mut(&fd_num)
    }
    
    /// Close a file descriptor
    pub fn close(&mut self, fd_num: i32) -> Result<(), &'static str> {
        self.descriptors.remove(&fd_num).ok_or("Invalid file descriptor")?;
        Ok(())
    }
    
    /// Get the number of open file descriptors
    pub fn count(&self) -> usize {
        self.descriptors.len()
    }
    
    /// Check if a file descriptor exists
    pub fn contains(&self, fd_num: i32) -> bool {
        self.descriptors.contains_key(&fd_num)
    }
}

impl Default for FileDescriptorTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Global file descriptor table manager (for kernel-level operations)
pub struct GlobalFdManager {
    /// Process ID to file descriptor table mapping
    tables: BTreeMap<u64, Arc<Mutex<FileDescriptorTable>>>,
}

impl GlobalFdManager {
    /// Create a new global FD manager
    pub const fn new() -> Self {
        Self {
            tables: BTreeMap::new(),
        }
    }
    
    /// Create a file descriptor table for a process
    pub fn create_table(&mut self, pid: u64) -> Arc<Mutex<FileDescriptorTable>> {
        let table = Arc::new(Mutex::new(FileDescriptorTable::new()));
        self.tables.insert(pid, table.clone());
        table
    }
    
    /// Get a process's file descriptor table
    pub fn get_table(&self, pid: u64) -> Option<Arc<Mutex<FileDescriptorTable>>> {
        self.tables.get(&pid).cloned()
    }
    
    /// Remove a process's file descriptor table
    pub fn remove_table(&mut self, pid: u64) {
        self.tables.remove(&pid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::vfs::VNodeType;
    use alloc::string::String;
    
    #[test]
    fn test_file_descriptor_creation() {
        let vnode = VNode::new(1, VNodeType::File, String::from("/test.txt"));
        let fd = FileDescriptor::new(vnode.clone(), OpenFlags::read_only());
        
        assert_eq!(fd.vnode.id, 1);
        assert_eq!(fd.offset, 0);
        assert!(fd.flags.read);
    }
    
    #[test]
    fn test_seek_start() {
        let vnode = VNode::new(1, VNodeType::File, String::from("/test.txt"));
        let mut fd = FileDescriptor::new(vnode, OpenFlags::read_only());
        
        let offset = fd.seek(10, SeekWhence::Start, 100).unwrap();
        assert_eq!(offset, 10);
        assert_eq!(fd.offset, 10);
    }
    
    #[test]
    fn test_seek_current() {
        let vnode = VNode::new(1, VNodeType::File, String::from("/test.txt"));
        let mut fd = FileDescriptor::new(vnode, OpenFlags::read_only());
        fd.offset = 10;
        
        let offset = fd.seek(5, SeekWhence::Current, 100).unwrap();
        assert_eq!(offset, 15);
        assert_eq!(fd.offset, 15);
    }
    
    #[test]
    fn test_seek_end() {
        let vnode = VNode::new(1, VNodeType::File, String::from("/test.txt"));
        let mut fd = FileDescriptor::new(vnode, OpenFlags::read_only());
        
        let offset = fd.seek(-10, SeekWhence::End, 100).unwrap();
        assert_eq!(offset, 90);
        assert_eq!(fd.offset, 90);
    }
    
    #[test]
    fn test_fd_table_alloc() {
        let mut table = FileDescriptorTable::new();
        let vnode = VNode::new(1, VNodeType::File, String::from("/test.txt"));
        let fd = FileDescriptor::new(vnode, OpenFlags::read_only());
        
        let fd_num = table.alloc(fd).unwrap();
        assert_eq!(fd_num, 0);
        assert!(table.contains(fd_num));
    }
    
    #[test]
    fn test_fd_table_get() {
        let mut table = FileDescriptorTable::new();
        let vnode = VNode::new(1, VNodeType::File, String::from("/test.txt"));
        let fd = FileDescriptor::new(vnode.clone(), OpenFlags::read_only());
        
        let fd_num = table.alloc(fd).unwrap();
        let retrieved = table.get(fd_num).unwrap();
        
        assert_eq!(retrieved.vnode.id, 1);
    }
    
    #[test]
    fn test_fd_table_close() {
        let mut table = FileDescriptorTable::new();
        let vnode = VNode::new(1, VNodeType::File, String::from("/test.txt"));
        let fd = FileDescriptor::new(vnode, OpenFlags::read_only());
        
        let fd_num = table.alloc(fd).unwrap();
        assert!(table.contains(fd_num));
        
        table.close(fd_num).unwrap();
        assert!(!table.contains(fd_num));
    }
    
    #[test]
    fn test_fd_table_multiple() {
        let mut table = FileDescriptorTable::new();
        
        for i in 0..5 {
            let vnode = VNode::new(i, VNodeType::File, String::from("/test.txt"));
            let fd = FileDescriptor::new(vnode, OpenFlags::read_only());
            table.alloc(fd).unwrap();
        }
        
        assert_eq!(table.count(), 5);
    }
}
