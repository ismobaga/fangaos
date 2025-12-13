# FangaOS Architecture Documentation

This document describes the high-level architecture and design decisions of FangaOS.

## Table of Contents

- [Overview](#overview)
- [Design Principles](#design-principles)
- [System Architecture](#system-architecture)
- [Memory Layout](#memory-layout)
- [Boot Process](#boot-process)
- [Module Organization](#module-organization)
- [Future Architecture](#future-architecture)

## Overview

FangaOS is a modern operating system kernel written in Rust, designed with safety, modularity, and multi-architecture support in mind. The kernel follows a monolithic design with modular components that can be extended or replaced independently.

## Design Principles

### 1. Safety First
- Leverage Rust's type system and ownership model
- Minimize unsafe code, document all unsafe blocks
- Use RAII patterns for resource management
- Validate all external inputs and hardware data

### 2. Modularity
- Clear separation between architecture-specific and portable code
- Well-defined interfaces between kernel subsystems
- Modular driver architecture
- Plugin system for optional features (future)

### 3. Performance
- Zero-cost abstractions where possible
- Efficient data structures and algorithms
- Lock-free designs where appropriate
- SIMD optimizations for critical paths (future)

### 4. Multi-Architecture
- Common kernel core shared across architectures
- Architecture-specific implementations in separate crates
- Traits define common interfaces
- Conditional compilation for platform features

### 5. Maintainability
- Clear code organization and naming
- Comprehensive documentation
- Consistent coding style
- Extensive testing (in progress)

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     User Space                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │  Shell   │  │   Apps   │  │ Services │  (Future)   │
│  └──────────┘  └──────────┘  └──────────┘             │
└─────────────────────────────────────────────────────────┘
                       │ System Calls
┌─────────────────────────────────────────────────────────┐
│                    Kernel Space                         │
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │           Kernel Core (fanga-kernel)           │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐    │    │
│  │  │   VFS    │  │Scheduler │  │   IPC    │    │    │
│  │  └──────────┘  └──────────┘  └──────────┘    │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐    │    │
│  │  │MemMgmt   │  │ Drivers  │  │ Network  │    │    │
│  │  └──────────┘  └──────────┘  └──────────┘    │    │
│  └────────────────────────────────────────────────┘    │
│                       │                                 │
│  ┌────────────────────────────────────────────────┐    │
│  │    Architecture Layer (fanga-arch-x86_64)      │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐    │    │
│  │  │Interrupts│  │  MMU     │  │  Ports   │    │    │
│  │  └──────────┘  └──────────┘  └──────────┘    │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐    │    │
│  │  │  Timers  │  │   SMP    │  │  ACPI    │    │    │
│  │  └──────────┘  └──────────┘  └──────────┘    │    │
│  └────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
                       │
┌─────────────────────────────────────────────────────────┐
│                     Hardware                            │
│    CPU  │  Memory  │  Devices  │  Storage  │  Network  │
└─────────────────────────────────────────────────────────┘
```

## Memory Layout

### Virtual Memory Map (x86_64)

```
0xFFFFFFFF_FFFFFFFF ┌────────────────────────┐
                    │   Kernel Stack         │
                    │   (grows down)         │
0xFFFFFFFF_80000000 ├────────────────────────┤
                    │   Kernel Code/Data     │
                    │   (2GB)                │
0xFFFFFFFF_00000000 ├────────────────────────┤
                    │   HHDM (Physical       │
                    │   Memory Direct Map)   │
                    │   (varies)             │
0xFFFF8000_00000000 ├────────────────────────┤
                    │   Canonical Hole       │
                    │   (Non-canonical       │
                    │    addresses)          │
0x00007FFF_FFFFFFFF ├────────────────────────┤
                    │   User Space           │
                    │   (128TB)              │
                    │                        │
0x00000000_00001000 ├────────────────────────┤
                    │   Null Page            │
0x00000000_00000000 └────────────────────────┘
```

### Physical Memory Regions

- **Kernel**: Loaded by bootloader at physical address
- **HHDM**: Physical memory mapped at fixed offset
- **MMIO**: Memory-mapped I/O devices
- **Framebuffer**: Graphics memory
- **Free Memory**: Available for allocation

## Boot Process

### 1. Pre-Boot (Firmware)
```
UEFI/BIOS → Limine Bootloader
```

### 2. Bootloader Phase
```
Limine:
  - Load kernel ELF binary
  - Set up initial page tables
  - Prepare boot information
  - Jump to kernel entry (_start)
```

### 3. Kernel Initialization
```rust
_start() {
    1. Architecture init (arch::init)
       - Initialize serial port
       - Set up IDT
       - Configure PIC
    
    2. Validate Limine protocol
    
    3. Parse boot information
       - Bootloader info
       - Memory map
       - HHDM offset
       - Framebuffer
    
    4. Initialize subsystems (future)
       - Physical memory allocator
       - Virtual memory manager
       - Heap allocator
       - Device drivers
    
    5. Start scheduler (future)
    
    6. Jump to init process (future)
}
```

### 4. Current State
```
→ Framebuffer test (fills screen)
→ Idle loop (HLT instruction)
```

## Module Organization

### Kernel Workspace Structure

```
kernel/
├── Cargo.toml                 # Workspace manifest
├── crates/
│   ├── fanga-kernel/          # Main kernel
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs        # Entry point, init logic
│   │
│   └── fanga-arch-x86_64/     # x86_64 architecture
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs         # Architecture init
│           ├── interrupts/    # Interrupt handling
│           │   ├── mod.rs
│           │   ├── idt.rs     # IDT setup
│           │   └── pic.rs     # PIC configuration
│           ├── serial.rs      # Serial driver
│           └── port.rs        # I/O port access
```

### Future Structure

```
kernel/crates/
├── fanga-kernel/              # Core kernel
├── fanga-arch-common/         # Shared architecture traits
├── fanga-arch-x86_64/         # x86_64 implementation
├── fanga-arch-aarch64/        # ARM64 implementation
├── fanga-arch-riscv64/        # RISC-V implementation
├── fanga-mm/                  # Memory management
├── fanga-fs/                  # Filesystem
├── fanga-drivers/             # Device drivers
├── fanga-net/                 # Networking
└── fanga-user/                # Userspace support
```

## Subsystem Details

### Interrupt Handling (Current)

```rust
Interrupts:
  IDT (Interrupt Descriptor Table)
    → 256 entries
    → Handlers for exceptions
    → IRQ routing through PIC
  
  PIC (Programmable Interrupt Controller)
    → Master/Slave configuration
    → IRQ 0-15 mapping
    → Mask management
```

### Serial I/O (Current)

```rust
Serial Port (COM1):
  - Base address: 0x3F8
  - Baud rate: 115200
  - Data bits: 8
  - Parity: None
  - Stop bits: 1
  - Atomic locking for concurrent access
```

### Memory Management (Planned)

```rust
Physical Allocator:
  - Bitmap or buddy allocator
  - 4KB page granularity
  - Free list management
  
Virtual Memory:
  - 4-level page tables (x86_64)
  - Kernel space: Higher half
  - User space: Lower half
  - Demand paging
  - Copy-on-write
  
Heap:
  - Dynamic kernel allocator
  - GlobalAlloc implementation
  - Segregated free lists
```

### Process Management (Planned)

```rust
Scheduler:
  - Preemptive multitasking
  - Round-robin or CFS
  - Per-CPU run queues
  - Priority levels
  
Context Switch:
  - Save/restore registers
  - FPU/SIMD state
  - Page table switch
  - TLS management
```

## Design Patterns

### Error Handling

```rust
// Use Result for recoverable errors
pub fn allocate_page() -> Result<PhysAddr, AllocError> {
    // ...
}

// Panic only for unrecoverable situations
if !BASE_REVISION.is_supported() {
    panic!("Limine base revision not supported");
}
```

### Resource Management

```rust
// Use RAII for automatic cleanup
struct Guard<'a> {
    lock: &'a AtomicBool,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}
```

### Hardware Access

```rust
// Volatile for MMIO
let framebuffer = fb.addr() as *mut u32;
unsafe {
    ptr.write_volatile(value);
}

// Port I/O abstraction
pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value);
}
```

## Future Architecture

### Microkernel Consideration

FangaOS currently uses a monolithic design, but could evolve toward a microkernel:

**Potential User Space Services:**
- File system servers
- Network stack
- Device drivers (non-critical)
- Window manager

**Remaining in Kernel:**
- Scheduler
- IPC (critical for performance)
- Memory management
- Hardware abstraction

**Benefits:**
- Better isolation
- Easier debugging
- More modular
- Enhanced security

**Challenges:**
- IPC overhead
- Complexity
- Performance concerns

### Modular Driver Framework

```rust
pub trait Driver {
    fn init(&mut self) -> Result<(), DriverError>;
    fn probe(&self) -> bool;
    fn remove(&mut self);
}

// Register drivers dynamically
register_driver!(MyDriver);
```

### Capability-Based Security

```rust
pub struct Capability {
    resource: ResourceId,
    permissions: Permissions,
}

// Processes hold capabilities, not direct access
process.check_capability(&cap)?;
```

## References

- [OSDev Wiki](https://wiki.osdev.org/)
- [Intel Software Developer Manuals](https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html)
- [Rust Embedded Book](https://rust-embedded.github.io/book/)
- [Writing an OS in Rust](https://os.phil-opp.com/)

---

**Last Updated:** December 2024  
**Maintainer:** FangaOS Team
