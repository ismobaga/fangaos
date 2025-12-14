//! Virtual File System (VFS) Integration Tests
//!
//! This module tests the complete VFS system including:
//! - In-memory file system implementation
//! - File operations (open, close, read, write, seek)
//! - Directory operations (mkdir, rmdir, readdir)
//! - Path resolution
//! - File descriptor management

use fanga_kernel::fs::{
    MemoryFileSystem, FileSystem, VNodeType, OpenFlags,
    FileDescriptor, FileDescriptorTable, PathResolver, SeekWhence,
};

#[test]
fn test_complete_file_workflow() {
    let mut fs = MemoryFileSystem::new();
    
    // Create a file
    let vnode = fs.create("/test.txt", VNodeType::File).unwrap();
    assert_eq!(vnode.path, "/test.txt");
    
    // Write to the file
    let data = b"Hello, VFS!";
    let written = fs.write(&vnode, 0, data).unwrap();
    assert_eq!(written, data.len());
    
    // Read from the file
    let mut buffer = vec![0u8; data.len()];
    let read = fs.read(&vnode, 0, &mut buffer).unwrap();
    assert_eq!(read, data.len());
    assert_eq!(&buffer, data);
    
    // Check file attributes
    let attr = fs.stat(&vnode).unwrap();
    assert_eq!(attr.size, data.len());
    
    // Remove the file
    fs.remove("/test.txt").unwrap();
    assert!(fs.lookup("/test.txt").is_err());
}

#[test]
fn test_directory_operations() {
    let mut fs = MemoryFileSystem::new();
    
    // Create directories
    fs.create("/dir1", VNodeType::Directory).unwrap();
    fs.create("/dir2", VNodeType::Directory).unwrap();
    
    // Create files in directories
    fs.create("/dir1/file1.txt", VNodeType::File).unwrap();
    fs.create("/dir1/file2.txt", VNodeType::File).unwrap();
    fs.create("/dir2/file3.txt", VNodeType::File).unwrap();
    
    // Read directory contents
    let dir1 = fs.lookup("/dir1").unwrap();
    let entries = fs.readdir(&dir1).unwrap();
    assert_eq!(entries.len(), 2);
    
    // Verify entry names
    let names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
    assert!(names.contains(&String::from("file1.txt")));
    assert!(names.contains(&String::from("file2.txt")));
}

#[test]
fn test_nested_directories() {
    let mut fs = MemoryFileSystem::new();
    
    // Create nested directory structure
    fs.create("/home", VNodeType::Directory).unwrap();
    fs.create("/home/user", VNodeType::Directory).unwrap();
    fs.create("/home/user/documents", VNodeType::Directory).unwrap();
    
    // Create file in nested directory
    let vnode = fs.create("/home/user/documents/file.txt", VNodeType::File).unwrap();
    assert_eq!(vnode.path, "/home/user/documents/file.txt");
    
    // Lookup file
    let found = fs.lookup("/home/user/documents/file.txt").unwrap();
    assert_eq!(found.id, vnode.id);
}

#[test]
fn test_file_descriptor_table() {
    let mut fs = MemoryFileSystem::new();
    let mut fd_table = FileDescriptorTable::new();
    
    // Create files
    let vnode1 = fs.create("/file1.txt", VNodeType::File).unwrap();
    let vnode2 = fs.create("/file2.txt", VNodeType::File).unwrap();
    
    // Open files (allocate file descriptors)
    let fd1 = FileDescriptor::new(vnode1.clone(), OpenFlags::read_write());
    let fd2 = FileDescriptor::new(vnode2.clone(), OpenFlags::read_only());
    
    let fd1_num = fd_table.alloc(fd1).unwrap();
    let fd2_num = fd_table.alloc(fd2).unwrap();
    
    assert_eq!(fd1_num, 0);
    assert_eq!(fd2_num, 1);
    assert_eq!(fd_table.count(), 2);
    
    // Close a file descriptor
    fd_table.close(fd1_num).unwrap();
    assert_eq!(fd_table.count(), 1);
    assert!(!fd_table.contains(fd1_num));
    assert!(fd_table.contains(fd2_num));
}

#[test]
fn test_file_seek() {
    let mut fs = MemoryFileSystem::new();
    let vnode = fs.create("/test.txt", VNodeType::File).unwrap();
    
    // Write test data
    let data = b"0123456789";
    fs.write(&vnode, 0, data).unwrap();
    
    // Create file descriptor
    let mut fd = FileDescriptor::new(vnode.clone(), OpenFlags::read_write());
    
    // Seek from start
    let pos = fd.seek(5, SeekWhence::Start, data.len()).unwrap();
    assert_eq!(pos, 5);
    
    // Read from current position
    let mut buffer = [0u8; 5];
    let read = fs.read(&vnode, fd.offset, &mut buffer).unwrap();
    assert_eq!(read, 5);
    assert_eq!(&buffer, b"56789");
    
    // Seek from current
    fd.offset += read;
    let pos = fd.seek(-3, SeekWhence::Current, data.len()).unwrap();
    assert_eq!(pos, 7);
    
    // Seek from end
    let pos = fd.seek(-2, SeekWhence::End, data.len()).unwrap();
    assert_eq!(pos, 8);
}

