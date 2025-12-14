//! Inter-Process Communication (IPC)
//!
//! This module provides comprehensive IPC mechanisms including:
//! - Message queues
//! - Synchronization primitives (mutex, semaphore)
//! - Pipes (anonymous and named)
//! - Shared memory segments
//! - Signal handling

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use super::tcb::TaskId;

/// Maximum message size in bytes
pub const MAX_MESSAGE_SIZE: usize = 256;

/// Message structure for IPC
#[derive(Debug, Clone)]
pub struct Message {
    /// Sender task ID
    pub sender: TaskId,
    
    /// Message data
    pub data: Vec<u8>,
}

impl Message {
    /// Create a new message
    pub fn new(sender: TaskId, data: Vec<u8>) -> Result<Self, &'static str> {
        if data.len() > MAX_MESSAGE_SIZE {
            return Err("Message too large");
        }
        
        Ok(Message { sender, data })
    }
    
    /// Get the message data as a slice
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Get the message size
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Message queue for inter-process communication
#[derive(Debug)]
pub struct MessageQueue {
    /// Queue of messages
    messages: VecDeque<Message>,
    
    /// Maximum queue size
    max_size: usize,
    
    /// Tasks waiting to receive messages
    waiting_tasks: Vec<TaskId>,
}

impl MessageQueue {
    /// Create a new message queue with the given maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_size),
            max_size,
            waiting_tasks: Vec::new(),
        }
    }
    
    /// Send a message to the queue
    pub fn send(&mut self, message: Message) -> Result<(), &'static str> {
        if self.messages.len() >= self.max_size {
            return Err("Message queue is full");
        }
        
        self.messages.push_back(message);
        Ok(())
    }
    
    /// Receive a message from the queue (non-blocking)
    pub fn receive(&mut self) -> Option<Message> {
        self.messages.pop_front()
    }
    
    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    
    /// Get the number of messages in the queue
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    /// Add a task to the waiting list
    pub fn add_waiting_task(&mut self, task_id: TaskId) {
        if !self.waiting_tasks.contains(&task_id) {
            self.waiting_tasks.push(task_id);
        }
    }
    
    /// Remove a task from the waiting list
    pub fn remove_waiting_task(&mut self, task_id: TaskId) {
        self.waiting_tasks.retain(|&id| id != task_id);
    }
    
    /// Get the list of waiting tasks
    pub fn waiting_tasks(&self) -> &[TaskId] {
        &self.waiting_tasks
    }
}

/// Semaphore for synchronization
#[derive(Debug)]
pub struct Semaphore {
    /// Current value
    value: isize,
    
    /// Tasks waiting on this semaphore
    waiting_tasks: Vec<TaskId>,
}

impl Semaphore {
    /// Create a new semaphore with the given initial value
    pub fn new(initial_value: isize) -> Self {
        Self {
            value: initial_value,
            waiting_tasks: Vec::new(),
        }
    }
    
    /// Wait (P operation, decrement)
    /// Returns true if the operation succeeded, false if task should block
    pub fn wait(&mut self, task_id: TaskId) -> bool {
        if self.value > 0 {
            self.value -= 1;
            true
        } else {
            if !self.waiting_tasks.contains(&task_id) {
                self.waiting_tasks.push(task_id);
            }
            false
        }
    }
    
    /// Signal (V operation, increment)
    /// Returns the next task to wake up, if any
    pub fn signal(&mut self) -> Option<TaskId> {
        if !self.waiting_tasks.is_empty() {
            // Wake up a waiting task
            Some(self.waiting_tasks.remove(0))
        } else {
            // No one waiting, just increment
            self.value += 1;
            None
        }
    }
    
    /// Get the current value
    pub fn value(&self) -> isize {
        self.value
    }
    
    /// Get the number of waiting tasks
    pub fn waiting_count(&self) -> usize {
        self.waiting_tasks.len()
    }
}

/// Simple mutex implementation for task synchronization
#[derive(Debug)]
pub struct TaskMutex {
    /// Is the mutex locked?
    locked: bool,
    
