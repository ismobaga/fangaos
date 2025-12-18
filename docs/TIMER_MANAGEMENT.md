# Timer and Time Management

This document describes the timer and time management subsystem in FangaOS.

## Overview

FangaOS implements a comprehensive timing system based on the Intel 8253/8254 Programmable Interval Timer (PIT). The timing system provides:

- Hardware timer configuration and management
- System uptime tracking
- Time-based delays (busy-wait and sleep)
- Preemptive multitasking through timer interrupts
- Scheduler integration for time-slice based task switching

## Architecture

### Components

#### 1. PIT (Programmable Interval Timer)
**Location**: `kernel/crates/fanga-arch-x86_64/src/interrupts/pit.rs`

The PIT module provides low-level control of the Intel 8253/8254 timer chip:

- **Base Frequency**: 1.193182 MHz (1193182 Hz)
- **Default Configuration**: 100 Hz (10ms per tick)
- **Mode**: Square wave generator (Mode 3)
- **Channel**: Channel 0 (system timer)

**Key Functions**:
- `init(frequency)` - Configure the PIT to a specific frequency
- `read_counter()` - Read the current counter value
- `calculate_divisor(frequency)` - Calculate divisor for a given frequency

#### 2. Timer IRQ Handler
**Location**: `kernel/crates/fanga-arch-x86_64/src/interrupts/idt.rs`

The timer IRQ handler (IRQ0) is invoked on each timer tick:

- Increments the global tick counter
- Invokes registered callback (if any)
- Sends End of Interrupt (EOI) signal to the PIC

**Key Functions**:
- `set_timer_callback(callback)` - Register a function to be called on each tick
- `timer_ticks()` - Get the current tick count
- `uptime_ms()` - Get system uptime in milliseconds
- `uptime_secs()` - Get system uptime in seconds

#### 3. Timer Bridge
**Location**: `kernel/crates/fanga-kernel/src/task/timer_bridge.rs`

The timer bridge connects the arch-specific timer interrupt to the kernel scheduler:

- Registers the scheduler callback with the arch layer
- Enables preemptive multitasking

**Flow**:
```
Timer IRQ → timer_irq_handler() → timer_callback() → schedule_on_timer() → scheduler
```

#### 4. Time Management API
**Location**: `kernel/crates/fanga-kernel/src/task/time.rs`

High-level time management functions for kernel and user code:

**Delay Functions** (busy-wait):
- `delay_ms(ms)` - Busy-wait for specified milliseconds
- `delay_us(us)` - Busy-wait for specified microseconds

**Sleep Functions** (blocking):
- `sleep_ms(ms)` - Sleep for specified milliseconds (yields CPU)
- `block_for_duration(task_id, ms)` - Block a task for a duration

**Uptime Functions**:
- `uptime_ms()` - System uptime in milliseconds
- `uptime_secs()` - System uptime in seconds
- `timer_ticks()` - Raw tick counter value

#### 5. Scheduler Timer
**Location**: `kernel/crates/fanga-kernel/src/task/sched_timer.rs`

Implements preemptive scheduling logic:

- **Time Slice**: 10 ticks = 100ms per task (with 100 Hz timer)
- `schedule_on_timer()` - Called on each timer tick to potentially trigger context switch
- Tracks ticks per task and triggers scheduler when time slice expires

## Configuration

### Timer Frequency

The default timer frequency is **100 Hz** (10ms per tick), configured via:
```rust
const PIT_DEFAULT_FREQ: u32 = 100;
```

This provides a good balance between:
- Timer resolution (10ms)
- Interrupt overhead
- Scheduling responsiveness

### Time Slice

Tasks are scheduled in **100ms time slices** (10 ticks):
```rust
pub const TIME_SLICE: u64 = 10;  // 10 ticks × 10ms = 100ms
```

## Usage Examples

### Getting System Uptime

```rust
use fanga_kernel::task::time;

// Get uptime in different units
let ticks = time::timer_ticks();
let ms = time::uptime_ms();
let secs = time::uptime_secs();

println!("System has been running for {} seconds", secs);
```

### Delay/Busy-Wait

```rust
use fanga_kernel::task::time;

// Wait 100ms (busy loop)
time::delay_ms(100);

// Wait 50 microseconds
time::delay_us(50);
```

### Sleep (Yielding)

```rust
use fanga_kernel::task::time;

// Sleep for 1 second (yields CPU to other tasks)
time::sleep_ms(1000);
```

