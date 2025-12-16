//! Advanced Synchronization Primitives
//!
//! This module provides advanced synchronization mechanisms including:
//! - Condition variables
//! - Read-Write locks
//! - Barriers
//! - Spin barriers

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use super::tcb::TaskId;

/// Condition variable for thread synchronization
/// 
/// Allows threads to wait until a particular condition occurs.
/// Must be used in conjunction with a mutex.
#[derive(Debug)]
pub struct ConditionVariable {
    /// Queue of waiting threads
    waiting_threads: VecDeque<TaskId>,
}

impl ConditionVariable {
    /// Create a new condition variable
    pub fn new() -> Self {
        Self {
            waiting_threads: VecDeque::new(),
        }
    }
    
    /// Wait on the condition variable
    /// Returns the task that should be blocked
    /// 
    /// The caller must ensure the associated mutex is held before calling this.
    /// The mutex will be released atomically with going to sleep.
    pub fn wait(&mut self, task_id: TaskId) -> TaskId {
        self.waiting_threads.push_back(task_id);
        task_id
    }
    
    /// Signal one waiting thread
    /// Returns the task ID to wake up, if any
    pub fn signal(&mut self) -> Option<TaskId> {
        self.waiting_threads.pop_front()
    }
    
    /// Broadcast to all waiting threads
    /// Returns all task IDs to wake up
    pub fn broadcast(&mut self) -> Vec<TaskId> {
        let mut tasks = Vec::new();
        while let Some(task_id) = self.waiting_threads.pop_front() {
            tasks.push(task_id);
        }
        tasks
    }
    
    /// Get the number of waiting threads
    pub fn waiting_count(&self) -> usize {
        self.waiting_threads.len()
    }
    
    /// Check if any threads are waiting
    pub fn has_waiters(&self) -> bool {
        !self.waiting_threads.is_empty()
    }
}

impl Default for ConditionVariable {
    fn default() -> Self {
        Self::new()
    }
}

/// Read-Write lock mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RwLockMode {
    /// No lock held
    Unlocked,
    /// Read lock(s) held
    Read,
    /// Write lock held
    Write,
}

/// Read-Write lock for shared/exclusive access
/// 
/// Allows multiple readers or a single writer at a time.
#[derive(Debug)]
pub struct RwLock {
    /// Current lock mode
    mode: RwLockMode,
    
    /// Number of readers (when in Read mode)
    reader_count: usize,
    
    /// Task holding the write lock (when in Write mode)
    writer: Option<TaskId>,
    
    /// Tasks waiting for read access
    read_waiters: VecDeque<TaskId>,
    
    /// Tasks waiting for write access
    write_waiters: VecDeque<TaskId>,
}

impl RwLock {
    /// Create a new unlocked read-write lock
    pub fn new() -> Self {
        Self {
            mode: RwLockMode::Unlocked,
            reader_count: 0,
            writer: None,
            read_waiters: VecDeque::new(),
            write_waiters: VecDeque::new(),
        }
    }
    
    /// Try to acquire a read lock
    /// Returns true if acquired, false if the task should block
    pub fn try_read_lock(&mut self, task_id: TaskId) -> bool {
        match self.mode {
            RwLockMode::Unlocked => {
                // Can acquire read lock
                self.mode = RwLockMode::Read;
                self.reader_count = 1;
                true
            }
            RwLockMode::Read => {
                // Can acquire read lock (multiple readers allowed)
                self.reader_count += 1;
                true
            }
            RwLockMode::Write => {
                // Cannot acquire, writer has exclusive access
                if !self.read_waiters.contains(&task_id) {
                    self.read_waiters.push_back(task_id);
                }
                false
            }
        }
    }
    
    /// Try to acquire a write lock
    /// Returns true if acquired, false if the task should block
    pub fn try_write_lock(&mut self, task_id: TaskId) -> bool {
        match self.mode {
            RwLockMode::Unlocked => {
                // Can acquire write lock
                self.mode = RwLockMode::Write;
                self.writer = Some(task_id);
                true
            }
            RwLockMode::Read | RwLockMode::Write => {
                // Cannot acquire, someone else has access
                if !self.write_waiters.contains(&task_id) {
                    self.write_waiters.push_back(task_id);
                }
                false
            }
        }
    }
    
    /// Release a read lock
    /// Returns tasks to wake up (may include readers and a writer)
    pub fn read_unlock(&mut self, _task_id: TaskId) -> Result<Vec<TaskId>, &'static str> {
        if self.mode != RwLockMode::Read {
            return Err("Lock is not held for reading");
        }
        