    /// Task that currently holds the mutex
    owner: Option<TaskId>,
    
    /// Tasks waiting for the mutex
    waiting_tasks: Vec<TaskId>,
}

impl TaskMutex {
    /// Create a new unlocked mutex
    pub fn new() -> Self {
        Self {
            locked: false,
            owner: None,
            waiting_tasks: Vec::new(),
        }
    }
    
    /// Try to acquire the mutex
    /// Returns true if acquired, false if task should block
    pub fn try_lock(&mut self, task_id: TaskId) -> bool {
        if !self.locked {
            self.locked = true;
            self.owner = Some(task_id);
            true
        } else {
            if !self.waiting_tasks.contains(&task_id) {
                self.waiting_tasks.push(task_id);
            }
            false
        }
    }
    
    /// Release the mutex
    /// Returns the next task to wake up, if any
    pub fn unlock(&mut self, task_id: TaskId) -> Result<Option<TaskId>, &'static str> {
        if self.owner != Some(task_id) {
            return Err("Task does not own the mutex");
        }
        
        self.locked = false;
        self.owner = None;
        
        Ok(if !self.waiting_tasks.is_empty() {
            Some(self.waiting_tasks.remove(0))
        } else {
            None
        })
    }
    
    /// Check if the mutex is locked
    pub fn is_locked(&self) -> bool {
        self.locked
    }
    
    /// Get the owner of the mutex
    pub fn owner(&self) -> Option<TaskId> {
        self.owner
    }
    
    /// Get the number of waiting tasks
    pub fn waiting_count(&self) -> usize {
        self.waiting_tasks.len()
    }
}

impl Default for TaskMutex {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipe buffer size in bytes
pub const PIPE_BUFFER_SIZE: usize = 4096;

/// Pipe for inter-process communication
#[derive(Debug)]
pub struct Pipe {
    /// Buffer for pipe data
    buffer: VecDeque<u8>,
    
    /// Maximum buffer size
    max_size: usize,
    
    /// Number of readers
    readers: usize,
    
    /// Number of writers
    writers: usize,
    
    /// Tasks waiting to read
    waiting_readers: Vec<TaskId>,
    
    /// Tasks waiting to write
    waiting_writers: Vec<TaskId>,
}

impl Pipe {
    /// Create a new pipe
    pub fn new() -> Self {
        Self::with_capacity(PIPE_BUFFER_SIZE)
    }
    
    /// Create a new pipe with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            max_size: capacity,
            readers: 0,
            writers: 0,
            waiting_readers: Vec::new(),
            waiting_writers: Vec::new(),
        }
    }
    
    /// Add a reader to the pipe
    pub fn add_reader(&mut self) {
        self.readers += 1;
    }
    
    /// Add a writer to the pipe
    pub fn add_writer(&mut self) {
        self.writers += 1;
    }
    
    /// Remove a reader from the pipe
    pub fn remove_reader(&mut self) {
        if self.readers > 0 {
            self.readers -= 1;
        }
    }
    
    /// Remove a writer from the pipe
    pub fn remove_writer(&mut self) {
        if self.writers > 0 {
            self.writers -= 1;
        }
    }
    
    /// Check if pipe is readable (has data or no writers)
    pub fn is_readable(&self) -> bool {
        !self.buffer.is_empty() || self.writers == 0
    }
    
    /// Check if pipe is writable (has space and has readers)
    pub fn is_writable(&self) -> bool {
        self.buffer.len() < self.max_size && self.readers > 0
    }
    
    /// Write data to the pipe (non-blocking)
    pub fn write(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        if self.readers == 0 {
            return Err("No readers (broken pipe)");
        }
        
        let available_space = self.max_size - self.buffer.len();
        if available_space == 0 {
            return Ok(0); // Would block
        }
        
        let bytes_to_write = core::cmp::min(data.len(), available_space);
        for &byte in &data[..bytes_to_write] {
            self.buffer.push_back(byte);
        }
        
        Ok(bytes_to_write)
    }
    
    /// Read data from the pipe (non-blocking)
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, &'static str> {
        if self.buffer.is_empty() {
            if self.writers == 0 {
                return Ok(0); // EOF
            }
            return Ok(0); // Would block
        }
        
        let bytes_to_read = core::cmp::min(buf.len(), self.buffer.len());
        for i in 0..bytes_to_read {
            buf[i] = self.buffer.pop_front().unwrap();
        }
        
        Ok(bytes_to_read)
    }
    
    /// Get the number of bytes available to read
    pub fn available(&self) -> usize {
        self.buffer.len()
    }
    
    /// Add a task waiting to read
    pub fn add_waiting_reader(&mut self, task_id: TaskId) {
        if !self.waiting_readers.contains(&task_id) {
            self.waiting_readers.push(task_id);
        }
    }
    
    /// Add a task waiting to write
    pub fn add_waiting_writer(&mut self, task_id: TaskId) {
        if !self.waiting_writers.contains(&task_id) {
            self.waiting_writers.push(task_id);
        }
    }
    
    /// Wake up waiting readers
    pub fn wake_readers(&mut self) -> Vec<TaskId> {
        core::mem::take(&mut self.waiting_readers)
    }
    
    /// Wake up waiting writers
    pub fn wake_writers(&mut self) -> Vec<TaskId> {
        core::mem::take(&mut self.waiting_writers)
    }
}

