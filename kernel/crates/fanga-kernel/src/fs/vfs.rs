//! VFS Core Abstraction
//!
//! This module defines the core VFS traits and types for pluggable file system implementations.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// File system operations trait
///
/// This trait defines the interface that all file system implementations must provide.
pub trait FileSystem: Send + Sync {
    /// Get the root vnode of the file system
    fn root(&self) -> Result<VNode, FsError>;
    
    /// Look up a file or directory by path
    fn lookup(&self, path: &str) -> Result<VNode, FsError>;
    
    /// Create a new file
    fn create(&mut self, path: &str, vtype: VNodeType) -> Result<VNode, FsError>;
    
    /// Remove a file or empty directory
    fn remove(&mut self, path: &str) -> Result<(), FsError>;
    
    /// Read from a file
    fn read(&self, vnode: &VNode, offset: usize, buffer: &mut [u8]) -> Result<usize, FsError>;
    
    /// Write to a file
    fn write(&mut self, vnode: &VNode, offset: usize, buffer: &[u8]) -> Result<usize, FsError>;
    
    /// Get file/directory attributes
    fn stat(&self, vnode: &VNode) -> Result<VNodeAttr, FsError>;
    
    /// Read directory entries
    fn readdir(&self, vnode: &VNode) -> Result<Vec<DirEntry>, FsError>;
    
    /// Truncate file to specified size
    fn truncate(&mut self, vnode: &VNode, size: usize) -> Result<(), FsError>;
}

/// Virtual node (inode equivalent)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VNode {
    /// Unique identifier for this vnode
    pub id: u64,
    /// Type of the vnode
    pub vtype: VNodeType,
    /// Path to this vnode
    pub path: String,
}

impl VNode {
    /// Create a new vnode
    pub fn new(id: u64, vtype: VNodeType, path: String) -> Self {
        Self { id, vtype, path }
    }
}

/// Virtual node type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VNodeType {
    /// Regular file
    File,
    /// Directory
    Directory,
}

/// Virtual node attributes
#[derive(Debug, Clone)]
pub struct VNodeAttr {
    /// File size in bytes
    pub size: usize,
    /// Node type
    pub vtype: VNodeType,
}

/// Directory entry
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// Entry name
    pub name: String,
    /// Entry type
    pub vtype: VNodeType,
}

impl DirEntry {
    /// Create a new directory entry
    pub fn new(name: String, vtype: VNodeType) -> Self {
        Self { name, vtype }
    }
}

/// File open flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenFlags {
    /// Read access
    pub read: bool,
    /// Write access
    pub write: bool,
    /// Create if doesn't exist
    pub create: bool,
    /// Truncate on open
    pub truncate: bool,
    /// Append mode
    pub append: bool,
}

impl OpenFlags {
    /// Open for reading only
    pub const fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            create: false,
            truncate: false,
            append: false,
        }
    }
    
    /// Open for writing only
    pub const fn write_only() -> Self {
        Self {
            read: false,
            write: true,
            create: false,
            truncate: false,
            append: false,
        }
    }
    
    /// Open for reading and writing
    pub const fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            create: false,
            truncate: false,
            append: false,
        }
    }
    
    /// Create new file (write + create + truncate)
    pub const fn create_new() -> Self {
        Self {
            read: false,
            write: true,
            create: true,
            truncate: true,
            append: false,
        }
    }
}

/// Seek operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekWhence {
    /// Seek from start of file
    Start,
    /// Seek from current position
    Current,
    /// Seek from end of file
    End,
}

/// File system errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsError {
    /// File or directory not found
    NotFound,
    /// File or directory already exists
    AlreadyExists,
    /// Not a directory
    NotADirectory,
    /// Is a directory
    IsADirectory,
    /// Invalid path
    InvalidPath,
    /// Permission denied
    PermissionDenied,
    /// Directory not empty
    DirectoryNotEmpty,
    /// No space left
    NoSpace,
    /// Invalid argument
    InvalidArgument,
    /// I/O error
    IoError,
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FsError::NotFound => write!(f, "File or directory not found"),
            FsError::AlreadyExists => write!(f, "File or directory already exists"),
            FsError::NotADirectory => write!(f, "Not a directory"),
            FsError::IsADirectory => write!(f, "Is a directory"),
            FsError::InvalidPath => write!(f, "Invalid path"),
            FsError::PermissionDenied => write!(f, "Permission denied"),
            FsError::DirectoryNotEmpty => write!(f, "Directory not empty"),
            FsError::NoSpace => write!(f, "No space left"),
            FsError::InvalidArgument => write!(f, "Invalid argument"),
            FsError::IoError => write!(f, "I/O error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vnode_creation() {
        let vnode = VNode::new(1, VNodeType::File, String::from("/test.txt"));
        assert_eq!(vnode.id, 1);
        assert_eq!(vnode.vtype, VNodeType::File);
        assert_eq!(vnode.path, "/test.txt");
    }
    
    #[test]
    fn test_open_flags() {
        let read_only = OpenFlags::read_only();
        assert!(read_only.read);
        assert!(!read_only.write);
        
        let write_only = OpenFlags::write_only();
        assert!(!write_only.read);
        assert!(write_only.write);
        
        let read_write = OpenFlags::read_write();
        assert!(read_write.read);
        assert!(read_write.write);
        
        let create_new = OpenFlags::create_new();
        assert!(create_new.write);
        assert!(create_new.create);
        assert!(create_new.truncate);
    }
    
    #[test]
    fn test_dir_entry() {
        let entry = DirEntry::new(String::from("test.txt"), VNodeType::File);
        assert_eq!(entry.name, "test.txt");
        assert_eq!(entry.vtype, VNodeType::File);
    }
}
