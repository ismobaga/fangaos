//! Virtual Memory Management (Paging)
//!
//! This module manages virtual memory through x86_64 page tables.

pub mod arch_x86_64;
pub mod mapper;

pub use arch_x86_64::{
    PageTable, PageTableEntry, PageTableFlags,
    pml4_index, pdpt_index, pd_index, pt_index, page_offset,
};
pub use mapper::PageTableMapper;
