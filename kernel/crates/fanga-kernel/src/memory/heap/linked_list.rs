//! Linked-List Heap Allocator
//!
//! This module implements a simple heap allocator for dynamic memory allocation.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, NonNull};
use core::mem;
use crate::memory::addr::PAGE_SIZE;

/// Minimum allocation size (must be at least size of FreeBlock)
const MIN_BLOCK_SIZE: usize = mem::size_of::<FreeBlock>();

/// Alignment for all allocations
const HEAP_ALIGN: usize = 8;

/// A free block in the heap
#[repr(C)]
struct FreeBlock {
    size: usize,
    next: Option<NonNull<FreeBlock>>,
}

/// Simple linked-list heap allocator
pub struct HeapAllocator {
    head: Option<NonNull<FreeBlock>>,
}

impl HeapAllocator {
    /// Creates a new, uninitialized heap allocator
    pub const fn new() -> Self {
        Self { head: None }
    }

    /// Initializes the heap with a memory region
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - The memory region [heap_start, heap_start + heap_size) is valid
    /// - The memory is properly aligned
    /// - The memory will not be accessed by other code
    /// - The heap_size is large enough to hold at least one FreeBlock
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        // Align heap start upwards
        let aligned_start = align_up(heap_start, HEAP_ALIGN);
        let adjusted_size = heap_size.saturating_sub(aligned_start - heap_start);

        if adjusted_size < MIN_BLOCK_SIZE {
            // Heap too small, can't initialize
            return;
        }

        // Create the initial free block
        let block = aligned_start as *mut FreeBlock;
        (*block).size = adjusted_size;
        (*block).next = None;

        self.head = NonNull::new(block);
    }

    /// Allocates memory with the given layout
    ///
    /// Returns a pointer to the allocated memory, or a null pointer if allocation fails.
    /// For zero-sized allocations, returns a dangling (non-null but invalid) pointer that
    /// should not be dereferenced.
    ///
    /// # Safety
    /// This method is not thread-safe and must be called with proper synchronization.
    /// The layout must have a power-of-two alignment.
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        // Handle zero-sized allocations
        if layout.size() == 0 {
            return core::ptr::NonNull::dangling().as_ptr();
        }

        let size = layout.size().max(MIN_BLOCK_SIZE);
        let align = layout.align().max(HEAP_ALIGN);

        // Search for a suitable block
        let mut current = self.head;
        let mut prev: Option<NonNull<FreeBlock>> = None;

        while let Some(mut block_ptr) = current {
            let block = block_ptr.as_mut();
            
            // Calculate aligned start address for the allocation
            let block_addr = block_ptr.as_ptr() as usize;
            let aligned_addr = align_up(block_addr, align);
            let alignment_offset = aligned_addr - block_addr;

            // Check if block is large enough considering alignment
            if block.size >= size + alignment_offset {
                // Found a suitable block
                let remaining = block.size - size - alignment_offset;

                if remaining >= MIN_BLOCK_SIZE {
                    // Split the block
                    let new_block_addr = aligned_addr + size;
                    let new_block = new_block_addr as *mut FreeBlock;
                    (*new_block).size = remaining;
                    (*new_block).next = block.next;

                    // If we need alignment, create a small free block before allocation
                    if alignment_offset >= MIN_BLOCK_SIZE {
                        block.size = alignment_offset;
                        block.next = NonNull::new(new_block);
                        return aligned_addr as *mut u8;
                    } else {
                        // Remove current block from list and link to new block
                        if let Some(mut p) = prev {
                            p.as_mut().next = NonNull::new(new_block);
                        } else {
                            self.head = NonNull::new(new_block);
                        }
                        return aligned_addr as *mut u8;
                    }
                } else {
                    // Use entire block
                    if let Some(mut p) = prev {
                        p.as_mut().next = block.next;
                    } else {
                        self.head = block.next;
                    }
                    return aligned_addr as *mut u8;
                }
            }

            prev = current;
            current = block.next;
        }

        // No suitable block found
        ptr::null_mut()
    }

    /// Deallocates memory at the given pointer
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - ptr was allocated by this allocator
    /// - ptr is not used after this call
    /// - The layout matches the one used for allocation
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        // Handle zero-sized deallocations
        if layout.size() == 0 {
            return;
        }

        let size = layout.size().max(MIN_BLOCK_SIZE);
        let addr = ptr as usize;

        // Create a new free block
        let new_block = ptr as *mut FreeBlock;
        (*new_block).size = size;
        (*new_block).next = None;

        // Insert into free list in address order and coalesce
        if self.head.is_none() {
            self.head = NonNull::new(new_block);
            return;
        }

        let mut current = self.head;
        let mut prev: Option<NonNull<FreeBlock>> = None;

        // Find insertion point
        while let Some(block_ptr) = current {
            let block_addr = block_ptr.as_ptr() as usize;

            if addr < block_addr {
                // Insert before current block
                (*new_block).next = current;

                // Try to coalesce with next block
                if addr + size == block_addr {
                    let next_block = block_ptr.as_ptr();
                    (*new_block).size += (*next_block).size;
                    (*new_block).next = (*next_block).next;
                }

                // Update previous or head
                if let Some(mut p) = prev {
                    let prev_addr = p.as_ptr() as usize;
                    let prev_ref = p.as_mut();

                    // Try to coalesce with previous block
                    if prev_addr + prev_ref.size == addr {
                        prev_ref.size += (*new_block).size;
                        prev_ref.next = (*new_block).next;
                    } else {
                        prev_ref.next = NonNull::new(new_block);
                    }
                } else {
                    self.head = NonNull::new(new_block);
                }
                return;
            }

            prev = current;
            current = (*block_ptr.as_ptr()).next;
        }

        // Insert at end
        if let Some(mut p) = prev {
            let prev_addr = p.as_ptr() as usize;
            let prev_ref = p.as_mut();

            // Try to coalesce with previous block
            if prev_addr + prev_ref.size == addr {
                prev_ref.size += size;
            } else {
                prev_ref.next = NonNull::new(new_block);
            }
        }
    }
}

/// Aligns a value up to the given alignment
#[inline]
fn align_up(val: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two(), "Alignment must be a power of two");
    (val + align - 1) & !(align - 1)
}

/// Global heap allocator instance
pub struct GlobalHeapAllocator {
    inner: spin::Mutex<HeapAllocator>,
}

impl GlobalHeapAllocator {
    /// Creates a new global heap allocator
    pub const fn new() -> Self {
        Self {
            inner: spin::Mutex::new(HeapAllocator::new()),
        }
    }

    /// Initializes the heap allocator
    ///
    /// # Safety
    /// See HeapAllocator::init for safety requirements
    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        self.inner.lock().init(heap_start, heap_size);
    }
}

unsafe impl GlobalAlloc for GlobalHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.lock().alloc(layout);
        if !ptr.is_null() && layout.size() > 0 {
            crate::memory::stats::stats().record_heap_alloc(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() > 0 {
            crate::memory::stats::stats().record_heap_dealloc(layout.size());
        }
        self.inner.lock().dealloc(ptr, layout)
    }
}

// Safety: The mutex ensures thread-safe access
unsafe impl Send for GlobalHeapAllocator {}
unsafe impl Sync for GlobalHeapAllocator {}