impl Default for Pipe {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared memory segment
#[derive(Debug)]
pub struct SharedMemory {
    /// Physical address of the shared memory
    phys_addr: crate::memory::PhysAddr,
    
    /// Size of the shared memory segment in bytes
    size: usize,
    
    /// Reference count (number of processes attached)
    ref_count: usize,
    
    /// Processes that have attached this segment
    attached_tasks: Vec<TaskId>,
}

impl SharedMemory {
    /// Create a new shared memory segment
    pub fn new(phys_addr: crate::memory::PhysAddr, size: usize) -> Self {
        Self {
            phys_addr,
            size,
            ref_count: 0,
            attached_tasks: Vec::new(),
        }
    }
    
    /// Attach a task to this shared memory segment
    pub fn attach(&mut self, task_id: TaskId) -> Result<(), &'static str> {
        if !self.attached_tasks.contains(&task_id) {
            self.attached_tasks.push(task_id);
            self.ref_count += 1;
        }
        Ok(())
    }
    
    /// Detach a task from this shared memory segment
    pub fn detach(&mut self, task_id: TaskId) -> Result<(), &'static str> {
        if let Some(pos) = self.attached_tasks.iter().position(|&id| id == task_id) {
            self.attached_tasks.remove(pos);
            if self.ref_count > 0 {
                self.ref_count -= 1;
            }
        }
        Ok(())
    }
    
    /// Get the physical address
    pub fn phys_addr(&self) -> crate::memory::PhysAddr {
        self.phys_addr
    }
    
    /// Get the size
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Get the reference count
    pub fn ref_count(&self) -> usize {
        self.ref_count
    }
    
    /// Check if any tasks are attached
    pub fn has_attachments(&self) -> bool {
        self.ref_count > 0
    }
}

/// Signal types (POSIX-like)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Signal {
    /// Hangup
    SIGHUP = 1,
    /// Interrupt
    SIGINT = 2,
    /// Quit
    SIGQUIT = 3,
    /// Illegal instruction
    SIGILL = 4,
    /// Trace/breakpoint trap
    SIGTRAP = 5,
    /// Abort
    SIGABRT = 6,
    /// Bus error
    SIGBUS = 7,
    /// Floating point exception
    SIGFPE = 8,
    /// Kill (cannot be caught)
    SIGKILL = 9,
    /// User-defined signal 1
    SIGUSR1 = 10,
    /// Segmentation fault
    SIGSEGV = 11,
    /// User-defined signal 2
    SIGUSR2 = 12,
    /// Broken pipe
    SIGPIPE = 13,
    /// Alarm clock
    SIGALRM = 14,
    /// Termination
    SIGTERM = 15,
    /// Child stopped or terminated
    SIGCHLD = 17,
    /// Continue
    SIGCONT = 18,
    /// Stop (cannot be caught)
    SIGSTOP = 19,
    /// Terminal stop
    SIGTSTP = 20,
}

