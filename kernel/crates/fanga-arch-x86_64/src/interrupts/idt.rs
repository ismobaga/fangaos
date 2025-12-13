use core::arch::asm;

use crate::interrupts::pic;
use crate::serial_println;

pub const IDT_LEN: usize = 256;

// Common vectors - CPU exceptions
pub const VEC_DIVIDE_ERROR: u8 = 0;
pub const VEC_DEBUG: u8 = 1;
pub const VEC_NMI: u8 = 2;
pub const VEC_BREAKPOINT: u8 = 3;
pub const VEC_OVERFLOW: u8 = 4;
pub const VEC_BOUND_RANGE: u8 = 5;
pub const VEC_INVALID_OPCODE: u8 = 6;
pub const VEC_DEVICE_NOT_AVAILABLE: u8 = 7;
pub const VEC_DOUBLE_FAULT: u8 = 8;
pub const VEC_COPROCESSOR_SEGMENT_OVERRUN: u8 = 9;
pub const VEC_INVALID_TSS: u8 = 10;
pub const VEC_SEGMENT_NOT_PRESENT: u8 = 11;
pub const VEC_STACK_FAULT: u8 = 12;
pub const VEC_GENERAL_PROTECTION: u8 = 13;
pub const VEC_PAGE_FAULT: u8 = 14;
// 15 is reserved
pub const VEC_X87_FPU: u8 = 16;
pub const VEC_ALIGNMENT_CHECK: u8 = 17;
pub const VEC_MACHINE_CHECK: u8 = 18;
pub const VEC_SIMD_FP: u8 = 19;
pub const VEC_VIRTUALIZATION: u8 = 20;
pub const VEC_CONTROL_PROTECTION: u8 = 21;

// PIC remap base
pub const PIC1_OFFSET: u8 = 32; // IRQ0..7  -> 32..39
pub const PIC2_OFFSET: u8 = 40; // IRQ8..15 -> 40..47

// IRQ numbers (hardware IRQs)
pub const IRQ_TIMER: u8 = 0;
pub const IRQ_KEYBOARD: u8 = 1;
pub const IRQ_CASCADE: u8 = 2; // Used internally by PICs
pub const IRQ_COM2: u8 = 3;
pub const IRQ_COM1: u8 = 4;
pub const IRQ_LPT2: u8 = 5;
pub const IRQ_FLOPPY: u8 = 6;
pub const IRQ_LPT1: u8 = 7;
pub const IRQ_RTC: u8 = 8;
pub const IRQ_FREE1: u8 = 9;
pub const IRQ_FREE2: u8 = 10;
pub const IRQ_FREE3: u8 = 11;
pub const IRQ_PS2_MOUSE: u8 = 12;
pub const IRQ_FPU: u8 = 13;
pub const IRQ_PRIMARY_ATA: u8 = 14;
pub const IRQ_SECONDARY_ATA: u8 = 15;

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
        self.ist = ist & 0x7;

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
#[derive(Clone, Copy)]
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

extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    serial_println!("[IDT] Divide Error (#DE)");
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn debug_handler(frame: InterruptStackFrame) {
    serial_println!("[IDT] Debug Exception (#DB)");
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
}

extern "x86-interrupt" fn nmi_handler(frame: InterruptStackFrame) {
    serial_println!("[IDT] Non-Maskable Interrupt (NMI)");
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
}

extern "x86-interrupt" fn breakpoint_handler(_frame: InterruptStackFrame) {
    serial_println!("[IDT] Breakpoint (#BP)");
}

