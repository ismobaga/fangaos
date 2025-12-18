# Virtual File System (VFS)

This document describes the Virtual File System (VFS) implementation in FangaOS.

## Overview

The VFS provides a file system abstraction layer that enables:
- Pluggable file system implementations
- Unified interface for file and directory operations
- Per-process file descriptor tables
- Path resolution for absolute and relative paths

## Architecture

### Core Components

#### 1. VFS Trait (`fs::vfs`)

The `FileSystem` trait defines the interface that all file system implementations must provide:

```rust
pub trait FileSystem: Send + Sync {
    fn root(&self) -> Result<VNode, FsError>;
    fn lookup(&self, path: &str) -> Result<VNode, FsError>;
    fn create(&mut self, path: &str, vtype: VNodeType) -> Result<VNode, FsError>;
    fn remove(&mut self, path: &str) -> Result<(), FsError>;
    fn read(&self, vnode: &VNode, offset: usize, buffer: &mut [u8]) -> Result<usize, FsError>;
    fn write(&mut self, vnode: &VNode, offset: usize, buffer: &[u8]) -> Result<usize, FsError>;
    fn stat(&self, vnode: &VNode) -> Result<VNodeAttr, FsError>;
    fn readdir(&self, vnode: &VNode) -> Result<Vec<DirEntry>, FsError>;
    fn truncate(&mut self, vnode: &VNode, size: usize) -> Result<(), FsError>;
}
```

**Key Types:**
- `VNode`: Virtual node representing a file or directory
- `VNodeType`: Either `File` or `Directory`
- `VNodeAttr`: File/directory attributes (size, type)
- `DirEntry`: Directory entry with name and type
- `FsError`: File system error types

#### 2. In-Memory File System (`fs::memfs`)

The `MemoryFileSystem` is a simple RAM-based implementation:

```rust
let mut fs = MemoryFileSystem::new();

// Create a file
let vnode = fs.create("/test.txt", VNodeType::File)?;

// Write data
let data = b"Hello, VFS!";
fs.write(&vnode, 0, data)?;

// Read data
let mut buffer = vec![0u8; data.len()];
fs.read(&vnode, 0, &mut buffer)?;
```

**Features:**
- Files stored as `Vec<u8>` in memory
- Directories stored as `BTreeMap<String, u64>`
- Hierarchical directory structure
- Path-based lookups with O(log n) complexity

#### 3. File Descriptor Management (`fs::file_descriptor`)

File descriptors provide per-process access to files:

```rust
// File descriptor table for a process
let mut fd_table = FileDescriptorTable::new();

// Open a file (allocate descriptor)
let fd = FileDescriptor::new(vnode, OpenFlags::read_write());
let fd_num = fd_table.alloc(fd)?;

// Use the file descriptor
let fd = fd_table.get_mut(fd_num)?;
fd.seek(10, SeekWhence::Start, file_size)?;

// Close the file descriptor
fd_table.close(fd_num)?;
```

**File Descriptor Features:**
- Maintains current offset
- Stores open flags (read, write, create, truncate, append)
- Supports seek operations (Start, Current, End)
- Per-process isolation via separate tables

#### 4. Path Resolution (`fs::path`)

The `PathResolver` handles absolute and relative path resolution:

```rust
let resolver = PathResolver::new();

// Resolve absolute path
let path = resolver.resolve("/home/user/file.txt")?;

// Resolve relative path
let resolver = PathResolver::with_cwd(String::from("/home/user"));
let path = resolver.resolve("documents/file.txt")?;
// Returns: "/home/user/documents/file.txt"

// Handle . and ..
let path = resolver.resolve("/home/user/../admin/file.txt")?;
// Returns: "/home/admin/file.txt"
```

**Path Operations:**
- `resolve()`: Resolve absolute or relative paths
- `normalize()`: Resolve `.` and `..` components
- `parent()`: Get parent directory
- `filename()`: Extract filename component
- `join()`: Join path components

## File Operations

### Open Flags

