//! Task Context
//!
//! This module defines the CPU context structure for saving and restoring
//! task state during context switches on x86_64 architecture.

/// Task context - contains saved CPU state
/// 
/// This structure stores all the general-purpose registers and special registers
/// needed to save and restore a task's execution state.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    // General purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    
    // Special registers
    /// Instruction pointer
    pub rip: u64,
    /// Stack pointer
    pub rsp: u64,
    /// RFLAGS register
    pub rflags: u64,
    
    // Segment selectors
    pub cs: u64,
    pub ss: u64,
}

impl TaskContext {
    /// Create a new task context with the given entry point and stack pointer
    pub fn new(entry_point: u64, stack_pointer: u64) -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: entry_point,
            rsp: stack_pointer,
            rflags: 0x202, // IF (interrupt flag) set
            cs: 0x08,  // Kernel code segment
            ss: 0x10,  // Kernel data segment
        }
    }
    
    /// Create a zeroed context
    pub const fn zero() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rsp: 0,
            rflags: 0,
            cs: 0,
            ss: 0,
        }
    }
}

impl Default for TaskContext {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = TaskContext::new(0x1000, 0x2000);
        assert_eq!(ctx.rip, 0x1000);
        assert_eq!(ctx.rsp, 0x2000);
        assert_eq!(ctx.rflags, 0x202);
        assert_eq!(ctx.cs, 0x08);
        assert_eq!(ctx.ss, 0x10);
    }

    #[test]
    fn test_context_zero() {
        let ctx = TaskContext::zero();
        assert_eq!(ctx.rip, 0);
        assert_eq!(ctx.rsp, 0);
        assert_eq!(ctx.rflags, 0);
    }

    #[test]
    fn test_context_default() {
        let ctx = TaskContext::default();
        assert_eq!(ctx.rip, 0);
        assert_eq!(ctx.rsp, 0);
    }
}
