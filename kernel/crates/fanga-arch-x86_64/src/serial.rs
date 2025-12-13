use core::fmt::{self, Write};
use super::port::{inb, outb};

const COM1: u16 = 0x3F8;

pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1 + 0, 0x03);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xC7);
        outb(COM1 + 4, 0x0B);
    }
}

fn tx_empty() -> bool {
    unsafe { (inb(COM1 + 5) & 0x20) != 0 }
}

fn write_byte(b: u8) {
    while !tx_empty() {
        core::hint::spin_loop();
    }
    unsafe { outb(COM1, b) }
}

struct Serial;

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' { write_byte(b'\r'); }
            write_byte(b);
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    let _ = Serial.write_fmt(args);
}