        if self.reader_count == 0 {
            return Err("Reader count is already zero");
        }
        
        self.reader_count -= 1;
        
        let mut tasks_to_wake = Vec::new();
        
        if self.reader_count == 0 {
            // Last reader, transition to unlocked
            self.mode = RwLockMode::Unlocked;
            
            // Prefer writers over readers to prevent writer starvation
            if let Some(writer) = self.write_waiters.pop_front() {
                self.mode = RwLockMode::Write;
                self.writer = Some(writer);
                tasks_to_wake.push(writer);
            } else {
                // Wake all waiting readers
                while let Some(reader) = self.read_waiters.pop_front() {
                    tasks_to_wake.push(reader);
                }
                if !tasks_to_wake.is_empty() {
                    self.mode = RwLockMode::Read;
                    self.reader_count = tasks_to_wake.len();
                }
            }
        }
        
        Ok(tasks_to_wake)
    }
    
    /// Release a write lock
    /// Returns tasks to wake up (may include readers or a writer)
    pub fn write_unlock(&mut self, task_id: TaskId) -> Result<Vec<TaskId>, &'static str> {
        if self.mode != RwLockMode::Write {
            return Err("Lock is not held for writing");
        }
        
        if self.writer != Some(task_id) {
            return Err("Task does not hold the write lock");
        }
        
        self.mode = RwLockMode::Unlocked;
        self.writer = None;
        
        let mut tasks_to_wake = Vec::new();
        
        // Prefer writers over readers to prevent writer starvation
        if let Some(writer) = self.write_waiters.pop_front() {
            self.mode = RwLockMode::Write;
            self.writer = Some(writer);
            tasks_to_wake.push(writer);
        } else {
            // Wake all waiting readers
            while let Some(reader) = self.read_waiters.pop_front() {
                tasks_to_wake.push(reader);
            }
            if !tasks_to_wake.is_empty() {
                self.mode = RwLockMode::Read;
                self.reader_count = tasks_to_wake.len();
            }
        }
        
        Ok(tasks_to_wake)
    }
    
    /// Check if the lock is held by any task
    pub fn is_locked(&self) -> bool {
        self.mode != RwLockMode::Unlocked
    }
    
    /// Get the number of active readers
    pub fn reader_count(&self) -> usize {
        if self.mode == RwLockMode::Read {
            self.reader_count
        } else {
            0
        }
    }
    
    /// Get the writer task ID, if any
    pub fn writer(&self) -> Option<TaskId> {
        self.writer
    }
}

impl Default for RwLock {
    fn default() -> Self {
        Self::new()
    }
}

/// Synchronization barrier
/// 
/// Allows multiple threads to wait until all have reached a synchronization point.
#[derive(Debug)]
pub struct Barrier {
    /// Total number of threads that must reach the barrier
    thread_count: usize,
    
    /// Number of threads currently waiting
    waiting_count: usize,
    
    /// Tasks waiting at the barrier
    waiting_tasks: Vec<TaskId>,
    
    /// Generation number (incremented each time barrier is released)
    generation: usize,
}

impl Barrier {
    /// Create a new barrier for the specified number of threads
    pub fn new(thread_count: usize) -> Self {
        Self {
            thread_count,
            waiting_count: 0,
            waiting_tasks: Vec::with_capacity(thread_count),
            generation: 0,
        }
    }
    
    /// Wait at the barrier
    /// Returns (should_block, tasks_to_wake)
    /// - should_block: true if this task should block
    /// - tasks_to_wake: list of tasks to wake (non-empty when barrier is released)
    pub fn wait(&mut self, task_id: TaskId) -> (bool, Vec<TaskId>) {
        self.waiting_count += 1;
        self.waiting_tasks.push(task_id);
        
        if self.waiting_count >= self.thread_count {
            // Barrier is complete, wake all tasks
            let tasks = core::mem::take(&mut self.waiting_tasks);
            self.waiting_count = 0;
            self.generation += 1;
            (false, tasks)
        } else {
            // Task should block
            (true, Vec::new())
        }
    }
    
    /// Get the number of waiting threads
    pub fn waiting_count(&self) -> usize {
        self.waiting_count
    }
    
    /// Get the total thread count
    pub fn thread_count(&self) -> usize {
        self.thread_count
    }
    
