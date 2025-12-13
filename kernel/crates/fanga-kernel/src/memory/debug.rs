//! Memory Debugging Utilities
//!
//! This module provides debugging utilities for memory management.

use crate::memory::paging::PageTableMapper;

/// Prints a memory dump of a region
///
/// # Safety
/// The caller must ensure that:
/// - The memory region [addr, addr + size) is valid and mapped
/// - The memory is readable and won't cause page faults
/// - The memory is not being modified concurrently
/// - The HHDM mapping is valid for physical addresses
pub unsafe fn dump_memory(addr: u64, size: usize, label: &str) {
    use fanga_arch_x86_64 as arch;

    arch::serial_println!("Memory Dump: {} @ 0x{:x} ({} bytes)", label, addr, size);

    let ptr = addr as *const u8;
    let mut offset = 0;

    while offset < size {
        // Print address
        arch::serial_print!("  0x{:08x}: ", addr + offset as u64);

        // Print hex bytes
        let line_size = core::cmp::min(16, size - offset);
        for i in 0..line_size {
            let byte = ptr.add(offset + i).read();
            arch::serial_print!("{:02x} ", byte);
        }

        // Padding for short lines
        for _ in line_size..16 {
            arch::serial_print!("   ");
        }

        // Print ASCII representation
        arch::serial_print!(" |");
        for i in 0..line_size {
            let byte = ptr.add(offset + i).read();
            if byte >= 0x20 && byte <= 0x7E {
                arch::serial_print!("{}", byte as char);
            } else {
                arch::serial_print!(".");
            }
        }
        arch::serial_println!("|");

        offset += line_size;
    }
}

/// Prints page table information for a virtual address
pub fn dump_page_table_entry(virt_addr: u64, mapper: &PageTableMapper) {
    use fanga_arch_x86_64 as arch;
    use crate::memory::paging::{pml4_index, pdpt_index, pd_index, pt_index, page_offset};

    arch::serial_println!("Page Table Entry for 0x{:x}:", virt_addr);

    if let Some(phys_addr) = mapper.translate(virt_addr) {
        arch::serial_println!("  Physical: 0x{:x}", phys_addr);
        arch::serial_println!("  Mapped: Yes");
    } else {
        arch::serial_println!("  Mapped: No");
    }

    // Print indices
    arch::serial_println!("  PML4 index: {}", pml4_index(virt_addr));
    arch::serial_println!("  PDPT index: {}", pdpt_index(virt_addr));
    arch::serial_println!("  PD index:   {}", pd_index(virt_addr));
    arch::serial_println!("  PT index:   {}", pt_index(virt_addr));
    arch::serial_println!("  Offset:     0x{:x}", page_offset(virt_addr));
}
