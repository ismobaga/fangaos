/// Advanced Programmable Interrupt Controller (APIC) support
///
/// This module provides basic Local APIC detection and initialization.
/// The APIC is the modern replacement for the legacy PIC (8259) and provides
/// better interrupt handling, support for multiple processors, and more features.

use crate::serial_println;
use core::arch::asm;
use spin::Once;

/// APIC base address (typically 0xFEE00000)
const APIC_BASE_MSR: u32 = 0x1B;

/// Local APIC register offsets
const APIC_ID: u32 = 0x020;
const APIC_EOI: u32 = 0x0B0; // End of Interrupt
const APIC_TIMER_INIT: u32 = 0x380; // Initial Count
const APIC_TIMER_CURRENT: u32 = 0x390; // Current Count
const APIC_TIMER_DIV: u32 = 0x3E0; // Divide Configuration

// These will be used when APIC MMIO is properly mapped
#[allow(dead_code)]
const APIC_VERSION: u32 = 0x030;
#[allow(dead_code)]
const APIC_TPR: u32 = 0x080; // Task Priority Register
#[allow(dead_code)]
const APIC_SPURIOUS: u32 = 0x0F0; // Spurious Interrupt Vector Register
#[allow(dead_code)]
const APIC_LVT_TIMER: u32 = 0x320; // Local Vector Table Timer
#[allow(dead_code)]
const APIC_LVT_LINT0: u32 = 0x350; // Local Vector Table LINT0
#[allow(dead_code)]
const APIC_LVT_LINT1: u32 = 0x360; // Local Vector Table LINT1
#[allow(dead_code)]
const APIC_LVT_ERROR: u32 = 0x370; // Local Vector Table Error

/// APIC state
pub struct Apic {
    base_addr: u64,
    enabled: bool,
}

impl Apic {
    /// Create a new APIC instance (not yet initialized)
    pub const fn new() -> Self {
        Self {
            base_addr: 0,
            enabled: false,
        }
    }

    /// Check if APIC is supported by the CPU
    pub fn is_supported() -> bool {
        let cpuid_result: u32;
        unsafe {
            asm!(
                "mov eax, 1",
                "cpuid",
                "mov {0:e}, edx",
                out(reg) cpuid_result,
                out("eax") _,
                out("ecx") _,
                out("edx") _,
            );
        }
        // Bit 9 of EDX indicates APIC support
        (cpuid_result & (1 << 9)) != 0
    }

    /// Read the APIC base address from MSR
    fn read_base_address() -> u64 {
        unsafe {
            let low: u32;
            let high: u32;
            asm!(
                "rdmsr",
                in("ecx") APIC_BASE_MSR,
                out("eax") low,
                out("edx") high,
            );
            ((high as u64) << 32) | (low as u64)
        }
    }

    /// Write to an APIC register
    unsafe fn write_register(&self, offset: u32, value: u32) {
        let addr = (self.base_addr & 0xFFFF_F000) + offset as u64;
        core::ptr::write_volatile(addr as *mut u32, value);
    }

    /// Read from an APIC register
    unsafe fn read_register(&self, offset: u32) -> u32 {
        let addr = (self.base_addr & 0xFFFF_F000) + offset as u64;
        core::ptr::read_volatile(addr as *const u32)
    }

    /// Initialize the Local APIC
    pub fn init(&mut self) -> Result<(), &'static str> {
        if !Self::is_supported() {
            return Err("APIC not supported by CPU");
        }

        // Read APIC base address from MSR
        self.base_addr = Self::read_base_address();
        
        // Check if APIC is enabled in the MSR
        let apic_enabled = (self.base_addr & (1 << 11)) != 0;
        let base_addr_only = self.base_addr & 0xFFFF_F000;

        serial_println!("[APIC] Base address: 0x{:x}", base_addr_only);
        serial_println!("[APIC] Enabled in MSR: {}", apic_enabled);
        serial_println!("[APIC] Note: MMIO access disabled until memory mapping is implemented");

        // TODO: Map APIC memory region before accessing registers
        // For now, we just detect APIC but don't enable it
        // This requires identity mapping or page table setup for 0xFEE00000

        // Mark as not enabled since we can't safely access MMIO yet
        self.enabled = false;
        Ok(())
    }

    /// Send End of Interrupt (EOI) signal
    pub fn eoi(&self) {
        if self.enabled {
            unsafe {
                self.write_register(APIC_EOI, 0);
            }
        }
    }

    /// Check if APIC is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get APIC ID of current processor
    pub fn get_id(&self) -> u8 {
        if self.enabled {
            unsafe {
                (self.read_register(APIC_ID) >> 24) as u8
            }
        } else {
            0
        }
    }
}

/// Global APIC instance
static LOCAL_APIC: Once<Apic> = Once::new();

/// Initialize the Local APIC
pub fn init() -> Result<(), &'static str> {
    let mut apic = Apic::new();
    let result = apic.init();
    LOCAL_APIC.call_once(|| apic);
    result
}

/// Get a reference to the Local APIC
pub fn local_apic() -> Option<&'static Apic> {
    LOCAL_APIC.get()
}

/// Check if APIC is available and enabled
pub fn is_available() -> bool {
    LOCAL_APIC.get().map_or(false, |apic: &Apic| apic.is_enabled())
}

/// Send EOI using APIC (if available) or fall back to PIC
pub fn eoi(irq: u8) {
    match LOCAL_APIC.get() {
        Some(apic) if apic.is_enabled() => apic.eoi(),
        _ => unsafe { crate::interrupts::pic::eoi(irq); }
    }
}
