//! ATA (IDE) Driver
//!
//! This module implements ATA PIO (Programmed I/O) mode for basic disk access.
//! ATA is the older IDE interface, useful for simple disk operations.

extern crate alloc;
use crate::storage::block_device::{BlockDevice, BlockDeviceError};
use spin::Mutex;

/// Simple port wrapper for ATA I/O
struct Port<T> {
    port: u16,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> Port<T> {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl Port<u8> {
    pub unsafe fn read(&self) -> u8 {
        fanga_arch_x86_64::port::inb(self.port)
    }
    
    pub unsafe fn write(&self, value: u8) {
        fanga_arch_x86_64::port::outb(self.port, value)
    }
}

impl Port<u16> {
    pub unsafe fn read(&self) -> u16 {
        // Use proper 16-bit port I/O instruction
        fanga_arch_x86_64::port::inw(self.port)
    }
    
    pub unsafe fn write(&self, value: u16) {
        // Use proper 16-bit port I/O instruction
        fanga_arch_x86_64::port::outw(self.port, value)
    }
}

/// ATA command ports for primary bus
const PRIMARY_IO_BASE: u16 = 0x1F0;
const PRIMARY_CONTROL: u16 = 0x3F6;

/// ATA command ports for secondary bus
const SECONDARY_IO_BASE: u16 = 0x170;
const SECONDARY_CONTROL: u16 = 0x376;

/// ATA status register bits
const ATA_SR_BSY: u8 = 0x80;  // Busy
const ATA_SR_DRDY: u8 = 0x40; // Drive ready
const ATA_SR_DF: u8 = 0x20;   // Drive fault
const ATA_SR_DSC: u8 = 0x10;  // Drive seek complete
const ATA_SR_DRQ: u8 = 0x08;  // Data request ready
const ATA_SR_CORR: u8 = 0x04; // Corrected data
const ATA_SR_IDX: u8 = 0x02;  // Index
const ATA_SR_ERR: u8 = 0x01;  // Error

/// ATA commands
const ATA_CMD_READ_PIO: u8 = 0x20;
const ATA_CMD_WRITE_PIO: u8 = 0x30;
const ATA_CMD_IDENTIFY: u8 = 0xEC;
const ATA_CMD_FLUSH: u8 = 0xE7;

/// ATA device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtaBus {
    Primary,
    Secondary,
}

/// ATA device (master or slave on a bus)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtaDrive {
    Master,
    Slave,
}

/// ATA device structure
pub struct AtaDevice {
    bus: AtaBus,
    drive: AtaDrive,
    io_base: u16,
    control_base: u16,
    data_port: Mutex<Port<u16>>,
    error_port: Mutex<Port<u8>>,
    sector_count_port: Mutex<Port<u8>>,
    lba_low_port: Mutex<Port<u8>>,
    lba_mid_port: Mutex<Port<u8>>,
    lba_high_port: Mutex<Port<u8>>,
    drive_port: Mutex<Port<u8>>,
    status_port: Mutex<Port<u8>>,
    command_port: Mutex<Port<u8>>,
    sector_size: usize,
    total_sectors: u64,
}

impl AtaDevice {
    /// Create a new ATA device
    pub fn new(bus: AtaBus, drive: AtaDrive) -> Self {
        let (io_base, control_base) = match bus {
            AtaBus::Primary => (PRIMARY_IO_BASE, PRIMARY_CONTROL),
            AtaBus::Secondary => (SECONDARY_IO_BASE, SECONDARY_CONTROL),
        };
        
        Self {
            bus,
            drive,
            io_base,
            control_base,
            data_port: Mutex::new(Port::new(io_base)),
            error_port: Mutex::new(Port::new(io_base + 1)),
            sector_count_port: Mutex::new(Port::new(io_base + 2)),
            lba_low_port: Mutex::new(Port::new(io_base + 3)),
            lba_mid_port: Mutex::new(Port::new(io_base + 4)),
            lba_high_port: Mutex::new(Port::new(io_base + 5)),
            drive_port: Mutex::new(Port::new(io_base + 6)),
            status_port: Mutex::new(Port::new(io_base + 7)),
            command_port: Mutex::new(Port::new(io_base + 7)),
            sector_size: 512,
            total_sectors: 0,
        }
    }
    
    /// Initialize the device and read identification info
    pub fn init(&mut self) -> Result<(), BlockDeviceError> {
        // Select the drive
        let drive_select = match self.drive {
            AtaDrive::Master => 0xA0,
            AtaDrive::Slave => 0xB0,
        };
        
        unsafe {
            self.drive_port.lock().write(drive_select);
        }
        
        // Small delay for drive selection
        self.io_delay();
        
        // Send IDENTIFY command
        unsafe {
            self.sector_count_port.lock().write(0);
            self.lba_low_port.lock().write(0);
            self.lba_mid_port.lock().write(0);
            self.lba_high_port.lock().write(0);
            self.command_port.lock().write(ATA_CMD_IDENTIFY);
        }
        
        // Wait for BSY to clear
        self.wait_not_busy()?;
        
        // Check if device exists
        let status = unsafe { self.status_port.lock().read() };
        if status == 0 {
            return Err(BlockDeviceError::NotFound);
        }
        
        // Wait for DRQ (data ready)
        self.wait_drq()?;
        
        // Read identification data (256 words = 512 bytes)
        let mut identify_data = [0u16; 256];
        for word in identify_data.iter_mut() {
            *word = unsafe { self.data_port.lock().read() };
        }
        
        // Extract total sectors (LBA28 or LBA48)
        // For simplicity, using LBA28 (max 128 GB)
        let lba28_sectors = ((identify_data[61] as u32) << 16) | (identify_data[60] as u32);
        self.total_sectors = lba28_sectors as u64;
        
        if self.total_sectors == 0 {
            return Err(BlockDeviceError::NotFound);
        }
        
        Ok(())
    }
    
