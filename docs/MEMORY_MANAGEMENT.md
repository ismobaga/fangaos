# Memory Management in FangaOS

This document describes the memory management subsystem in FangaOS.

## Overview

The FangaOS kernel implements a comprehensive memory management system with the following components:

1. **Physical Memory Manager (PMM)** - Manages physical memory pages using a bitmap allocator
2. **Virtual Memory Manager (VMM)** - Manages page tables and virtual-to-physical address translation
3. **Heap Allocator** - Provides dynamic memory allocation via Rust's `GlobalAlloc` trait
4. **Memory Regions** - Tracks different types of memory regions (kernel, user, MMIO, etc.)
5. **Memory Statistics** - Tracks and reports memory usage statistics

## Physical Memory Manager (PMM)

**Location**: `kernel/crates/fanga-kernel/src/pmm.rs`

The PMM manages physical memory at the page level (4 KiB pages) using a bitmap allocator.

### Features

- **Bitmap-based allocation**: Each bit represents one physical page
  - Bit = 0: Page is free
  - Bit = 1: Page is used/allocated
- **Fast allocation**: First-fit strategy with O(n) complexity
- **Thread-safe counters**: Uses atomic operations for free page counting
- **Memory map integration**: Initializes from bootloader memory map

### API

```rust
// Initialize the PMM
PMM.init(memmap_response, hhdm_offset);

// Allocate a physical page (returns physical address)
let phys_addr = PMM.alloc_page().expect("Out of memory");

// Free a physical page
PMM.free_page(phys_addr);

// Query statistics
let total = PMM.total_pages();
let free = PMM.free_pages();
let used = PMM.used_pages();
```

### Thread Safety

⚠️ **Warning**: The current PMM implementation is NOT thread-safe. The bitmap operations are not atomic and will race if called from multiple threads. A locking mechanism (Mutex/SpinLock) must be added before using in a multi-threaded environment.

## Virtual Memory Manager (VMM)

**Location**: `kernel/crates/fanga-kernel/src/vmm.rs`

The VMM manages the x86_64 page tables and provides virtual memory mapping functionality.

### Features

- **4-level paging**: PML4 → PDPT → PD → PT (standard x86_64)
- **Page table creation**: Automatically allocates page tables as needed
- **Address translation**: Translates virtual addresses to physical addresses
- **TLB management**: Flushes TLB entries after map/unmap operations
- **Flexible flags**: Supports all x86_64 page table flags (Present, Writable, User, NX, etc.)

### API

```rust
// Create a new page table
let mut mapper = PageTableMapper::new(&mut PMM, hhdm_offset)?;

// Map a virtual address to a physical address
mapper.map(virt_addr, phys_addr, flags, &mut PMM)?;

// Translate a virtual address
let phys = mapper.translate(virt_addr)?;

// Unmap a virtual address
let phys = mapper.unmap(virt_addr)?;

// Get current CR3 (active page table)
let cr3 = PageTableMapper::current_cr3();
```

### Page Table Flags

- `PRESENT`: Page is present in memory
- `WRITABLE`: Page is writable
- `USER`: Page is accessible from user mode
- `WRITE_THROUGH`: Write-through caching
- `NO_CACHE`: Disable caching
- `ACCESSED`: Page has been accessed (set by CPU)
- `DIRTY`: Page has been written to (set by CPU)
- `HUGE_PAGE`: Use 2 MiB or 1 GiB pages
- `GLOBAL`: Don't flush from TLB on CR3 reload
- `NO_EXECUTE`: Disable execution (NX bit)

## Heap Allocator

**Location**: `kernel/crates/fanga-kernel/src/heap.rs`

The heap allocator provides dynamic memory allocation for kernel data structures.

### Features

- **Linked-list based**: Simple first-fit allocation strategy
- **Automatic coalescing**: Adjacent free blocks are merged on deallocation
- **GlobalAlloc implementation**: Integrates with Rust's standard allocator interface
- **Thread-safe**: Uses spin locks for synchronization
- **Zero-sized allocation handling**: Properly handles zero-sized allocations

### API

The heap allocator is used transparently through Rust's allocation APIs:

```rust
// Vec (dynamic array)
let mut vec = Vec::new();
vec.push(1);
vec.push(2);

// Box (heap-allocated value)
let boxed = Box::new(42);

// String (heap-allocated string)
let s = String::from("Hello, FangaOS!");
```

### Initialization

```rust
// Initialize heap with a memory region
GLOBAL_ALLOCATOR.init(heap_start, heap_size);
```

### Limitations

