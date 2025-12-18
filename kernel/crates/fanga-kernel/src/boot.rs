//! Kernel Boot Sequence Module
//!
//! This module implements a structured, phased boot sequence for FangaOS.
//! The boot process is divided into clear phases to ensure proper initialization
//! order and to make the boot sequence more maintainable and debuggable.
//!
//! # Boot Phases
//!
//! 1. **Early Boot**: Architecture-specific initialization (GDT, IDT, interrupts)
//! 2. **Bootloader Protocol**: Process bootloader-provided information
//! 3. **Memory Initialization**: Physical memory manager, heap allocator, virtual memory
//! 4. **Driver Initialization**: Essential drivers (framebuffer, keyboard, timer)
//! 5. **Subsystem Initialization**: Shell, scheduler, power management
//! 6. **Post-Init**: Final setup and demonstration code
//!
//! # Boot Flow
//!
//! ```text
//! _start (entry point)
//!   └─> boot::initialize()
//!       ├─> Phase 1: Early Boot
//!       │   ├─> Architecture init (GDT, IDT)
//!       │   └─> Serial output ready
//!       │
//!       ├─> Phase 2: Bootloader Protocol
//!       │   ├─> Verify Limine support
//!       │   ├─> Parse bootloader info
//!       │   └─> Get memory map & HHDM
//!       │
//!       ├─> Phase 3: Memory Initialization
//!       │   ├─> Initialize PMM
//!       │   ├─> Setup heap allocator
//!       │   ├─> Initialize VMM
//!       │   └─> Setup memory regions
//!       │
//!       ├─> Phase 4: Driver Initialization
//!       │   ├─> Framebuffer console
//!       │   ├─> Keyboard driver
//!       │   └─> Timer (PIT/APIC)
//!       │
//!       ├─> Phase 5: Subsystem Initialization
//!       │   ├─> Task scheduler
//!       │   ├─> Process management
//!       │   ├─> Power management
//!       │   └─> Shell/REPL
//!       │
//!       └─> Phase 6: Post-Init
//!           ├─> Run demonstrations
//!           └─> Display welcome message
//! ```

use crate::memory;
use crate::io;
use crate::shell;
use crate::task;
use crate::power;

use fanga_arch_x86_64 as arch;
use limine::request::{
    BootloaderInfoRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest,
};

/* -------------------------------------------------------------------------- */
/*                              BOOT PHASE 1: EARLY                            */
/* -------------------------------------------------------------------------- */

/// Phase 1: Early boot initialization
///
/// This phase performs the most basic initialization required to get the
/// system to a point where we can start using more advanced features.
///
/// # Operations
/// - Initialize architecture-specific components (GDT, IDT, interrupts)
/// - Setup serial output for early debugging
/// - Enable interrupts
pub fn phase1_early_boot() {
    arch::init();
    arch::serial_println!("[Boot Phase 1] Early boot initialization complete ✅");
}

/* -------------------------------------------------------------------------- */
/*                         BOOT PHASE 2: BOOTLOADER                            */
/* -------------------------------------------------------------------------- */

/// Context information from the bootloader
pub struct BootloaderContext {
    pub memory_map: &'static limine::response::MemoryMapResponse,
    pub hhdm_offset: u64,
    pub framebuffer: Option<&'static limine::response::FramebufferResponse>,
}

