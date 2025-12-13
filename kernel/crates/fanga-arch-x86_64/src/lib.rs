#![no_std]
#![feature(abi_x86_interrupt)]

pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod port;
pub mod serial;
pub mod context;
pub mod syscall;

pub fn init() {
    serial::init();
    gdt::init();
    interrupts::idt::init();

    // Try to initialize APIC, fall back to PIC if not available
    match interrupts::apic::init() {
        Ok(()) => {
            serial_println!("[APIC] initialized ✅");
        }
        Err(e) => {
            serial_println!("[APIC] not available: {}, using PIC", e);
        }
    }

    // Initialize system call interface
    syscall::init();

    // Keep interrupts OFF for a moment? You can enable now if you want IRQs.
    unsafe {
        core::arch::asm!("sti");
    }
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {{
        $crate::serial::_print(core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! serial_println{
    () => {$crate::serial_print("\n")};
    ($fmt:expr) => {$crate::serial_print!(concat!($fmt, "\n"))};
    ($fmt:expr, $($arg:tt)*) => {$crate::serial_print!(concat!($fmt, "\n"), $($arg)*)};
}