```rust
pub struct OpenFlags {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub truncate: bool,
    pub append: bool,
}

// Predefined flag combinations
OpenFlags::read_only()     // Read access only
OpenFlags::write_only()    // Write access only
OpenFlags::read_write()    // Read and write access
OpenFlags::create_new()    // Create new file (write + create + truncate)
```

### File Operations API

**Creating Files and Directories:**
```rust
// Create a file
let file = fs.create("/data/file.txt", VNodeType::File)?;

// Create a directory
let dir = fs.create("/data/subdir", VNodeType::Directory)?;
```

**Reading and Writing:**
```rust
// Write to file
let data = b"Hello, World!";
let written = fs.write(&vnode, 0, data)?;

// Read from file
let mut buffer = vec![0u8; 100];
let read = fs.read(&vnode, 0, &mut buffer)?;
```

**File Attributes:**
```rust
let attr = fs.stat(&vnode)?;
println!("Size: {} bytes", attr.size);
println!("Type: {:?}", attr.vtype);
```

**Seeking:**
```rust
let mut fd = FileDescriptor::new(vnode, OpenFlags::read_write());

// Seek to absolute position
fd.seek(10, SeekWhence::Start, file_size)?;

// Seek relative to current position
fd.seek(5, SeekWhence::Current, file_size)?;

// Seek from end
fd.seek(-10, SeekWhence::End, file_size)?;
```

## Directory Operations

### Creating Directories

```rust
// Create a directory
fs.create("/mydir", VNodeType::Directory)?;

// Create nested directories (requires parent to exist)
fs.create("/home", VNodeType::Directory)?;
fs.create("/home/user", VNodeType::Directory)?;
fs.create("/home/user/documents", VNodeType::Directory)?;
```

### Removing Files and Directories

```rust
// Remove a file
fs.remove("/data/file.txt")?;

// Remove an empty directory
fs.remove("/mydir")?;

// Removing non-empty directory returns error
fs.remove("/home")?; // Error: DirectoryNotEmpty
```

### Reading Directory Entries

```rust
let dir = fs.lookup("/home")?;
let entries = fs.readdir(&dir)?;

for entry in entries {
    println!("{} - {:?}", entry.name, entry.vtype);
}
```

## System Call Interface

The VFS is integrated with the system call layer. The following syscalls are defined:

| Syscall | Number | Description |
|---------|--------|-------------|
| `SYS_OPEN` | 2 | Open a file |
| `SYS_CLOSE` | 3 | Close a file descriptor |
| `SYS_READ` | 0 | Read from a file descriptor |
| `SYS_WRITE` | 1 | Write to a file descriptor |
| `SYS_LSEEK` | 8 | Seek in a file |
| `SYS_MKDIR` | 83 | Create a directory |
| `SYS_RMDIR` | 84 | Remove a directory |
| `SYS_UNLINK` | 87 | Remove a file |
| `SYS_GETDENTS` | 78 | Get directory entries |

**Error Codes:**
- `ENOENT` (-2): No such file or directory
- `EBADF` (-9): Bad file descriptor
- `EACCES` (-13): Permission denied
- `EEXIST` (-17): File exists
- `ENOTDIR` (-20): Not a directory
- `EISDIR` (-21): Is a directory
- `EINVAL` (-22): Invalid argument
- `ENOSYS` (-38): Function not implemented
- `ENOTEMPTY` (-39): Directory not empty

## Error Handling

The VFS uses the `FsError` enum for error handling:

```rust
pub enum FsError {
    NotFound,          // File or directory not found
    AlreadyExists,     // File or directory already exists
    NotADirectory,     // Not a directory
    IsADirectory,      // Is a directory
    InvalidPath,       // Invalid path
    PermissionDenied,  // Permission denied
    DirectoryNotEmpty, // Directory not empty
    NoSpace,           // No space left
    InvalidArgument,   // Invalid argument
    IoError,           // I/O error
}
```

## Usage Examples

### Complete File Workflow

