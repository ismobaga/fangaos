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

/// Writes a word (16-bit) to the specified I/O port.
///
/// # Safety
/// This function is unsafe because writing to arbitrary I/O ports can cause
/// undefined behavior, system instability, or hardware damage if the port
/// and value are not valid for the system.
pub unsafe fn outw(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
}

/// Reads a word (16-bit) from the specified I/O port.
///
/// # Safety
/// This function is unsafe because reading from arbitrary I/O ports can cause
/// undefined behavior or system instability if the port is not valid for the system.
pub unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    asm!("in ax, dx", in("dx") port, out("ax") value, options(nomem, nostack, preserves_flags));
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_operations_compile() {
        // This test verifies that the port operations compile correctly.
        // Actual execution requires hardware or QEMU, so we just verify the API.
        
        // These operations would be unsafe to actually execute in a test environment
        // without proper hardware or emulation context, but we can verify the
        // function signatures and basic properties.
        
        // Verify that port numbers are u16
        let _port: u16 = 0x3F8;
        
        // Verify that byte values are u8
        let _value: u8 = 0x42;
        
        // Functions should be available (this ensures they're public)
        let _outb_fn = outb;
        let _inb_fn = inb;
        let _outw_fn = outw;
        let _inw_fn = inw;
    }
}
