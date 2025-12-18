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

mod boot;
mod io;
mod memory;
mod shell;
mod task;
mod power;

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
/*                               KERNEL ENTRY                                  */
/* -------------------------------------------------------------------------- */

/// Kernel entry point (_start)
///
/// This is the very first code that runs after the bootloader hands control
/// to the kernel. It follows a clean, structured boot sequence organized into
/// distinct phases for maintainability and clarity.
///
/// # Boot Sequence
///
/// The boot process is handled by the `boot` module and consists of:
/// 1. Early boot (architecture init)
/// 2. Bootloader protocol processing
/// 3. Memory subsystem initialization
/// 4. Driver initialization
/// 5. Kernel subsystem initialization
/// 6. Post-initialization and demonstrations
///
/// See the `boot` module for detailed documentation of each phase.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Execute the structured boot sequence
    match boot::initialize(
        &FRAMEBUFFER_REQ,
        &BOOTLOADER_INFO_REQ,
        &MEMMAP_REQ,
        &HHDM_REQ,
        &BASE_REVISION,
    ) {
        Ok(()) => {
            // Boot successful - enter idle loop
            arch::serial_println!("[Kernel] Boot complete, entering idle loop");
        }
        Err(e) => {
            // Boot failed - print error and halt
            arch::serial_println!("[Kernel] BOOT FAILED: {}", e);
            console_println!();
            console_println!("!!! KERNEL BOOT FAILURE !!!");
            console_println!("Error: {}", e);
        }
    }

    // Infinite idle loop - the CPU will be woken by interrupts
    loop {
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
