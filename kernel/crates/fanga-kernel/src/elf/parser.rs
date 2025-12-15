//! ELF Format Parser
//!
//! This module parses ELF binary headers and provides structures for
//! reading ELF files.

use core::mem::size_of;

/// ELF Magic bytes
pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

/// ELF Class - 32-bit or 64-bit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ElfClass {
    None = 0,
    Elf32 = 1,
    Elf64 = 2,
}

/// ELF Data encoding - Little or Big Endian
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ElfData {
    None = 0,
    Little = 1, // Little-endian
    Big = 2,    // Big-endian
}

/// ELF Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ElfType {
    None = 0,
    Relocatable = 1, // ET_REL
    Executable = 2,  // ET_EXEC
    Shared = 3,      // ET_DYN
    Core = 4,        // ET_CORE
}

/// ELF Machine architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ElfMachine {
    None = 0,
    X86 = 3,      // EM_386
    X86_64 = 62,  // EM_X86_64
    Arm = 40,     // EM_ARM
    Aarch64 = 183, // EM_AARCH64
}

/// Program header type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ProgramType {
    Null = 0,         // PT_NULL
    Load = 1,         // PT_LOAD
    Dynamic = 2,      // PT_DYNAMIC
    Interp = 3,       // PT_INTERP
    Note = 4,         // PT_NOTE
    Shlib = 5,        // PT_SHLIB
    Phdr = 6,         // PT_PHDR
    Tls = 7,          // PT_TLS
    GnuStack = 0x6474e551, // PT_GNU_STACK
}

impl ProgramType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(ProgramType::Null),
            1 => Some(ProgramType::Load),
            2 => Some(ProgramType::Dynamic),
            3 => Some(ProgramType::Interp),
            4 => Some(ProgramType::Note),
            5 => Some(ProgramType::Shlib),
            6 => Some(ProgramType::Phdr),
            7 => Some(ProgramType::Tls),
            0x6474e551 => Some(ProgramType::GnuStack),
            _ => None,
        }
    }
}

/// Program header flags
pub mod program_flags {
    pub const PF_X: u32 = 1; // Executable
    pub const PF_W: u32 = 2; // Writable
    pub const PF_R: u32 = 4; // Readable
}

/// ELF Header (64-bit)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ElfHeader {
    /// ELF identification
    pub e_ident: [u8; 16],
    /// Object file type
    pub e_type: u16,
    /// Machine architecture
    pub e_machine: u16,
    /// Object file version
    pub e_version: u32,
    /// Entry point virtual address
    pub e_entry: u64,
    /// Program header table file offset
    pub e_phoff: u64,
    /// Section header table file offset
    pub e_shoff: u64,
    /// Processor-specific flags
    pub e_flags: u32,
    /// ELF header size in bytes
    pub e_ehsize: u16,
    /// Program header table entry size
    pub e_phentsize: u16,
    /// Program header table entry count
    pub e_phnum: u16,
    /// Section header table entry size
    pub e_shentsize: u16,
    /// Section header table entry count
    pub e_shnum: u16,
    /// Section header string table index
    pub e_shstrndx: u16,
}

impl ElfHeader {
    /// Parse an ELF header from a byte slice
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < size_of::<ElfHeader>() {
            return Err("Data too small for ELF header");
        }

        // Safety: We've checked the size, and we use read_unaligned to handle potentially unaligned data
        let header = unsafe { core::ptr::read_unaligned(data.as_ptr() as *const ElfHeader) };

        // Verify magic bytes
        if header.e_ident[0..4] != ELF_MAGIC {
            return Err("Invalid ELF magic");
        }

        // Verify this is a 64-bit ELF
        if header.e_ident[4] != ElfClass::Elf64 as u8 {
            return Err("Not a 64-bit ELF");
        }

        // Verify little-endian
        if header.e_ident[5] != ElfData::Little as u8 {
            return Err("Not little-endian");
        }

        // Verify version
        if header.e_ident[6] != 1 {
            return Err("Invalid ELF version");
        }

