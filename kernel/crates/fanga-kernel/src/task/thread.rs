//! Thread Management
//!
//! This module provides kernel and user thread support including:
//! - Thread Control Block (TCB) for threads
//! - Thread creation and lifecycle management
//! - Thread-local storage (TLS)
//! - Thread attributes and capabilities

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

use super::tcb::{TaskId, TaskState, TaskPriority};
use super::context::TaskContext;
use crate::memory::{PhysAddr, VirtAddr};

/// Thread ID - unique identifier for each thread
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(pub usize);

impl ThreadId {
    /// Create a new thread ID
    pub const fn new(id: usize) -> Self {
        ThreadId(id)
    }
    
    /// Get the raw ID value
    pub const fn as_usize(&self) -> usize {
        self.0
    }
}

/// Thread type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadType {
    /// Kernel thread (runs in kernel space)
    Kernel,
    /// User thread (runs in user space)
    User,
}

/// Thread attributes
#[derive(Debug, Clone)]
pub struct ThreadAttributes {
    /// Thread priority
    pub priority: TaskPriority,
    
    /// Stack size in bytes
    pub stack_size: usize,
    
    /// Thread type
    pub thread_type: ThreadType,
    
    /// CPU affinity (None means no affinity)
    pub cpu_affinity: Option<usize>,
    
    /// CPU affinity mask (bitmask for multiple CPUs)
    /// If set, overrides cpu_affinity
    pub cpu_affinity_mask: Option<u64>,
    
    /// Real-time scheduling policy
    pub rt_policy: RtSchedulingPolicy,
}

impl Default for ThreadAttributes {
    fn default() -> Self {
        Self {
            priority: TaskPriority::Normal,
            stack_size: 8192, // 8 KB default stack
            thread_type: ThreadType::Kernel,
            cpu_affinity: None,
            cpu_affinity_mask: None,
            rt_policy: RtSchedulingPolicy::Normal,
        }
    }
}

impl ThreadAttributes {
    /// Set CPU affinity to a specific CPU
    pub fn with_cpu_affinity(mut self, cpu: usize) -> Self {
        // Ensure CPU ID is within valid range (0-63 for 64-bit mask)
        if cpu < 64 {
            self.cpu_affinity = Some(cpu);
            self.cpu_affinity_mask = Some(1u64 << cpu);
        }
        self
    }
    
    /// Set CPU affinity mask (multiple CPUs)
    pub fn with_affinity_mask(mut self, mask: u64) -> Self {
        self.cpu_affinity_mask = Some(mask);
        self.cpu_affinity = None;
        self
    }
    
    /// Check if thread can run on a specific CPU
    pub fn can_run_on_cpu(&self, cpu_id: usize) -> bool {
        // CPU ID must be within valid range
        if cpu_id >= 64 {
            return false;
        }
        
        if let Some(mask) = self.cpu_affinity_mask {
            (mask & (1u64 << cpu_id)) != 0
        } else if let Some(affinity) = self.cpu_affinity {
            affinity == cpu_id
        } else {
            true // No affinity set, can run on any CPU
        }
    }
}

/// Real-time scheduling policies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtSchedulingPolicy {
    /// Normal scheduling (non-realtime)
    Normal,
    /// Real-time FIFO (first-in, first-out)
    RtFifo,
    /// Real-time Round-Robin
    RtRoundRobin,
    /// Deadline scheduling
    Deadline,
}

/// Thread Control Block
#[derive(Debug)]
pub struct Thread {
    /// Unique thread identifier
    pub id: ThreadId,
    
    /// Parent process ID
    pub process_id: TaskId,
    
    /// Thread name
    pub name: String,
    
    /// Current thread state
    pub state: TaskState,
    
    /// Thread attributes
    pub attributes: ThreadAttributes,
    
    /// CPU context (saved registers)
    pub context: TaskContext,
    
    /// Thread stack address
    pub stack: VirtAddr,
    
    /// Thread stack size
    pub stack_size: usize,
    
    /// Thread-local storage pointer
    pub tls_base: VirtAddr,
    
    /// Exit code (valid when state is Terminated)
    pub exit_code: i32,
}

impl Thread {
    /// Create a new thread
    pub fn new(
        id: ThreadId,
        process_id: TaskId,
        name: String,
        entry_point: VirtAddr,
        stack: VirtAddr,
        stack_size: usize,
        attributes: ThreadAttributes,
    ) -> Self {
        let stack_top = stack.as_u64() + stack_size as u64;
        
        Thread {
            id,
            process_id,
            name,
            state: TaskState::Ready,
            attributes,
            context: TaskContext::new(entry_point.as_u64(), stack_top),
            stack,
            stack_size,
            tls_base: VirtAddr::new(0),
            exit_code: 0,
        }
    }
    
    /// Set thread-local storage base address
    pub fn set_tls_base(&mut self, base: VirtAddr) {
        self.tls_base = base;
    }
    
    /// Get thread-local storage base address
    pub fn tls_base(&self) -> VirtAddr {
        self.tls_base
    }
    
    /// Check if the thread is a kernel thread
    pub fn is_kernel_thread(&self) -> bool {
        self.attributes.thread_type == ThreadType::Kernel
    }
    
    /// Check if the thread is a user thread
    pub fn is_user_thread(&self) -> bool {
        self.attributes.thread_type == ThreadType::User
    }
    
    /// Set CPU affinity
    pub fn set_cpu_affinity(&mut self, cpu: usize) {
        self.attributes.cpu_affinity = Some(cpu);
    }
    
    /// Clear CPU affinity
    pub fn clear_cpu_affinity(&mut self) {
        self.attributes.cpu_affinity = None;
    }
    
