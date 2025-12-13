//! Inter-Process Communication (IPC)
//!
//! This module provides basic IPC mechanisms including:
//! - Message queues
//! - Synchronization primitives (mutex, semaphore)

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
}
