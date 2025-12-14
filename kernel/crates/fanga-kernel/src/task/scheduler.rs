//! Task Scheduler
//!
//! This module implements the task scheduler with support for:
//! - Round-robin scheduling
//! - Priority-based scheduling
//! - Task queue management

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use super::tcb::{Task, TaskId, TaskState, TaskPriority};
use spin::Mutex;

/// Maximum number of tasks the scheduler can manage
pub const MAX_TASKS: usize = 256;

/// Scheduler implementation
pub struct Scheduler {
    /// All tasks indexed by task ID
    tasks: Vec<Option<Task>>,
    
    /// Ready queue for each priority level
    ready_queues: [VecDeque<TaskId>; 4],
    
    /// Currently running task ID
    current_task: Option<TaskId>,
    
    /// Next available task ID
    next_task_id: usize,
}

impl Scheduler {
    /// Create a new scheduler
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            ready_queues: [
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
            ],
            current_task: None,
            next_task_id: 1,
        }
    }
    
    /// Initialize the scheduler with capacity
    pub fn init(&mut self) {
        // Clear and reserve space for tasks
        self.tasks.clear();
        if MAX_TASKS > 0 {
            self.tasks.reserve(MAX_TASKS);
            for _ in 0..MAX_TASKS {
                self.tasks.push(None);
            }
        }
        
        // Initialize ready queues with capacity to avoid 0-byte allocation panics
        self.ready_queues = [
            VecDeque::with_capacity(16),
            VecDeque::with_capacity(16),
            VecDeque::with_capacity(16),
            VecDeque::with_capacity(16),
        ];
    }
    
    /// Add a new task to the scheduler
    pub fn add_task(&mut self, mut task: Task) -> Result<TaskId, &'static str> {
        if self.next_task_id >= MAX_TASKS {
            return Err("Maximum number of tasks reached");
        }
        
        let task_id = TaskId::new(self.next_task_id);
        task.id = task_id;
        task.state = TaskState::Ready;
        
        self.next_task_id += 1;
        
        // Add to tasks list
        self.tasks[task_id.as_usize()] = Some(task);
        
        // Add to appropriate ready queue
        let priority_index = self.tasks[task_id.as_usize()]
            .as_ref()
            .unwrap()
            .priority as usize;
        self.ready_queues[priority_index].push_back(task_id);
        
        Ok(task_id)
    }
    
    /// Get a reference to a task
    pub fn get_task(&self, task_id: TaskId) -> Option<&Task> {
        self.tasks.get(task_id.as_usize())?.as_ref()
    }
    
    /// Get a mutable reference to a task
    pub fn get_task_mut(&mut self, task_id: TaskId) -> Option<&mut Task> {
        self.tasks.get_mut(task_id.as_usize())?.as_mut()
    }
    
    /// Get the currently running task ID
    pub fn current_task(&self) -> Option<TaskId> {
        self.current_task
    }
    
    /// Get a reference to the currently running task
    pub fn current_task_ref(&self) -> Option<&Task> {
        self.current_task.and_then(|id| self.get_task(id))
    }
    
    /// Get a mutable reference to the currently running task
    pub fn current_task_mut(&mut self) -> Option<&mut Task> {
        self.current_task.and_then(|id| self.get_task_mut(id))
    }
    
    /// Select the next task to run using priority-based round-robin
    /// Returns (previous_task_id, next_task_id, should_switch)
    pub fn schedule(&mut self) -> (Option<TaskId>, Option<TaskId>, bool) {
        let prev_task = self.current_task;
        
        // If there's a currently running task, move it back to ready queue
        if let Some(task_id) = self.current_task {
            if let Some(task) = self.get_task_mut(task_id) {
                if task.state == TaskState::Running {
                    task.state = TaskState::Ready;
                    let priority_index = task.priority as usize;
                    self.ready_queues[priority_index].push_back(task_id);
                }
            }
        }
        
        // Find the next task to run (highest priority first)
        let mut next_task = None;
        for priority_queue in self.ready_queues.iter_mut().rev() {
            while let Some(task_id) = priority_queue.pop_front() {
                // Check if task is still ready
                let is_ready = self.tasks.get(task_id.as_usize())
                    .and_then(|t| t.as_ref())
                    .map(|t| t.state == TaskState::Ready)
                    .unwrap_or(false);
                    
                if is_ready {
                    next_task = Some(task_id);
                    break;
                }
            }
            if next_task.is_some() {
                break;
            }
        }
        
        // Update current task and state
        if let Some(task_id) = next_task {
            if let Some(task) = self.get_task_mut(task_id) {
                task.state = TaskState::Running;
            }
            self.current_task = Some(task_id);
        } else {
            self.current_task = None;
        }
        
        let should_switch = prev_task != next_task;
        (prev_task, next_task, should_switch)
    }
    
    /// Terminate a task
    pub fn terminate_task(&mut self, task_id: TaskId) -> Result<(), &'static str> {
        if let Some(task) = self.get_task_mut(task_id) {
            task.state = TaskState::Terminated;
            
            if self.current_task == Some(task_id) {
                self.current_task = None;
            }
            
            Ok(())
        } else {
            Err("Task not found")
        }
    }
    
    /// Block a task (remove from ready queue)
    pub fn block_task(&mut self, task_id: TaskId) -> Result<(), &'static str> {
        if let Some(task) = self.get_task_mut(task_id) {
            task.state = TaskState::Blocked;
            Ok(())
        } else {
            Err("Task not found")
        }
    }
    
    /// Unblock a task (add back to ready queue)
    pub fn unblock_task(&mut self, task_id: TaskId) -> Result<(), &'static str> {
        if let Some(task) = self.get_task_mut(task_id) {
            task.state = TaskState::Ready;
            let priority_index = task.priority as usize;
            self.ready_queues[priority_index].push_back(task_id);
            Ok(())
        } else {
            Err("Task not found")
        }
    }
    
    /// Get the number of ready tasks
    pub fn ready_task_count(&self) -> usize {
        self.ready_queues.iter().map(|q| q.len()).sum()
    }
    
    /// Get the total number of tasks (excluding terminated)
    pub fn total_task_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.is_some()).count()
    }
}