```rust
use fanga_kernel::fs::{MemoryFileSystem, FileSystem, VNodeType};

// Create file system
let mut fs = MemoryFileSystem::new();

// Create a file
let vnode = fs.create("/myfile.txt", VNodeType::File)?;

// Write data
let data = b"Hello, VFS!";
fs.write(&vnode, 0, data)?;

// Read data
let mut buffer = vec![0u8; data.len()];
fs.read(&vnode, 0, &mut buffer)?;
assert_eq!(&buffer, data);

// Get file size
let attr = fs.stat(&vnode)?;
assert_eq!(attr.size, data.len());

// Remove file
fs.remove("/myfile.txt")?;
```

### Directory Hierarchy

```rust
// Create directory structure
fs.create("/home", VNodeType::Directory)?;
fs.create("/home/user", VNodeType::Directory)?;
fs.create("/home/user/documents", VNodeType::Directory)?;
fs.create("/home/user/documents/file.txt", VNodeType::File)?;

// List directory contents
let dir = fs.lookup("/home/user")?;
let entries = fs.readdir(&dir)?;
// entries contains "documents"

// Navigate using path resolution
let resolver = PathResolver::with_cwd(String::from("/home/user"));
let path = resolver.resolve("documents/file.txt")?;
// path is "/home/user/documents/file.txt"
```

### File Descriptor Management

```rust
use fanga_kernel::fs::{FileDescriptor, FileDescriptorTable, OpenFlags, SeekWhence};

// Create FD table for a process
let mut fd_table = FileDescriptorTable::new();

// Open files
let fd1 = FileDescriptor::new(vnode1, OpenFlags::read_only());
let fd2 = FileDescriptor::new(vnode2, OpenFlags::read_write());

let fd1_num = fd_table.alloc(fd1)?;  // Returns 0
let fd2_num = fd_table.alloc(fd2)?;  // Returns 1

// Use file descriptor
if let Some(fd) = fd_table.get_mut(fd1_num) {
    fd.seek(100, SeekWhence::Start, file_size)?;
    // Read at offset 100
}

// Close file descriptor
fd_table.close(fd1_num)?;
```

## Testing

The VFS implementation includes comprehensive tests:

### Unit Tests
- Path resolution and normalization
- File descriptor operations
- VNode and directory entry creation
- Open flags combinations

### Integration Tests
- Complete file workflows
- Directory operations
- Nested directory structures
- File descriptor table management
- Error handling scenarios
- Multiple reads/writes
- File truncation

Run tests:
```bash
# Run all VFS tests
cd kernel/crates/fanga-kernel
cargo test --target x86_64-unknown-linux-gnu fs

# Run integration tests
cargo test --target x86_64-unknown-linux-gnu --test vfs_integration
```

## Future Enhancements

Planned enhancements for the VFS:

1. **Additional File Systems**
   - EXT2/3/4 support
   - FAT32 support
   - Device file systems (devfs)
   - Proc file system (procfs)

2. **Advanced Features**
   - File permissions and ownership
   - Symbolic and hard links
   - File locking mechanisms
   - Memory-mapped files
   - Asynchronous I/O

3. **Performance Optimizations**
   - Page cache implementation
   - Read-ahead mechanisms
   - Write-back caching
   - Directory entry caching

4. **System Call Integration**
   - Full implementation of file syscalls
   - Integration with process management
   - User/kernel space buffer copying
   - Access control checks

## Implementation Notes

### Thread Safety

The in-memory file system uses `RwLock` for concurrent access:
- Multiple readers can access the file system simultaneously
- Write operations require exclusive access
- File descriptor tables are process-local

### Memory Management

- Files stored as contiguous vectors in memory
- Automatic resizing when writing beyond current size
- Memory freed when files are removed
- No disk I/O (fully in-memory)

### Limitations

Current implementation limitations:
- No persistence (data lost on reboot)
- No file permissions or ownership
- No symbolic links
- Fixed in-memory storage
- Single file system instance

## See Also

- [System Calls](SYSTEM_CALLS.md) - System call interface
- [Process Management](PROCESS_MANAGEMENT.md) - Process management
- [Memory Management](MEMORY_MANAGEMENT.md) - Memory subsystem
