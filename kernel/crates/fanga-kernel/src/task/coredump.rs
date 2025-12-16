//! Core Dump Support
//!
//! This module provides process state dumps for debugging:
//! - Capture process state on crash
//! - Dump CPU registers
//! - Dump memory contents
//! - Store core dumps for later analysis

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

use super::tcb::{TaskId, TaskState, TaskPriority};
use super::context::TaskContext;
use super::thread::ThreadId;
use crate::memory::{PhysAddr, VirtAddr};

/// Reason for core dump
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreDumpReason {
    /// Segmentation fault
    SegmentationFault,
    /// Illegal instruction
    IllegalInstruction,
    /// Floating point exception
    FloatingPointException,
    /// Bus error
    BusError,
    /// Abort signal
    Abort,
    /// User-requested dump
    UserRequested,
    /// Other/unknown
    Other,
}

impl CoreDumpReason {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            CoreDumpReason::SegmentationFault => "Segmentation fault",
            CoreDumpReason::IllegalInstruction => "Illegal instruction",
            CoreDumpReason::FloatingPointException => "Floating point exception",
            CoreDumpReason::BusError => "Bus error",
            CoreDumpReason::Abort => "Abort signal",
            CoreDumpReason::UserRequested => "User requested",
            CoreDumpReason::Other => "Unknown",
        }
    }
}

/// CPU register state snapshot
#[derive(Debug, Clone, Copy)]
pub struct RegisterDump {
    /// General purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    
    /// Instruction pointer
    pub rip: u64,
    
    /// Flags register
    pub rflags: u64,
    
    /// Segment registers
    pub cs: u16,
    pub ss: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
    
    /// Control registers
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
}

impl RegisterDump {
    /// Create a register dump from a task context
    pub fn from_context(context: &TaskContext) -> Self {
        Self {
            rax: context.rax,
            rbx: context.rbx,
            rcx: context.rcx,
            rdx: context.rdx,
            rsi: context.rsi,
            rdi: context.rdi,
            rbp: context.rbp,
            rsp: context.rsp,
            r8: context.r8,
            r9: context.r9,
            r10: context.r10,
            r11: context.r11,
            r12: context.r12,
            r13: context.r13,
            r14: context.r14,
            r15: context.r15,
            rip: context.rip,
            rflags: context.rflags,
            cs: context.cs as u16,
            ss: context.ss as u16,
            ds: 0,
            es: 0,
            fs: 0,
            gs: 0,
            cr0: 0,
            cr2: 0,
            cr3: 0,
            cr4: 0,
        }
    }
    
    /// Format register dump as a string
    pub fn format(&self) -> String {
        format!(
            "Registers:\n\
             RAX: 0x{:016x}  RBX: 0x{:016x}  RCX: 0x{:016x}  RDX: 0x{:016x}\n\
             RSI: 0x{:016x}  RDI: 0x{:016x}  RBP: 0x{:016x}  RSP: 0x{:016x}\n\
             R8:  0x{:016x}  R9:  0x{:016x}  R10: 0x{:016x}  R11: 0x{:016x}\n\
             R12: 0x{:016x}  R13: 0x{:016x}  R14: 0x{:016x}  R15: 0x{:016x}\n\
             RIP: 0x{:016x}  RFLAGS: 0x{:016x}\n\
             CS: 0x{:04x}  SS: 0x{:04x}",
            self.rax, self.rbx, self.rcx, self.rdx,
            self.rsi, self.rdi, self.rbp, self.rsp,
            self.r8, self.r9, self.r10, self.r11,
            self.r12, self.r13, self.r14, self.r15,
            self.rip, self.rflags,
            self.cs, self.ss
        )
    }
}

/// Memory region dump
#[derive(Debug, Clone)]
pub struct MemoryDump {
    /// Start address
    pub start_addr: VirtAddr,
    
    /// Size in bytes
    pub size: usize,
    
    /// Memory contents (limited to avoid excessive memory use)
    pub data: Vec<u8>,
}

impl MemoryDump {
    /// Create a new memory dump
    pub fn new(start_addr: VirtAddr, size: usize) -> Self {
        Self {
            start_addr,
            size,
            data: Vec::new(),
        }
    }
    
    /// Add data to the dump
    pub fn add_data(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }
}

/// Thread information in core dump
#[derive(Debug, Clone)]
pub struct ThreadInfo {
    /// Thread ID
    pub thread_id: ThreadId,
    
    /// Thread name
    pub name: String,
    
    /// Thread state
    pub state: TaskState,
    
    /// Thread priority
    pub priority: TaskPriority,
    
    /// Register state
    pub registers: RegisterDump,
    
    /// Stack pointer
    pub stack_ptr: VirtAddr,
    
    /// Stack size
    pub stack_size: usize,
}

/// Core dump structure
#[derive(Debug, Clone)]
pub struct CoreDump {
    /// Process ID
    pub process_id: TaskId,
    
