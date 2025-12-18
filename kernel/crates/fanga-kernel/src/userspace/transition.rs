//! User Mode Transition
//!
//! This module handles transitioning from kernel mode to user mode.

use crate::memory::VirtAddr;

/// Prepare a user mode stack with arguments
///
/// # Arguments
/// * `stack_top` - Top of the user stack
/// * `argc` - Number of arguments
/// * `argv` - Array of argument string pointers
///
/// # Returns
/// The adjusted stack pointer with arguments pushed
pub fn prepare_usermode_stack(
    stack_top: VirtAddr,
    _argc: usize,
    _argv: &[*const u8],
) -> VirtAddr {
    // TODO: In a real implementation, we would:
    // 1. Push environment variables
    // 2. Push argv array
    // 3. Push argc
    // 4. Align stack to 16 bytes (required by System V ABI)

    // Stack alignment mask (16 bytes)
    const STACK_ALIGNMENT_MASK: u64 = 0xF;

    // For now, just return the stack top aligned
    let aligned = stack_top.as_u64() & !STACK_ALIGNMENT_MASK;
    VirtAddr::new(aligned)
}

/// Enter user mode and start executing at the given entry point
///
/// This function does not return normally - it switches to user mode.
///
/// # Arguments
/// * `entry_point` - Virtual address to jump to in user mode
/// * `stack_pointer` - User stack pointer
///
/// # Safety
/// This function is unsafe because it changes privilege level and
/// switches to user mode code. The caller must ensure:
/// - The entry point is valid user code
/// - The stack pointer points to valid user stack
/// - Page tables are set up correctly for user space
pub unsafe fn enter_usermode(entry_point: VirtAddr, stack_pointer: VirtAddr) -> ! {
    #[cfg(not(test))]
    {
        use fanga_arch_x86_64::gdt::{USER_CODE_SELECTOR, USER_DATA_SELECTOR};
        
        fanga_arch_x86_64::serial_println!(
            "[USERMODE] Entering user mode: entry={:#x}, stack={:#x}",
            entry_point.as_u64(),
            stack_pointer.as_u64()
        );

        // Prepare user mode segment selectors with RPL=3 (user privilege)
        let user_cs = (USER_CODE_SELECTOR | 3) as u64;
        let user_ss = (USER_DATA_SELECTOR | 3) as u64;

        // Use IRET to switch to user mode
        // IRET expects the following on the stack (in this order, from low to high addresses):
        // 1. RIP (instruction pointer)
        // 2. CS (code segment)
        // 3. RFLAGS
        // 4. RSP (stack pointer)
        // 5. SS (stack segment)
        
        // RFLAGS bits
        const RFLAGS_RESERVED: u64 = 1 << 1; // Bit 1 is always 1
        const RFLAGS_IF: u64 = 1 << 9;       // Interrupt Enable Flag
        let rflags: u64 = RFLAGS_RESERVED | RFLAGS_IF;

        core::arch::asm!(
            // Push SS
            "push {ss}",
            // Push RSP
            "push {rsp}",
            // Push RFLAGS
            "push {rflags}",
            // Push CS
            "push {cs}",
            // Push RIP
            "push {rip}",
            // Execute IRET
            "iretq",
            ss = in(reg) user_ss,
            rsp = in(reg) stack_pointer.as_u64(),
            rflags = in(reg) rflags,
            cs = in(reg) user_cs,
            rip = in(reg) entry_point.as_u64(),
            options(noreturn)
        );
    }

    #[cfg(test)]
    {
        // In test mode, we can't actually enter user mode
        panic!("enter_usermode should not be called in test mode");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_usermode_stack() {
        let stack_top = VirtAddr::new(0x7fff_ffff_f000);
        let argv = [];
        let stack = prepare_usermode_stack(stack_top, 0, &argv);
        
        // Should be aligned to 16 bytes
        assert_eq!(stack.as_u64() & 0xF, 0);
    }

    #[test]
    fn test_prepare_usermode_stack_alignment() {
        // Test with unaligned address
        let stack_top = VirtAddr::new(0x7fff_ffff_f008);
        let argv = [];
        let stack = prepare_usermode_stack(stack_top, 0, &argv);
        
        // Should be aligned down to 16 bytes
        assert_eq!(stack.as_u64() & 0xF, 0);
        assert!(stack.as_u64() <= stack_top.as_u64());
    }
}
