//! Integration tests for memory subsystem
//!
//! These tests verify the interaction between different memory components.

#![cfg(test)]

use fanga_kernel::memory::{PhysAddr, VirtAddr, PAGE_SIZE, align_up, align_down};

#[test]
fn test_address_type_interoperability() {
    // Test that PhysAddr and VirtAddr work together in typical scenarios
    let phys = PhysAddr::new(0x1000);
    let virt = VirtAddr::new(0xFFFF_8000_0000_1000);
    
    // Convert to u64 and back
    let phys_val: u64 = phys.into();
    let virt_val: u64 = virt.into();
    
    let phys2: PhysAddr = phys_val.into();
    let virt2: VirtAddr = virt_val.into();
    
    assert_eq!(phys, phys2);
    assert_eq!(virt, virt2);
}

#[test]
fn test_page_alignment_consistency() {
    // Test that alignment functions work consistently across address types
    let test_values = [0u64, 1, 4095, 4096, 4097, 8192];
    
    for &val in &test_values {
        let phys = PhysAddr::new(val);
        let virt = VirtAddr::new(val);
        
        let phys_aligned_up = phys.align_up(PAGE_SIZE as u64);
        let virt_aligned_up = virt.align_up(PAGE_SIZE as u64);
        let direct_aligned_up = align_up(val, PAGE_SIZE as u64);
        
        assert_eq!(phys_aligned_up.as_u64(), direct_aligned_up);
        assert_eq!(virt_aligned_up.as_u64(), direct_aligned_up);
        
        let phys_aligned_down = phys.align_down(PAGE_SIZE as u64);
        let virt_aligned_down = virt.align_down(PAGE_SIZE as u64);
        let direct_aligned_down = align_down(val, PAGE_SIZE as u64);
        
        assert_eq!(phys_aligned_down.as_u64(), direct_aligned_down);
        assert_eq!(virt_aligned_down.as_u64(), direct_aligned_down);
    }
}

#[test]
fn test_multiple_page_allocations() {
    // Test patterns that would occur in real memory allocation scenarios
    let base_addr = PhysAddr::new(0x10_0000);
    let mut current_addr = base_addr;
    
    // Simulate allocating 10 pages
    let mut allocated_pages = Vec::new();
    for _ in 0..10 {
        allocated_pages.push(current_addr);
        current_addr = PhysAddr::new(current_addr.as_u64() + PAGE_SIZE as u64);
    }
    
    // Verify all pages are properly aligned
    for page in &allocated_pages {
        assert!(page.is_aligned(PAGE_SIZE as u64));
    }
    
    // Verify pages don't overlap
    for i in 0..allocated_pages.len() - 1 {
        let current = allocated_pages[i].as_u64();
        let next = allocated_pages[i + 1].as_u64();
        assert_eq!(next - current, PAGE_SIZE as u64);
    }
}

#[test]
fn test_canonical_address_ranges() {
    // Test various canonical address ranges
    let lower_half_addresses = [
        0x0000_0000_0000_0000,
        0x0000_0000_0000_1000,
        0x0000_7FFF_FFFF_F000,
        0x0000_7FFF_FFFF_FFFF,
    ];
    
    for &addr in &lower_half_addresses {
        let virt = VirtAddr::new(addr);
        assert!(virt.is_canonical(), "Address 0x{:x} should be canonical", addr);
    }
    
    let higher_half_addresses = [
        0xFFFF_8000_0000_0000,
        0xFFFF_8000_0000_1000,
        0xFFFF_FFFF_FFFF_F000,
        0xFFFF_FFFF_FFFF_FFFF,
    ];
    
    for &addr in &higher_half_addresses {
        let virt = VirtAddr::new(addr);
        assert!(virt.is_canonical(), "Address 0x{:x} should be canonical", addr);
    }
    
    let non_canonical_addresses = [
        0x0000_8000_0000_0000,
        0x0000_FFFF_FFFF_FFFF,
        0x7FFF_FFFF_FFFF_FFFF,
    ];
    
    for &addr in &non_canonical_addresses {
        let virt = VirtAddr::new(addr);
        assert!(!virt.is_canonical(), "Address 0x{:x} should not be canonical", addr);
    }
}

#[test]
fn test_address_arithmetic_patterns() {
    // Test common address arithmetic patterns used in memory management
    let base = PhysAddr::new(0x100000);
    
    // Page table indexing pattern
    let page_table_size = 512 * 8; // 512 entries * 8 bytes per entry
    let pt_addr = base.as_u64();
    let next_pt = PhysAddr::new(align_up(pt_addr + page_table_size as u64, PAGE_SIZE as u64));
    
    assert!(next_pt.is_aligned(PAGE_SIZE as u64));
    assert!(next_pt.as_u64() >= base.as_u64() + page_table_size as u64);
}