    /// Process name
    pub process_name: String,
    
    /// Reason for dump
    pub reason: CoreDumpReason,
    
    /// Timestamp (system ticks)
    pub timestamp: u64,
    
    /// Main thread/process register state
    pub registers: RegisterDump,
    
    /// Thread information (for multi-threaded processes)
    pub threads: Vec<ThreadInfo>,
    
    /// Memory dumps
    pub memory_regions: Vec<MemoryDump>,
    
    /// Stack dump
    pub stack_dump: Option<MemoryDump>,
    
    /// Page table address
    pub page_table: PhysAddr,
    
    /// Exit code
    pub exit_code: i32,
}

impl CoreDump {
    /// Create a new core dump
    pub fn new(
        process_id: TaskId,
        process_name: String,
        reason: CoreDumpReason,
        timestamp: u64,
        registers: RegisterDump,
        page_table: PhysAddr,
        exit_code: i32,
    ) -> Self {
        Self {
            process_id,
            process_name,
            reason,
            timestamp,
            registers,
            threads: Vec::new(),
            memory_regions: Vec::new(),
            stack_dump: None,
            page_table,
            exit_code,
        }
    }
    
    /// Add thread information
    pub fn add_thread(&mut self, thread: ThreadInfo) {
        self.threads.push(thread);
    }
    
    /// Add memory region
    pub fn add_memory_region(&mut self, region: MemoryDump) {
        self.memory_regions.push(region);
    }
    
    /// Set stack dump
    pub fn set_stack_dump(&mut self, dump: MemoryDump) {
        self.stack_dump = Some(dump);
    }
    
    /// Format core dump as a readable string
    pub fn format(&self) -> String {
        let mut output = String::new();
        
        output.push_str("=== CORE DUMP ===\n");
        output.push_str(&format!("Process ID: {}\n", self.process_id.as_usize()));
        output.push_str(&format!("Process Name: {}\n", self.process_name));
        output.push_str(&format!("Reason: {}\n", self.reason.description()));
        output.push_str(&format!("Timestamp: {} ticks\n", self.timestamp));
        output.push_str(&format!("Exit Code: {}\n", self.exit_code));
        output.push_str(&format!("Page Table: 0x{:016x}\n", self.page_table.as_u64()));
        output.push_str("\n");
        
        output.push_str(&self.registers.format());
        output.push_str("\n\n");
        
        if !self.threads.is_empty() {
            output.push_str(&format!("Threads: {}\n", self.threads.len()));
            for (idx, thread) in self.threads.iter().enumerate() {
                output.push_str(&format!(
                    "  Thread {}: {} (ID: {}, State: {:?}, Priority: {:?})\n",
                    idx, thread.name, thread.thread_id.as_usize(), thread.state, thread.priority
                ));
            }
            output.push_str("\n");
        }
        
        if let Some(ref stack) = self.stack_dump {
            output.push_str(&format!(
                "Stack: 0x{:016x} - 0x{:016x} ({} bytes)\n",
                stack.start_addr.as_u64(),
                stack.start_addr.as_u64() + stack.size as u64,
                stack.size
            ));
        }
        
        if !self.memory_regions.is_empty() {
            output.push_str(&format!("\nMemory Regions: {}\n", self.memory_regions.len()));
            for (idx, region) in self.memory_regions.iter().enumerate() {
                output.push_str(&format!(
                    "  Region {}: 0x{:016x} - 0x{:016x} ({} bytes)\n",
                    idx,
                    region.start_addr.as_u64(),
                    region.start_addr.as_u64() + region.size as u64,
                    region.size
                ));
            }
        }
        
        output.push_str("\n=== END CORE DUMP ===\n");
        output
    }
    
    /// Get the size of the core dump in bytes (estimate)
    pub fn size(&self) -> usize {
        let mut size = core::mem::size_of::<Self>();
        
        // Add thread info
        size += self.threads.len() * core::mem::size_of::<ThreadInfo>();
        
        // Add memory regions
        for region in &self.memory_regions {
            size += region.data.len();
        }
        
        // Add stack dump
        if let Some(ref stack) = self.stack_dump {
            size += stack.data.len();
        }
        
        size
    }
}

/// Core dump manager
pub struct CoreDumpManager {
    /// Collection of core dumps
    dumps: Vec<CoreDump>,
    
    /// Maximum number of core dumps to keep
    max_dumps: usize,
    
    /// Whether core dumps are enabled
    enabled: bool,
}

impl CoreDumpManager {
    /// Create a new core dump manager
    pub fn new(max_dumps: usize) -> Self {
        Self {
            dumps: Vec::new(),
            max_dumps,
            enabled: true,
        }
    }
    
