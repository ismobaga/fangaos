//! Process Management Integration Tests
//!
//! This module tests the complete process management system including:
//! - Process creation and termination
//! - Task scheduling
//! - Process states
//!
//! Note: These tests use local scheduler instances to avoid global state issues

use fanga_kernel::task::{Task, TaskId, TaskState, TaskPriority, Scheduler};
use fanga_kernel::memory::{PhysAddr, VirtAddr};

#[test]
fn test_local_scheduler_basic() {
    let mut scheduler = Scheduler::new();
    scheduler.init();
    
    // Create a task
    let task = Task::new(
        TaskId::new(0),
        VirtAddr::new(0x1000),
        VirtAddr::new(0x10000),
        4096,
        PhysAddr::new(0x0),
        TaskPriority::Normal,
    );
    
    let task_id = scheduler.add_task(task).expect("Failed to add task");
    
    // Verify task exists and is in ready state
    let task = scheduler.get_task(task_id).expect("Task not found");
    assert_eq!(task.state, TaskState::Ready);
    assert_eq!(task.priority, TaskPriority::Normal);
    
    // Terminate the task
    scheduler.terminate_task(task_id).expect("Failed to terminate task");
    
    // Verify task is terminated
    let task = scheduler.get_task(task_id).expect("Task not found");
    assert_eq!(task.state, TaskState::Terminated);
}

#[test]
fn test_local_scheduler_priority() {
    let mut scheduler = Scheduler::new();
    scheduler.init();
    
    // Create multiple tasks with different priorities
    let task1 = Task::new(
        TaskId::new(0),
        VirtAddr::new(0x1000),
        VirtAddr::new(0x10000),
        4096,
        PhysAddr::new(0x0),
        TaskPriority::Low,
    );
    
    let task2 = Task::new(
        TaskId::new(0),
        VirtAddr::new(0x2000),
        VirtAddr::new(0x20000),
        4096,
        PhysAddr::new(0x0),
        TaskPriority::High,
    );
    
    let task3 = Task::new(
        TaskId::new(0),
        VirtAddr::new(0x3000),
        VirtAddr::new(0x30000),
        4096,
        PhysAddr::new(0x0),
        TaskPriority::Normal,
    );
    
    let id1 = scheduler.add_task(task1).expect("Failed to add task1");
    let id2 = scheduler.add_task(task2).expect("Failed to add task2");
    let id3 = scheduler.add_task(task3).expect("Failed to add task3");
    
    // Schedule - high priority task should be selected first
    let (_, next, should_switch) = scheduler.schedule();
    assert!(should_switch);
    assert_eq!(next, Some(id2)); // High priority task
    
    // Get the task and verify priority
    let task = scheduler.get_task(id2).expect("Task not found");
    assert_eq!(task.priority, TaskPriority::High);
    
    // Terminate high priority task
    scheduler.terminate_task(id2).expect("Failed to terminate task");
    
    // Next schedule should select normal priority task
    let (_, next, _) = scheduler.schedule();
    assert_eq!(next, Some(id3)); // Normal priority task
    
    // Terminate normal priority task
    scheduler.terminate_task(id3).expect("Failed to terminate task");
    
    // Now only low priority task left
    let (_, next, _) = scheduler.schedule();
    assert_eq!(next, Some(id1)); // Low priority task
}

#[test]
fn test_local_scheduler_round_robin() {
    let mut scheduler = Scheduler::new();
    scheduler.init();
    
    // Create multiple tasks with same priority
    let task1 = Task::new(
        TaskId::new(0),
        VirtAddr::new(0x1000),
        VirtAddr::new(0x10000),
        4096,
        PhysAddr::new(0x0),
        TaskPriority::Normal,
    );
    
    let task2 = Task::new(
        TaskId::new(0),
        VirtAddr::new(0x2000),
        VirtAddr::new(0x20000),
        4096,
        PhysAddr::new(0x0),
        TaskPriority::Normal,
    );
    
    let task3 = Task::new(
        TaskId::new(0),
        VirtAddr::new(0x3000),
        VirtAddr::new(0x30000),
        4096,
        PhysAddr::new(0x0),
        TaskPriority::Normal,
    );
    
    let id1 = scheduler.add_task(task1).expect("Failed to add task1");
    let id2 = scheduler.add_task(task2).expect("Failed to add task2");
    let id3 = scheduler.add_task(task3).expect("Failed to add task3");
    
    // Track scheduling order
    let mut schedule_order = Vec::new();
    
    // Schedule multiple times to see round-robin behavior
    for _ in 0..6 {
        let (_, next, _) = scheduler.schedule();
        if let Some(task_id) = next {
            schedule_order.push(task_id);
        }
    }
    
    // Verify round-robin: should cycle through tasks
    assert_eq!(schedule_order.len(), 6);
    assert_eq!(schedule_order[0], id1);
    assert_eq!(schedule_order[1], id2);
    assert_eq!(schedule_order[2], id3);
    assert_eq!(schedule_order[3], id1); // Back to first task
    assert_eq!(schedule_order[4], id2);
    assert_eq!(schedule_order[5], id3);
}

#[test]
fn test_local_scheduler_state_transitions() {
    let mut scheduler = Scheduler::new();
    scheduler.init();
    
    // Create a task
    let task = Task::new(
        TaskId::new(0),
        VirtAddr::new(0x1000),
        VirtAddr::new(0x10000),
        4096,
        PhysAddr::new(0x0),
        TaskPriority::Normal,
    );
    
    let task_id = scheduler.add_task(task).expect("Failed to add task");
    
    // Initially ready
    let task = scheduler.get_task(task_id).expect("Task not found");
    assert_eq!(task.state, TaskState::Ready);
    
    // Schedule it - should transition to running
    let (_, next, _) = scheduler.schedule();
    assert_eq!(next, Some(task_id));
    let task = scheduler.get_task(task_id).expect("Task not found");
    assert_eq!(task.state, TaskState::Running);
    
    // Block it
    scheduler.block_task(task_id).expect("Failed to block task");
    let task = scheduler.get_task(task_id).expect("Task not found");
    assert_eq!(task.state, TaskState::Blocked);
    
    // Unblock it - should go back to ready
    scheduler.unblock_task(task_id).expect("Failed to unblock task");
    let task = scheduler.get_task(task_id).expect("Task not found");
    assert_eq!(task.state, TaskState::Ready);
    
    // Terminate it
    scheduler.terminate_task(task_id).expect("Failed to terminate task");
    let task = scheduler.get_task(task_id).expect("Task not found");
    assert_eq!(task.state, TaskState::Terminated);
}