### Block Task for Duration

```rust
use fanga_kernel::task::{time, TaskId};

let task_id = TaskId::new(1);

// Block task for 500ms
time::block_for_duration(task_id, 500)?;
```

## Initialization

Timer initialization happens during kernel boot in this order:

1. **PIC Remap and IDT Setup** (`arch::interrupts::idt::init()`)
   - Remaps PIC to avoid conflicts with CPU exceptions
   - Sets up Interrupt Descriptor Table (IDT)
   
2. **PIT Configuration** (`arch::interrupts::pit::init(100)`)
   - Configures PIT to 100 Hz
   - Enables timer interrupts (IRQ0)
   
3. **Scheduler Initialization** (`task::scheduler::init()`)
   - Initializes task scheduler data structures
   
4. **Timer Bridge Setup** (`task::timer_bridge::init()`)
   - Registers scheduler callback with timer IRQ handler
   - Enables preemptive multitasking

## Preemptive Multitasking

The timer system enables preemptive multitasking through the following mechanism:

1. PIT generates interrupt every 10ms (IRQ0)
2. CPU jumps to `timer_irq_handler()`
3. Handler increments tick counter
4. Handler invokes registered callback (`timer_callback()`)
5. Callback calls `schedule_on_timer()`
6. `schedule_on_timer()` tracks ticks per task
7. After TIME_SLICE ticks (100ms), scheduler is invoked
8. Scheduler selects next task based on priority
9. Context switch occurs (when implemented)

### Current Status

**Implemented**:
- ✅ PIT timer configuration (100 Hz)
- ✅ Timer interrupt handling
- ✅ Tick counter and uptime tracking
- ✅ Delay and sleep functions
- ✅ Timer-to-scheduler callback bridge
- ✅ Schedule decision logic (which task to run next)

**TODO** (future work):
- ⏳ Actual context switching (save/restore CPU state)
- ⏳ Sleep queue (wake tasks after specific time)
- ⏳ High-resolution timers (for sub-millisecond precision)
- ⏳ APIC timer support (for modern systems)

## Technical Details

### PIT Programming

The PIT is programmed using I/O ports:
- **0x40**: Channel 0 data port
- **0x43**: Command/mode register

**Divisor Calculation**:
```
divisor = PIT_BASE_FREQ / desired_frequency
divisor = 1193182 / 100 = 11931
```

**Command Byte** (0x36):
- Bits 7-6: Channel 0
- Bits 5-4: Access mode (low/high byte)
- Bits 3-1: Mode 3 (square wave)
- Bit 0: Binary mode

### Interrupt Flow

```
Timer Hardware (PIT)
  ↓
IRQ0 (PIC)
  ↓
Vector 32 (PIC1_OFFSET + IRQ_TIMER)
  ↓
IDT Entry 32
  ↓
timer_irq_handler()
  ↓
TIMER_CALLBACK (if registered)
  ↓
timer_callback()
  ↓
schedule_on_timer()
  ↓
scheduler::schedule()
```

### Atomic Operations

The timer tick counter uses atomic operations to ensure thread-safety:
```rust
static TIMER_TICKS: u64  // Protected by interrupt disable
static TICK_COUNTER: AtomicU64  // Atomic for scheduler
```

## Performance Considerations

### Timer Frequency Trade-offs

**Higher Frequency (e.g., 1000 Hz)**:
- ✅ Better time resolution
- ✅ More responsive scheduling
- ❌ More interrupt overhead
- ❌ Higher CPU usage

**Lower Frequency (e.g., 18 Hz)**:
- ✅ Less interrupt overhead
- ✅ Lower CPU usage
- ❌ Worse time resolution
- ❌ Less responsive scheduling

**Current Choice (100 Hz)**:
- Good balance for general-purpose OS
- 10ms resolution sufficient for most tasks
- ~1% CPU overhead on modern processors

### Busy-Wait vs Sleep

**Busy-Wait (`delay_ms`)**:
- Spins in a loop checking time
- Wastes CPU cycles
- Use for: Short delays (<10ms), interrupt context

**Sleep (`sleep_ms`)**:
- Blocks task and yields to scheduler
- Efficient CPU usage
- Use for: Long delays (>10ms), user tasks

## References

- Intel 8253/8254 PIT Datasheet
- OSDev Wiki: Programmable Interval Timer
- Intel 64 and IA-32 Architectures Software Developer's Manual
- OSDev Wiki: Interrupts
