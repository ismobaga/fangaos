//! In-Memory File System
//!
//! This module provides a simple RAM-based file system implementation.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use spin::RwLock;

use super::vfs::{FileSystem, VNode, VNodeType, VNodeAttr, DirEntry, FsError};
use super::path::PathResolver;

/// In-memory file data
#[derive(Debug, Clone)]
struct MemFile {
    /// File content
    data: Vec<u8>,
}

impl MemFile {
    /// Create a new empty file
    fn new() -> Self {
        Self { data: Vec::new() }
    }
    
    /// Read from the file
    fn read(&self, offset: usize, buffer: &mut [u8]) -> usize {
        if offset >= self.data.len() {
            return 0;
        }
        
        let available = self.data.len() - offset;
        let to_read = buffer.len().min(available);
        
        buffer[..to_read].copy_from_slice(&self.data[offset..offset + to_read]);
        to_read
    }
    
    /// Write to the file
    fn write(&mut self, offset: usize, buffer: &[u8]) -> usize {
        // Extend file if necessary
        if offset + buffer.len() > self.data.len() {
            self.data.resize(offset + buffer.len(), 0);
        }
        
        self.data[offset..offset + buffer.len()].copy_from_slice(buffer);
        buffer.len()
    }
    
    /// Get file size
    fn size(&self) -> usize {
        self.data.len()
    }
    
    /// Truncate file to specified size
    fn truncate(&mut self, size: usize) {
        self.data.resize(size, 0);
    }
}

/// In-memory directory
#[derive(Debug, Clone)]
struct MemDir {
    /// Directory entries (name -> vnode id)
    entries: BTreeMap<String, u64>,
}

impl MemDir {
    /// Create a new empty directory
    fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
    
    /// Add an entry to the directory
    fn add_entry(&mut self, name: String, vnode_id: u64) -> Result<(), FsError> {
        if self.entries.contains_key(&name) {
            return Err(FsError::AlreadyExists);
        }
        self.entries.insert(name, vnode_id);
        Ok(())
    }
    
    /// Remove an entry from the directory
    fn remove_entry(&mut self, name: &str) -> Result<u64, FsError> {
        self.entries.remove(name).ok_or(FsError::NotFound)
    }
    
    /// Get an entry's vnode id
    fn get_entry(&self, name: &str) -> Option<u64> {
        self.entries.get(name).copied()
    }
    
