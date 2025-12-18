//! ATA/AHCI Disk Drivers
//!
//! This module provides drivers for ATA (IDE) and AHCI (SATA) disk access.

pub mod ata;
pub mod ahci;

pub use ata::AtaDevice;
pub use ahci::AhciController;
