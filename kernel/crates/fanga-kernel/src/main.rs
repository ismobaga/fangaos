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

mod pmm;
mod vmm;
mod heap;
mod memory_regions;
mod memory_stats;

/* -------------------------------------------------------------------------- */
/*                             GLOBAL ALLOCATOR                                */
/* -------------------------------------------------------------------------- */

#[global_allocator]
static GLOBAL_ALLOCATOR: heap::GlobalHeapAllocator = heap::GlobalHeapAllocator::new();

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
/*                               FRAMEBUFFER DRAW                              */
/* -------------------------------------------------------------------------- */

fn fb_fill_color(argb: u32) {
    let fb_resp = FRAMEBUFFER_REQ
        .get_response()
        .expect("No framebuffer response");
    let fb = fb_resp.framebuffers().next().expect("No framebuffer");

    let addr = fb.addr() as *mut u8;
    let pitch = fb.pitch() as usize;
    let height = fb.height() as usize;
    let bpp = fb.bpp() as usize;

    // We only handle 32bpp here (common in QEMU/UEFI). If not, just do nothing.
    if bpp != 32 {
        arch::serial_println!("Framebuffer bpp={} (expected 32). Skipping fill.", bpp);
        return;
    }

    unsafe {
        for y in 0..height {
            let row = addr.add(y * pitch) as *mut u32;
            for x in 0..(pitch / 4) {
                // Write whole row in pitch units (includes padding)
                row.add(x).write_volatile(argb);
            }
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                               KERNEL ENTRY                                  */
/* -------------------------------------------------------------------------- */

#[no_mangle]
pub extern "C" fn _start() -> ! {
    arch::init();

    if !BASE_REVISION.is_supported() {
        arch::serial_println!("[Fanga] Limine base revision NOT supported");
        loop {
            unsafe {
                core::arch::asm!("hlt");
            }
        }
    }

    unsafe {
        // core::arch::asm!("int3");
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
    static mut PMM: pmm::PhysicalMemoryManager = pmm::PhysicalMemoryManager::new();
    
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
        const HEAP_PAGES: usize = HEAP_SIZE / pmm::PAGE_SIZE;
        
        unsafe {
            if let Some(heap_start_phys) = PMM.alloc_page() {
                // Allocate remaining pages
                let mut heap_phys_pages = alloc::vec::Vec::new();
                heap_phys_pages.push(heap_start_phys);
                
                for _ in 1..HEAP_PAGES {
                    if let Some(page) = PMM.alloc_page() {
                        heap_phys_pages.push(page);
                    } else {
                        arch::serial_println!("[Fanga] Warning: Could not allocate all heap pages");
                        break;
                    }
                }
                
                // For simplicity, we'll use the HHDM mapping for the heap
                let heap_start_virt = hhdm.offset() + heap_start_phys;
                let actual_heap_size = heap_phys_pages.len() * pmm::PAGE_SIZE;
                
                GLOBAL_ALLOCATOR.init(heap_start_virt as usize, actual_heap_size);
                
                arch::serial_println!(
                    "[Fanga] Heap initialized: {} KiB at 0x{:x}",
                    actual_heap_size / 1024,
                    heap_start_virt
                );
                
                // Update memory statistics
                memory_stats::stats().set_total_heap(actual_heap_size);
                
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
        
        static mut MEMORY_REGIONS: memory_regions::MemoryRegionManager = 
            memory_regions::MemoryRegionManager::new();
        
        unsafe {
            // Add regions based on memory map
            for entry in mm.entries() {
                let region_type = match entry.entry_type {
                    limine::memory_map::EntryType::USABLE => {
                        memory_regions::MemoryRegionType::Available
                    }
                    limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE |
                    limine::memory_map::EntryType::ACPI_RECLAIMABLE |
                    limine::memory_map::EntryType::ACPI_NVS |
                    limine::memory_map::EntryType::BAD_MEMORY |
                    limine::memory_map::EntryType::RESERVED => {
                        memory_regions::MemoryRegionType::Reserved
                    }
                    limine::memory_map::EntryType::FRAMEBUFFER |
                    limine::memory_map::EntryType::KERNEL_AND_MODULES => {
                        memory_regions::MemoryRegionType::KernelData
                    }
                    _ => memory_regions::MemoryRegionType::Reserved,
                };
                
                let region = memory_regions::MemoryRegion::new(
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
                memory_regions::MemoryRegionType::Available
            ) + MEMORY_REGIONS.count_by_type(
                memory_regions::MemoryRegionType::Reserved
            ));
            
            // Print some region statistics
            for region_type in [
                memory_regions::MemoryRegionType::Available,
                memory_regions::MemoryRegionType::Reserved,
                memory_regions::MemoryRegionType::KernelData,
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
        let total_mem = unsafe { PMM.total_pages() * pmm::PAGE_SIZE };
        let used_mem = unsafe { PMM.used_pages() * pmm::PAGE_SIZE };
        memory_stats::stats().set_total_physical(total_mem);
        memory_stats::stats().set_used_physical(used_mem);
        
        // Print memory statistics
        arch::serial_println!("[Fanga] Memory Statistics:");
        arch::serial_println!("[Fanga]   Total Physical: {} MiB", total_mem / (1024 * 1024));
        arch::serial_println!("[Fanga]   Used Physical:  {} MiB", used_mem / (1024 * 1024));
        arch::serial_println!("[Fanga]   Free Physical:  {} MiB", 
            (total_mem - used_mem) / (1024 * 1024)
        );
        
        arch::serial_println!("[Fanga] PMM test completed ✅");
        
        // Test VMM (Virtual Memory Manager)
        arch::serial_println!("[Fanga] Testing VMM...");
        
        unsafe {
            // Create a new page table
            if let Some(mut mapper) = vmm::PageTableMapper::new(&mut PMM, hhdm.offset()) {
                arch::serial_println!("[Fanga] Created new page table at: 0x{:x}", mapper.pml4_addr());
                
                // Test mapping: map virtual address 0x1000_0000 to a physical page
                if let Some(test_phys) = PMM.alloc_page() {
                    arch::serial_println!("[Fanga] Allocated test page at: 0x{:x}", test_phys);
                    
                    let test_virt = 0x1000_0000u64;
                    let flags = vmm::PageTableFlags::PRESENT
                        .with(vmm::PageTableFlags::WRITABLE);
                    
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
                let current_cr3 = vmm::PageTableMapper::current_cr3();
                arch::serial_println!("[Fanga] Current CR3 (page table): 0x{:x}", current_cr3);
            } else {
                arch::serial_println!("[Fanga] Failed to create page table mapper");
            }
        }
    } else {
        arch::serial_println!("[Fanga] Cannot initialize PMM: missing memory map or HHDM");
    }

    // Framebuffer test: fill screen with a solid color
    // (ARGB) 0xFFRRGGBB
    fb_fill_color(0xFF1E1E2E); // dark-ish

    arch::serial_println!("[Fanga] framebuffer filled ✅");

    // Uncomment the following to test double fault handling:
    // This will trigger a stack overflow which causes a page fault,
    // and since the stack is corrupted, it will then trigger a double fault.
    // The double fault handler uses IST so it won't cascade into a triple fault.
    arch::serial_println!("[Fanga] Testing double fault handler...");
    // unsafe {
    //     fn trigger_stack_overflow() {
    //         // Infinite recursion to overflow the stack
    //         let x = [0u8; 4096];  // Use some stack space
    //         core::hint::black_box(&x); // Prevent optimization
    //         trigger_stack_overflow();
    //     }
    //     trigger_stack_overflow();
    // }

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
    arch::serial_println!("");
    arch::serial_println!("!!! KERNEL PANIC !!!");
    arch::serial_println!("{}", info);

    loop {
        // core::hint::spin_loop();
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
