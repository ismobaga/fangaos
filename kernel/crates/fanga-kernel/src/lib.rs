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

// IO module
pub mod io;

// USB module
pub mod usb;

// Shell module
pub mod shell;

// File system module
pub mod fs;

// Storage module (ATA/AHCI, partitions, FAT32, caching)
pub mod storage;

// ELF loader module
pub mod elf;

// User space support
pub mod userspace;

// Networking module (E1000 driver, Ethernet, ARP, IPv4, UDP, TCP, sockets, DHCP)
pub mod net;

// Power management module
pub mod power;