#[test]
fn test_path_resolution() {
    let resolver = PathResolver::new();
    
    // Test absolute paths
    assert_eq!(resolver.resolve("/home/user/file.txt").unwrap(), "/home/user/file.txt");
    
    // Test path normalization with .
    assert_eq!(resolver.resolve("/home/./user/file.txt").unwrap(), "/home/user/file.txt");
    
    // Test path normalization with ..
    assert_eq!(resolver.resolve("/home/user/../admin/file.txt").unwrap(), "/home/admin/file.txt");
    
    // Test relative paths
    let resolver = PathResolver::with_cwd(String::from("/home/user"));
    assert_eq!(resolver.resolve("documents/file.txt").unwrap(), "/home/user/documents/file.txt");
}

#[test]
fn test_file_append_mode() {
    let mut fs = MemoryFileSystem::new();
    let vnode = fs.create("/test.txt", VNodeType::File).unwrap();
    
    // Write initial data
    let data1 = b"Hello, ";
    fs.write(&vnode, 0, data1).unwrap();
    
    // Append more data
    let attr = fs.stat(&vnode).unwrap();
    let data2 = b"World!";
    fs.write(&vnode, attr.size, data2).unwrap();
    
    // Read all data
    let total_size = data1.len() + data2.len();
    let mut buffer = vec![0u8; total_size];
    let read = fs.read(&vnode, 0, &mut buffer).unwrap();
    assert_eq!(read, total_size);
    assert_eq!(&buffer, b"Hello, World!");
}

#[test]
fn test_truncate_file() {
    let mut fs = MemoryFileSystem::new();
    let vnode = fs.create("/test.txt", VNodeType::File).unwrap();
    
    // Write data
    let data = b"Hello, World!";
    fs.write(&vnode, 0, data).unwrap();
    
    // Truncate to smaller size
    fs.truncate(&vnode, 5).unwrap();
    
    // Verify new size
    let attr = fs.stat(&vnode).unwrap();
    assert_eq!(attr.size, 5);
    
    // Read truncated data
    let mut buffer = vec![0u8; 5];
    let read = fs.read(&vnode, 0, &mut buffer).unwrap();
    assert_eq!(read, 5);
    assert_eq!(&buffer, b"Hello");
}

#[test]
fn test_readdir_root() {
    let mut fs = MemoryFileSystem::new();
    
    // Create files and directories in root
    fs.create("/file1.txt", VNodeType::File).unwrap();
    fs.create("/file2.txt", VNodeType::File).unwrap();
    fs.create("/dir1", VNodeType::Directory).unwrap();
    fs.create("/dir2", VNodeType::Directory).unwrap();
    
    // Read root directory
    let root = fs.root().unwrap();
    let entries = fs.readdir(&root).unwrap();
    
    assert_eq!(entries.len(), 4);
    
    // Count files and directories
    let file_count = entries.iter().filter(|e| e.vtype == VNodeType::File).count();
    let dir_count = entries.iter().filter(|e| e.vtype == VNodeType::Directory).count();
    
    assert_eq!(file_count, 2);
    assert_eq!(dir_count, 2);
}

#[test]
fn test_error_handling() {
    let mut fs = MemoryFileSystem::new();
    
    // Try to lookup non-existent file
    assert!(fs.lookup("/nonexistent.txt").is_err());
    
    // Try to create duplicate file
    fs.create("/test.txt", VNodeType::File).unwrap();
    assert!(fs.create("/test.txt", VNodeType::File).is_err());
    
    // Try to remove non-empty directory
    fs.create("/dir", VNodeType::Directory).unwrap();
    fs.create("/dir/file.txt", VNodeType::File).unwrap();
    assert!(fs.remove("/dir").is_err());
    
    // Try to read from a directory
    let dir = fs.lookup("/dir").unwrap();
    let mut buffer = [0u8; 10];
    assert!(fs.read(&dir, 0, &mut buffer).is_err());
    
    // Try to write to a directory
    assert!(fs.write(&dir, 0, b"data").is_err());
}

#[test]
fn test_multiple_reads_writes() {
    let mut fs = MemoryFileSystem::new();
    let vnode = fs.create("/test.txt", VNodeType::File).unwrap();
    
    // Write at different offsets
    fs.write(&vnode, 0, b"Hello").unwrap();
    fs.write(&vnode, 10, b"World").unwrap();
    
    // The file should be extended with zeros
    let attr = fs.stat(&vnode).unwrap();
    assert_eq!(attr.size, 15);
    
    // Read from different offsets
    let mut buffer1 = [0u8; 5];
    fs.read(&vnode, 0, &mut buffer1).unwrap();
    assert_eq!(&buffer1, b"Hello");
    
    let mut buffer2 = [0u8; 5];
    fs.read(&vnode, 10, &mut buffer2).unwrap();
    assert_eq!(&buffer2, b"World");
}
