use core::arch::asm;

/// Writes a byte to the specified I/O port.
///
/// # Safety
/// This function is unsafe because writing to arbitrary I/O ports can cause
/// undefined behavior, system instability, or hardware damage if the port
/// and value are not valid for the system.
pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port , in("al") value, options(nomem, nostack, preserves_flags));
}

/// Reads a byte from the specified I/O port.
///
/// # Safety
/// This function is unsafe because reading from arbitrary I/O ports can cause
/// undefined behavior or system instability if the port is not valid for the system.
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}
