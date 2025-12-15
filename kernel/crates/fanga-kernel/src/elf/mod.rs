//! ELF Binary Loader
//!
//! This module implements a basic ELF (Executable and Linkable Format) loader
//! for loading user-space applications into memory.

mod parser;
mod loader;

pub use parser::{ElfHeader, ElfProgramHeader, ElfType, ElfMachine, ProgramType};
pub use loader::{load_elf, ElfLoadError, LoadedElf};