impl Signal {
    /// Convert from signal number
    pub fn from_num(num: u8) -> Option<Self> {
        match num {
            1 => Some(Signal::SIGHUP),
            2 => Some(Signal::SIGINT),
            3 => Some(Signal::SIGQUIT),
            4 => Some(Signal::SIGILL),
            5 => Some(Signal::SIGTRAP),
            6 => Some(Signal::SIGABRT),
            7 => Some(Signal::SIGBUS),
            8 => Some(Signal::SIGFPE),
            9 => Some(Signal::SIGKILL),
            10 => Some(Signal::SIGUSR1),
            11 => Some(Signal::SIGSEGV),
            12 => Some(Signal::SIGUSR2),
            13 => Some(Signal::SIGPIPE),
            14 => Some(Signal::SIGALRM),
            15 => Some(Signal::SIGTERM),
            17 => Some(Signal::SIGCHLD),
            18 => Some(Signal::SIGCONT),
            19 => Some(Signal::SIGSTOP),
            20 => Some(Signal::SIGTSTP),
            _ => None,
        }
    }
    
    /// Get the signal number
    pub fn num(&self) -> u8 {
        *self as u8
    }
}

/// Signal handler
#[derive(Debug)]
pub struct SignalHandler {
    /// Pending signals (bit mask)
    pending: u32,
    
    /// Blocked signals (bit mask)
    blocked: u32,
}

impl SignalHandler {
    /// Create a new signal handler
    pub fn new() -> Self {
        Self {
            pending: 0,
            blocked: 0,
        }
    }
    
    /// Helper to get the bit mask for a signal
    fn signal_bit(signal: Signal) -> u32 {
        1u32 << (signal.num() as u32)
    }
    
    /// Send a signal
    pub fn send(&mut self, signal: Signal) {
        let bit = Self::signal_bit(signal);
        self.pending |= bit;
    }
    
    /// Check if a signal is pending
    pub fn is_pending(&self, signal: Signal) -> bool {
        let bit = Self::signal_bit(signal);
        (self.pending & bit) != 0
    }
    
    /// Check if a signal is blocked
    pub fn is_blocked(&self, signal: Signal) -> bool {
        let bit = Self::signal_bit(signal);
        (self.blocked & bit) != 0
    }
    
    /// Block a signal
    pub fn block(&mut self, signal: Signal) {
        let bit = Self::signal_bit(signal);
        self.blocked |= bit;
    }
    
    /// Unblock a signal
    pub fn unblock(&mut self, signal: Signal) {
        let bit = Self::signal_bit(signal);
        self.blocked &= !bit;
    }
    
    /// Clear a pending signal
    pub fn clear(&mut self, signal: Signal) {
        let bit = Self::signal_bit(signal);
        self.pending &= !bit;
    }
    
    /// Get the next unblocked pending signal
    pub fn next_unblocked(&self) -> Option<Signal> {
        let unblocked_pending = self.pending & !self.blocked;
        if unblocked_pending == 0 {
            return None;
        }
        
        // Find the first set bit (0-indexed)
        let bit_pos = unblocked_pending.trailing_zeros() as u8;
        
        // Signal numbers use the bit position directly
        // (bit 1 = SIGHUP (1), bit 2 = SIGINT (2), etc.)
        Signal::from_num(bit_pos)
    }
    
