# Scalability & Performance Features

## Overview

FangaOS has been enhanced with advanced scalability and performance features to support multi-core systems, NUMA architectures, and comprehensive performance monitoring. This document describes the implementation and usage of these features.

## Table of Contents

1. [SMP (Symmetric Multi-Processing) Support](#smp-support)
2. [CPU Affinity](#cpu-affinity)
3. [NUMA (Non-Uniform Memory Access) Awareness](#numa-awareness)
4. [Performance Profiling](#performance-profiling)
5. [Kernel Preemption](#kernel-preemption)

---

## SMP Support

### Overview

The SMP subsystem provides multi-core CPU support, enabling FangaOS to utilize multiple processors simultaneously for improved performance and scalability.

### Features

- **CPU Detection and Enumeration**: Automatic detection of all available CPUs via ACPI MADT
- **Per-CPU Data Structures**: Dedicated data structures for each CPU to avoid contention
- **Application Processor (AP) Startup**: Framework for initializing secondary CPUs
- **Inter-Processor Interrupts (IPI)**: Communication mechanism between CPUs
- **TLB Shootdown**: Coordinate TLB invalidation across multiple CPUs
- **SMP-safe Spinlocks**: Synchronization primitives for multi-core environments

### Module Structure

```
kernel/crates/fanga-kernel/src/smp/
├── mod.rs          - Main SMP module
├── cpu.rs          - CPU detection and management
├── percpu.rs       - Per-CPU data structures
├── ipi.rs          - Inter-Processor Interrupts
├── spinlock.rs     - SMP-safe spinlocks
└── acpi.rs         - ACPI parsing for CPU info
```

### Usage

#### Initialization

```rust
use fangaos::smp;

// Initialize SMP subsystem
smp::init()?;

// Start application processors
smp::start_application_processors()?;
```

#### CPU Information

```rust
use fangaos::smp::{cpu_count, current_cpu_id, cpu_manager};

// Get number of CPUs
let num_cpus = cpu_count();

// Get current CPU ID
let cpu = current_cpu_id();

// Access CPU manager
let manager = cpu_manager().lock();
let cpu_info = manager.get_cpu(cpu).unwrap();
println!("Running on CPU {}, APIC ID: {}", 
    cpu_info.id.as_usize(), cpu_info.apic_id);
```

#### Per-CPU Data

```rust
use fangaos::smp::percpu::{current_cpu_data, get_cpu_data};

// Get current CPU's data
let cpu_data = current_cpu_data();
cpu_data.total_ticks += 1;

// Check if preemptible
if cpu_data.preemptible() {
    // Can be preempted
}
```

#### Spinlocks

```rust
use fangaos::smp::SpinLock;

static DATA: SpinLock<u32> = SpinLock::new(0);

{
    let mut guard = DATA.lock();
    *guard += 1;
} // Lock automatically released
```

#### Inter-Processor Interrupts

```rust
use fangaos::smp::ipi::{Ipi, IpiType, IpiTarget, send_ipi};

// Send reschedule IPI to CPU 1
let ipi = Ipi::new(IpiType::Reschedule, IpiTarget::Cpu(CpuId::new(1)));
send_ipi(ipi)?;

// Broadcast TLB flush to all CPUs
let ipi = Ipi::with_data(
    IpiType::TlbFlush, 
    IpiTarget::AllExceptSelf, 
    0x1000
);
send_ipi(ipi)?;
```

### API Reference

#### CpuManager

- `detect_cpus()` - Detect and enumerate all CPUs
- `cpu_count()` - Get number of CPUs
- `online_count()` - Get number of online CPUs
- `get_cpu(id)` - Get CPU information
- `add_cpu(apic_id)` - Add a CPU to the manager
- `bring_cpu_online(id)` - Bring a CPU online

#### PerCpuData

- `in_interrupt()` - Check if in interrupt context
- `preemptible()` - Check if preemption is enabled

---

## CPU Affinity

### Overview

CPU affinity allows binding processes and threads to specific CPUs or sets of CPUs, improving cache locality and providing deterministic scheduling.

### Features

- **Single CPU Affinity**: Bind thread to a specific CPU
- **Affinity Masks**: Bind thread to multiple CPUs
- **Dynamic Affinity**: Change affinity at runtime
- **Scheduler Integration**: Affinity-aware scheduling

### Usage

#### Setting CPU Affinity

```rust
use fangaos::task::{ThreadAttributes, ThreadManager};

// Create thread with affinity to CPU 0
let attrs = ThreadAttributes::default()
    .with_cpu_affinity(0);

let thread_id = thread_manager.create_thread(
    process_id,
    "my_thread".into(),
    entry_point,
    stack,
    attrs
)?;

// Set affinity to multiple CPUs (mask)
let attrs = ThreadAttributes::default()
    .with_affinity_mask(0b1111); // CPUs 0-3

// Check if thread can run on a CPU
if attrs.can_run_on_cpu(2) {
    // Thread can run on CPU 2
}
```

#### Runtime Affinity Changes

```rust
// Get thread and update affinity
let mut thread = thread_manager.get_thread_mut(thread_id).unwrap();
thread.set_cpu_affinity(1);  // Pin to CPU 1
thread.clear_cpu_affinity(); // Clear affinity
```

### API Reference

#### ThreadAttributes

- `with_cpu_affinity(cpu)` - Set affinity to specific CPU
- `with_affinity_mask(mask)` - Set affinity mask
- `can_run_on_cpu(cpu_id)` - Check if can run on CPU

#### Thread

- `set_cpu_affinity(cpu)` - Set CPU affinity
- `clear_cpu_affinity()` - Clear CPU affinity
- `cpu_affinity()` - Get CPU affinity

---

## NUMA Awareness

### Overview

NUMA (Non-Uniform Memory Access) awareness optimizes memory allocation by preferring memory that is local to the CPU, reducing memory access latency.

### Features

- **NUMA Topology Detection**: Automatic detection via ACPI SRAT/SLIT
- **NUMA Nodes**: Organize CPUs and memory into nodes
- **NUMA-aware Allocation**: Allocate memory from specific nodes
- **Memory Policies**: Configure allocation strategies
- **Distance Metrics**: Track inter-node latencies

### Module Structure

```
kernel/crates/fanga-kernel/src/numa/
├── mod.rs          - Main NUMA module
├── topology.rs     - NUMA topology detection
├── allocator.rs    - NUMA-aware allocator
└── policy.rs       - Memory allocation policies
```

### Usage

#### Initialization

```rust
use fangaos::numa;

// Initialize NUMA subsystem
numa::init()?;
```

#### NUMA Topology

```rust
use fangaos::numa::{numa_topology, NumaNodeId};

let topology = numa_topology().lock();

// Check if NUMA is enabled
if topology.is_enabled() {
    println!("NUMA system with {} nodes", topology.node_count());
}

// Get node for CPU
let node_id = topology.node_for_cpu(CpuId::new(0)).unwrap();

// Get node information
let node = topology.get_node(node_id).unwrap();
println!("Node {} has {} CPUs", node.id.as_usize(), node.cpus.len());
println!("Memory: {} GB at 0x{:x}", 
    node.mem_size / (1024*1024*1024), node.mem_base);

// Get distance to another node
let distance = node.distance_to(NumaNodeId::new(1));
println!("Distance to node 1: {}", distance);
```

#### NUMA-aware Allocation

```rust
use fangaos::numa::{NumaAllocator, AllocHint, NumaNodeId};

let allocator = NumaAllocator::new();

// Allocate from any node
let addr = allocator.alloc(4096, AllocHint::Any)?;

// Prefer specific node
let addr = allocator.alloc(
    4096, 
    AllocHint::PreferNode(NumaNodeId::new(0))
)?;

// Strict allocation (only from specific node)
let addr = allocator.alloc(
    4096, 
    AllocHint::StrictNode(NumaNodeId::new(1))
)?;

// Allocate from local node
let addr = allocator.alloc(4096, AllocHint::Local)?;

// Free memory
allocator.free(addr, 4096)?;
```

#### Memory Policies

```rust
use fangaos::numa::{NumaMemoryPolicy, NumaPolicy, NumaNodeId};

// Default policy (any node)
let policy = NumaMemoryPolicy::default();

// Bind to specific nodes
let policy = NumaMemoryPolicy::bind(&[
    NumaNodeId::new(0), 
    NumaNodeId::new(2)
]);

// Interleave across nodes
let policy = NumaMemoryPolicy::interleave(&[
    NumaNodeId::new(0), 
    NumaNodeId::new(1)
]);

// Prefer specific node
let policy = NumaMemoryPolicy::preferred(NumaNodeId::new(0));

// Check if node is allowed
if policy.is_node_allowed(NumaNodeId::new(0)) {
    // Can allocate from node 0
}
```

### API Reference

#### NumaTopology

- `detect()` - Detect NUMA topology
- `node_count()` - Get number of nodes
- `is_enabled()` - Check if NUMA is enabled
- `get_node(id)` - Get node information
- `node_for_cpu(cpu_id)` - Find node for CPU
- `closest_node(from)` - Find closest node

#### NumaAllocator

- `alloc(size, hint)` - Allocate memory with NUMA hint
- `free(addr, size)` - Free allocated memory

#### NumaMemoryPolicy

- `default()` - Default policy
- `bind(nodes)` - Bind to specific nodes
- `interleave(nodes)` - Interleave across nodes
- `preferred(node)` - Prefer specific node
- `is_node_allowed(node)` - Check if node is allowed

---

## Performance Profiling

### Overview

The profiling subsystem provides built-in performance analysis tools for identifying bottlenecks and optimizing kernel and application code.

### Features

- **Sampling-based Profiler**: Low-overhead statistical profiling
- **Performance Counters**: Hardware PMU integration
- **Call Stack Sampling**: Capture execution context
- **Multiple Output Formats**: Text, JSON, CSV
- **Hotspot Analysis**: Identify most time-consuming functions

### Module Structure

```
kernel/crates/fanga-kernel/src/profiling/
├── mod.rs          - Main profiling module
├── sampler.rs      - Sampling-based profiler
├── pmu.rs          - Performance counter support
├── stats.rs        - Statistics collection
└── output.rs       - Output formatting
```

### Usage

#### Initialization

```rust
use fangaos::profiling;

// Initialize profiling subsystem
profiling::init()?;
```

#### Basic Profiling

```rust
use fangaos::profiling::{start_profiling, stop_profiling, get_stats};

// Start profiling
start_profiling()?;

// ... code to profile ...

// Stop profiling
stop_profiling()?;

// Get statistics
let stats = get_stats();
println!("Total samples: {}", stats.total_samples);
println!("Kernel: {:.2}%", stats.kernel_percentage());
println!("User: {:.2}%", stats.user_percentage());
```

#### Advanced Configuration

```rust
use fangaos::profiling::{profiler, SamplingConfig};

let mut prof = profiler().lock();

// Configure sampling
let config = SamplingConfig {
    interval_us: 1000,      // 1ms sampling interval
    max_samples: 10000,     // Max 10k samples
    include_kernel: true,   // Include kernel samples
    include_user: true,     // Include user samples
};
prof.configure(config);

prof.start()?;
// ... profiling ...
prof.stop()?;

// Get top hotspots
let hotspots = prof.top_hotspots(10);
for (rip, count) in hotspots {
    println!("0x{:016x}: {} samples", rip, count);
}
```

#### Performance Counters

```rust
use fangaos::profiling::{PerformanceCounter, CounterType};

// Create performance counter
let mut counter = PerformanceCounter::new(CounterType::Cycles);

// Enable counter
counter.enable()?;

// ... code to measure ...

// Read counter
let cycles = counter.read();
println!("Cycles: {}", cycles);

// Disable counter
counter.disable()?;
```

#### Output Generation

```rust
use fangaos::profiling::{ProfileOutput, OutputFormat, get_stats};

let stats = get_stats();

// Text output
let output = ProfileOutput::new(stats.clone(), OutputFormat::Text);
println!("{}", output.generate());

// JSON output
let output = ProfileOutput::new(stats.clone(), OutputFormat::Json);
let json = output.generate();

// CSV output
let output = ProfileOutput::new(stats, OutputFormat::Csv);
let csv = output.generate();
```

### API Reference

#### Profiler

- `start()` - Start profiling
- `stop()` - Stop profiling
- `pause()` - Pause profiling
- `resume()` - Resume profiling
- `record_sample(sample)` - Record a sample
- `get_stats()` - Get statistics
- `top_hotspots(n)` - Get top N hotspots

#### PerformanceCounter

- `enable()` - Enable counter
- `disable()` - Disable counter
- `read()` - Read counter value
- `reset()` - Reset counter

#### ProfileStats

- `total_samples` - Total sample count
- `kernel_samples` - Kernel sample count
- `user_samples` - User sample count
- `kernel_percentage()` - Kernel time percentage
- `user_percentage()` - User time percentage
- `top_functions(n)` - Get top N functions

---

## Kernel Preemption

### Overview

Kernel preemption allows the scheduler to interrupt and reschedule kernel code, improving responsiveness and enabling better real-time performance.

### Features

- **Preemption Counter**: Track preemption disable depth
- **Preemption Points**: Strategic reschedule locations
- **RAII Guards**: Automatic preemption management
- **Statistics**: Track voluntary/involuntary preemptions
- **Latency Tracking**: Monitor maximum preemption latency

### Module Structure

```
kernel/crates/fanga-kernel/src/preempt/
├── mod.rs          - Main preemption module
├── counter.rs      - Preemption counter management
├── points.rs       - Preemption points
└── stats.rs        - Preemption statistics
```

### Usage

#### Initialization

```rust
use fangaos::preempt;

// Initialize preemption subsystem
preempt::init();
```

#### Disable/Enable Preemption

```rust
use fangaos::preempt::{preempt_disable, preempt_enable};

// Manually disable preemption
preempt_disable();

// Critical section here

// Re-enable preemption
preempt_enable(); // May trigger reschedule
```

#### Using RAII Guards

```rust
use fangaos::preempt::counter::PreemptGuard;

{
    let _guard = PreemptGuard::new();
    // Preemption disabled in this scope
    
    // ... critical section ...
    
} // Preemption automatically re-enabled
```

#### Preemption Points

```rust
use fangaos::preempt::preempt_check;

// Manual preemption check
preempt_check();

// Using macro
preempt_point!();
```

#### Checking Preemption State

```rust
use fangaos::preempt::{preemptible, need_resched};

if preemptible() {
    // Can be preempted
}

if need_resched() {
    // Should reschedule
}
```

#### Statistics

```rust
use fangaos::preempt::{get_preemption_stats, stats};

let stats = get_preemption_stats();
println!("Total preemptions: {}", stats.total_preemptions);
println!("Voluntary: {}", stats.voluntary_preemptions);
println!("Involuntary: {}", stats.involuntary_preemptions);
println!("Max latency: {} μs", stats.max_latency_us);

// Record preemption
stats::record_voluntary_preempt();
stats::record_involuntary_preempt();
stats::record_latency(50); // 50 μs
```

### API Reference

#### Preemption Control

- `preempt_disable()` - Disable preemption
- `preempt_enable()` - Enable preemption
- `preempt_count()` - Get preemption count
- `preemptible()` - Check if preemptible

#### Preemption Points

- `preempt_check()` - Check and reschedule if needed
- `should_reschedule()` - Check if reschedule needed
- `set_need_resched()` - Set reschedule flag
- `clear_need_resched()` - Clear reschedule flag

#### Statistics

- `get_preemption_stats()` - Get statistics
- `record_voluntary_preempt()` - Record voluntary preemption
- `record_involuntary_preempt()` - Record involuntary preemption
- `record_latency(us)` - Record latency
- `reset_stats()` - Reset statistics

---

## Integration Example

Here's a complete example showing how these features work together:

```rust
use fangaos::{smp, numa, profiling, preempt, task};

fn main() {
    // Initialize all subsystems
    smp::init().unwrap();
    numa::init().unwrap();
    profiling::init().unwrap();
    preempt::init();
    
    // Create a thread with CPU affinity and NUMA awareness
    let cpu_id = 2;
    let numa_node = numa::numa_topology().lock()
        .node_for_cpu(smp::CpuId::new(cpu_id))
        .unwrap();
    
    let attrs = task::ThreadAttributes::default()
        .with_cpu_affinity(cpu_id);
    
    let thread_id = create_thread_with_attrs(attrs);
    
    // Start profiling
    profiling::start_profiling().unwrap();
    
    // Run workload
    {
        let _guard = preempt::counter::PreemptGuard::new();
        
        // Allocate memory from local NUMA node
        let allocator = numa::NumaAllocator::new();
        let memory = allocator.alloc(
            4096,
            numa::AllocHint::StrictNode(numa_node)
        ).unwrap();
        
        // Do work with preemption disabled
        do_critical_work(memory);
        
        allocator.free(memory, 4096).unwrap();
    }
    
    // Stop profiling and get results
    profiling::stop_profiling().unwrap();
    let stats = profiling::get_stats();
    
    // Generate report
    let output = profiling::ProfileOutput::new(
        stats,
        profiling::OutputFormat::Text
    );
    println!("{}", output.generate());
    
    // Check preemption statistics
    let preempt_stats = preempt::get_preemption_stats();
    println!("Preemptions: {}", preempt_stats.total_preemptions);
    println!("Max latency: {} μs", preempt_stats.max_latency_us);
}
```

---

## Performance Considerations

### SMP

- **Lock Contention**: Use per-CPU data structures to minimize locking
- **Cache Effects**: Consider false sharing when designing data structures
- **IPI Overhead**: Minimize IPI usage as it has significant overhead

### NUMA

- **Memory Locality**: Allocate memory from local nodes when possible
- **Migration Cost**: Consider NUMA topology when migrating threads
- **Interleaving**: Use interleave policy for shared data

### Profiling

- **Sampling Rate**: Higher rates give more accuracy but increase overhead
- **Overhead**: Profiling has ~1-5% overhead depending on sampling rate
- **Sample Storage**: Limit max_samples to avoid memory exhaustion

### Preemption

- **Critical Sections**: Keep preemption-disabled sections short
- **Latency**: Long disabled sections increase scheduling latency
- **Nesting**: Avoid deep nesting of preemption disable/enable

---

## Testing

All modules include comprehensive unit tests. Run with:

```bash
cd kernel/crates/fanga-kernel
cargo test --lib --target x86_64-unknown-linux-gnu
```

Specific test modules:
- `smp::*::tests` - SMP tests
- `numa::*::tests` - NUMA tests
- `profiling::*::tests` - Profiling tests
- `preempt::*::tests` - Preemption tests

---

## Future Enhancements

### SMP
- Complete ACPI MADT parsing
- Full AP startup sequence
- Hardware APIC programming
- Load balancing across CPUs

### NUMA
- Complete SRAT/SLIT parsing
- Dynamic migration based on access patterns
- NUMA-aware page replacement
- Cross-node memory migration

### Profiling
- Hardware PMU programming
- Stack unwinding for call graphs
- Per-function timing
- Flame graph generation

### Preemption
- Full scheduler integration
- Preemption tracing
- Real-time latency analysis
- Preemption debugging tools

---

## References

- [Intel Software Developer Manual](https://software.intel.com/en-us/articles/intel-sdm)
- [ACPI Specification](https://uefi.org/specifications)
- [Linux Kernel Documentation](https://www.kernel.org/doc/)
- [OSDev Wiki](https://wiki.osdev.org/)