/// Phase 2: Process bootloader protocol
///
/// This phase processes information provided by the Limine bootloader,
/// including memory maps, framebuffer configuration, and HHDM offset.
///
/// # Returns
/// BootloaderContext containing essential bootloader information, or None if critical info is missing
pub fn phase2_bootloader_protocol(
    framebuffer_req: &'static FramebufferRequest,
    bootloader_info_req: &'static BootloaderInfoRequest,
    memmap_req: &'static MemoryMapRequest,
    hhdm_req: &'static HhdmRequest,
) -> Option<BootloaderContext> {
    arch::serial_println!("[Boot Phase 2] Processing bootloader protocol...");

    // Initialize framebuffer early if available
    if let Some(fb_resp) = framebuffer_req.get_response() {
        if let Some(fb) = fb_resp.framebuffers().next() {
            let addr = fb.addr() as *mut u8;
            let width = fb.width() as usize;
            let height = fb.height() as usize;
            let pitch = fb.pitch() as usize;
            let bpp = fb.bpp() as usize;

            if bpp == 32 {
                io::framebuffer::init(addr, width, height, pitch, bpp);
                arch::serial_println!(
                    "[Boot Phase 2] Framebuffer console: {}x{} @ {}bpp",
                    width, height, bpp
                );
            } else {
                arch::serial_println!(
                    "[Boot Phase 2] Framebuffer bpp={} (expected 32). Console disabled.",
                    bpp
                );
            }
        }
    }

    // Log bootloader information
    if let Some(info) = bootloader_info_req.get_response() {
        arch::serial_println!("[Boot Phase 2] Bootloader: {} {}", info.name(), info.version());
    }

    // Get critical bootloader information
    let memory_map = memmap_req.get_response()?;
    let hhdm_response = hhdm_req.get_response()?;
    let hhdm_offset = hhdm_response.offset();

    arch::serial_println!("[Boot Phase 2] HHDM offset: 0x{:x}", hhdm_offset);

    // Log memory map summary
    let mut usable: u64 = 0;
    let mut total: u64 = 0;
    for entry in memory_map.entries() {
        total += entry.length;
        if entry.entry_type == limine::memory_map::EntryType::USABLE {
            usable += entry.length;
        }
    }
    arch::serial_println!("[Boot Phase 2] Total memory: {} MiB", total / (1024 * 1024));
    arch::serial_println!("[Boot Phase 2] Usable memory: {} MiB", usable / (1024 * 1024));

    arch::serial_println!("[Boot Phase 2] Bootloader protocol complete ✅");

    Some(BootloaderContext {
        memory_map,
        hhdm_offset,
        framebuffer: framebuffer_req.get_response(),
    })
}

/* -------------------------------------------------------------------------- */
/*                         BOOT PHASE 3: MEMORY                                */
/* -------------------------------------------------------------------------- */

/// Phase 3: Initialize memory subsystems
///
/// This phase sets up all memory management subsystems in the correct order:
/// PMM -> Heap -> VMM -> Memory regions
///
/// # Safety
/// This function manipulates static mutable memory managers and must be called
/// exactly once during boot.
pub unsafe fn phase3_memory_init(ctx: &BootloaderContext) {
    arch::serial_println!("[Boot Phase 3] Initializing memory subsystems...");

    // Initialize Physical Memory Manager (PMM)
    arch::serial_println!("[Boot Phase 3] Initializing PMM...");
    static mut PMM: memory::PhysicalMemoryManager = memory::PhysicalMemoryManager::new();
    
    PMM.init(ctx.memory_map, ctx.hhdm_offset);
    arch::serial_println!(
        "[Boot Phase 3] PMM: {} pages total, {} free",
        PMM.total_pages(),
        PMM.free_pages()
    );

    // Initialize heap allocator
    arch::serial_println!("[Boot Phase 3] Initializing heap allocator...");
    const HEAP_PAGES: usize = 3; // 12KB initial heap

    if let Some(heap_start_phys) = PMM.alloc_page() {
        // Allocate additional pages
        for i in 1..HEAP_PAGES {
            if PMM.alloc_page().is_none() {
                arch::serial_println!(
                    "[Boot Phase 3] Warning: Only allocated {} heap pages",
                    i
                );
                break;
            }
        }

        let heap_start_virt = ctx.hhdm_offset + heap_start_phys;
        let heap_size = memory::PAGE_SIZE * HEAP_PAGES;

        crate::GLOBAL_ALLOCATOR.init(heap_start_virt as usize, heap_size);
        arch::serial_println!(
            "[Boot Phase 3] Heap: {} KiB at 0x{:x}",
            heap_size / 1024,
            heap_start_virt
        );

        memory::stats::stats().set_total_heap(heap_size);
    } else {
        panic!("Failed to allocate heap memory");
    }

    // Test Virtual Memory Manager (VMM)
    arch::serial_println!("[Boot Phase 3] Testing VMM...");
    if let Some(mapper) = memory::PageTableMapper::new(&mut PMM, ctx.hhdm_offset) {
        arch::serial_println!(
            "[Boot Phase 3] VMM: Page table at 0x{:x}",
            mapper.pml4_addr()
        );
    } else {
        arch::serial_println!("[Boot Phase 3] Warning: VMM test skipped");
    }

    // Initialize memory regions
    arch::serial_println!("[Boot Phase 3] Initializing memory regions...");
    static mut MEMORY_REGIONS: memory::regions::MemoryRegionManager =
        memory::regions::MemoryRegionManager::new();

    for entry in ctx.memory_map.entries() {
        let region_type = match entry.entry_type {
            limine::memory_map::EntryType::USABLE => {
                memory::regions::MemoryRegionType::Available
            }
            limine::memory_map::EntryType::FRAMEBUFFER
            | limine::memory_map::EntryType::EXECUTABLE_AND_MODULES => {
                memory::regions::MemoryRegionType::KernelData
            }
            _ => memory::regions::MemoryRegionType::Reserved,
        };

        let region = memory::regions::MemoryRegion::new(entry.base, entry.length, region_type);
        if !MEMORY_REGIONS.add_region(region) {
            break;
        }
    }

    // Update memory statistics
    let total_mem = PMM.total_pages() * memory::PAGE_SIZE;
    let used_mem = PMM.used_pages() * memory::PAGE_SIZE;
    memory::stats::stats().set_total_physical(total_mem);
    memory::stats::stats().set_used_physical(used_mem);

    arch::serial_println!(
        "[Boot Phase 3] Memory: {} MiB total, {} MiB free",
        total_mem / (1024 * 1024),
        (total_mem - used_mem) / (1024 * 1024)
    );

    arch::serial_println!("[Boot Phase 3] Memory initialization complete ✅");
}

