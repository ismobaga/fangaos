# Process Management Implementation

## Overview

This document describes the complete process management system implemented in FangaOS.

## Components Implemented

### 1. Task Control Block (TCB)
**Location:** `kernel/crates/fanga-kernel/src/task/tcb.rs`

The Task Control Block contains all metadata for each process:
- **TaskId**: Unique process identifier
- **TaskState**: Process state (Ready, Running, Blocked, Terminated)
- **TaskPriority**: Four priority levels (Low, Normal, High, Critical)
- **TaskContext**: Saved CPU registers for context switching
- **Kernel Stack**: Stack address and size
- **Page Table**: Physical address of process's page table (CR3)
- **Name**: Human-readable process name (32 bytes)

### 2. Process Scheduler
**Location:** `kernel/crates/fanga-kernel/src/task/scheduler.rs`

Implements a hybrid scheduling algorithm:
- **Priority-based**: Tasks with higher priority are selected first
- **Round-robin**: Tasks of same priority are scheduled in round-robin fashion
- **Ready Queues**: Separate queue for each priority level
- **Capacity**: Supports up to 256 concurrent processes (MAX_TASKS)

Key methods:
- `add_task()`: Add a new task to the scheduler
- `schedule()`: Select the next task to run
- `terminate_task()`: Mark a task as terminated
- `block_task()` / `unblock_task()`: Manage task blocking

### 3. Context Switching
**Location:** `kernel/crates/fanga-arch-x86_64/src/context.rs`

Full x86_64 context save/restore implementation:
- Saves all general-purpose registers (rax-r15)
- Saves instruction pointer (rip)
- Saves stack pointer (rsp)
- Saves RFLAGS register
- Saves segment selectors (cs, ss)

The `switch_context()` function is a naked assembly function that performs the low-level context switch.

### 4. Process Management
**Location:** `kernel/crates/fanga-kernel/src/task/process.rs`

High-level process lifecycle management:
- **ProcessManager**: Manages process creation and termination
- `create_process()`: Create a new process with specified parameters
- `fork_process()`: Duplicate an existing process (returns child PID to parent, 0 to child)
- `exit_process()`: Terminate a process with exit code

### 5. System Calls
**Locations:**
- `kernel/crates/fanga-arch-x86_64/src/syscall.rs` (low-level)
- `kernel/crates/fanga-kernel/src/syscall_handlers.rs` (high-level)

Implemented syscalls:
- **fork()**: Create child process (SYS_FORK = 57)
- **exit()**: Terminate current process (SYS_EXIT = 60)
- **read()**: Read from file descriptor (SYS_READ = 0)
- **write()**: Write to file descriptor (SYS_WRITE = 1)

### 6. Timer-based Preemptive Scheduling
**Location:** `kernel/crates/fanga-kernel/src/task/sched_timer.rs`

Provides timer interrupt-based task switching:
- Configurable time slice (default: 1 timer tick)
- Integrates with timer interrupt handler
- Calls scheduler to select next task
- Can trigger context switch

### 7. Inter-Process Communication (IPC)
**Location:** `kernel/crates/fanga-kernel/src/task/ipc.rs`

Implements multiple IPC mechanisms:
- **Message Queues**: Send/receive messages between processes
- **Semaphores**: P/V operations for synchronization
- **Mutexes**: Mutual exclusion locks

### 8. Example Tasks
**Location:** `kernel/crates/fanga-kernel/src/task/examples.rs`

Provides example task functions for demonstration:
- `task1()`: Counter task
- `task2()`: Computation task
- `task3()`: Background task
- `idle_task()`: Idle task (lowest priority)

## Process States

The system implements four process states with proper transitions:

```
┌──────────┐  create     ┌─────────┐
│   Init   │────────────>│  Ready  │
└──────────┘             └─────────┘
                              │  schedule()
                              v
             terminate() ┌─────────┐ schedule()
         ┌──────────────│ Running │────────┐
         │               └─────────┘        │
         │                    │  block()    │
         v                    v             v
    ┌──────────┐         ┌─────────┐  ┌─────────┐
    │Terminated│         │ Blocked │  │  Ready  │
    └──────────┘         └─────────┘  └─────────┘
                              │  unblock()    ^
                              └───────────────┘
```

## Multi-tasking Demonstration

The kernel demonstrates multi-tasking with 2-3 concurrent processes:
- Creates tasks with different priorities
- Shows priority-based scheduling
- Demonstrates state transitions
- Shows process termination

See `kernel/crates/fanga-kernel/src/main.rs` for the full demonstration code.

## Testing

Comprehensive test suite in `kernel/crates/fanga-kernel/tests/process_integration.rs`:
- Process creation and termination
- Priority-based scheduling
- Round-robin scheduling
- State transitions
- Scheduler operations

All tests pass successfully.

## Known Limitations

1. **Context Switching**: Assembly implementation exists but is not fully integrated with the scheduler due to complexity of switching in kernel mode
2. **Memory Management**: Each process currently shares the same page table; full address space isolation not yet implemented
3. **Stack Allocation**: Kernel stacks are allocated at fixed addresses for demo purposes
4. **Timer Integration**: Preemptive scheduling hook exists but not connected to actual timer interrupt

## Future Enhancements

1. Implement full context switching in timer interrupt
2. Add per-process page tables with copy-on-write fork()
3. Implement dynamic kernel stack allocation
4. Add process wait() and waitpid() syscalls
5. Implement exec() with ELF loader
6. Add process signals and signal handlers
7. Implement process groups and sessions

## References

- Intel® 64 and IA-32 Architectures Software Developer's Manual
- OSDev Wiki: Scheduling, Context Switching
- Linux Kernel Documentation
