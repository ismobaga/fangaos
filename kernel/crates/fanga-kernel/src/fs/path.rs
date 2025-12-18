//! Path Resolution
//!
//! This module provides path resolution for both absolute and relative paths.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Path resolver for handling absolute and relative paths
pub struct PathResolver {
    /// Current working directory
    cwd: String,
}

impl PathResolver {
    /// Create a new path resolver with the root as CWD
    pub fn new() -> Self {
        Self {
            cwd: String::from("/"),
        }
    }
    
    /// Create a path resolver with a specific CWD
    pub fn with_cwd(cwd: String) -> Self {
        Self { cwd }
    }
    
    /// Get the current working directory
    pub fn cwd(&self) -> &str {
        &self.cwd
    }
    
    /// Set the current working directory
    pub fn set_cwd(&mut self, path: String) {
        self.cwd = path;
    }
    
    /// Resolve a path (absolute or relative) to an absolute path
    pub fn resolve(&self, path: &str) -> Result<String, &'static str> {
        if path.is_empty() {
            return Err("Empty path");
        }
        
        let absolute_path = if path.starts_with('/') {
            // Already absolute
            String::from(path)
        } else {
            // Relative path - combine with CWD
            if self.cwd.ends_with('/') {
                format!("{}{}", self.cwd, path)
            } else {
                format!("{}/{}", self.cwd, path)
            }
        };
        
        // Normalize the path (handle . and ..)
        Self::normalize(&absolute_path)
    }
    
    /// Normalize a path by resolving . and .. components
    pub fn normalize(path: &str) -> Result<String, &'static str> {
        if !path.starts_with('/') {
            return Err("Path must be absolute after resolution");
        }
        
        let mut components: Vec<&str> = Vec::new();
        
        for component in path.split('/').filter(|s| !s.is_empty()) {
            match component {
                "." => {
                    // Current directory - skip
                }
                ".." => {
                    // Parent directory - pop last component
                    components.pop();
                }
                _ => {
                    // Regular component
                    components.push(component);
                }
            }
        }
        
        // Reconstruct the path
        if components.is_empty() {
            Ok(String::from("/"))
        } else {
            let mut result = String::from("/");
            for (i, component) in components.iter().enumerate() {
                result.push_str(component);
                if i < components.len() - 1 {
                    result.push('/');
                }
            }
            Ok(result)
        }
    }
    
    /// Get the parent directory of a path
    pub fn parent(path: &str) -> Option<String> {
        if path == "/" {
            return None;
        }
        
        let path = path.trim_end_matches('/');
        if let Some(pos) = path.rfind('/') {
            if pos == 0 {
                Some(String::from("/"))
            } else {
                Some(String::from(&path[..pos]))
            }
        } else {
            None
        }
    }
    
    /// Get the filename component of a path
    pub fn filename(path: &str) -> Option<&str> {
        if path == "/" {
            return None;
        }
        
        let path = path.trim_end_matches('/');
        if let Some(pos) = path.rfind('/') {
            Some(&path[pos + 1..])
        } else {
            Some(path)
        }
    }
    
    /// Join two path components
    pub fn join(base: &str, component: &str) -> String {
        if base.ends_with('/') {
            format!("{}{}", base, component)
        } else {
            format!("{}/{}", base, component)
        }
    }
}

impl Default for PathResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resolve_absolute() {
        let resolver = PathResolver::new();
        assert_eq!(resolver.resolve("/foo/bar").unwrap(), "/foo/bar");
    }
    
    #[test]
    fn test_resolve_relative() {
        let resolver = PathResolver::with_cwd(String::from("/home/user"));
        assert_eq!(resolver.resolve("file.txt").unwrap(), "/home/user/file.txt");
    }
    
    #[test]
    fn test_normalize_current_dir() {
        let resolver = PathResolver::new();
        assert_eq!(resolver.resolve("/foo/./bar").unwrap(), "/foo/bar");
    }
    
    #[test]
    fn test_normalize_parent_dir() {
        let resolver = PathResolver::new();
        assert_eq!(resolver.resolve("/foo/bar/../baz").unwrap(), "/foo/baz");
    }
    
    #[test]
    fn test_normalize_root() {
        let resolver = PathResolver::new();
        assert_eq!(resolver.resolve("/").unwrap(), "/");
        assert_eq!(resolver.resolve("/..").unwrap(), "/");
    }
    
    #[test]
    fn test_parent() {
        assert_eq!(PathResolver::parent("/foo/bar").unwrap(), "/foo");
        assert_eq!(PathResolver::parent("/foo").unwrap(), "/");
        assert_eq!(PathResolver::parent("/"), None);
    }
    
    #[test]
    fn test_filename() {
        assert_eq!(PathResolver::filename("/foo/bar"), Some("bar"));
        assert_eq!(PathResolver::filename("/foo"), Some("foo"));
        assert_eq!(PathResolver::filename("/"), None);
    }
    
    #[test]
    fn test_join() {
        assert_eq!(PathResolver::join("/foo", "bar"), "/foo/bar");
        assert_eq!(PathResolver::join("/foo/", "bar"), "/foo/bar");
    }
}
