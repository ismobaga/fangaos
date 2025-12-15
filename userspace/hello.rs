#![no_std]
#![no_main]

// Import our minimal libc
mod libc;
use libc::{println, exit};

// Panic handler (required for no_std)
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(1);
}

// Entry point - this is called from _start
#[no_mangle]
pub extern "C" fn main() -> i32 {
    println("Hello from user space!");
    println("This is a user-mode application running in FangaOS.");
    0
}

// _start is the actual entry point
// It sets up the stack and calls main
#[no_mangle]
#[link_section = ".text.start"]
pub extern "C" fn _start() -> ! {
    let exit_code = main();
    exit(exit_code);
}
