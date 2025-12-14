# FangaOS Architecture

This document describes the high-level architecture and design decisions of the FangaOS kernel.

## Table of Contents

- [Overview](#overview)
- [Design Philosophy](#design-philosophy)
- [System Architecture](#system-architecture)
- [Memory Architecture](#memory-architecture)
- [Process Model](#process-model)
- [Interrupt Handling](#interrupt-handling)
- [I/O Architecture](#io-architecture)
- [Security Model](#security-model)
- [Performance Considerations](#performance-considerations)
- [Future Directions](#future-directions)

---

## Overview

FangaOS is a modern, monolithic kernel written in Rust for the x86_64 architecture. It leverages Rust's memory safety guarantees to build a secure and reliable operating system while maintaining performance.

### Key Characteristics

- **Language**: Rust (no_std, nightly)
- **Architecture**: x86_64 (future: ARM64, RISC-V)
- **Kernel Type**: Monolithic (with modular design)
- **Boot Protocol**: UEFI via Limine bootloader
- **License**: [TBD]

### Design Goals

1. **Safety**: Leverage Rust's type system for memory safety
2. **Performance**: Achieve performance comparable to C kernels
3. **Modularity**: Clean separation of concerns
4. **Portability**: Abstract architecture-specific code
5. **Testability**: Comprehensive test coverage
6. **Documentation**: Well-documented codebase

---

## Design Philosophy

### Rust-First Approach

FangaOS embraces Rust's philosophy:
- **Zero-cost abstractions**: Use high-level constructs without runtime cost
- **Memory safety**: Eliminate undefined behavior through ownership
- **Fearless concurrency**: Safe concurrent programming (future)
- **Explicit over implicit**: Clear code over clever code

### Monolithic with Modularity

While FangaOS is a monolithic kernel (all services in kernel space), it maintains clear module boundaries:
- Loose coupling between subsystems
- Well-defined interfaces
- Testable components
- Potential for microkernel conversion (future research)

### Minimalism

Start simple, add complexity only when needed:
- Implement core functionality first
- Avoid premature optimization
- Prefer simple algorithms that work
- Add advanced features incrementally

---

## System Architecture

### Kernel Crates Structure

```
kernel/
├── crates/
│   ├── fanga-kernel/          # Platform-independent kernel
│   │   ├── memory/            # Memory management
│   │   ├── task/              # Process/thread management
│   │   ├── io/                # I/O subsystems
│   │   └── syscall.rs         # System call handler
│   └── fanga-arch-x86_64/     # x86_64-specific code
│       ├── interrupts/        # IDT, exception handlers
│       ├── gdt.rs             # Global Descriptor Table
│       ├── syscall.rs         # SYSCALL instruction setup
│       └── keyboard.rs        # Keyboard driver
```

### Layered Architecture

```
┌─────────────────────────────────────┐
│        User Space (Ring 3)          │
│  (Shell, Applications, Libraries)   │
└─────────────────────────────────────┘
              ↕ System Calls
┌─────────────────────────────────────┐
│     Kernel Space (Ring 0)           │
│  ┌───────────────────────────────┐  │
│  │   System Call Interface       │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │   Process Management          │  │
│  │   (Scheduler, IPC, Signals)   │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │   Memory Management           │  │
│  │   (PMM, VMM, Heap)            │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │   File Systems (VFS)          │  │
│  │   (FAT32, ext2, InitramFS)    │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │   Device Drivers              │  │
│  │   (Storage, Network, USB)     │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │   I/O Subsystems              │  │
│  │   (Console, Serial, Keyboard) │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │   Interrupt Handling          │  │
│  │   (IDT, IRQ, Exceptions)      │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
              ↕ Hardware Interface
┌─────────────────────────────────────┐
│         Hardware (x86_64)           │
│   (CPU, Memory, Devices, I/O)       │
└─────────────────────────────────────┘
```

### Boot Sequence

1. **Firmware (UEFI)**
   - Initialize hardware
   - Load Limine bootloader

2. **Limine Bootloader**
   - Load kernel ELF binary
   - Set up higher half direct map (HHDM)
   - Provide memory map and framebuffer info
   - Jump to kernel entry point

3. **Kernel Initialization** (`main.rs`)
   ```rust
   fn kernel_main() -> ! {
       // 1. Architecture-specific init
       arch::init();          // GDT, IDT, serial
       
       // 2. Memory subsystem
       memory::init();        // PMM, VMM, heap
       
       // 3. I/O subsystem
       io::init();            // Framebuffer, console
       
       // 4. Interrupt system
       interrupts::init();    // Enable interrupts
       
       // 5. System calls
       syscall::init();       // SYSCALL instruction
       
       // 6. Process management (future)
       task::init();          // Scheduler, init process
       
       // 7. Idle loop or scheduler
       loop { arch::halt(); }
   }
   ```

---

## Memory Architecture

### Address Space Layout

#### Physical Memory
- **0x0000_0000_0000_0000 - 0x0000_0000_0007_FFFF**: Low memory (< 512 KiB)
- **0x0000_0000_0010_0000 - 0x0000_0000_FFFF_FFFF**: Conventional memory
- **Higher addresses**: Extended memory (system-dependent)

#### Virtual Memory (x86_64 Canonical Form)
```
User Space:
0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF  (128 TiB)

Canonical Hole:
0x0000_8000_0000_0000 - 0xFFFF_7FFF_FFFF_FFFF  (Unusable)

Kernel Space:
0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_7FFF_FFFF  (128 TiB)
  ├─ 0xFFFF_8000_0000_0000 - ...  Higher Half Direct Map (HHDM)
  │                                 Physical memory mapped 1:1
  └─ 0xFFFF_FFFF_8000_0000 - ...  Kernel image, heap, stacks
```

### Memory Management Subsystems

#### 1. Physical Memory Manager (PMM)
- **Allocator**: Bitmap-based
- **Granularity**: 4 KiB pages
- **Strategy**: First-fit
- **Data Structure**: Bitmap (1 bit per page)

**Trade-offs:**
- ✅ Simple implementation
- ✅ Low memory overhead (~0.003% of RAM)
- ✅ Fast free operation (O(1))
- ❌ Slower allocation (O(n) worst case)
- 🔄 Future: Buddy allocator for better performance

#### 2. Virtual Memory Manager (VMM)
- **Paging**: 4-level page tables (PML4 → PDPT → PD → PT)
- **Page Size**: 4 KiB (future: 2 MiB huge pages)
- **TLB Management**: Explicit flushes after map/unmap

**Page Table Hierarchy:**
```
Virtual Address (48-bit):
┌─────────┬─────────┬─────────┬─────────┬──────────────┐
│ PML4    │ PDPT    │ PD      │ PT      │ Page Offset  │
│ [47:39] │ [38:30] │ [29:21] │ [20:12] │ [11:0]       │
└─────────┴─────────┴─────────┴─────────┴──────────────┘
   9 bits    9 bits    9 bits    9 bits      12 bits

Each table has 512 entries (9 bits)
Each entry is 8 bytes (64 bits)
```

#### 3. Heap Allocator
- **Algorithm**: Linked-list allocator
- **Features**: Automatic coalescing
- **Interface**: Rust `GlobalAlloc` trait

**Data Structure:**
```rust
struct FreeBlock {
    size: usize,
    next: Option<NonNull<FreeBlock>>,
}
```

---

## Process Model

### Current State (v0.1.0)
- Single-threaded kernel
- No user processes yet
- Task structures defined (TCB, scheduler)

### Planned Design

#### Task Control Block (TCB)
```rust
struct TaskControlBlock {
    tid: TaskId,
    state: TaskState,              // Running, Ready, Blocked, Zombie
    priority: Priority,
    context: Context,              // Saved CPU state
    page_table: PhysAddr,          // CR3 value
    kernel_stack: VirtAddr,
    user_stack: VirtAddr,
    parent: Option<TaskId>,
    children: Vec<TaskId>,
    open_files: Vec<FileDescriptor>,
    // ...
}
```

#### Scheduling
- **Algorithm**: Round-robin with priority levels
- **Preemption**: Timer-based (future)
- **Quantum**: 10ms (configurable)

#### Context Switching
```
1. Save current task's register state
2. Save current task's FPU/SSE state
3. Update task state (Running → Ready)
4. Select next task (scheduler)
5. Switch page table (load CR3)
6. Restore next task's state
7. Jump to next task
```

---

## Interrupt Handling

### Interrupt Architecture

```
Hardware Interrupt → IDT Entry → Interrupt Handler → EOI → Return

Exception → IDT Entry → Exception Handler → Handle/Panic
```

### Interrupt Descriptor Table (IDT)

- **Size**: 256 entries
- **Entry Types**:
  - 0-31: CPU exceptions (divide by zero, page fault, etc.)
  - 32-47: Hardware IRQs (PIC remapped)
  - 48+: Software interrupts, IPIs (future)
  - 0x80: System call (legacy, using SYSCALL now)

### PIC vs APIC

**Current**: Using legacy PIC (8259A)
- Simple to configure
- Sufficient for single-core

**Future**: APIC/x2APIC
- Multi-core support
- Better interrupt routing
- Message-based interrupts

### Exception Handling

Critical exceptions:
- **Page Fault (#PF)**: Handle on-demand paging (future)
- **General Protection (#GP)**: Security violation
- **Double Fault (#DF)**: Unrecoverable error
- **Divide Error (#DE)**: Division by zero

Non-critical exceptions:
- **Breakpoint (#BP)**: Debugging support
- **Invalid Opcode (#UD)**: Emulation potential

---

## I/O Architecture

### Console Abstraction

```
Application
     ↓
console_println!()
     ↓
┌────────────────┐
│   Console      │
│   Abstraction  │
└────┬──────┬────┘
     │      │
     ↓      ↓
┌─────────┐ ┌──────────┐
│ Serial  │ │Framebuffer│
│ Port    │ │  Console  │
└─────────┘ └──────────┘
```

### Framebuffer Management

- **Font**: 8x16 bitmap font (embedded)
- **Features**: Scrolling, colors, special characters
- **Synchronization**: SpinLock
- **Performance**: ~100 cycles per character

### Serial Communication

- **Port**: COM1 (0x3F8)
- **Baud Rate**: 115200
- **Usage**: Debugging, logging
- **Buffering**: None (write-through)

### Keyboard Input

- **Controller**: PS/2 (8042)
- **Scancode Set**: Set 1
- **Layout**: US QWERTY
- **Buffer**: 1024-character ring buffer
- **Future**: USB HID, international layouts

---

## Security Model

### Current Security Features

1. **Memory Safety**
   - Rust ownership prevents use-after-free, double-free
   - Bounds checking on array access
   - Type safety prevents type confusion

2. **Privilege Separation**
   - Kernel runs in Ring 0
   - User code will run in Ring 3 (future)
   - No user code yet

3. **NX Bit**
   - Code pages: executable, not writable
   - Data pages: writable, not executable
   - Stack pages: not executable

### Planned Security Features

1. **ASLR** (Address Space Layout Randomization)
   - Randomize user space layout
   - Randomize heap base
   - Randomize stack location

2. **KASLR** (Kernel ASLR)
   - Randomize kernel base address
   - Requires bootloader support

3. **Stack Canaries**
   - Detect stack buffer overflows
   - Per-thread random canary

4. **Capability-Based Security** (Research)
   - Fine-grained permissions
   - Principle of least privilege

### Threat Model

**In Scope:**
- Malicious user space programs
- Stack/heap buffer overflows
- Use-after-free vulnerabilities
- Privilege escalation attempts

**Out of Scope (Currently):**
- Physical attacks (DMA, cold boot)
- Side-channel attacks (Spectre, Meltdown)
- Cryptographic security (no crypto yet)

---

## Performance Considerations

### Optimization Strategy

1. **Correctness First**: Get it working correctly
2. **Profile**: Measure before optimizing
3. **Optimize Hot Paths**: Focus on critical sections
4. **Leverage Rust**: Trust the optimizer

### Performance Targets

| Operation | Target | Current | Notes |
|-----------|--------|---------|-------|
| Page allocation | < 1 µs | ~2 µs | Bitmap scan |
| Page mapping | < 500 ns | ~800 ns | 4-level walk |
| Context switch | < 5 µs | N/A | Not implemented |
| System call | < 100 ns | ~200 ns | Includes validation |
| Heap allocation | < 200 ns | ~500 ns | Linked list |

### Future Optimizations

- [ ] Buddy allocator for PMM
- [ ] Slab allocator for common objects
- [ ] TLB shootdown batching
- [ ] Lazy TLB flushing
- [ ] Lock-free data structures
- [ ] Per-CPU data structures

---

## Future Directions

### Multi-Core Support

**Challenges:**
- Per-CPU data structures
- Spinlock contention
- TLB shootdown
- Cache coherency

**Approach:**
- Start with coarse-grained locking
- Profile and identify bottlenecks
- Gradually add per-CPU structures
- Consider lock-free algorithms

### Microkernel Exploration

**Research Question**: Can we convert to microkernel?

**Pros:**
- Better isolation
- Easier to debug individual services
- More secure

**Cons:**
- IPC overhead
- Complexity
- Performance impact

**Status**: Research only, not planned for v1.0

### Formal Verification

**Inspiration**: seL4 (formally verified microkernel)

**Potential Targets:**
- Memory allocator correctness
- Page table invariants
- Scheduler fairness properties

**Status**: Long-term research goal

---

## References

### Operating System Theory
- [Operating Systems: Three Easy Pieces](https://pages.cs.wisc.edu/~remzi/OSTEP/)
- [The Design and Implementation of the 4.4BSD Operating System](https://www.freebsd.org/doc/en/books/design-44bsd/)

### x86_64 Architecture
- [Intel® 64 and IA-32 Architectures Software Developer Manuals](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
- [AMD64 Architecture Programmer's Manual](https://www.amd.com/en/support/tech-docs)

### Rust OS Development
- [Writing an OS in Rust](https://os.phil-opp.com/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Redox OS](https://www.redox-os.org/)

### Other Operating Systems
- [Linux Kernel](https://www.kernel.org/)
- [FreeBSD](https://www.freebsd.org/)
- [seL4](https://sel4.systems/)
- [Fuchsia](https://fuchsia.dev/)

---

**Document Version**: 1.0  
**Last Updated**: December 2024  
**Maintainers**: FangaOS Development Team