    /// Check if directory is empty
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// List all entries
    fn list_entries(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

/// In-memory node
#[derive(Debug, Clone)]
enum MemNode {
    File(MemFile),
    Directory(MemDir),
}

/// In-Memory File System
pub struct MemoryFileSystem {
    /// Node storage (vnode id -> node data)
    nodes: RwLock<BTreeMap<u64, MemNode>>,
    /// Vnode metadata (vnode id -> vnode)
    vnodes: RwLock<BTreeMap<u64, VNode>>,
    /// Path to vnode id mapping
    paths: RwLock<BTreeMap<String, u64>>,
    /// Next vnode id
    next_id: RwLock<u64>,
}

impl MemoryFileSystem {
    /// Create a new in-memory file system
    pub fn new() -> Self {
        let mut nodes = BTreeMap::new();
        let mut vnodes = BTreeMap::new();
        let mut paths = BTreeMap::new();
        
        // Create root directory
        let root_node = MemNode::Directory(MemDir::new());
        let root_vnode = VNode::new(0, VNodeType::Directory, String::from("/"));
        
        nodes.insert(0, root_node);
        vnodes.insert(0, root_vnode);
        paths.insert(String::from("/"), 0);
        
        Self {
            nodes: RwLock::new(nodes),
            vnodes: RwLock::new(vnodes),
            paths: RwLock::new(paths),
            next_id: RwLock::new(1),
        }
    }
    
    /// Allocate a new vnode id
    fn alloc_id(&self) -> u64 {
        let mut next_id = self.next_id.write();
        let id = *next_id;
        *next_id += 1;
        id
    }
    
    /// Get a vnode by id
    fn get_vnode_by_id(&self, id: u64) -> Option<VNode> {
        self.vnodes.read().get(&id).cloned()
    }
}

impl Default for MemoryFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for MemoryFileSystem {
    fn root(&self) -> Result<VNode, FsError> {
        self.get_vnode_by_id(0).ok_or(FsError::NotFound)
    }
    
    fn lookup(&self, path: &str) -> Result<VNode, FsError> {
        // Normalize the path
        let path = PathResolver::normalize(path).map_err(|_| FsError::InvalidPath)?;
        
        // Look up in path mapping
        let paths = self.paths.read();
        let vnode_id = paths.get(&path).copied().ok_or(FsError::NotFound)?;
        drop(paths);
        
        // Get vnode
        self.get_vnode_by_id(vnode_id).ok_or(FsError::NotFound)
    }
    
    fn create(&mut self, path: &str, vtype: VNodeType) -> Result<VNode, FsError> {
        // Normalize the path
        let path = PathResolver::normalize(path).map_err(|_| FsError::InvalidPath)?;
        
        // Check if already exists
        if self.paths.read().contains_key(&path) {
            return Err(FsError::AlreadyExists);
        }
        
        // Get parent directory
        let parent_path = PathResolver::parent(&path).ok_or(FsError::InvalidPath)?;
        let parent_id = self.paths.read().get(&parent_path).copied().ok_or(FsError::NotFound)?;
        
        // Verify parent is a directory
        let nodes = self.nodes.read();
        if !matches!(nodes.get(&parent_id), Some(MemNode::Directory(_))) {
            return Err(FsError::NotADirectory);
        }
        drop(nodes);
        
        // Create new node
        let id = self.alloc_id();
        let filename = PathResolver::filename(&path).ok_or(FsError::InvalidPath)?;
        
        let node = match vtype {
            VNodeType::File => MemNode::File(MemFile::new()),
            VNodeType::Directory => MemNode::Directory(MemDir::new()),
        };
        
        let vnode = VNode::new(id, vtype, path.clone());
        
        // Add to parent directory
        let mut nodes = self.nodes.write();
        if let Some(MemNode::Directory(parent_dir)) = nodes.get_mut(&parent_id) {
            parent_dir.add_entry(String::from(filename), id)?;
        } else {
            return Err(FsError::NotADirectory);
        }
        
        // Store node and vnode
        nodes.insert(id, node);
        drop(nodes);
        
        self.vnodes.write().insert(id, vnode.clone());
        self.paths.write().insert(path, id);
        
        Ok(vnode)
    }
    
    fn remove(&mut self, path: &str) -> Result<(), FsError> {
        // Normalize the path
        let path = PathResolver::normalize(path).map_err(|_| FsError::InvalidPath)?;
        
        // Can't remove root
        if path == "/" {
            return Err(FsError::InvalidArgument);
        }
        
        // Get vnode
        let vnode_id = self.paths.read().get(&path).copied().ok_or(FsError::NotFound)?;
        
        // Check if directory is empty
        let nodes = self.nodes.read();
        if let Some(MemNode::Directory(dir)) = nodes.get(&vnode_id) {
            if !dir.is_empty() {
                return Err(FsError::DirectoryNotEmpty);
            }
        }
        drop(nodes);
        
        // Get parent directory
        let parent_path = PathResolver::parent(&path).ok_or(FsError::InvalidPath)?;
        let parent_id = self.paths.read().get(&parent_path).copied().ok_or(FsError::NotFound)?;
        let filename = PathResolver::filename(&path).ok_or(FsError::InvalidPath)?;
        
        // Remove from parent directory
        let mut nodes = self.nodes.write();
        if let Some(MemNode::Directory(parent_dir)) = nodes.get_mut(&parent_id) {
            parent_dir.remove_entry(filename)?;
        } else {
            return Err(FsError::NotADirectory);
        }
        
        // Remove node
        nodes.remove(&vnode_id);
        drop(nodes);
        
        self.vnodes.write().remove(&vnode_id);
        self.paths.write().remove(&path);
        
        Ok(())
    }
    
    fn read(&self, vnode: &VNode, offset: usize, buffer: &mut [u8]) -> Result<usize, FsError> {
        let nodes = self.nodes.read();
        
        match nodes.get(&vnode.id) {
            Some(MemNode::File(file)) => Ok(file.read(offset, buffer)),
            Some(MemNode::Directory(_)) => Err(FsError::IsADirectory),
            None => Err(FsError::NotFound),
        }
    }
    
    fn write(&mut self, vnode: &VNode, offset: usize, buffer: &[u8]) -> Result<usize, FsError> {
        let mut nodes = self.nodes.write();
        
        match nodes.get_mut(&vnode.id) {
            Some(MemNode::File(file)) => Ok(file.write(offset, buffer)),
            Some(MemNode::Directory(_)) => Err(FsError::IsADirectory),
            None => Err(FsError::NotFound),
        }
    }
    
    fn stat(&self, vnode: &VNode) -> Result<VNodeAttr, FsError> {
        let nodes = self.nodes.read();
        
        match nodes.get(&vnode.id) {
            Some(MemNode::File(file)) => Ok(VNodeAttr {
                size: file.size(),
                vtype: VNodeType::File,
            }),
            Some(MemNode::Directory(_)) => Ok(VNodeAttr {
                size: 0,
                vtype: VNodeType::Directory,
            }),
            None => Err(FsError::NotFound),
        }
    }
    
    fn readdir(&self, vnode: &VNode) -> Result<Vec<DirEntry>, FsError> {
        let nodes = self.nodes.read();
        
        match nodes.get(&vnode.id) {
            Some(MemNode::Directory(dir)) => {
                let vnodes = self.vnodes.read();
                let mut entries = Vec::new();
                
                for name in dir.list_entries() {
                    if let Some(entry_id) = dir.get_entry(&name) {
                        if let Some(entry_vnode) = vnodes.get(&entry_id) {
                            entries.push(DirEntry::new(name, entry_vnode.vtype));
                        }
                    }
                }
                
                Ok(entries)
            }
            Some(MemNode::File(_)) => Err(FsError::NotADirectory),
            None => Err(FsError::NotFound),
        }
    }
    
    fn truncate(&mut self, vnode: &VNode, size: usize) -> Result<(), FsError> {
        let mut nodes = self.nodes.write();
        
        match nodes.get_mut(&vnode.id) {
            Some(MemNode::File(file)) => {
                file.truncate(size);
                Ok(())
            }
            Some(MemNode::Directory(_)) => Err(FsError::IsADirectory),
            None => Err(FsError::NotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memfs_creation() {
        let fs = MemoryFileSystem::new();
        let root = fs.root().unwrap();
        
        assert_eq!(root.id, 0);
        assert_eq!(root.vtype, VNodeType::Directory);
        assert_eq!(root.path, "/");
    }
    
    #[test]
    fn test_create_file() {
        let mut fs = MemoryFileSystem::new();
        let vnode = fs.create("/test.txt", VNodeType::File).unwrap();
        
        assert_eq!(vnode.vtype, VNodeType::File);
        assert_eq!(vnode.path, "/test.txt");
    }
    
    #[test]
    fn test_create_directory() {
        let mut fs = MemoryFileSystem::new();
        let vnode = fs.create("/testdir", VNodeType::Directory).unwrap();
        
        assert_eq!(vnode.vtype, VNodeType::Directory);
        assert_eq!(vnode.path, "/testdir");
    }
    
    #[test]
    fn test_lookup() {
        let mut fs = MemoryFileSystem::new();
        fs.create("/test.txt", VNodeType::File).unwrap();
        
        let vnode = fs.lookup("/test.txt").unwrap();
        assert_eq!(vnode.path, "/test.txt");
    }
    
    #[test]
    fn test_write_read() {
        let mut fs = MemoryFileSystem::new();
        let vnode = fs.create("/test.txt", VNodeType::File).unwrap();
        
        let data = b"Hello, World!";
        let written = fs.write(&vnode, 0, data).unwrap();
        assert_eq!(written, data.len());
        
        let mut buffer = vec![0u8; data.len()];
        let read = fs.read(&vnode, 0, &mut buffer).unwrap();
        assert_eq!(read, data.len());
        assert_eq!(&buffer, data);
    }
    
    #[test]
    fn test_stat() {
        let mut fs = MemoryFileSystem::new();
        let vnode = fs.create("/test.txt", VNodeType::File).unwrap();
        
        let data = b"Hello!";
        fs.write(&vnode, 0, data).unwrap();
        
        let attr = fs.stat(&vnode).unwrap();
        assert_eq!(attr.size, data.len());
        assert_eq!(attr.vtype, VNodeType::File);
    }
    
    #[test]
    fn test_remove_file() {
        let mut fs = MemoryFileSystem::new();
        fs.create("/test.txt", VNodeType::File).unwrap();
        
        fs.remove("/test.txt").unwrap();
        assert!(fs.lookup("/test.txt").is_err());
    }
    
    #[test]
    fn test_remove_empty_directory() {
        let mut fs = MemoryFileSystem::new();
        fs.create("/testdir", VNodeType::Directory).unwrap();
        
        fs.remove("/testdir").unwrap();
        assert!(fs.lookup("/testdir").is_err());
    }
    
    #[test]
    fn test_readdir() {
        let mut fs = MemoryFileSystem::new();
        fs.create("/file1.txt", VNodeType::File).unwrap();
        fs.create("/file2.txt", VNodeType::File).unwrap();
        fs.create("/dir1", VNodeType::Directory).unwrap();
        
        let root = fs.root().unwrap();
        let entries = fs.readdir(&root).unwrap();
        
        assert_eq!(entries.len(), 3);
    }
    
    #[test]
    fn test_nested_create() {
        let mut fs = MemoryFileSystem::new();
        fs.create("/dir1", VNodeType::Directory).unwrap();
        let vnode = fs.create("/dir1/file.txt", VNodeType::File).unwrap();
        
        assert_eq!(vnode.path, "/dir1/file.txt");
    }
    
    #[test]
    fn test_truncate() {
        let mut fs = MemoryFileSystem::new();
        let vnode = fs.create("/test.txt", VNodeType::File).unwrap();
        
        let data = b"Hello, World!";
        fs.write(&vnode, 0, data).unwrap();
        
        fs.truncate(&vnode, 5).unwrap();
        
        let attr = fs.stat(&vnode).unwrap();
        assert_eq!(attr.size, 5);
    }
}