        Ok(header)
    }

    /// Get ELF type
    pub fn elf_type(&self) -> Option<ElfType> {
        match self.e_type {
            1 => Some(ElfType::Relocatable),
            2 => Some(ElfType::Executable),
            3 => Some(ElfType::Shared),
            4 => Some(ElfType::Core),
            _ => None,
        }
    }

    /// Get machine architecture
    pub fn machine(&self) -> Option<ElfMachine> {
        match self.e_machine {
            3 => Some(ElfMachine::X86),
            62 => Some(ElfMachine::X86_64),
            40 => Some(ElfMachine::Arm),
            183 => Some(ElfMachine::Aarch64),
            _ => None,
        }
    }

    /// Validate that this ELF is suitable for execution
    pub fn validate(&self) -> Result<(), &'static str> {
        // Check type is executable or shared
        match self.elf_type() {
            Some(ElfType::Executable) | Some(ElfType::Shared) => {},
            _ => return Err("ELF must be executable or shared object"),
        }

        // Check machine is x86_64
        match self.machine() {
            Some(ElfMachine::X86_64) => {},
            _ => return Err("ELF must be x86_64"),
        }

        // Check we have program headers
        if self.e_phnum == 0 {
            return Err("No program headers");
        }

        Ok(())
    }
}

/// ELF Program Header (64-bit)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ElfProgramHeader {
    /// Segment type
    pub p_type: u32,
    /// Segment flags
    pub p_flags: u32,
    /// Segment file offset
    pub p_offset: u64,
    /// Segment virtual address
    pub p_vaddr: u64,
    /// Segment physical address
    pub p_paddr: u64,
    /// Segment size in file
    pub p_filesz: u64,
    /// Segment size in memory
    pub p_memsz: u64,
    /// Segment alignment
    pub p_align: u64,
}

impl ElfProgramHeader {
    /// Parse a program header from a byte slice
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < size_of::<ElfProgramHeader>() {
            return Err("Data too small for program header");
        }

        // Safety: We've checked the size, and we use read_unaligned to handle potentially unaligned data
        let header = unsafe { core::ptr::read_unaligned(data.as_ptr() as *const ElfProgramHeader) };
        Ok(header)
    }

    /// Get program type
    pub fn program_type(&self) -> Option<ProgramType> {
        ProgramType::from_u32(self.p_type)
    }

    /// Check if segment is executable
    pub fn is_executable(&self) -> bool {
        (self.p_flags & program_flags::PF_X) != 0
    }

    /// Check if segment is writable
    pub fn is_writable(&self) -> bool {
        (self.p_flags & program_flags::PF_W) != 0
    }

    /// Check if segment is readable
    pub fn is_readable(&self) -> bool {
        (self.p_flags & program_flags::PF_R) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_magic() {
        assert_eq!(ELF_MAGIC, [0x7f, b'E', b'L', b'F']);
    }

    #[test]
    fn test_elf_header_size() {
        // ELF64 header should be 64 bytes
        assert_eq!(size_of::<ElfHeader>(), 64);
    }

    #[test]
    fn test_program_header_size() {
        // ELF64 program header should be 56 bytes
        assert_eq!(size_of::<ElfProgramHeader>(), 56);
    }

    #[test]
    fn test_program_type_from_u32() {
        assert_eq!(ProgramType::from_u32(0), Some(ProgramType::Null));
        assert_eq!(ProgramType::from_u32(1), Some(ProgramType::Load));
        assert_eq!(ProgramType::from_u32(2), Some(ProgramType::Dynamic));
        assert_eq!(ProgramType::from_u32(3), Some(ProgramType::Interp));
        assert_eq!(ProgramType::from_u32(999), None);
    }

    #[test]
    fn test_program_flags() {
        let mut header = ElfProgramHeader {
            p_type: 1,
            p_flags: program_flags::PF_R | program_flags::PF_X,
            p_offset: 0,
            p_vaddr: 0,
            p_paddr: 0,
            p_filesz: 0,
            p_memsz: 0,
            p_align: 0,
        };

        assert!(header.is_readable());
        assert!(!header.is_writable());
        assert!(header.is_executable());

        header.p_flags = program_flags::PF_R | program_flags::PF_W;
        assert!(header.is_readable());
        assert!(header.is_writable());
        assert!(!header.is_executable());
    }

    #[test]
    fn test_invalid_elf_header() {
        let data = [0u8; 64];
        assert!(ElfHeader::parse(&data).is_err());
    }

    #[test]
    fn test_elf_header_too_small() {
        let data = [0u8; 32];
        assert!(ElfHeader::parse(&data).is_err());
    }
}
