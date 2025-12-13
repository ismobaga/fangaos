/// Advanced Programmable Interrupt Controller (APIC) support
///
/// This module provides basic Local APIC detection and initialization.
/// The APIC is the modern replacement for the legacy PIC (8259) and provides
/// better interrupt handling, support for multiple processors, and more features.

use crate::serial_println;
use core::arch::asm;

/// APIC base address (typically 0xFEE00000)
const APIC_BASE_MSR: u32 = 0x1B;

/// Local APIC register offsets
const APIC_ID: u32 = 0x020;
const APIC_VERSION: u32 = 0x030;
const APIC_TPR: u32 = 0x080; // Task Priority Register
const APIC_EOI: u32 = 0x0B0; // End of Interrupt
const APIC_SPURIOUS: u32 = 0x0F0; // Spurious Interrupt Vector Register
const APIC_LVT_TIMER: u32 = 0x320; // Local Vector Table Timer
const APIC_LVT_LINT0: u32 = 0x350; // Local Vector Table LINT0
const APIC_LVT_LINT1: u32 = 0x360; // Local Vector Table LINT1
const APIC_LVT_ERROR: u32 = 0x370; // Local Vector Table Error
const APIC_TIMER_INIT: u32 = 0x380; // Initial Count
const APIC_TIMER_CURRENT: u32 = 0x390; // Current Count
const APIC_TIMER_DIV: u32 = 0x3E0; // Divide Configuration

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

        unsafe {
            // Enable APIC by setting bit 8 in Spurious Interrupt Vector Register
            // Also set the spurious vector to 255
            let spurious = self.read_register(APIC_SPURIOUS);
            self.write_register(APIC_SPURIOUS, spurious | 0x100 | 0xFF);

            // Set Task Priority Register to 0 (accept all interrupts)
            self.write_register(APIC_TPR, 0);

            // Mask all LVT entries initially
            self.write_register(APIC_LVT_TIMER, 0x10000); // Masked
            self.write_register(APIC_LVT_LINT0, 0x10000); // Masked
            self.write_register(APIC_LVT_LINT1, 0x10000); // Masked
            self.write_register(APIC_LVT_ERROR, 0x10000); // Masked

            // Read APIC ID and version
            let apic_id = self.read_register(APIC_ID) >> 24;
            let apic_version = self.read_register(APIC_VERSION) & 0xFF;

            serial_println!("[APIC] ID: {}", apic_id);
            serial_println!("[APIC] Version: 0x{:x}", apic_version);
        }

        self.enabled = true;
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
static mut LOCAL_APIC: Apic = Apic::new();

/// Initialize the Local APIC
pub fn init() -> Result<(), &'static str> {
    unsafe {
        LOCAL_APIC.init()
    }
}

/// Get a reference to the Local APIC
pub fn local_apic() -> &'static Apic {
    unsafe { &LOCAL_APIC }
}

/// Check if APIC is available and enabled
pub fn is_available() -> bool {
    unsafe { LOCAL_APIC.is_enabled() }
}

/// Send EOI using APIC (if available) or fall back to PIC
pub fn eoi(irq: u8) {
    unsafe {
        if LOCAL_APIC.is_enabled() {
            LOCAL_APIC.eoi();
        } else {
            crate::interrupts::pic::eoi(irq);
        }
    }
}