/* -------------------------------------------------------------------------- */
/*                         BOOT PHASE 4: DRIVERS                               */
/* -------------------------------------------------------------------------- */

/// Phase 4: Initialize essential drivers
///
/// This phase initializes all essential hardware drivers in the correct order.
/// Requires heap allocator to be ready for dynamic allocations.
pub fn phase4_driver_init() {
    arch::serial_println!("[Boot Phase 4] Initializing drivers...");

    // Keyboard input system (requires heap for Vec)
    io::keyboard_bridge::init();
    arch::serial_println!("[Boot Phase 4] Keyboard driver initialized");

    // Timer is initialized as part of architecture init, but we log it here for clarity
    arch::serial_println!("[Boot Phase 4] Timer (PIT) ready");

    arch::serial_println!("[Boot Phase 4] Driver initialization complete ✅");
}

/* -------------------------------------------------------------------------- */
/*                         BOOT PHASE 5: SUBSYSTEMS                            */
/* -------------------------------------------------------------------------- */

/// Phase 5: Initialize kernel subsystems
///
/// This phase initializes higher-level kernel subsystems that depend on
/// memory and drivers being ready.
pub fn phase5_subsystem_init() {
    arch::serial_println!("[Boot Phase 5] Initializing kernel subsystems...");

    // Shell and command history
    shell::init();
    shell::history::init();
    io::line_editor::init();
    arch::serial_println!("[Boot Phase 5] Shell initialized");

    // Task scheduler and process management
    task::scheduler::init();
    task::process::init();
    task::timer_bridge::init();
    arch::serial_println!(
        "[Boot Phase 5] Task scheduler initialized (time slice: {}ms)",
        task::sched_timer::TIME_SLICE * 10
    );

    // Power management
    power::init();
    arch::serial_println!("[Boot Phase 5] Power management initialized");

    arch::serial_println!("[Boot Phase 5] Subsystem initialization complete ✅");
}

/* -------------------------------------------------------------------------- */
/*                         BOOT PHASE 6: POST-INIT                             */
/* -------------------------------------------------------------------------- */

/// Phase 6: Post-initialization and demonstrations
///
/// This phase runs demonstration code and displays the welcome message.
/// This is where you can add system tests or feature demonstrations.
pub fn phase6_post_init() {
    arch::serial_println!("[Boot Phase 6] Running post-initialization...");

    // Run process management demonstration
    run_process_demo();

    // Display welcome message
    display_welcome();

    arch::serial_println!("[Boot Phase 6] Post-initialization complete ✅");
    arch::serial_println!("");
    arch::serial_println!("===========================================");
    arch::serial_println!("   KERNEL BOOT SEQUENCE COMPLETE");
    arch::serial_println!("===========================================");
}