- **Single region**: Currently supports only one contiguous heap region
- **No growth**: The heap cannot grow after initialization
- **First-fit**: May lead to fragmentation over time

## Memory Regions

**Location**: `kernel/crates/fanga-kernel/src/memory_regions.rs`

The memory regions module tracks different types of memory in the system.

### Region Types

- **KernelCode**: Kernel code (read-only, executable)
- **KernelData**: Kernel data (read-write, non-executable)
- **KernelStack**: Kernel stack (read-write, non-executable)
- **KernelHeap**: Kernel heap (read-write, non-executable)
- **UserSpace**: User space memory
- **MMIO**: Memory-Mapped I/O (read-write, non-cacheable)
- **Available**: Free physical memory
- **Reserved**: Reserved memory (bootloader, firmware, etc.)

### API

```rust
// Create a region manager
let mut manager = MemoryRegionManager::new();

// Add a region
let region = MemoryRegion::new(base, size, MemoryRegionType::KernelData);
manager.add_region(region);

// Find region containing an address
let region = manager.find_region(addr);

// Query statistics
let total_size = manager.total_size_by_type(MemoryRegionType::Available);
let count = manager.count_by_type(MemoryRegionType::KernelData);
```

### Address Spaces

```rust
use memory_regions::address_space::*;

// Check address space
if is_kernel_space(addr) { /* ... */ }
if is_user_space(addr) { /* ... */ }

// Convert addresses
let virt = phys_to_hhdm(phys, hhdm_offset);
let phys = hhdm_to_phys(virt, hhdm_offset);
```

## Memory Statistics

**Location**: `kernel/crates/fanga-kernel/src/memory_stats.rs`

The memory statistics module tracks memory usage and provides debugging utilities.

### Features

- **Physical memory tracking**: Total, used, and free physical memory
- **Heap tracking**: Total and used heap memory
- **Allocation counters**: Tracks allocation/deallocation counts
- **Thread-safe**: Uses atomic operations

### API

```rust
use memory_stats::stats;

// Query statistics
let total_phys = stats().total_physical();
let used_phys = stats().used_physical();
let free_phys = stats().free_physical();

let total_heap = stats().total_heap();
let used_heap = stats().used_heap();
let free_heap = stats().free_heap();

// Record allocations (done automatically by allocators)
stats().record_heap_alloc(size);
stats().record_heap_dealloc(size);
stats().record_page_alloc();
stats().record_page_dealloc();

// Print all statistics
println!("{}", stats());
```

### Debugging Utilities

```rust
use memory_stats::debug;

// Dump memory contents
unsafe {
    debug::dump_memory(addr, size, "Label");
}

// Dump page table entry
debug::dump_page_table_entry(virt_addr, &mapper);
```

## Memory Layout

### Physical Memory

```
0x0000_0000_0000_0000 - 0x0000_0000_0007_FFFF : Low memory (< 512 KiB)
0x0000_0000_0010_0000 - 0x0000_0000_FFFF_FFFF : Conventional memory
0x0001_0000_0000_0000 - ...                   : High memory (varies by system)
```

### Virtual Memory (x86_64)

```
0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF : User space (128 TiB)
0x0000_8000_0000_0000 - 0xFFFF_7FFF_FFFF_FFFF : Canonical hole (unusable)
0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_7FFF_FFFF : Kernel space (128 TiB)
  ├─ 0xFFFF_8000_0000_0000 - ...              : Higher Half Direct Map (HHDM)
  └─ 0xFFFF_FFFF_8000_0000 - ...              : Kernel code/data
0xFFFF_FFFF_8000_0000 - 0xFFFF_FFFF_FFFF_FFFF : Kernel (2 GiB)
```

## Initialization Sequence

1. **Bootloader**: Limine bootloader provides memory map and HHDM offset
2. **PMM**: Initialize physical memory manager from memory map
3. **Heap**: Allocate pages for heap and initialize heap allocator
4. **Regions**: Build memory regions database from memory map
5. **VMM**: Create page tables for kernel mapping (if needed)

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| PMM alloc_page | O(n) | Linear search through bitmap |
| PMM free_page | O(1) | Direct bit manipulation |
| Heap alloc | O(n) | First-fit search through free list |
| Heap dealloc | O(n) | Insertion with coalescing |
| VMM map | O(1) | 4 page table lookups + TLB flush |
| VMM unmap | O(1) | 4 page table lookups + TLB flush |
| VMM translate | O(1) | 4 page table lookups |

## Future Improvements