    /// Get CPU affinity
    pub fn cpu_affinity(&self) -> Option<usize> {
        self.attributes.cpu_affinity
    }
}

/// Thread manager for creating and managing threads
pub struct ThreadManager {
    /// All threads
    threads: Vec<Option<Thread>>,
    
    /// Next available thread ID
    next_thread_id: usize,
    
    /// Maximum number of threads
    max_threads: usize,
}

impl ThreadManager {
    /// Create a new thread manager
    pub fn new(max_threads: usize) -> Self {
        let mut threads = Vec::new();
        threads.resize_with(max_threads, || None);
        
        Self {
            threads,
            next_thread_id: 0,
            max_threads,
        }
    }
    
    /// Create a new thread
    pub fn create_thread(
        &mut self,
        process_id: TaskId,
        name: String,
        entry_point: VirtAddr,
        stack: VirtAddr,
        attributes: ThreadAttributes,
    ) -> Result<ThreadId, &'static str> {
        if self.next_thread_id >= self.max_threads {
            return Err("Maximum number of threads reached");
        }
        
        let thread_id = ThreadId::new(self.next_thread_id);
        self.next_thread_id += 1;
        
        let thread = Thread::new(
            thread_id,
            process_id,
            name,
            entry_point,
            stack,
            attributes.stack_size,
            attributes,
        );
        
        self.threads[thread_id.as_usize()] = Some(thread);
        
        Ok(thread_id)
    }
    
    /// Get a thread by ID
    pub fn get_thread(&self, thread_id: ThreadId) -> Option<&Thread> {
        self.threads.get(thread_id.as_usize())?.as_ref()
    }
    
    /// Get a mutable thread by ID
    pub fn get_thread_mut(&mut self, thread_id: ThreadId) -> Option<&mut Thread> {
        self.threads.get_mut(thread_id.as_usize())?.as_mut()
    }
    
    /// Terminate a thread
    pub fn terminate_thread(&mut self, thread_id: ThreadId, exit_code: i32) -> Result<(), &'static str> {
        if let Some(thread) = self.get_thread_mut(thread_id) {
            thread.state = TaskState::Terminated;
            thread.exit_code = exit_code;
            Ok(())
        } else {
            Err("Thread not found")
        }
    }
    
    /// Get all threads for a process
    pub fn get_process_threads(&self, process_id: TaskId) -> Vec<ThreadId> {
        self.threads
            .iter()
            .enumerate()
            .filter_map(|(idx, thread_opt)| {
                thread_opt.as_ref().and_then(|thread| {
                    if thread.process_id == process_id && thread.state != TaskState::Terminated {
                        Some(thread.id)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
    
    /// Get thread count
    pub fn thread_count(&self) -> usize {
        self.threads.iter().filter(|t| t.is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_id() {
        let id1 = ThreadId::new(1);
        let id2 = ThreadId::new(2);
        
        assert_eq!(id1.as_usize(), 1);
        assert_eq!(id2.as_usize(), 2);
        assert!(id1 < id2);
    }

    #[test]
    fn test_thread_attributes_default() {
        let attrs = ThreadAttributes::default();
        assert_eq!(attrs.priority, TaskPriority::Normal);
        assert_eq!(attrs.stack_size, 8192);
        assert_eq!(attrs.thread_type, ThreadType::Kernel);
        assert_eq!(attrs.cpu_affinity, None);
    }

    #[test]
    fn test_thread_creation() {
        let thread = Thread::new(
            ThreadId::new(1),
            TaskId::new(1),
            String::from("test_thread"),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            8192,
            ThreadAttributes::default(),
        );
        
        assert_eq!(thread.id, ThreadId::new(1));
        assert_eq!(thread.name, "test_thread");
        assert_eq!(thread.state, TaskState::Ready);
        assert!(thread.is_kernel_thread());
    }

    #[test]
    fn test_thread_manager() {
        let mut manager = ThreadManager::new(32);
        
        let thread_id = manager.create_thread(
            TaskId::new(1),
            String::from("thread1"),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            ThreadAttributes::default(),
        ).unwrap();
        
        assert_eq!(thread_id, ThreadId::new(0));
        assert_eq!(manager.thread_count(), 1);
        
        let thread = manager.get_thread(thread_id).unwrap();
        assert_eq!(thread.name, "thread1");
    }

    #[test]
    fn test_thread_termination() {
        let mut manager = ThreadManager::new(32);
        
        let thread_id = manager.create_thread(
            TaskId::new(1),
            String::from("thread1"),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            ThreadAttributes::default(),
        ).unwrap();
        
        manager.terminate_thread(thread_id, 0).unwrap();
        
        let thread = manager.get_thread(thread_id).unwrap();
        assert_eq!(thread.state, TaskState::Terminated);
        assert_eq!(thread.exit_code, 0);
    }

    #[test]
    fn test_cpu_affinity() {
        let mut thread = Thread::new(
            ThreadId::new(1),
            TaskId::new(1),
            String::from("test_thread"),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            8192,
            ThreadAttributes::default(),
        );
        
        assert_eq!(thread.cpu_affinity(), None);
        
        thread.set_cpu_affinity(2);
        assert_eq!(thread.cpu_affinity(), Some(2));
        
        thread.clear_cpu_affinity();
        assert_eq!(thread.cpu_affinity(), None);
    }

    #[test]
    fn test_tls() {
        let mut thread = Thread::new(
            ThreadId::new(1),
            TaskId::new(1),
            String::from("test_thread"),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            8192,
            ThreadAttributes::default(),
        );
        
        assert_eq!(thread.tls_base(), VirtAddr::new(0));
        
        thread.set_tls_base(VirtAddr::new(0x3000));
        assert_eq!(thread.tls_base(), VirtAddr::new(0x3000));
    }
}
