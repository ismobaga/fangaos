#![no_std]
#![no_main]

use core::panic::PanicInfo;

use limine::request::{
    BootloaderInfoRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest, RequestsEndMarker,
    RequestsStartMarker,
};
use limine::BaseRevision;

use fanga_arch_x86_64 as arch;

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
        core::arch::asm!("int3");
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
            let base = entry.base;
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

    // Framebuffer test: fill screen with a solid color
    // (ARGB) 0xFFRRGGBB
    fb_fill_color(0xFF1E1E2E); // dark-ish

    arch::serial_println!("[Fanga] framebuffer filled ✅");

    // Uncomment the following to test double fault handling:
    // This will trigger a stack overflow which causes a page fault,
    // and since the stack is corrupted, it will then trigger a double fault.
    // The double fault handler uses IST so it won't cascade into a triple fault.
    // arch::serial_println!("[Fanga] Testing double fault handler...");
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