### Physical Memory Manager
- [ ] Implement buddy allocator for faster allocation
- [ ] Add support for large pages (2 MiB, 1 GiB)
- [ ] Add NUMA awareness
- [ ] Add memory zones (DMA, Normal, High)

### Virtual Memory Manager
- [x] Implement copy-on-write (COW)
- [x] Add memory-mapped files (mmap/munmap infrastructure)
- [x] Add shared memory support
- [x] Implement demand paging

### Heap Allocator
- [ ] Implement slab allocator for small objects
- [ ] Add heap growth support
- [ ] Implement best-fit or buddy algorithm
- [ ] Add memory pool support

### Memory Regions
- [x] Add memory protection keys
- [x] Implement guard pages
- [ ] Add memory access profiling
- [ ] Implement kernel address space layout randomization (KASLR)

## New Features (Recently Added)

### Copy-on-Write (CoW)

**Location**: `kernel/crates/fanga-kernel/src/memory/cow.rs`

CoW allows efficient memory sharing between processes by marking shared pages as read-only. When a process attempts to write to a CoW page, a page fault occurs, and a private copy is made for that process.

**Features:**
- Reference counting for shared pages
- Automatic copy on write access
- Integration with fork()

**API:**
```rust
use fanga_kernel::memory::*;

// Mark a page as CoW (shared)
add_cow_page(PhysAddr::new(0x1000));
mark_cow_page(PhysAddr::new(0x1000)); // Increment ref count

// Check if page is shared
if is_cow_page(PhysAddr::new(0x1000)) {
    // Make a private copy
}

// Release reference
let ref_count = release_cow_page(PhysAddr::new(0x1000));
if ref_count == 0 {
    // Page can be freed
}
```

### Memory Mapping (mmap/munmap)

**Location**: `kernel/crates/fanga-kernel/src/memory/mmap.rs`

Provides POSIX-like mmap/munmap functionality for mapping memory regions.

**Features:**
- Anonymous mappings (MAP_ANONYMOUS)
- Shared and private mappings (MAP_SHARED, MAP_PRIVATE)
- Fixed address placement (MAP_FIXED)
- Automatic address selection
- Protection flags (READ, WRITE, EXEC)

**API:**
```rust
use fanga_kernel::memory::*;

let mut manager = MmapManager::new(0x1000_0000);

// Map anonymous memory
let addr = manager.mmap(
    0,  // Auto-select address
    4096,  // Size
    MmapProt::READ.with(MmapProt::WRITE),
    MmapFlags::PRIVATE.with(MmapFlags::ANONYMOUS),
).unwrap();

// Unmap
manager.munmap(addr.as_u64(), 4096);
```

**Syscalls:**
- `SYS_MMAP (9)` - Map memory region
- `SYS_MUNMAP (11)` - Unmap memory region

### Demand Paging

**Location**: `kernel/crates/fanga-kernel/src/memory/demand_paging.rs`

Implements demand paging where pages are allocated only when first accessed, reducing memory usage.

**Features:**
- Page state tracking (NotAllocated, InMemory, SwappedOut)
- Reserve virtual address ranges without physical allocation
- Allocate on page fault
- Statistics tracking

**API:**
```rust
use fanga_kernel::memory::*;

// Reserve pages without allocating physical memory
reserve_demand_pages(VirtAddr::new(0x1000), 10);

// On page fault, allocate the page
if should_allocate_on_fault(VirtAddr::new(0x1000)) {
    allocate_demand_page(VirtAddr::new(0x1000));
}

// Get statistics
let stats = get_demand_paging_stats();
println!("In memory: {}, Not allocated: {}", 
    stats.in_memory, stats.not_allocated);
```

### Page Replacement (LRU)

**Location**: `kernel/crates/fanga-kernel/src/memory/demand_paging.rs`

Implements Least Recently Used (LRU) page replacement for efficient memory usage when physical memory is scarce.

**Features:**
- Queue-based LRU tracking
- Access time recording
- Automatic eviction of least recently used pages
- Statistics and access patterns

**API:**
```rust
use fanga_kernel::memory::*;

// Record page access (moves to back of LRU queue)
record_page_access(VirtAddr::new(0x1000), PhysAddr::new(0x10000));

// Get least recently used page (for eviction)
if let Some((virt, phys)) = get_lru_page() {
    // Evict this page
}

// Get statistics
let stats = get_lru_stats();
println!("Tracking {} pages, {} total accesses", 
    stats.tracked_pages, stats.total_accesses);
```

### Memory Protection

**Location**: `kernel/crates/fanga-kernel/src/memory/protection.rs`

Provides memory protection mechanisms including guard pages and access control.

