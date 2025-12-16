# Boot Sequence Documentation

## Overview

FangaOS implements a structured, phased boot sequence that initializes the kernel in a clean and maintainable manner. This document describes the boot process from the moment the bootloader hands control to the kernel until the system is fully operational.

## Architecture

The boot process is organized into distinct phases, each with specific responsibilities and dependencies. This modular approach provides:

- **Clarity**: Each phase has a clear purpose
- **Maintainability**: Easy to modify individual phases
- **Debuggability**: Phases can be tested and validated independently
- **Reliability**: Proper initialization order prevents subtle bugs

## Boot Phases

```text
┌─────────────────────────────────────────────────────────┐
│                     _start (entry point)                 │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│            Phase 1: Early Boot                           │
│  - Architecture initialization (GDT, IDT)                │
│  - Serial port for debugging                             │
│  - Enable interrupts                                     │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│       Phase 2: Bootloader Protocol                       │
│  - Verify Limine support                                 │
│  - Parse bootloader information                          │
│  - Process memory map                                    │
│  - Get HHDM offset                                       │
│  - Initialize framebuffer (early console)                │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│       Phase 3: Memory Initialization                     │
│  - Physical Memory Manager (PMM)                         │
│  - Heap allocator                                        │
│  - Virtual Memory Manager (VMM)                          │
│  - Memory regions                                        │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│       Phase 4: Driver Initialization                     │
│  - Keyboard driver                                       │
│  - Timer (PIT/APIC)                                      │
│  - Other essential drivers                               │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│      Phase 5: Subsystem Initialization                   │
│  - Task scheduler                                        │
│  - Process management                                    │
│  - Power management                                      │
│  - Shell/REPL                                            │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│         Phase 6: Post-Initialization                     │
│  - Run demonstrations                                    │
│  - Display welcome message                               │
│  - System ready                                          │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
              ┌────────────┐
              │ Idle Loop  │
              │ (hlt loop) │
              └────────────┘
```

## Detailed Phase Descriptions

### Phase 1: Early Boot

**Purpose**: Initialize the bare minimum required to get the system running.

**Operations**:
1. Call `arch::init()` which:
   - Initializes the serial port for debugging output
   - Sets up the Global Descriptor Table (GDT)
   - Configures the Interrupt Descriptor Table (IDT)
   - Initializes the interrupt controller (APIC or PIC)
   - Enables interrupts with `sti`

**Dependencies**: None (this is the first code that runs)

**Outputs**: Serial debugging available, interrupts enabled

**Location**: `kernel/crates/fanga-kernel/src/boot.rs::phase1_early_boot()`

---

### Phase 2: Bootloader Protocol

**Purpose**: Process information provided by the Limine bootloader.

**Operations**:
1. Verify Limine base revision is supported
2. Parse bootloader name and version
3. Get memory map from bootloader
4. Obtain HHDM (Higher Half Direct Map) offset
5. Initialize framebuffer console if available
6. Log memory statistics

**Dependencies**: Phase 1 (serial output)

**Outputs**: `BootloaderContext` containing:
- Memory map
- HHDM offset
- Framebuffer information

**Location**: `kernel/crates/fanga-kernel/src/boot.rs::phase2_bootloader_protocol()`

---

### Phase 3: Memory Initialization

**Purpose**: Set up all memory management subsystems.

**Operations**:
1. **Physical Memory Manager (PMM)**
   - Parse memory map
   - Initialize bitmap allocator
   - Track free/used pages
   
2. **Heap Allocator**
   - Allocate physical pages for heap
   - Map to virtual memory via HHDM
   - Initialize global allocator
   - Enable `alloc` crate (Vec, Box, etc.)

3. **Virtual Memory Manager (VMM)**
   - Test page table operations
   - Verify mapping/unmapping works

4. **Memory Regions**
   - Categorize memory (Available, Reserved, Kernel)
   - Build region database

**Dependencies**: Phase 2 (bootloader context)