    /// Enable core dumps
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// Disable core dumps
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// Check if core dumps are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Add a core dump
    pub fn add_dump(&mut self, dump: CoreDump) {
        if !self.enabled {
            return;
        }
        
        // Remove oldest dump if we've reached the limit
        if self.dumps.len() >= self.max_dumps {
            self.dumps.remove(0);
        }
        
        self.dumps.push(dump);
    }
    
    /// Get all core dumps
    pub fn get_dumps(&self) -> &[CoreDump] {
        &self.dumps
    }
    
    /// Get a specific core dump by index
    pub fn get_dump(&self, index: usize) -> Option<&CoreDump> {
        self.dumps.get(index)
    }
    
    /// Get the most recent core dump
    pub fn get_latest(&self) -> Option<&CoreDump> {
        self.dumps.last()
    }
    
    /// Clear all core dumps
    pub fn clear(&mut self) {
        self.dumps.clear();
    }
    
    /// Get the number of core dumps
    pub fn count(&self) -> usize {
        self.dumps.len()
    }
    
    /// Get total size of all core dumps
    pub fn total_size(&self) -> usize {
        self.dumps.iter().map(|d| d.size()).sum()
    }
}

impl Default for CoreDumpManager {
    fn default() -> Self {
        Self::new(10) // Keep up to 10 core dumps by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_dump_reason() {
        assert_eq!(CoreDumpReason::SegmentationFault.description(), "Segmentation fault");
        assert_eq!(CoreDumpReason::Abort.description(), "Abort signal");
    }

    #[test]
    fn test_register_dump() {
        let context = TaskContext::new(0x1000, 0x2000);
        let regs = RegisterDump::from_context(&context);
        
        assert_eq!(regs.rip, 0x1000);
        assert_eq!(regs.rsp, 0x2000);
        
        let formatted = regs.format();
        assert!(formatted.contains("RIP: 0x0000000000001000"));
    }

    #[test]
    fn test_memory_dump() {
        let mut dump = MemoryDump::new(VirtAddr::new(0x1000), 256);
        let data = vec![0x90, 0xCC, 0xC3]; // nop, int3, ret
        dump.add_data(&data);
        
        assert_eq!(dump.start_addr, VirtAddr::new(0x1000));
        assert_eq!(dump.size, 256);
        assert_eq!(dump.data.len(), 3);
    }

    #[test]
    fn test_core_dump() {
        let context = TaskContext::new(0x1000, 0x2000);
        let regs = RegisterDump::from_context(&context);
        
        let mut dump = CoreDump::new(
            TaskId::new(42),
            String::from("test_process"),
            CoreDumpReason::SegmentationFault,
            1000,
            regs,
            PhysAddr::new(0x3000),
            -11,
        );
        
        assert_eq!(dump.process_id, TaskId::new(42));
        assert_eq!(dump.process_name, "test_process");
        assert_eq!(dump.reason, CoreDumpReason::SegmentationFault);
        assert_eq!(dump.exit_code, -11);
        
        // Add thread
        let thread = ThreadInfo {
            thread_id: ThreadId::new(1),
            name: String::from("worker"),
            state: TaskState::Running,
            priority: TaskPriority::Normal,
            registers: regs,
            stack_ptr: VirtAddr::new(0x2000),
            stack_size: 8192,
        };
        dump.add_thread(thread);
        assert_eq!(dump.threads.len(), 1);
        
        // Format
        let formatted = dump.format();
        assert!(formatted.contains("CORE DUMP"));
        assert!(formatted.contains("test_process"));
        assert!(formatted.contains("Segmentation fault"));
    }

    #[test]
    fn test_core_dump_manager() {
        let mut manager = CoreDumpManager::new(3);
        assert!(manager.is_enabled());
        assert_eq!(manager.count(), 0);
        
        // Add dumps
        for i in 0..5 {
            let context = TaskContext::new(0x1000, 0x2000);
            let regs = RegisterDump::from_context(&context);
            let dump = CoreDump::new(
                TaskId::new(i),
                format!("process_{}", i),
                CoreDumpReason::Abort,
                i as u64,
                regs,
                PhysAddr::new(0x3000),
                0,
            );
            manager.add_dump(dump);
        }
        
        // Should only keep the last 3
        assert_eq!(manager.count(), 3);
        
        let latest = manager.get_latest().unwrap();
        assert_eq!(latest.process_id, TaskId::new(4));
        
        manager.clear();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_core_dump_disable() {
        let mut manager = CoreDumpManager::new(10);
        manager.disable();
        assert!(!manager.is_enabled());
        
        let context = TaskContext::new(0x1000, 0x2000);
        let regs = RegisterDump::from_context(&context);
        let dump = CoreDump::new(
            TaskId::new(1),
            String::from("test"),
            CoreDumpReason::Abort,
            0,
            regs,
            PhysAddr::new(0x3000),
            0,
        );
        manager.add_dump(dump);
        
        assert_eq!(manager.count(), 0); // Should not be added
    }
}
