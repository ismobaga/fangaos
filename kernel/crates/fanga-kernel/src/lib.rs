//! FangaOS Kernel Library
//!
//! This library exports the testable components of the kernel for unit and integration testing.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, allow(dead_code))]

#[cfg(not(test))]
extern crate alloc;

#[cfg(test)]
extern crate alloc;

pub mod memory;
pub mod task;
pub mod syscall;
pub mod syscall_handlers;

// IO module only available in no_std builds
#[cfg(not(test))]
pub mod io;