    /// Get the current generation
    pub fn generation(&self) -> usize {
        self.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_variable_signal() {
        let mut cv = ConditionVariable::new();
        
        let task1 = TaskId::new(1);
        let task2 = TaskId::new(2);
        
        // Add waiting tasks
        cv.wait(task1);
        cv.wait(task2);
        
        assert_eq!(cv.waiting_count(), 2);
        
        // Signal one task
        let woken = cv.signal();
        assert_eq!(woken, Some(task1));
        assert_eq!(cv.waiting_count(), 1);
        
        // Signal another task
        let woken = cv.signal();
        assert_eq!(woken, Some(task2));
        assert_eq!(cv.waiting_count(), 0);
        
        // No more tasks
        let woken = cv.signal();
        assert_eq!(woken, None);
    }

    #[test]
    fn test_condition_variable_broadcast() {
        let mut cv = ConditionVariable::new();
        
        let task1 = TaskId::new(1);
        let task2 = TaskId::new(2);
        let task3 = TaskId::new(3);
        
        cv.wait(task1);
        cv.wait(task2);
        cv.wait(task3);
        
        assert_eq!(cv.waiting_count(), 3);
        
        // Broadcast to all
        let woken = cv.broadcast();
        assert_eq!(woken.len(), 3);
        assert_eq!(cv.waiting_count(), 0);
    }

    #[test]
    fn test_rwlock_multiple_readers() {
        let mut lock = RwLock::new();
        
        let task1 = TaskId::new(1);
        let task2 = TaskId::new(2);
        
        // First reader
        assert!(lock.try_read_lock(task1));
        assert_eq!(lock.reader_count(), 1);
        
        // Second reader
        assert!(lock.try_read_lock(task2));
        assert_eq!(lock.reader_count(), 2);
        
        // Release first reader
        let woken = lock.read_unlock(task1).unwrap();
        assert_eq!(woken.len(), 0);
        assert_eq!(lock.reader_count(), 1);
        
        // Release second reader
        let woken = lock.read_unlock(task2).unwrap();
        assert_eq!(woken.len(), 0);
        assert!(!lock.is_locked());
    }

    #[test]
    fn test_rwlock_writer_blocks_readers() {
        let mut lock = RwLock::new();
        
        let writer = TaskId::new(1);
        let reader = TaskId::new(2);
        
        // Writer acquires lock
        assert!(lock.try_write_lock(writer));
        assert_eq!(lock.writer(), Some(writer));
        
        // Reader is blocked
        assert!(!lock.try_read_lock(reader));
        
        // Writer releases
        let woken = lock.write_unlock(writer).unwrap();
        assert_eq!(woken.len(), 1);
        assert_eq!(woken[0], reader);
    }

    #[test]
    fn test_rwlock_readers_block_writer() {
        let mut lock = RwLock::new();
        
        let reader = TaskId::new(1);
        let writer = TaskId::new(2);
        
        // Reader acquires lock
        assert!(lock.try_read_lock(reader));
        
        // Writer is blocked
        assert!(!lock.try_write_lock(writer));
        
        // Reader releases
        let woken = lock.read_unlock(reader).unwrap();
        assert_eq!(woken.len(), 1);
        assert_eq!(woken[0], writer);
    }

    #[test]
    fn test_barrier() {
        let mut barrier = Barrier::new(3);
        
        let task1 = TaskId::new(1);
        let task2 = TaskId::new(2);
        let task3 = TaskId::new(3);
        
        // First task waits
        let (should_block, woken) = barrier.wait(task1);
        assert!(should_block);
        assert_eq!(woken.len(), 0);
        assert_eq!(barrier.waiting_count(), 1);
        
        // Second task waits
        let (should_block, woken) = barrier.wait(task2);
        assert!(should_block);
        assert_eq!(woken.len(), 0);
        assert_eq!(barrier.waiting_count(), 2);
        
        // Third task completes the barrier
        let (should_block, woken) = barrier.wait(task3);
        assert!(!should_block);
        assert_eq!(woken.len(), 3);
        assert_eq!(barrier.waiting_count(), 0);
        assert_eq!(barrier.generation(), 1);
    }

    #[test]
    fn test_barrier_reuse() {
        let mut barrier = Barrier::new(2);
        
        let task1 = TaskId::new(1);
        let task2 = TaskId::new(2);
        
        // First round
        barrier.wait(task1);
        let (_, woken) = barrier.wait(task2);
        assert_eq!(woken.len(), 2);
        assert_eq!(barrier.generation(), 1);
        
        // Second round
        barrier.wait(task1);
        let (_, woken) = barrier.wait(task2);
        assert_eq!(woken.len(), 2);
        assert_eq!(barrier.generation(), 2);
    }
}
