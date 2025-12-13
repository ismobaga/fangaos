//! Context Switching
//!
//! This module provides low-level context switching functionality for x86_64.
//! It handles saving and restoring CPU state when switching between tasks.

use core::arch::asm;

/// Perform a context switch from one task to another
///
/// # Arguments
/// * `old_context` - Pointer to save the old task's context
/// * `new_context` - Pointer to load the new task's context
///
/// # Safety
/// This function is unsafe because it:
/// - Modifies CPU registers
/// - Changes the stack pointer
/// - Assumes the contexts are valid and properly aligned
#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(old_context: *mut TaskContext, new_context: *const TaskContext) {
    core::arch::naked_asm!(
        // Save old context (callee-saved registers + rip/rsp)
        // rdi = old_context, rsi = new_context
        
        // Save general purpose registers
        "mov [rdi + 0x00], rax",     // rax
        "mov [rdi + 0x08], rbx",     // rbx
        "mov [rdi + 0x10], rcx",     // rcx
        "mov [rdi + 0x18], rdx",     // rdx
        "mov [rdi + 0x20], rsi",     // rsi
        "mov [rdi + 0x28], rdi",     // rdi
        "mov [rdi + 0x30], rbp",     // rbp
        "mov [rdi + 0x38], r8",      // r8
        "mov [rdi + 0x40], r9",      // r9
        "mov [rdi + 0x48], r10",     // r10
        "mov [rdi + 0x50], r11",     // r11
        "mov [rdi + 0x58], r12",     // r12
        "mov [rdi + 0x60], r13",     // r13
        "mov [rdi + 0x68], r14",     // r14
        "mov [rdi + 0x70], r15",     // r15
        
        // Save return address as rip
        "mov rax, [rsp]",
        "mov [rdi + 0x78], rax",     // rip
        
        // Save stack pointer (after return address)
        "lea rax, [rsp + 8]",
        "mov [rdi + 0x80], rax",     // rsp
        
        // Save rflags
        "pushfq",
        "pop rax",
        "mov [rdi + 0x88], rax",     // rflags
        
        // Save segment selectors (cs and ss)
        "mov ax, cs",
        "mov [rdi + 0x90], rax",     // cs
        "mov ax, ss",
        "mov [rdi + 0x98], rax",     // ss
        
        // Load new context
        // Restore general purpose registers
        "mov rax, [rsi + 0x00]",     // rax
        "mov rbx, [rsi + 0x08]",     // rbx
        "mov rcx, [rsi + 0x10]",     // rcx
        "mov rdx, [rsi + 0x18]",     // rdx
        // Skip rsi for now, we need it
        "mov rbp, [rsi + 0x30]",     // rbp
        "mov r8,  [rsi + 0x38]",     // r8
        "mov r9,  [rsi + 0x40]",     // r9
        "mov r10, [rsi + 0x48]",     // r10
        "mov r11, [rsi + 0x50]",     // r11
        "mov r12, [rsi + 0x58]",     // r12
        "mov r13, [rsi + 0x60]",     // r13
        "mov r14, [rsi + 0x68]",     // r14
        "mov r15, [rsi + 0x70]",     // r15
        
        // Restore stack pointer
        "mov rsp, [rsi + 0x80]",     // rsp
        
        // Push rip for ret instruction
        "push [rsi + 0x78]",         // rip
        
        // Restore rflags
        "push [rsi + 0x88]",         // rflags
        "popfq",
        
        // Restore rdi and rsi last
        "mov rdi, [rsi + 0x28]",     // rdi
        "mov rsi, [rsi + 0x20]",     // rsi (restore last)
        
        // Jump to new task
        "ret",
    );
}

/// TaskContext structure must match the layout expected by switch_context
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    pub rax: u64,      // 0x00
    pub rbx: u64,      // 0x08
    pub rcx: u64,      // 0x10
    pub rdx: u64,      // 0x18
    pub rsi: u64,      // 0x20
    pub rdi: u64,      // 0x28
    pub rbp: u64,      // 0x30
    pub r8: u64,       // 0x38
    pub r9: u64,       // 0x40
    pub r10: u64,      // 0x48
    pub r11: u64,      // 0x50
    pub r12: u64,      // 0x58
    pub r13: u64,      // 0x60
    pub r14: u64,      // 0x68
    pub r15: u64,      // 0x70
    pub rip: u64,      // 0x78
    pub rsp: u64,      // 0x80
    pub rflags: u64,   // 0x88
    pub cs: u64,       // 0x90
    pub ss: u64,       // 0x98
}

impl TaskContext {
    /// Create a new task context
    pub const fn new() -> Self {
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
            rflags: 0x202, // IF (interrupt flag) set
            cs: 0x08,      // Kernel code segment
            ss: 0x10,      // Kernel data segment
        }
    }
    
    /// Initialize context for a new task
    pub fn init(&mut self, entry_point: u64, stack_pointer: u64) {
        self.rip = entry_point;
        self.rsp = stack_pointer;
        self.rflags = 0x202; // IF set
        self.cs = 0x08;      // Kernel code segment
        self.ss = 0x10;      // Kernel data segment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = TaskContext::new();
        assert_eq!(ctx.rip, 0);
        assert_eq!(ctx.rsp, 0);
        assert_eq!(ctx.rflags, 0x202);
        assert_eq!(ctx.cs, 0x08);
        assert_eq!(ctx.ss, 0x10);
    }

    #[test]
    fn test_context_init() {
        let mut ctx = TaskContext::new();
        ctx.init(0x1000, 0x2000);
        assert_eq!(ctx.rip, 0x1000);
        assert_eq!(ctx.rsp, 0x2000);
    }
}
