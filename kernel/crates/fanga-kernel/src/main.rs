#![no_std]
#![no_main]

use core::panic::PanicInfo;

use limine::request::{RequestsEndMarker, RequestsStartMarker};
use limine::BaseRevision;

use fanga_arch_x86_64 as arch;

/* -------------------------------------------------------------------------- */
/*                          LIMINE REQUIRED MARKERS                            */
/* -------------------------------------------------------------------------- */

#[used]
#[link_section = ".limine_requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".limine_requests_start"]
static LIMINE_REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[link_section = ".limine_requests_end"]
static LIMINE_REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();

/* -------------------------------------------------------------------------- */
/*                               KERNEL ENTRY                                  */
/* -------------------------------------------------------------------------- */

#[no_mangle]
pub extern "C" fn _start() -> ! {
    arch::init();

    arch::serial_println!("");
    arch::serial_println!("====================================");
    arch::serial_println!("   Fanga Kernel booted successfully  ");
    arch::serial_println!("====================================");
    arch::serial_println!("");

    loop {
        core::hint::spin_loop();
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
        core::hint::spin_loop();
    }
}
