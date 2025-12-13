use crate::port::{inb, outb};

const PIC1: u16 = 0x20;
const PIC2: u16 = 0xA0;

const PIC1_COMMAND: u16 = PIC1;
const PIC1_DATA: u16 = PIC1 + 1;
const PIC2_COMMAND: u16 = PIC2;
const PIC2_DATA: u16 = PIC2 + 1;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

#[inline(always)]
unsafe fn io_wait() {
    // Port 0x80 is traditionally used for 'wait'
    outb(0x80, 0);
}

pub unsafe fn remap(offset1: u8, offset2: u8) {
    let a1 = inb(PIC1_DATA);
    let a2 = inb(PIC2_DATA);

    outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();
    outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();

    outb(PIC1_DATA, offset1);
    io_wait();
    outb(PIC2_DATA, offset2);
    io_wait();

    outb(PIC1_DATA, 4); // PIC2 at IRQ2
    io_wait();
    outb(PIC2_DATA, 2);
    io_wait();

    outb(PIC1_DATA, ICW4_8086);
    io_wait();
    outb(PIC2_DATA, ICW4_8086);
    io_wait();

    // restore masks
    outb(PIC1_DATA, a1);
    outb(PIC2_DATA, a2);
}

pub unsafe fn set_masks(pic1_mask: u8, pic2_mask: u8) {
    outb(PIC1_DATA, pic1_mask);
    outb(PIC2_DATA, pic2_mask);
}

pub unsafe fn get_masks() -> (u8, u8) {
    (inb(PIC1_DATA), inb(PIC2_DATA))
}

pub unsafe fn mask_irq(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let value = inb(port) | (1 << (irq % 8));
    outb(port, value);
}

pub unsafe fn unmask_irq(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let value = inb(port) & !(1 << (irq % 8));
    outb(port, value);
}

pub unsafe fn eoi(irq: u8) {
    // If IRQ came from PIC2, we must ACK PIC2 as well
    if irq >= 8 {
        outb(PIC2_COMMAND, 0x20);
    }
    outb(PIC1_COMMAND, 0x20);
}