/// Run process management demonstration
fn run_process_demo() {
    use crate::memory::{VirtAddr, PhysAddr};
    
    arch::serial_println!("");
    arch::serial_println!("===========================================");
    arch::serial_println!("   PROCESS MANAGEMENT DEMONSTRATION");
    arch::serial_println!("===========================================");

    // Create and schedule some demo tasks
    let mut scheduler_guard = task::scheduler::scheduler();
    let scheduler = &mut *scheduler_guard;

    // Create tasks with different priorities
    let mut task1 = task::Task::new(
        task::TaskId::new(0),
        VirtAddr::new(task::examples::task1 as *const () as u64),
        VirtAddr::new(0x10000),
        4096,
        PhysAddr::new(0x0),
        task::TaskPriority::Normal,
    );
    task1.set_name("counter_task");

    let mut task2 = task::Task::new(
        task::TaskId::new(0),
        VirtAddr::new(task::examples::task2 as *const () as u64),
        VirtAddr::new(0x20000),
        4096,
        PhysAddr::new(0x0),
        task::TaskPriority::High,
    );
    task2.set_name("compute_task");

    if let Ok(id1) = scheduler.add_task(task1) {
        arch::serial_println!("  Created task {:?}: counter_task", id1);
    }
    if let Ok(id2) = scheduler.add_task(task2) {
        arch::serial_println!("  Created task {:?}: compute_task", id2);
    }

    arch::serial_println!("  Total tasks: {}", scheduler.total_task_count());
}

/// Display welcome message on console
fn display_welcome() {
    crate::console_println!();
    crate::console_println!("===========================================");
    crate::console_println!("    FangaOS - Operating System Kernel");
    crate::console_println!("===========================================");
    crate::console_println!();

    crate::log_info!("Kernel boot complete");
    crate::log_info!("All subsystems operational");

    crate::console_println!();
    crate::console_println!("Kernel Features:");
    crate::console_println!("  [x] Memory Management (PMM, VMM, Heap)");
    crate::console_println!("  [x] Interrupt Handling (IDT, PIC/APIC)");
    crate::console_println!("  [x] System Calls (SYSCALL/SYSRET)");
    crate::console_println!("  [x] Task Scheduling (Round-Robin & Priority)");
    crate::console_println!("  [x] Process Management");
    crate::console_println!("  [x] Inter-Process Communication");
    crate::console_println!("  [x] Interactive Shell/REPL");
    crate::console_println!();
    crate::console_println!("Welcome to FangaOS Interactive Shell!");
    crate::console_println!("Type 'help' for available commands.");
    crate::console_println!();

    // Show initial prompt
    {
        let shell_guard = shell::shell();
        if let Some(shell) = shell_guard.as_ref() {
            let mut fb = io::framebuffer::framebuffer();
            fb.write_string(shell.prompt());
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                         MAIN BOOT ORCHESTRATOR                              */
/* -------------------------------------------------------------------------- */

/// Main boot initialization function
///
/// This function orchestrates the entire kernel boot sequence by calling
/// each boot phase in the correct order.
///
/// # Arguments
/// * `framebuffer_req` - Limine framebuffer request
/// * `bootloader_info_req` - Limine bootloader info request
/// * `memmap_req` - Limine memory map request
/// * `hhdm_req` - Limine HHDM request
/// * `base_revision` - Limine base revision for compatibility check
///
/// # Returns
/// Returns `Ok(())` if boot succeeds, or an error message if a critical phase fails
pub fn initialize(
    framebuffer_req: &'static FramebufferRequest,
    bootloader_info_req: &'static BootloaderInfoRequest,
    memmap_req: &'static MemoryMapRequest,
    hhdm_req: &'static HhdmRequest,
    base_revision: &'static limine::BaseRevision,
) -> Result<(), &'static str> {
    // Phase 1: Early boot
    phase1_early_boot();

    // Check Limine base revision
    if !base_revision.is_supported() {
        arch::serial_println!("[Boot] ERROR: Limine base revision not supported");
        return Err("Limine base revision not supported");
    }

    // Phase 2: Bootloader protocol
    let ctx = phase2_bootloader_protocol(
        framebuffer_req,
        bootloader_info_req,
        memmap_req,
        hhdm_req,
    )
    .ok_or("Failed to process bootloader protocol")?;

    // Phase 3: Memory initialization
    unsafe {
        phase3_memory_init(&ctx);
    }

    // Phase 4: Driver initialization
    phase4_driver_init();

    // Phase 5: Subsystem initialization
    phase5_subsystem_init();

    // Phase 6: Post-initialization
    phase6_post_init();

    Ok(())
}
