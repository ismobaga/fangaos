use core::arch::asm;

use crate::interrupts::pic;
use crate::serial_println;

pub const IDT_LEN: usize = 256;

// Common vectors
pub const VEC_BREAKPOINT: u8 = 3;
pub const VEC_DOUBLE_FAULT: u8 = 8;
pub const VEC_GENERAL_PROTECTION: u8 = 13;
pub const VEC_PAGE_FAULT: u8 = 14;

// PIC remap base
pub const PIC1_OFFSET: u8 = 32; // IRQ0..7  -> 32..39
pub const PIC2_OFFSET: u8 = 40; // IRQ8..15 -> 40..47

pub const IRQ_TIMER: u8 = 0;
pub const IRQ_KEYBOARD: u8 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    pub const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    pub fn set_handler(&mut self, handler: u64) {
        self.set_handler_with_ist(handler, 0);
    }

    pub fn set_handler_with_ist(&mut self, handler: u64, ist: u8) {
        self.offset_low = handler as u16;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;

        // Kernel CS selector (GDT code segment)
        self.selector = crate::gdt::KERNEL_CODE_SELECTOR;

        // IST index (0 = don't use IST, 1-7 = use IST entry)
        self.ist = ist;

        // 0x8E = present | DPL=0 | interrupt gate
        self.type_attr = 0x8E;

        self.zero = 0;
    }
}

#[repr(C, packed)]
pub struct Idtr {
    limit: u16,
    base: u64,
}

#[repr(C)]
pub struct InterruptStackFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

static mut IDT: [IdtEntry; IDT_LEN] = [IdtEntry::missing(); IDT_LEN];

#[inline(always)]
unsafe fn lidt(idtr: &Idtr) {
    asm!("lidt [{}]", in(reg) idtr, options(readonly, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn read_cr2() -> u64 {
    let value: u64;
    asm!("mov {}, cr2", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

// --------- Exception Handlers (x86-interrupt ABI) ---------

extern "x86-interrupt" fn breakpoint_handler(_frame: InterruptStackFrame) {
    serial_println!("[IDT] Breakpoint (#BP)");
}

extern "x86-interrupt" fn gp_fault_handler(frame: InterruptStackFrame, error_code: u64) {
    serial_println!("[IDT] General Protection Fault (#GP) ec=0x{:x}", error_code);
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, error_code: u64) -> ! {
    serial_println!("[IDT] Double Fault (#DF) ec=0x{:x}", error_code);
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn page_fault_handler(frame: InterruptStackFrame, error_code: u64) {
    let cr2 = unsafe { read_cr2() };
    serial_println!(
        "[IDT] Page Fault (#PF) ec=0x{:x} cr2=0x{:x}",
        error_code,
        cr2
    );
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

// --------- IRQ Handlers (PIC) ---------

extern "x86-interrupt" fn timer_irq_handler(_frame: InterruptStackFrame) {
    // Keep it quiet for now — printing every tick is noisy.
    unsafe {
        pic::eoi(IRQ_TIMER);
    }
}

extern "x86-interrupt" fn keyboard_irq_handler(_frame: InterruptStackFrame) {
    // Read scancode from PS/2 data port 0x60 (prevents stuck IRQ)
    let scancode = unsafe { crate::port::inb(0x60) };
    serial_println!("[IRQ] keyboard scancode=0x{:x}", scancode);
    unsafe {
        pic::eoi(IRQ_KEYBOARD);
    }
}

// --------- Public init ---------

pub fn init() {
    unsafe {
        // Fill all with "missing"
        for e in IDT.iter_mut() {
            *e = IdtEntry::missing();
        }

        // Exceptions
        IDT[VEC_BREAKPOINT as usize].set_handler(breakpoint_handler as u64);
        IDT[VEC_GENERAL_PROTECTION as usize].set_handler(gp_fault_handler as u64);
        IDT[VEC_PAGE_FAULT as usize].set_handler(page_fault_handler as u64);
        IDT[VEC_DOUBLE_FAULT as usize].set_handler_with_ist(double_fault_handler as u64, crate::gdt::DOUBLE_FAULT_IST_INDEX);

        // PIC remap + enable timer/keyboard only
        pic::remap(PIC1_OFFSET, PIC2_OFFSET);
        // Mask bits: 1 = masked(disabled). Enable IRQ0 & IRQ1 => mask others.
        pic::set_masks(0b1111_1100, 0b1111_1111);

        // IRQ handlers (after remap)
        IDT[(PIC1_OFFSET + IRQ_TIMER) as usize].set_handler(timer_irq_handler as u64);
        IDT[(PIC1_OFFSET + IRQ_KEYBOARD) as usize].set_handler(keyboard_irq_handler as u64);

        let idtr = Idtr {
            limit: (core::mem::size_of::<[IdtEntry; IDT_LEN]>() - 1) as u16,
            base: (&raw const IDT as *const _) as u64,
        };

        lidt(&idtr);
    }

    serial_println!("[IDT] loaded ✅");
}
