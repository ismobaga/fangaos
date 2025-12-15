//! ELF Binary Loader
//!
//! This module loads ELF binaries into memory and prepares them for execution.

use super::parser::{ElfHeader, ElfProgramHeader, ProgramType};
use crate::memory::{VirtAddr, PhysAddr};
use core::mem::size_of;

/// Error type for ELF loading
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfLoadError {
    /// Invalid ELF format
    InvalidFormat,
    /// Unsupported ELF type
    UnsupportedType,
    /// Out of memory
    OutOfMemory,
    /// Invalid address
    InvalidAddress,
    /// Failed to map memory
    MappingFailed,
}

/// Information about a loaded ELF binary
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadedElf {
    /// Entry point virtual address
    pub entry_point: VirtAddr,
    /// Start of loaded segments (lowest address)
    pub load_start: VirtAddr,
    /// End of loaded segments (highest address)
    pub load_end: VirtAddr,
    /// Dynamic linker path (if PT_INTERP present)
    pub interp: Option<VirtAddr>,
}

/// Load an ELF binary from memory
///
/// # Arguments
/// * `data` - The ELF binary data
///
/// # Returns
/// Information about the loaded ELF binary
pub fn load_elf(data: &[u8]) -> Result<LoadedElf, ElfLoadError> {
    // Parse ELF header
    let header = ElfHeader::parse(data)
        .map_err(|_| ElfLoadError::InvalidFormat)?;

    // Validate header
    header.validate()
        .map_err(|_| ElfLoadError::UnsupportedType)?;

    // Track address range
    let mut load_start = VirtAddr::new(u64::MAX);
    let mut load_end = VirtAddr::new(0);
    let mut interp = None;

    // Process program headers
    let phoff = header.e_phoff as usize;
    let phentsize = header.e_phentsize as usize;
    let phnum = header.e_phnum as usize;

    for i in 0..phnum {
        let offset = phoff + i * phentsize;
        if offset + size_of::<ElfProgramHeader>() > data.len() {
            return Err(ElfLoadError::InvalidFormat);
        }

        let ph_data = &data[offset..offset + size_of::<ElfProgramHeader>()];
        let phdr = ElfProgramHeader::parse(ph_data)
            .map_err(|_| ElfLoadError::InvalidFormat)?;

        match phdr.program_type() {
            Some(ProgramType::Load) => {
                // Load segment into memory
                load_segment(&phdr, data)?;

                // Update address range
                let seg_start = VirtAddr::new(phdr.p_vaddr);
                let seg_end = VirtAddr::new(phdr.p_vaddr + phdr.p_memsz);

                if seg_start < load_start {
                    load_start = seg_start;
                }
                if seg_end > load_end {
                    load_end = seg_end;
                }
            }
            Some(ProgramType::Interp) => {
                // Store interpreter path location
                interp = Some(VirtAddr::new(phdr.p_vaddr));
            }
            _ => {
                // Ignore other segment types for now
            }
        }
    }

    // If no segments were loaded, that's an error
    if load_start.as_u64() == u64::MAX {
        return Err(ElfLoadError::InvalidFormat);
    }

    Ok(LoadedElf {
        entry_point: VirtAddr::new(header.e_entry),
        load_start,
        load_end,
        interp,
    })
}

/// Load a single PT_LOAD segment into memory
fn load_segment(phdr: &ElfProgramHeader, data: &[u8]) -> Result<(), ElfLoadError> {
    // Validate segment
    if phdr.p_filesz > phdr.p_memsz {
        return Err(ElfLoadError::InvalidFormat);
    }

    let offset = phdr.p_offset as usize;
    let filesz = phdr.p_filesz as usize;
    let memsz = phdr.p_memsz as usize;

    // Validate file offset and size
    if offset + filesz > data.len() {
        return Err(ElfLoadError::InvalidFormat);
    }

    // TODO: In a real implementation, we would:
    // 1. Allocate physical pages for the segment
    // 2. Map the pages into virtual memory at p_vaddr
    // 3. Copy data from the ELF file to the mapped memory
    // 4. Zero-fill the remainder (memsz - filesz)
    // 5. Set appropriate page permissions based on p_flags

    // For now, we just validate and return success
    // The actual memory allocation and mapping will be done when
    // we integrate this with the memory management system

    #[cfg(not(test))]
    {
        use crate::serial_println;
        serial_println!(
            "[ELF] Loading segment: vaddr={:#x}, filesz={:#x}, memsz={:#x}, flags={:#x}",
            phdr.p_vaddr, phdr.p_filesz, phdr.p_memsz, phdr.p_flags
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a minimal valid ELF header
    fn create_test_elf_header() -> [u8; 64] {
        let mut header = [0u8; 64];
        // Magic
        header[0..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']);
        // Class (64-bit)
        header[4] = 2;
        // Data (little-endian)
        header[5] = 1;
        // Version
        header[6] = 1;
        // Type (executable) - little-endian u16 at offset 16
        header[16] = 2;
        header[17] = 0;
        // Machine (x86_64) - little-endian u16 at offset 18
        header[18] = 62;
        header[19] = 0;
        // Version - little-endian u32 at offset 20
        header[20] = 1;
        header[21] = 0;
        header[22] = 0;
        header[23] = 0;
        // Entry point - little-endian u64 at offset 24
        header[24..32].copy_from_slice(&0x400000u64.to_le_bytes());
        // No program headers for this test
        // Program header offset at offset 32
        header[32..40].copy_from_slice(&0u64.to_le_bytes());
        // Section header offset at offset 40
        header[40..48].copy_from_slice(&0u64.to_le_bytes());
        // Header size at offset 52
        header[52] = 64;
        header[53] = 0;
        // Program header entry size at offset 54
        header[54] = 56;
        header[55] = 0;
        // Program header count at offset 56
        header[56] = 0;
        header[57] = 0;

        header
    }

    #[test]
    fn test_load_elf_invalid_data() {
        let data = [0u8; 32];
        assert_eq!(load_elf(&data), Err(ElfLoadError::InvalidFormat));
    }

    #[test]
    fn test_load_elf_no_segments() {
        let header = create_test_elf_header();
        // This header has no program headers, should fail
        assert_eq!(load_elf(&header), Err(ElfLoadError::UnsupportedType));
    }

    #[test]
    fn test_loaded_elf_struct() {
        let loaded = LoadedElf {
            entry_point: VirtAddr::new(0x400000),
            load_start: VirtAddr::new(0x400000),
            load_end: VirtAddr::new(0x401000),
            interp: None,
        };

        assert_eq!(loaded.entry_point.as_u64(), 0x400000);
        assert_eq!(loaded.load_start.as_u64(), 0x400000);
        assert_eq!(loaded.load_end.as_u64(), 0x401000);
        assert!(loaded.interp.is_none());
    }
}