**Features:**
- Guard pages for stack overflow detection
- Memory region access control (read/write/exec)
- Per-region protection flags
- Access validation

**API:**
```rust
use fanga_kernel::memory::*;

// Add a guard page
add_guard_page(VirtAddr::new(0x1000)).unwrap();

// Check if address is a guard page
if is_guard_page(VirtAddr::new(0x1000)) {
    // Deny access
}

// Create a protected region
let region = ProtectedRegion::new(
    VirtAddr::new(0x2000),
    4096,
    MemoryProtection::READ,  // Read-only
    "code section"
);
add_protected_region(region);

// Check access permissions
if check_memory_access(VirtAddr::new(0x2000), true, false) {
    // Write is not allowed
}
```

### Swap Support

**Location**: `kernel/crates/fanga-kernel/src/memory/swap.rs`

Provides basic swap space support for paging memory to disk when physical memory is exhausted.

**Features:**
- Swap slot allocation
- Simulated disk I/O (for testing)
- Page swap out/swap in
- Integration with page replacement

**API:**
```rust
use fanga_kernel::memory::*;

// Initialize swap with 1000 page slots
init_swap(1000);

// Swap out a page
unsafe {
    if let Some(slot) = swap_out_page(virt_addr, phys_addr) {
        println!("Page swapped to slot {}", slot);
    }
}

// Swap in a page
unsafe {
    if swap_in_page(virt_addr, phys_addr) {
        println!("Page swapped back in");
    }
}

// Check if page is swapped
if is_page_swapped(virt_addr) {
    // Page is on disk
}

// Get statistics
let stats = get_swap_stats();
println!("Used {} of {} swap slots", stats.used_slots, stats.total_slots);
```

## Integration Example

Here's how these features work together:

```rust
use fanga_kernel::memory::*;

// 1. Initialize swap space
init_swap(1024);

// 2. Create a process with demand paging
reserve_demand_pages(VirtAddr::new(0x1000), 100);

// 3. Add guard page for stack protection
add_guard_page(VirtAddr::new(0xFFFF_FFFF_0000)).unwrap();

// 4. Set up memory mapping
let mut mmap_mgr = MmapManager::new(0x4000_0000);
let mapped = mmap_mgr.mmap(
    0, 
    0x10000,
    MmapProt::READ_WRITE,
    MmapFlags::PRIVATE.with(MmapFlags::ANONYMOUS)
).unwrap();

// 5. On page fault:
if should_allocate_on_fault(fault_addr) {
    // Allocate physical page
    let phys = alloc_frame(&mut pmm).unwrap();
    
    // Record access for LRU
    record_page_access(fault_addr, PhysAddr::new(phys));
    
    // If memory pressure, evict LRU page
    if should_evict() {
        if let Some((lru_virt, lru_phys)) = get_lru_page() {
            unsafe {
                swap_out_page(lru_virt, lru_phys);
            }
        }
    }
    
    allocate_demand_page(fault_addr);
}
```

## Testing

All memory management components have been tested:

- ✅ PMM: Allocate and free pages
- ✅ VMM: Map, translate, and unmap virtual addresses  
- ✅ Heap: Allocate Vec and Box
- ✅ Regions: Track different memory types
- ✅ Statistics: Report memory usage
- ✅ CoW: Reference counting and sharing
- ✅ mmap/munmap: Memory mapping operations
- ✅ Demand paging: State tracking and allocation
- ✅ LRU: Page replacement and eviction
- ✅ Memory protection: Guard pages and access control
- ✅ Swap: Page swap out/in operations

Test results: **154 kernel tests passing**

Example test output:
```
[Fanga] PMM initialized: 268435456 pages total, 515695 pages free, 267919761 pages used
[Fanga] Heap initialized: 4 KiB at 0xffff800000052000
[Fanga] Created Vec with 5 elements
[Fanga] Vec now has 7 elements
[Fanga] Created Box with value: 42
[Fanga] Memory regions initialized
[Fanga] Total regions: 14
[Fanga] CoW page tracking: 2 pages shared
[Fanga] mmap: Mapped 4096 bytes at 0x40000000
[Fanga] LRU: Tracking 10 pages, 25 accesses
[Fanga] Swap: 5 of 1024 slots used
```

## References

- [OSDev Wiki - Memory Management](https://wiki.osdev.org/Memory_Management)
- [x86_64 Paging](https://wiki.osdev.org/Paging)
- [Rust Embedded Book - Memory Management](https://docs.rust-embedded.org/book/)
- [Intel 64 and IA-32 Architectures Software Developer's Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