    /// Check if there are any pending unblocked signals
    pub fn has_pending(&self) -> bool {
        (self.pending & !self.blocked) != 0
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_new() {
        let sender = TaskId::new(1);
        let data = vec![1, 2, 3, 4];
        
        let msg = Message::new(sender, data.clone()).unwrap();
        assert_eq!(msg.sender, sender);
        assert_eq!(msg.data(), &data[..]);
        assert_eq!(msg.size(), 4);
    }

    #[test]
    fn test_message_too_large() {
        let sender = TaskId::new(1);
        let data = vec![0u8; MAX_MESSAGE_SIZE + 1];
        
        let result = Message::new(sender, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_queue() {
        let mut queue = MessageQueue::new(10);
        
        let msg1 = Message::new(TaskId::new(1), vec![1, 2, 3]).unwrap();
        let msg2 = Message::new(TaskId::new(2), vec![4, 5, 6]).unwrap();
        
        assert!(queue.send(msg1).is_ok());
        assert!(queue.send(msg2).is_ok());
        assert_eq!(queue.len(), 2);
        
        let received = queue.receive().unwrap();
        assert_eq!(received.sender, TaskId::new(1));
        assert_eq!(received.data(), &[1, 2, 3]);
        
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_message_queue_full() {
        let mut queue = MessageQueue::new(2);
        
        let msg1 = Message::new(TaskId::new(1), vec![1]).unwrap();
        let msg2 = Message::new(TaskId::new(1), vec![2]).unwrap();
        let msg3 = Message::new(TaskId::new(1), vec![3]).unwrap();
        
        assert!(queue.send(msg1).is_ok());
        assert!(queue.send(msg2).is_ok());
        assert!(queue.send(msg3).is_err());
    }

    #[test]
    fn test_semaphore() {
        let mut sem = Semaphore::new(1);
        let task1 = TaskId::new(1);
        let task2 = TaskId::new(2);
        
        // First wait should succeed
        assert!(sem.wait(task1));
        assert_eq!(sem.value(), 0);
        
        // Second wait should block
        assert!(!sem.wait(task2));
        assert_eq!(sem.waiting_count(), 1);
        
        // Signal should wake up task2
        let woken = sem.signal();
        assert_eq!(woken, Some(task2));
    }

    #[test]
    fn test_mutex() {
        let mut mutex = TaskMutex::new();
        let task1 = TaskId::new(1);
        let task2 = TaskId::new(2);
        
        // First lock should succeed
        assert!(mutex.try_lock(task1));
        assert!(mutex.is_locked());
        assert_eq!(mutex.owner(), Some(task1));
        
        // Second lock should fail
        assert!(!mutex.try_lock(task2));
        assert_eq!(mutex.waiting_count(), 1);
        
        // Unlock should wake up task2
        let woken = mutex.unlock(task1).unwrap();
        assert_eq!(woken, Some(task2));
        assert!(!mutex.is_locked());
    }

    #[test]
    fn test_mutex_wrong_owner() {
        let mut mutex = TaskMutex::new();
        let task1 = TaskId::new(1);
        let task2 = TaskId::new(2);
        
        mutex.try_lock(task1);
        
        // Task2 cannot unlock task1's mutex
        let result = mutex.unlock(task2);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipe_basic() {
        let mut pipe = Pipe::new();
        
        // Add reader and writer
        pipe.add_reader();
        pipe.add_writer();
        
        // Write data
        let data = b"Hello, pipe!";
        let written = pipe.write(data).unwrap();
        assert_eq!(written, data.len());
        assert_eq!(pipe.available(), data.len());
        
        // Read data
        let mut buf = [0u8; 32];
        let read = pipe.read(&mut buf).unwrap();
        assert_eq!(read, data.len());
        assert_eq!(&buf[..read], data);
        assert_eq!(pipe.available(), 0);
    }

    #[test]
    fn test_pipe_broken() {
        let mut pipe = Pipe::new();
        
        // Add only writer, no readers
        pipe.add_writer();
        
        // Write should fail (broken pipe)
        let data = b"test";
        let result = pipe.write(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipe_eof() {
        let mut pipe = Pipe::new();
        
        // Add reader but no writers
        pipe.add_reader();
        
        // Read should return 0 (EOF)
        let mut buf = [0u8; 32];
        let read = pipe.read(&mut buf).unwrap();
        assert_eq!(read, 0);
    }

    #[test]
    fn test_pipe_full() {
        let mut pipe = Pipe::with_capacity(4);
        
        pipe.add_reader();
        pipe.add_writer();
        
        // Fill the pipe
        let data = b"12345678";
        let written1 = pipe.write(&data[..4]).unwrap();
        assert_eq!(written1, 4);
        
        // Pipe is full, should write 0 bytes
        let written2 = pipe.write(&data[4..]).unwrap();
        assert_eq!(written2, 0);
    }

    #[test]
    fn test_shared_memory() {
        use crate::memory::PhysAddr;
        
        let mut shm = SharedMemory::new(PhysAddr::new(0x1000), 4096);
        let task1 = TaskId::new(1);
        let task2 = TaskId::new(2);
        
        // Initially no attachments
        assert_eq!(shm.ref_count(), 0);
        assert!(!shm.has_attachments());
        
        // Attach task1
        assert!(shm.attach(task1).is_ok());
        assert_eq!(shm.ref_count(), 1);
        assert!(shm.has_attachments());
        
        // Attach task2
        assert!(shm.attach(task2).is_ok());
        assert_eq!(shm.ref_count(), 2);
        
        // Detach task1
        assert!(shm.detach(task1).is_ok());
        assert_eq!(shm.ref_count(), 1);
        
        // Detach task2
        assert!(shm.detach(task2).is_ok());
        assert_eq!(shm.ref_count(), 0);
        assert!(!shm.has_attachments());
    }

    #[test]
    fn test_signal_send_and_check() {
        let mut handler = SignalHandler::new();
        
        // Send a signal
        handler.send(Signal::SIGINT);
        assert!(handler.is_pending(Signal::SIGINT));
        assert!(!handler.is_pending(Signal::SIGTERM));
        
        // Clear the signal
        handler.clear(Signal::SIGINT);
        assert!(!handler.is_pending(Signal::SIGINT));
    }

    #[test]
    fn test_signal_blocking() {
        let mut handler = SignalHandler::new();
        
        // Send and block a signal
        handler.send(Signal::SIGUSR1);
        handler.block(Signal::SIGUSR1);
        
        assert!(handler.is_pending(Signal::SIGUSR1));
        assert!(handler.is_blocked(Signal::SIGUSR1));
        assert!(!handler.has_pending()); // Blocked, so not available
        
        // Unblock
        handler.unblock(Signal::SIGUSR1);
        assert!(handler.has_pending());
        
        let next = handler.next_unblocked();
        assert_eq!(next, Some(Signal::SIGUSR1));
    }

    #[test]
    fn test_signal_priority() {
        let mut handler = SignalHandler::new();
        
        // Send multiple signals
        handler.send(Signal::SIGTERM); // 15
        handler.send(Signal::SIGINT);  // 2
        handler.send(Signal::SIGHUP);  // 1
        
        // Should get SIGHUP first (lowest signal number)
        let next = handler.next_unblocked();
        assert_eq!(next, Some(Signal::SIGHUP));
        
        handler.clear(Signal::SIGHUP);
        
        // Should get SIGINT next
        let next = handler.next_unblocked();
        assert_eq!(next, Some(Signal::SIGINT));
        
        handler.clear(Signal::SIGINT);
        
        // Should get SIGTERM last
        let next = handler.next_unblocked();
        assert_eq!(next, Some(Signal::SIGTERM));
    }

    #[test]
    fn test_signal_from_num() {
        assert_eq!(Signal::from_num(2), Some(Signal::SIGINT));
        assert_eq!(Signal::from_num(9), Some(Signal::SIGKILL));
        assert_eq!(Signal::from_num(15), Some(Signal::SIGTERM));
        assert_eq!(Signal::from_num(99), None);
    }

    #[test]
    fn test_signal_num() {
        assert_eq!(Signal::SIGINT.num(), 2);
        assert_eq!(Signal::SIGKILL.num(), 9);
        assert_eq!(Signal::SIGTERM.num(), 15);
    }
}