**Outputs**: 
- Working memory allocation
- Heap allocator ready
- Memory statistics

**Location**: `kernel/crates/fanga-kernel/src/boot.rs::phase3_memory_init()`

**Safety**: Uses `unsafe` due to static mutable memory managers

---

### Phase 4: Driver Initialization

**Purpose**: Initialize essential hardware drivers.

**Operations**:
1. Keyboard driver (PS/2)
2. Timer already initialized in arch init
3. Log driver status

**Dependencies**: Phase 3 (heap allocator for driver data structures)

**Outputs**: Working keyboard input, timer interrupts

**Location**: `kernel/crates/fanga-kernel/src/boot.rs::phase4_driver_init()`

---

### Phase 5: Subsystem Initialization

**Purpose**: Initialize high-level kernel subsystems.

**Operations**:
1. **Shell System**
   - Initialize shell state
   - Setup command history
   - Initialize line editor

2. **Task Scheduler**
   - Initialize scheduler
   - Setup process management
   - Enable preemptive multitasking
   - Configure timer-based scheduling

3. **Power Management**
   - Initialize power states
   - Setup CPU frequency scaling
   - Enable power management features

**Dependencies**: Phases 3-4 (memory and drivers)

**Outputs**: 
- Interactive shell ready
- Task scheduler operational
- Power management active

**Location**: `kernel/crates/fanga-kernel/src/boot.rs::phase5_subsystem_init()`

---

### Phase 6: Post-Initialization

**Purpose**: Run demonstrations and display welcome message.

**Operations**:
1. Create and schedule demo tasks
2. Display process management demonstration
3. Show welcome message and feature list
4. Display shell prompt

**Dependencies**: Phase 5 (all subsystems ready)

**Outputs**: 
- System ready for user interaction
- Welcome message displayed

**Location**: `kernel/crates/fanga-kernel/src/boot.rs::phase6_post_init()`

---

## Entry Point

### `_start()` Function

The `_start()` function in `main.rs` is the kernel entry point called by the bootloader.

**Responsibilities**:
1. Call `boot::initialize()` with bootloader requests
2. Handle initialization success/failure
3. Enter idle loop (CPU halts waiting for interrupts)

**Code Structure**:
```rust
#[no_mangle]
pub extern "C" fn _start() -> ! {
    match boot::initialize(...) {
        Ok(()) => {
            // Boot successful
        }
        Err(e) => {
            // Boot failed - log error
        }
    }
    
    // Idle loop
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
```

**Location**: `kernel/crates/fanga-kernel/src/main.rs::_start()`

---

## Boot Module API

The `boot` module (`kernel/crates/fanga-kernel/src/boot.rs`) provides:

### Main Initialization Function

```rust
pub fn initialize(
    framebuffer_req: &FramebufferRequest,
    bootloader_info_req: &BootloaderInfoRequest,
    memmap_req: &MemoryMapRequest,
    hhdm_req: &HhdmRequest,
    base_revision: &limine::BaseRevision,
) -> Result<(), &'static str>
```

Orchestrates the entire boot sequence.

### Individual Phase Functions

- `phase1_early_boot()` - Architecture initialization
- `phase2_bootloader_protocol()` - Bootloader info processing
- `phase3_memory_init()` - Memory subsystems
- `phase4_driver_init()` - Hardware drivers
- `phase5_subsystem_init()` - Kernel subsystems
- `phase6_post_init()` - Demonstrations and welcome

### Data Structures

```rust
pub struct BootloaderContext {
    pub memory_map: &'static limine::response::MemoryMapResponse,
    pub hhdm_offset: u64,
    pub framebuffer: Option<&'static limine::response::FramebufferResponse>,
}
```

Contains critical bootloader-provided information.

---

## Error Handling

Each boot phase can fail, and errors are propagated up to `_start()`:

1. **Phase 2 failure**: Missing memory map or HHDM
   - System cannot continue without memory information
   - Error logged and system halts

2. **Phase 3 failure**: Memory allocation failure
   - Panic if heap cannot be initialized
   - Critical failure, cannot continue

