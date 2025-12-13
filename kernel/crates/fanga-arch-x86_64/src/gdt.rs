use core::arch::asm;
use core::mem::size_of;

/// GDT Entry structure (8 bytes)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    pub const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }

    pub const fn new(base: u32, limit: u32, access: u8, granularity: u8) -> Self {
        Self {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_mid: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: (granularity & 0xF0) | (((limit >> 16) & 0x0F) as u8),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

/// TSS Entry structure (16 bytes in 64-bit mode)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct TssEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
    base_upper: u32,
    reserved: u32,
}

impl TssEntry {
    pub const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
            base_upper: 0,
            reserved: 0,
        }
    }

    pub fn new(base: u64, limit: u32) -> Self {
        Self {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_mid: ((base >> 16) & 0xFF) as u8,
            access: 0x89, // Present, DPL=0, TSS Available (64-bit)
            granularity: ((limit >> 16) & 0x0F) as u8,
            base_high: ((base >> 24) & 0xFF) as u8,
            base_upper: ((base >> 32) & 0xFFFFFFFF) as u32,
            reserved: 0,
        }
    }
}

/// Task State Segment (64-bit)
#[repr(C, packed)]
pub struct Tss {
    _reserved1: u32,
    /// Privilege Stack Table - RSP values for privilege levels 0-2
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    _reserved2: u64,
    /// Interrupt Stack Table - 7 separate stacks for interrupts
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    _reserved3: u64,
    _reserved4: u16,
    pub iomap_base: u16,
}

impl Tss {
    pub const fn new() -> Self {
        Self {
            _reserved1: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            _reserved2: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            _reserved3: 0,
            _reserved4: 0,
            iomap_base: size_of::<Tss>() as u16,
        }
    }
}

/// GDTR structure
#[repr(C, packed)]
pub struct Gdtr {
    limit: u16,
    base: u64,
}

/// Global Descriptor Table with 6 entries:
/// 0: Null descriptor
/// 1: Kernel code segment
/// 2: Kernel data segment
/// 3: User code segment (for future use)
/// 4: User data segment (for future use)
/// 5-6: TSS descriptor (takes 2 entries in 64-bit mode)
#[repr(C, align(16))]
struct GdtTable {
    null: GdtEntry,
    kernel_code: GdtEntry,
    kernel_data: GdtEntry,
    user_code: GdtEntry,
    user_data: GdtEntry,
    tss: TssEntry,
}

// Static GDT and TSS
static mut GDT: GdtTable = GdtTable {
    null: GdtEntry::null(),
    // Kernel Code: base=0, limit=0xFFFFF, access=0x9A (present, DPL=0, code, readable)
    // granularity=0xA0 (4KB granularity, 64-bit mode)
    kernel_code: GdtEntry::new(0, 0xFFFFF, 0x9A, 0xA0),
    // Kernel Data: base=0, limit=0xFFFFF, access=0x92 (present, DPL=0, data, writable)
    // granularity=0xC0 (4KB granularity, 32-bit)
    kernel_data: GdtEntry::new(0, 0xFFFFF, 0x92, 0xC0),
    // User Code: access=0xFA (present, DPL=3, code, readable)
    user_code: GdtEntry::new(0, 0xFFFFF, 0xFA, 0xA0),
    // User Data: access=0xF2 (present, DPL=3, data, writable)
    user_data: GdtEntry::new(0, 0xFFFFF, 0xF2, 0xC0),
    tss: TssEntry::null(),
};

static mut TSS: Tss = Tss::new();

// Double fault stack - 128KB should be enough for double fault handling
const DOUBLE_FAULT_STACK_SIZE: usize = 32 * 4096;
static mut DOUBLE_FAULT_STACK: [u8; DOUBLE_FAULT_STACK_SIZE] = [0; DOUBLE_FAULT_STACK_SIZE];

/// GDT segment selectors
pub const KERNEL_CODE_SELECTOR: u16 = 0x08; // offset 1 * 8
pub const KERNEL_DATA_SELECTOR: u16 = 0x10; // offset 2 * 8
pub const TSS_SELECTOR: u16 = 0x28; // offset 5 * 8

/// IST index for double fault (1-based)
pub const DOUBLE_FAULT_IST_INDEX: u8 = 1;

#[inline(always)]
unsafe fn lgdt(gdtr: &Gdtr) {
    asm!("lgdt [{}]", in(reg) gdtr, options(readonly, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn ltr(selector: u16) {
    asm!("ltr {:x}", in(reg) selector, options(nostack, preserves_flags));
}

pub fn init() {
    unsafe {
        // Set up the double fault stack in IST1
        // The stack grows downward, so we need the address after the last byte
        let stack_start = &raw const DOUBLE_FAULT_STACK as *const u8 as u64;
        let stack_top = stack_start + DOUBLE_FAULT_STACK_SIZE as u64;
        TSS.ist1 = stack_top;

        // Create TSS descriptor
        let tss_addr = &raw const TSS as *const _ as u64;
        let tss_limit = size_of::<Tss>() as u32 - 1;
        GDT.tss = TssEntry::new(tss_addr, tss_limit);

        // Load GDT
        let gdtr = Gdtr {
            limit: (size_of::<GdtTable>() - 1) as u16,
            base: &raw const GDT as *const _ as u64,
        };
        lgdt(&gdtr);

        // Reload segment registers
        // CS is reloaded via far return
        // Note: This modifies RSP by pushing/popping values
        asm!(
            "push {sel}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            sel = in(reg) KERNEL_CODE_SELECTOR as u64,
            tmp = lateout(reg) _,
        );

        // Reload data segments
        asm!(
            "mov ds, {0:x}",
            "mov es, {0:x}",
            "mov fs, {0:x}",
            "mov gs, {0:x}",
            "mov ss, {0:x}",
            in(reg) KERNEL_DATA_SELECTOR,
            options(nostack, preserves_flags),
        );

        // Load TSS
        ltr(TSS_SELECTOR);
    }

    crate::serial_println!("[GDT] loaded with TSS ✅");
}