extern "x86-interrupt" fn overflow_handler(frame: InterruptStackFrame) {
    serial_println!("[IDT] Overflow (#OF)");
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn bound_range_handler(frame: InterruptStackFrame) {
    serial_println!("[IDT] Bound Range Exceeded (#BR)");
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    serial_println!("[IDT] Invalid Opcode (#UD)");
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn device_not_available_handler(frame: InterruptStackFrame) {
    serial_println!("[IDT] Device Not Available (#NM)");
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn invalid_tss_handler(frame: InterruptStackFrame, error_code: u64) {
    serial_println!("[IDT] Invalid TSS (#TS) ec=0x{:x}", error_code);
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn segment_not_present_handler(frame: InterruptStackFrame, error_code: u64) {
    serial_println!("[IDT] Segment Not Present (#NP) ec=0x{:x}", error_code);
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn stack_fault_handler(frame: InterruptStackFrame, error_code: u64) {
    serial_println!("[IDT] Stack Fault (#SS) ec=0x{:x}", error_code);
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
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

    // Error code bits:
    // 0: P (present) - 0 = non-present page, 1 = protection violation
    // 1: W (write) - 0 = read access, 1 = write access
    // 2: U (user) - 0 = kernel mode, 1 = user mode
    // 3: R (reserved write) - 1 = reserved bits set in page table
    // 4: I (instruction fetch) - 1 = instruction fetch caused fault
    serial_println!("      Access: {} {} {}",
        if error_code & 0x1 != 0 { "protection-violation" } else { "non-present" },
        if error_code & 0x2 != 0 { "write" } else { "read" },
        if error_code & 0x4 != 0 { "user" } else { "kernel" }
    );

    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn x87_fpu_handler(frame: InterruptStackFrame) {
    serial_println!("[IDT] x87 FPU Exception (#MF)");
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn alignment_check_handler(frame: InterruptStackFrame, error_code: u64) {
    serial_println!("[IDT] Alignment Check (#AC) ec=0x{:x}", error_code);
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn machine_check_handler(frame: InterruptStackFrame) -> ! {
    serial_println!("[IDT] Machine Check (#MC)");
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn simd_fp_handler(frame: InterruptStackFrame) {
    serial_println!("[IDT] SIMD Floating Point (#XM/#XF)");
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn virtualization_handler(frame: InterruptStackFrame) {
    serial_println!("[IDT] Virtualization Exception (#VE)");
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

extern "x86-interrupt" fn control_protection_handler(frame: InterruptStackFrame, error_code: u64) {
    serial_println!("[IDT] Control Protection Exception (#CP) ec=0x{:x}", error_code);
    serial_println!("      rip=0x{:x} rflags=0x{:x}", frame.rip, frame.rflags);
    loop {
        unsafe {
            asm!("cli; hlt");
        }
    }
}

// --------- IRQ Handlers (PIC) ---------

static mut TIMER_TICKS: u64 = 0;

extern "x86-interrupt" fn timer_irq_handler(_frame: InterruptStackFrame) {
    unsafe {
        TIMER_TICKS = TIMER_TICKS.wrapping_add(1);
        // Print every ~18.2 seconds (PIT default frequency)
        if TIMER_TICKS % (18 * 1000) == 0 {
            // serial_println!("[IRQ] Timer tick: {}", TIMER_TICKS);
        }
        pic::eoi(IRQ_TIMER);
    }
}

extern "x86-interrupt" fn keyboard_irq_handler(_frame: InterruptStackFrame) {
    // Read scancode from PS/2 data port 0x60
    let kbd = crate::keyboard::keyboard();
    let scancode = kbd.read_scancode();
    
    if let Some(event) = kbd.process_scancode(scancode) {
        // Only handle key presses for now
        if let crate::keyboard::KeyEvent::Press(keycode) = event {
            if let Some(ascii) = kbd.to_ascii(keycode) {
                serial_println!("[Keyboard] '{}'", ascii);
            } else {
                serial_println!("[Keyboard] {:?}", keycode);
            }
        }
    }
    
    unsafe {
        pic::eoi(IRQ_KEYBOARD);
    }
}

// Generic spurious IRQ handler
extern "x86-interrupt" fn spurious_irq_handler(_frame: InterruptStackFrame) {
    serial_println!("[IRQ] Spurious interrupt detected");
    // Note: Don't send EOI for spurious interrupts from PIC
}

// --------- Public init ---------

pub fn init() {
    unsafe {
        // Fill all with "missing"
        for e in IDT.iter_mut() {
            *e = IdtEntry::missing();
        }

        // CPU Exceptions (0-21)
        IDT[VEC_DIVIDE_ERROR as usize].set_handler(divide_error_handler as u64);
        IDT[VEC_DEBUG as usize].set_handler(debug_handler as u64);
        IDT[VEC_NMI as usize].set_handler(nmi_handler as u64);
        IDT[VEC_BREAKPOINT as usize].set_handler(breakpoint_handler as u64);
        IDT[VEC_OVERFLOW as usize].set_handler(overflow_handler as u64);
        IDT[VEC_BOUND_RANGE as usize].set_handler(bound_range_handler as u64);
        IDT[VEC_INVALID_OPCODE as usize].set_handler(invalid_opcode_handler as u64);
        IDT[VEC_DEVICE_NOT_AVAILABLE as usize].set_handler(device_not_available_handler as u64);
        IDT[VEC_DOUBLE_FAULT as usize].set_handler_with_ist(
            double_fault_handler as u64,
            crate::gdt::DOUBLE_FAULT_IST_INDEX,
        );
        IDT[VEC_INVALID_TSS as usize].set_handler(invalid_tss_handler as u64);
        IDT[VEC_SEGMENT_NOT_PRESENT as usize].set_handler(segment_not_present_handler as u64);
        IDT[VEC_STACK_FAULT as usize].set_handler(stack_fault_handler as u64);
        IDT[VEC_GENERAL_PROTECTION as usize].set_handler(gp_fault_handler as u64);
        IDT[VEC_PAGE_FAULT as usize].set_handler(page_fault_handler as u64);
        IDT[VEC_X87_FPU as usize].set_handler(x87_fpu_handler as u64);
        IDT[VEC_ALIGNMENT_CHECK as usize].set_handler(alignment_check_handler as u64);
        IDT[VEC_MACHINE_CHECK as usize].set_handler(machine_check_handler as u64);
        IDT[VEC_SIMD_FP as usize].set_handler(simd_fp_handler as u64);
        IDT[VEC_VIRTUALIZATION as usize].set_handler(virtualization_handler as u64);
        IDT[VEC_CONTROL_PROTECTION as usize].set_handler(control_protection_handler as u64);

        // PIC remap + enable timer/keyboard only
        pic::remap(PIC1_OFFSET, PIC2_OFFSET);
        // Mask bits: 1 = masked(disabled). Enable IRQ0 & IRQ1 => mask others.
        pic::set_masks(0b1111_1100, 0b1111_1111);

        // IRQ handlers (after remap)
        IDT[(PIC1_OFFSET + IRQ_TIMER) as usize].set_handler(timer_irq_handler as u64);
        IDT[(PIC1_OFFSET + IRQ_KEYBOARD) as usize].set_handler(keyboard_irq_handler as u64);
        
        // Set spurious IRQ handler for PIC1 IRQ7 and PIC2 IRQ15
        IDT[(PIC1_OFFSET + 7) as usize].set_handler(spurious_irq_handler as u64);
        IDT[(PIC2_OFFSET + 15) as usize].set_handler(spurious_irq_handler as u64);

        let idtr = Idtr {
            limit: (core::mem::size_of::<[IdtEntry; IDT_LEN]>() - 1) as u16,
            base: (&raw const IDT as *const _) as u64,
        };

        lidt(&idtr);
    }

    serial_println!("[IDT] loaded with {} exception handlers ✅", 16);
}

/// Get the current timer tick count
pub fn timer_ticks() -> u64 {
    unsafe { TIMER_TICKS }
}
