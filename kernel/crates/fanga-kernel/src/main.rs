#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use limine::request::{
    BootloaderInfoRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest, RequestsEndMarker,
    RequestsStartMarker,
};
use limine::BaseRevision;

use fanga_arch_x86_64 as arch;

mod memory;
mod io;
mod task;

/* -------------------------------------------------------------------------- */
/*                             GLOBAL ALLOCATOR                                */
/* -------------------------------------------------------------------------- */

#[global_allocator]
static GLOBAL_ALLOCATOR: memory::GlobalHeapAllocator = memory::GlobalHeapAllocator::new();

/* -------------------------------------------------------------------------- */
/*                          LIMINE REQUIRED MARKERS                            */
/* -------------------------------------------------------------------------- */

#[used]
#[link_section = ".limine_requests_start"]
static LIMINE_REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[link_section = ".limine_requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".limine_requests"]
static BOOTLOADER_INFO_REQ: BootloaderInfoRequest = BootloaderInfoRequest::new();

#[used]
#[link_section = ".limine_requests"]
static FRAMEBUFFER_REQ: FramebufferRequest = FramebufferRequest::new();

#[used]
#[link_section = ".limine_requests"]
static MEMMAP_REQ: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[link_section = ".limine_requests"]
static HHDM_REQ: HhdmRequest = HhdmRequest::new();

#[used]
#[link_section = ".limine_requests_end"]
static LIMINE_REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();

/* -------------------------------------------------------------------------- */
/*                          FRAMEBUFFER INITIALIZATION                         */
/* -------------------------------------------------------------------------- */

fn init_framebuffer() {
    let fb_resp = FRAMEBUFFER_REQ
        .get_response()
        .expect("No framebuffer response");
    let fb = fb_resp.framebuffers().next().expect("No framebuffer");

    let addr = fb.addr() as *mut u8;
    let width = fb.width() as usize;
    let height = fb.height() as usize;
    let pitch = fb.pitch() as usize;
    let bpp = fb.bpp() as usize;

    // We only handle 32bpp here (common in QEMU/UEFI)
    if bpp != 32 {
        arch::serial_println!("Framebuffer bpp={} (expected 32). Console disabled.", bpp);
        return;
    }

    // Initialize the framebuffer console
    io::framebuffer::init(addr, width, height, pitch, bpp);
    arch::serial_println!("[Fanga] Framebuffer console initialized: {}x{} @ {}bpp", width, height, bpp);
}

/* -------------------------------------------------------------------------- */
/*                               KERNEL ENTRY                                  */
/* -------------------------------------------------------------------------- */

#[no_mangle]
pub extern "C" fn _start() -> ! {
    arch::init();

    // Initialize framebuffer console early
    init_framebuffer();

    if !BASE_REVISION.is_supported() {
        arch::serial_println!("[Fanga] Limine base revision NOT supported");
        loop {
            unsafe {
                core::arch::asm!("hlt");
            }
        }
    }

    arch::serial_println!("[Fanga] entered _start");

    // Bootloader info (nice sanity check)
    if let Some(info) = BOOTLOADER_INFO_REQ.get_response() {
        arch::serial_println!("[Fanga] bootloader: {}", info.name());
        arch::serial_println!("[Fanga] bootloader version: {}", info.version());
    }

    // HHDM offset (useful later for phys->virt mapping)
    if let Some(hhdm) = HHDM_REQ.get_response() {
        arch::serial_println!("[Fanga] HHDM offset: 0x{:x}", hhdm.offset());
    }

    // Memory map summary
    if let Some(mm) = MEMMAP_REQ.get_response() {
        let mut usable: u64 = 0;
        let mut total: u64 = 0;

        for entry in mm.entries() {
            let e = *entry;
            let _base = entry.base;
            let len = entry.length;
            total += len;

            // Usable memory type varies by crate version; this is the common pattern:
            if e.entry_type == limine::memory_map::EntryType::USABLE {
                usable += len;
            }

            // You can uncomment to dump entries:
            // arch::serial_println!(
            //   "MM: base=0x{:x} len=0x{:x} type={:?}",
            //   base, len, entry.entry_type()
            // );
        }

        arch::serial_println!("[Fanga] mem total:  {} KiB", total / 1024);
        arch::serial_println!("[Fanga] mem usable: {} KiB", usable / 1024);
    } else {
        arch::serial_println!("[Fanga] No memory map response");
    }

    // Initialize PMM (Physical Memory Manager)
    static mut PMM: memory::PhysicalMemoryManager = memory::PhysicalMemoryManager::new();
    
    if let (Some(mm), Some(hhdm)) = (MEMMAP_REQ.get_response(), HHDM_REQ.get_response()) {
        arch::serial_println!("[Fanga] Initializing PMM...");
        unsafe {
            PMM.init(mm, hhdm.offset());
        }
        
        arch::serial_println!(
            "[Fanga] PMM initialized: {} pages total, {} pages free, {} pages used",
            unsafe { PMM.total_pages() },
            unsafe { PMM.free_pages() },
            unsafe { PMM.used_pages() }
        );

        // Test PMM allocation
        arch::serial_println!("[Fanga] Testing PMM allocation...");
        unsafe {
            if let Some(page1) = PMM.alloc_page() {
                arch::serial_println!("[Fanga] Allocated page at: 0x{:x}", page1);
                
                if let Some(page2) = PMM.alloc_page() {
                    arch::serial_println!("[Fanga] Allocated page at: 0x{:x}", page2);
                    
                    // Free the pages
                    PMM.free_page(page1);
                    arch::serial_println!("[Fanga] Freed page at: 0x{:x}", page1);
                    
                    PMM.free_page(page2);
                    arch::serial_println!("[Fanga] Freed page at: 0x{:x}", page2);
                    
                    arch::serial_println!(
                        "[Fanga] After test: {} pages free",
                        PMM.free_pages()
                    );
                } else {
                    arch::serial_println!("[Fanga] Failed to allocate second page");
                }
            } else {
                arch::serial_println!("[Fanga] Failed to allocate first page");
            }
        }
        
        arch::serial_println!("[Fanga] PMM test completed ✅");
        
        // Initialize heap allocator
        arch::serial_println!("[Fanga] Initializing heap allocator...");
        
        // Allocate pages for the heap (1 MiB = 256 pages)
        const HEAP_SIZE: usize = 1024 * 1024; // 1 MiB
        const HEAP_PAGES: usize = HEAP_SIZE / memory::PAGE_SIZE;
        
        unsafe {
            // Allocate contiguous physical pages for the heap
            // We need to find a contiguous region, or just use the first page
            // and allocate more pages (though they may not be contiguous in physical memory,
            // they will be contiguous in virtual memory via HHDM)
            
            if let Some(heap_start_phys) = PMM.alloc_page() {
                // For simplicity, we'll use just the first allocated page to start
                // In a real kernel, we'd want to allocate all pages first
                // But to avoid the chicken-egg problem with Vec, we'll start with one page
                
                // For simplicity, we'll use the HHDM mapping for the heap
                let heap_start_virt = hhdm.offset() + heap_start_phys;
                let initial_heap_size = memory::PAGE_SIZE;
                
                GLOBAL_ALLOCATOR.init(heap_start_virt as usize, initial_heap_size);
                
                arch::serial_println!(
                    "[Fanga] Heap initialized: {} KiB at 0x{:x}",
                    initial_heap_size / 1024,
                    heap_start_virt
                );
                
                // Update memory statistics
                memory::stats::stats().set_total_heap(initial_heap_size);
                
                // Test heap allocation
                arch::serial_println!("[Fanga] Testing heap allocation...");
                
                // Test with Vec (requires alloc)
                let mut test_vec = alloc::vec![1, 2, 3, 4, 5];
                arch::serial_println!("[Fanga] Created Vec with {} elements", test_vec.len());
                
                test_vec.push(6);
                test_vec.push(7);
                arch::serial_println!("[Fanga] Vec now has {} elements", test_vec.len());
                
                // Test with Box
                let test_box = alloc::boxed::Box::new(42u64);
                arch::serial_println!("[Fanga] Created Box with value: {}", *test_box);
                
                arch::serial_println!("[Fanga] Heap allocation test completed ✅");
            } else {
                arch::serial_println!("[Fanga] Failed to allocate heap memory");
            }
        }
        
        // Initialize memory regions
        arch::serial_println!("[Fanga] Initializing memory regions...");
        
        static mut MEMORY_REGIONS: memory::regions::MemoryRegionManager = 
            memory::regions::MemoryRegionManager::new();
        
        unsafe {
            // Add regions based on memory map
            for entry in mm.entries() {
                let region_type = match entry.entry_type {
                    limine::memory_map::EntryType::USABLE => {
                        memory::regions::MemoryRegionType::Available
                    }
                    limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE |
                    limine::memory_map::EntryType::ACPI_RECLAIMABLE |
                    limine::memory_map::EntryType::ACPI_NVS |
                    limine::memory_map::EntryType::BAD_MEMORY |
                    limine::memory_map::EntryType::RESERVED => {
                        memory::regions::MemoryRegionType::Reserved
                    }
                    limine::memory_map::EntryType::FRAMEBUFFER |
                    limine::memory_map::EntryType::KERNEL_AND_MODULES => {
                        memory::regions::MemoryRegionType::KernelData
                    }
                    _ => memory::regions::MemoryRegionType::Reserved,
                };
                
                let region = memory::regions::MemoryRegion::new(
                    entry.base,
                    entry.length,
                    region_type,
                );
                
                if !MEMORY_REGIONS.add_region(region) {
                    arch::serial_println!("[Fanga] Warning: Memory region manager is full");
                    break;
                }
            }
            
            arch::serial_println!("[Fanga] Memory regions initialized");
            arch::serial_println!("[Fanga] Total regions: {}", MEMORY_REGIONS.count_by_type(
                memory::regions::MemoryRegionType::Available
            ) + MEMORY_REGIONS.count_by_type(
                memory::regions::MemoryRegionType::Reserved
            ));
            
            // Print some region statistics
            for region_type in [
                memory::regions::MemoryRegionType::Available,
                memory::regions::MemoryRegionType::Reserved,
                memory::regions::MemoryRegionType::KernelData,
            ] {
                let count = MEMORY_REGIONS.count_by_type(region_type);
                let size = MEMORY_REGIONS.total_size_by_type(region_type);
                if count > 0 {
                    arch::serial_println!(
                        "[Fanga]   {}: {} regions, {} KiB",
                        region_type,
                        count,
                        size / 1024
                    );
                }
            }
        }
        
        // Update memory statistics
        let total_mem = unsafe { PMM.total_pages() * memory::PAGE_SIZE };
        let used_mem = unsafe { PMM.used_pages() * memory::PAGE_SIZE };
        memory::stats::stats().set_total_physical(total_mem);
        memory::stats::stats().set_used_physical(used_mem);
        
        // Print memory statistics
        arch::serial_println!("[Fanga] Memory Statistics:");
        arch::serial_println!("[Fanga]   Total Physical: {} MiB", total_mem / (1024 * 1024));
        arch::serial_println!("[Fanga]   Used Physical:  {} MiB", used_mem / (1024 * 1024));
        arch::serial_println!("[Fanga]   Free Physical:  {} MiB", 
            (total_mem - used_mem) / (1024 * 1024)
        );
        
        // Test VMM (Virtual Memory Manager)
        arch::serial_println!("[Fanga] Testing VMM...");
        
        unsafe {
            // Create a new page table
            if let Some(mut mapper) = memory::PageTableMapper::new(&mut PMM, hhdm.offset()) {
                arch::serial_println!("[Fanga] Created new page table at: 0x{:x}", mapper.pml4_addr());
                
                // Test mapping: map virtual address 0x1000_0000 to a physical page
                if let Some(test_phys) = PMM.alloc_page() {
                    arch::serial_println!("[Fanga] Allocated test page at: 0x{:x}", test_phys);
                    
                    let test_virt = 0x1000_0000u64;
                    let flags = memory::PageTableFlags::PRESENT
                        .with(memory::PageTableFlags::WRITABLE);
                    
                    match mapper.map(test_virt, test_phys, flags, &mut PMM) {
                        Ok(()) => {
                            arch::serial_println!("[Fanga] Mapped 0x{:x} -> 0x{:x}", test_virt, test_phys);
                            
                            // Test translation
                            if let Some(translated) = mapper.translate(test_virt) {
                                arch::serial_println!("[Fanga] Translation: 0x{:x} -> 0x{:x}", test_virt, translated);
                                
                                if translated == test_phys {
                                    arch::serial_println!("[Fanga] Translation correct ✅");
                                } else {
                                    arch::serial_println!("[Fanga] Translation incorrect ❌");
                                }
                            } else {
                                arch::serial_println!("[Fanga] Translation failed ❌");
                            }
                            
                            // Test unmapping
                            match mapper.unmap(test_virt) {
                                Ok(unmapped_phys) => {
                                    arch::serial_println!("[Fanga] Unmapped 0x{:x}, got phys: 0x{:x}", test_virt, unmapped_phys);
                                    
                                    if unmapped_phys == test_phys {
                                        arch::serial_println!("[Fanga] Unmap correct ✅");
                                    } else {
                                        arch::serial_println!("[Fanga] Unmap returned wrong address ❌");
                                    }
                                    
                                    // Verify translation returns None after unmap
                                    if mapper.translate(test_virt).is_none() {
                                        arch::serial_println!("[Fanga] Translation after unmap: None (correct) ✅");
                                    } else {
                                        arch::serial_println!("[Fanga] Translation after unmap still works ❌");
                                    }
                                }
                                Err(e) => {
                                    arch::serial_println!("[Fanga] Unmap failed: {}", e);
                                }
                            }
                            
                            // Free the test page
                            PMM.free_page(test_phys);
                            arch::serial_println!("[Fanga] Freed test page");
                        }
                        Err(e) => {
                            arch::serial_println!("[Fanga] Map failed: {}", e);
                            PMM.free_page(test_phys);
                        }
                    }
                } else {
                    arch::serial_println!("[Fanga] Failed to allocate test page for VMM");
                }
                
                arch::serial_println!("[Fanga] VMM test completed ✅");
                
                // Get current page table (CR3)
                let current_cr3 = memory::PageTableMapper::current_cr3();
                arch::serial_println!("[Fanga] Current CR3 (page table): 0x{:x}", current_cr3);
            } else {
                arch::serial_println!("[Fanga] Failed to create page table mapper");
            }
        }
    } else {
        arch::serial_println!("[Fanga] Cannot initialize PMM: missing memory map or HHDM");
    }

    // Initialize task scheduler
    arch::serial_println!("[Fanga] Initializing task scheduler...");
    task::scheduler::init();
    arch::serial_println!("[Fanga] Task scheduler initialized ✅");
    
    // Demonstrate task creation
    arch::serial_println!("[Fanga] Testing task management...");
    
    {
        let mut scheduler_guard = task::scheduler::scheduler();
        let scheduler = scheduler_guard.as_mut().expect("Scheduler not initialized");
        
        // Create a simple task
        let task1 = task::Task::new(
            task::TaskId::new(0), // Will be overwritten by scheduler
            memory::VirtAddr::new(0x1000), // Dummy entry point
            memory::VirtAddr::new(0x10000), // Dummy stack
            4096, // 4KB stack
            memory::PhysAddr::new(0x0), // Dummy page table
            task::TaskPriority::Normal,
        );
        
        let mut task2 = task::Task::new(
            task::TaskId::new(0),
            memory::VirtAddr::new(0x2000),
            memory::VirtAddr::new(0x20000),
            4096,
            memory::PhysAddr::new(0x0),
            task::TaskPriority::High,
        );
        task2.set_name("high_priority");
        
        let mut task3 = task::Task::new(
            task::TaskId::new(0),
            memory::VirtAddr::new(0x3000),
            memory::VirtAddr::new(0x30000),
            4096,
            memory::PhysAddr::new(0x0),
            task::TaskPriority::Low,
        );
        task3.set_name("low_priority");
        
        // Add tasks to scheduler
        match scheduler.add_task(task1) {
            Ok(id) => arch::serial_println!("[Fanga] Created task: {:?}", id),
            Err(e) => arch::serial_println!("[Fanga] Failed to create task: {}", e),
        }
        
        match scheduler.add_task(task2) {
            Ok(id) => arch::serial_println!("[Fanga] Created task: {:?} (high priority)", id),
            Err(e) => arch::serial_println!("[Fanga] Failed to create task: {}", e),
        }
        
        match scheduler.add_task(task3) {
            Ok(id) => arch::serial_println!("[Fanga] Created task: {:?} (low priority)", id),
            Err(e) => arch::serial_println!("[Fanga] Failed to create task: {}", e),
        }
        
        arch::serial_println!("[Fanga] Total tasks: {}", scheduler.total_task_count());
        arch::serial_println!("[Fanga] Ready tasks: {}", scheduler.ready_task_count());
        
        // Test scheduling
        arch::serial_println!("[Fanga] Testing scheduler...");
        for i in 0..5 {
            let (prev, next, switched) = scheduler.schedule();
            arch::serial_println!(
                "[Fanga]   Schedule #{}: prev={:?}, next={:?}, switched={}",
                i + 1, prev, next, switched
            );
            
            if let Some(task_id) = next {
                if let Some(task) = scheduler.get_task(task_id) {
                    arch::serial_println!(
                        "[Fanga]     -> Running task '{}' (priority: {:?})",
                        task.name(),
                        task.priority
                    );
                }
            }
        }
    }
    
    arch::serial_println!("[Fanga] Task management test completed ✅");
    
    // Test IPC
    arch::serial_println!("[Fanga] Testing IPC...");
    
    use alloc::vec;
    let sender_id = task::TaskId::new(1);
    let mut msg_queue = task::MessageQueue::new(10);
    
    let msg1 = task::Message::new(sender_id, vec![1, 2, 3, 4]).unwrap();
    let msg2 = task::Message::new(sender_id, vec![5, 6, 7, 8]).unwrap();
    
    msg_queue.send(msg1).unwrap();
    msg_queue.send(msg2).unwrap();
    arch::serial_println!("[Fanga] Sent 2 messages, queue length: {}", msg_queue.len());
    
    if let Some(msg) = msg_queue.receive() {
        arch::serial_println!("[Fanga] Received message from {:?}: {:?}", msg.sender, msg.data());
    }
    
    arch::serial_println!("[Fanga] IPC test completed ✅");

    // Test the new console and logging system
    console_println!("===========================================");
    console_println!("    FangaOS - Operating System Kernel");
    console_println!("===========================================");
    console_println!();
    
    log_info!("Console system initialized");
    log_info!("Logging framework active");
    log_info!("Task scheduler initialized");
    log_info!("Process management ready");
    
    // Test different log levels
    log_debug!("Debug message example");
    log_info!("Info message example");
    log_warn!("Warning message example");
    log_error!("Error message example");
    
    console_println!();
    console_println!("Kernel Features:");
    console_println!("  [x] Memory Management (PMM, VMM, Heap)");
    console_println!("  [x] Interrupt Handling");
    console_println!("  [x] Task Scheduling (Round-Robin & Priority)");
    console_println!("  [x] Inter-Process Communication");
    console_println!();
    console_println!("Keyboard input is now active!");
    console_println!("Type something and see it appear in serial output...");
    console_println!();

    loop {
        // core::hint::spin_loop();
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                               PANIC HANDLER                                 */
/* -------------------------------------------------------------------------- */

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    console_println!();
    console_println!("!!! KERNEL PANIC !!!");
    console_println!("{}", info);

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