    /// Wait for the device to not be busy
    fn wait_not_busy(&self) -> Result<(), BlockDeviceError> {
        for _ in 0..1000000 {
            let status = unsafe { self.status_port.lock().read() };
            if status & ATA_SR_BSY == 0 {
                return Ok(());
            }
        }
        Err(BlockDeviceError::Timeout)
    }
    
    /// Wait for data request ready
    fn wait_drq(&self) -> Result<(), BlockDeviceError> {
        for _ in 0..1000000 {
            let status = unsafe { self.status_port.lock().read() };
            if status & ATA_SR_DRQ != 0 {
                return Ok(());
            }
            if status & ATA_SR_ERR != 0 {
                return Err(BlockDeviceError::IoError);
            }
        }
        Err(BlockDeviceError::Timeout)
    }
    
    /// IO delay (read status register 4 times)
    fn io_delay(&self) {
        for _ in 0..4 {
            unsafe { self.status_port.lock().read(); }
        }
    }
    
    /// Read a single sector using LBA28
    fn read_sector_pio(&self, lba: u64, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if buffer.len() < 512 {
            return Err(BlockDeviceError::InvalidBufferSize);
        }
        
        if lba >= (1 << 28) {
            return Err(BlockDeviceError::InvalidBlock);
        }
        
        // Select drive and set LBA mode
        let drive_select = match self.drive {
            AtaDrive::Master => 0xE0,
            AtaDrive::Slave => 0xF0,
        } | ((lba >> 24) as u8 & 0x0F);
        
        unsafe {
            self.drive_port.lock().write(drive_select);
            self.sector_count_port.lock().write(1);
            self.lba_low_port.lock().write(lba as u8);
            self.lba_mid_port.lock().write((lba >> 8) as u8);
            self.lba_high_port.lock().write((lba >> 16) as u8);
            self.command_port.lock().write(ATA_CMD_READ_PIO);
        }
        
        // Wait for device ready
        self.wait_drq()?;
        
        // Read data (256 words = 512 bytes)
        let buffer_words = unsafe {
            core::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u16, 256)
        };
        
        for word in buffer_words.iter_mut() {
            *word = unsafe { self.data_port.lock().read() };
        }
        
        Ok(())
    }
    
    /// Write a single sector using LBA28
    fn write_sector_pio(&self, lba: u64, buffer: &[u8]) -> Result<(), BlockDeviceError> {
        if buffer.len() < 512 {
            return Err(BlockDeviceError::InvalidBufferSize);
        }
        
        if lba >= (1 << 28) {
            return Err(BlockDeviceError::InvalidBlock);
        }
        
        // Select drive and set LBA mode
        let drive_select = match self.drive {
            AtaDrive::Master => 0xE0,
            AtaDrive::Slave => 0xF0,
        } | ((lba >> 24) as u8 & 0x0F);
        
        unsafe {
            self.drive_port.lock().write(drive_select);
            self.sector_count_port.lock().write(1);
            self.lba_low_port.lock().write(lba as u8);
            self.lba_mid_port.lock().write((lba >> 8) as u8);
            self.lba_high_port.lock().write((lba >> 16) as u8);
            self.command_port.lock().write(ATA_CMD_WRITE_PIO);
        }
        
        // Wait for device ready
        self.wait_drq()?;
        
        // Write data (256 words = 512 bytes)
        let buffer_words = unsafe {
            core::slice::from_raw_parts(buffer.as_ptr() as *const u16, 256)
        };
        
        for word in buffer_words.iter() {
            unsafe { self.data_port.lock().write(*word); }
        }
        
        // Flush cache
        unsafe {
            self.command_port.lock().write(ATA_CMD_FLUSH);
        }
        self.wait_not_busy()?;
        
        Ok(())
    }
}

impl BlockDevice for AtaDevice {
    fn block_size(&self) -> usize {
        self.sector_size
    }
    
    fn block_count(&self) -> u64 {
        self.total_sectors
    }
    
    fn read_blocks(&self, start_block: u64, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if start_block >= self.total_sectors {
            return Err(BlockDeviceError::InvalidBlock);
        }
        
        let num_sectors = buffer.len() / self.sector_size;
        for i in 0..num_sectors {
            let offset = i * self.sector_size;
            self.read_sector_pio(start_block + i as u64, &mut buffer[offset..offset + self.sector_size])?;
        }
        
        Ok(())
    }
    
    fn write_blocks(&self, start_block: u64, buffer: &[u8]) -> Result<(), BlockDeviceError> {
        if start_block >= self.total_sectors {
            return Err(BlockDeviceError::InvalidBlock);
        }
        
        let num_sectors = buffer.len() / self.sector_size;
        for i in 0..num_sectors {
            let offset = i * self.sector_size;
            self.write_sector_pio(start_block + i as u64, &buffer[offset..offset + self.sector_size])?;
        }
        
        Ok(())
    }
    
    fn flush(&mut self) -> Result<(), BlockDeviceError> {
        unsafe {
            self.command_port.lock().write(ATA_CMD_FLUSH);
        }
        self.wait_not_busy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ata_device_creation() {
        let device = AtaDevice::new(AtaBus::Primary, AtaDrive::Master);
        assert_eq!(device.io_base, PRIMARY_IO_BASE);
        assert_eq!(device.bus, AtaBus::Primary);
        assert_eq!(device.drive, AtaDrive::Master);
    }
}