/// Global scheduler instance
static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

/// Initialize the global scheduler
pub fn init() {
    let mut scheduler_guard = SCHEDULER.lock();
    scheduler_guard.init();
}

/// Get a reference to the global scheduler
pub fn scheduler() -> spin::MutexGuard<'static, Scheduler> {
    SCHEDULER.lock()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{PhysAddr, VirtAddr};

    #[test]
    fn test_scheduler_new() {
        let scheduler = Scheduler::new();
        assert_eq!(scheduler.current_task(), None);
        assert_eq!(scheduler.next_task_id, 1);
    }

    #[test]
    fn test_scheduler_add_task() {
        let mut scheduler = Scheduler::new();
        scheduler.init();
        
        let task = Task::new(
            TaskId::new(0),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            4096,
            PhysAddr::new(0x3000),
            TaskPriority::Normal,
        );
        
        let result = scheduler.add_task(task);
        assert!(result.is_ok());
        
        let task_id = result.unwrap();
        assert_eq!(task_id.as_usize(), 1);
    }

    #[test]
    fn test_scheduler_schedule() {
        let mut scheduler = Scheduler::new();
        scheduler.init();
        
        // Add two tasks
        let task1 = Task::new(
            TaskId::new(0),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            4096,
            PhysAddr::new(0x3000),
            TaskPriority::Normal,
        );
        
        let task2 = Task::new(
            TaskId::new(0),
            VirtAddr::new(0x1100),
            VirtAddr::new(0x2100),
            4096,
            PhysAddr::new(0x3100),
            TaskPriority::Normal,
        );
        
        let id1 = scheduler.add_task(task1).unwrap();
        let id2 = scheduler.add_task(task2).unwrap();
        
        // First schedule should select task1
        let (prev, next, should_switch) = scheduler.schedule();
        assert_eq!(prev, None);
        assert_eq!(next, Some(id1));
        assert!(should_switch);
        
        // Second schedule should switch to task2
        let (prev, next, should_switch) = scheduler.schedule();
        assert_eq!(prev, Some(id1));
        assert_eq!(next, Some(id2));
        assert!(should_switch);
    }

    #[test]
    fn test_scheduler_priority() {
        let mut scheduler = Scheduler::new();
        scheduler.init();
        
        // Add low priority task
        let task_low = Task::new(
            TaskId::new(0),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            4096,
            PhysAddr::new(0x3000),
            TaskPriority::Low,
        );
        
        // Add high priority task
        let task_high = Task::new(
            TaskId::new(0),
            VirtAddr::new(0x1100),
            VirtAddr::new(0x2100),
            4096,
            PhysAddr::new(0x3100),
            TaskPriority::High,
        );
        
        let id_low = scheduler.add_task(task_low).unwrap();
        let id_high = scheduler.add_task(task_high).unwrap();
        
        // Schedule should select high priority task first
        let (_, next, _) = scheduler.schedule();
        assert_eq!(next, Some(id_high));
        
        // Next schedule should select high priority task again (it goes back to ready queue)
        let (_, next, _) = scheduler.schedule();
        assert_eq!(next, Some(id_high));
        
        // Now terminate the high priority task
        scheduler.terminate_task(id_high).unwrap();
        
        // Next schedule should now select low priority task
        let (_, next, _) = scheduler.schedule();
        assert_eq!(next, Some(id_low));
    }

    #[test]
    fn test_scheduler_terminate() {
        let mut scheduler = Scheduler::new();
        scheduler.init();
        
        let task = Task::new(
            TaskId::new(0),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            4096,
            PhysAddr::new(0x3000),
            TaskPriority::Normal,
        );
        
        let task_id = scheduler.add_task(task).unwrap();
        
        // Schedule the task
        scheduler.schedule();
        
        // Terminate it
        let result = scheduler.terminate_task(task_id);
        assert!(result.is_ok());
        
        // Verify task is terminated
        let task = scheduler.get_task(task_id).unwrap();
        assert!(task.is_terminated());
    }

    #[test]
    fn test_scheduler_block_unblock() {
        let mut scheduler = Scheduler::new();
        scheduler.init();
        
        let task = Task::new(
            TaskId::new(0),
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            4096,
            PhysAddr::new(0x3000),
            TaskPriority::Normal,
        );
        
        let task_id = scheduler.add_task(task).unwrap();
        
        // Block the task
        scheduler.block_task(task_id).unwrap();
        let task = scheduler.get_task(task_id).unwrap();
        assert_eq!(task.state, TaskState::Blocked);
        
        // Unblock the task
        scheduler.unblock_task(task_id).unwrap();
        let task = scheduler.get_task(task_id).unwrap();
        assert_eq!(task.state, TaskState::Ready);
    }
}