3. **Other phases**: Generally don't fail fatally
   - May log warnings and continue with reduced functionality

---

## Debugging the Boot Process

### Serial Output

All phases log their progress via serial output:
- Connect to COM1 (serial port)
- View boot progress in real-time
- Format: `[Boot Phase N] Message`

### Boot Failure Diagnosis

If boot fails:
1. Check serial output for last successful phase
2. Look for error messages
3. Verify QEMU/hardware configuration
4. Check bootloader (Limine) version compatibility

### Common Issues

**Issue**: "Limine base revision not supported"
- **Cause**: Bootloader version incompatible
- **Fix**: Update Limine bootloader

**Issue**: Boot hangs after Phase 1
- **Cause**: Interrupt configuration problem
- **Fix**: Check IDT and interrupt controller init

**Issue**: "Failed to allocate heap memory"
- **Cause**: No usable memory pages available
- **Fix**: Check memory map, increase QEMU RAM

---

## Performance Considerations

### Boot Time

The structured boot approach adds minimal overhead:
- Phase transitions are function calls (near-zero cost)
- Logging can be disabled in release builds
- Each phase executes in microseconds to milliseconds

### Memory Usage

Boot phase organization doesn't increase memory usage:
- No additional data structures for phase management
- All memory allocated is for actual kernel use
- Logging strings are compile-time constants

---

## Future Enhancements

Potential improvements to the boot sequence:

1. **Parallel Initialization**: Some phases could run concurrently
2. **Boot Profiles**: Different init sequences for different targets
3. **Early Boot Services**: Logging buffer available before serial init
4. **Boot Parameters**: Parse kernel command line arguments
5. **Recovery Mode**: Alternative boot path for debugging
6. **Boot Splash**: Display graphical boot progress

---

## Related Documentation

- [Memory Management](MEMORY_MANAGEMENT.md) - Details on PMM, VMM, heap
- [Interrupt Handling](INTERRUPT_HANDLING.md) - IDT, PIC, APIC setup
- [Timer Management](TIMER_MANAGEMENT.md) - PIT initialization
- [Input/Output](INPUT_OUTPUT.md) - Framebuffer and keyboard drivers
- [Process Management](PROCESS_MANAGEMENT.md) - Task scheduler

---

## Code Organization

```
kernel/crates/fanga-kernel/src/
├── main.rs                  # Entry point (_start)
├── boot.rs                  # Boot sequence implementation
├── memory/
│   ├── pmm.rs              # Physical memory (Phase 3)
│   ├── heap.rs             # Heap allocator (Phase 3)
│   └── vmm.rs              # Virtual memory (Phase 3)
├── io/
│   ├── framebuffer.rs      # Console output (Phase 2/4)
│   └── keyboard_bridge.rs  # Keyboard input (Phase 4)
├── task/
│   ├── scheduler.rs        # Task scheduler (Phase 5)
│   └── process.rs          # Process management (Phase 5)
├── power/
│   └── mod.rs              # Power management (Phase 5)
└── shell/
    └── mod.rs              # Interactive shell (Phase 5)

kernel/crates/fanga-arch-x86_64/src/
└── lib.rs                  # arch::init() (Phase 1)
    ├── gdt.rs              # GDT setup
    ├── interrupts/
    │   ├── idt.rs          # IDT setup
    │   ├── apic.rs         # APIC init
    │   └── pic.rs          # PIC fallback
    └── serial.rs           # Serial output
```

---

## Summary

FangaOS uses a **clean, phased boot sequence** that:

✅ **Separates concerns**: Each phase has a specific purpose  
✅ **Ensures proper ordering**: Dependencies are explicit  
✅ **Improves maintainability**: Easy to understand and modify  
✅ **Enables debugging**: Clear logging of each step  
✅ **Handles errors**: Graceful failure with informative messages  

The boot sequence provides a solid foundation for the operating system, ensuring all components are initialized in the correct order before the system becomes operational.
