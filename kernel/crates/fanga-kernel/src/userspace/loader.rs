//! User Binary Loader
//!
//! Loads ELF binaries and prepares them for execution in user mode.

use crate::elf::{load_elf, ElfLoadError, LoadedElf};
use crate::memory::{VirtAddr, PhysAddr};

/// Information about a loaded user binary
#[derive(Debug, Clone, Copy)]
pub struct UserBinaryInfo {
    /// Entry point to jump to
    pub entry_point: VirtAddr,
    /// User stack pointer
    pub stack_pointer: VirtAddr,
    /// Page table for the process
    pub page_table: PhysAddr,
}

/// Load a user binary from memory
///
/// # Arguments
/// * `binary_data` - The ELF binary data to load
/// * `stack_size` - Size of user stack to allocate (in bytes)
///
/// # Returns
/// Information needed to start the user binary
pub fn load_user_binary(
    binary_data: &[u8],
    stack_size: usize,
) -> Result<UserBinaryInfo, ElfLoadError> {
    // Parse and load the ELF binary
    let loaded_elf = load_elf(binary_data)?;

    // TODO: In a real implementation, we would:
    // 1. Create a new page table for the process
    // 2. Allocate and map physical pages for each PT_LOAD segment
    // 3. Copy data from the ELF file to the mapped pages
    // 4. Allocate and map a user stack
    // 5. Set up proper page permissions (user-accessible, read/write/exec)

    // For now, use a dummy stack address in user space
    // Real user space typically starts at 0x400000, stack grows down from high address
    let stack_top = VirtAddr::new(0x7fff_ffff_f000);

    // Use current page table (in reality we'd create a new one)
    let page_table = PhysAddr::new(0); // Placeholder

    Ok(UserBinaryInfo {
        entry_point: loaded_elf.entry_point,
        stack_pointer: stack_top,
        page_table,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_invalid_binary() {
        let data = [0u8; 32];
        assert!(load_user_binary(&data, 4096).is_err());
    }

    #[test]
    fn test_user_binary_info() {
        let info = UserBinaryInfo {
            entry_point: VirtAddr::new(0x400000),
            stack_pointer: VirtAddr::new(0x7fff_ffff_f000),
            page_table: PhysAddr::new(0),
        };

        assert_eq!(info.entry_point.as_u64(), 0x400000);
        assert_eq!(info.stack_pointer.as_u64(), 0x7fff_ffff_f000);
    }
}
