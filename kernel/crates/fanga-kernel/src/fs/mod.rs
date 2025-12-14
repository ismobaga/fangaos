//! File System Abstraction Layer
//!
//! This module provides a Virtual File System (VFS) abstraction layer that supports:
//! - Pluggable file system implementations
//! - File operations (open, close, read, write, seek)
//! - Directory operations (mkdir, rmdir, readdir)
//! - Path resolution (absolute and relative)
//! - Per-process file descriptor tables

pub mod vfs;
pub mod memfs;
pub mod file_descriptor;
pub mod path;

// Re-export commonly used types
pub use vfs::{FileSystem, VNode, VNodeType, OpenFlags, SeekWhence};
pub use memfs::MemoryFileSystem;
pub use file_descriptor::{FileDescriptor, FileDescriptorTable};
pub use path::PathResolver;
