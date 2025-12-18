/// USB (Universal Serial Bus) subsystem
///
/// This module provides USB host controller support and device management.
/// It implements the basic USB protocol stack including device enumeration,
/// descriptor parsing, and HID device support.

pub mod controller;
pub mod device;
pub mod hid;
pub mod descriptor;

use alloc::vec::Vec;
use spin::Mutex;

/// USB device speed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    Low,    // 1.5 Mbps
    Full,   // 12 Mbps
    High,   // 480 Mbps
    Super,  // 5 Gbps
}

/// USB transfer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferType {
    Control,
    Interrupt,
    Bulk,
    Isochronous,
}

/// USB device address (1-127, 0 is reserved)
pub type DeviceAddress = u8;

/// USB endpoint number (0-15)
pub type EndpointNum = u8;

/// USB manager - coordinates all USB operations
pub struct UsbManager {
    // controllers: Vec<Box<dyn controller::UsbController>>,
    devices: Vec<device::UsbDevice>,
    next_address: DeviceAddress,
}

impl UsbManager {
    pub const fn new() -> Self {
        Self {
            // controllers: Vec::new(),
            devices: Vec::new(),
            next_address: 1,
        }
    }
    
    /// Initialize USB subsystem
    pub fn init(&mut self) {
        // Scan for USB controllers
        // self.scan_controllers();
        
        // Initialize each controller
        // for controller in &mut self.controllers {
        //     if let Err(e) = controller.init() {
        //         // Log error but continue with other controllers
        //     }
        // }
    }
    
    /// Scan for USB host controllers on PCI bus
    fn scan_controllers(&mut self) {
        // TODO: Scan PCI bus for USB controllers (UHCI, OHCI, EHCI, XHCI)
        // For now, this is a placeholder
    }
    
    /// Allocate a new device address
    pub fn allocate_address(&mut self) -> Option<DeviceAddress> {
        if self.next_address > 127 {
            return None;
        }
        
        let addr = self.next_address;
        self.next_address += 1;
        Some(addr)
    }
    
    /// Register a new USB device
    pub fn register_device(&mut self, device: device::UsbDevice) {
        self.devices.push(device);
    }
    
    /// Get all connected devices
    pub fn devices(&self) -> &[device::UsbDevice] {
        &self.devices
    }
    
    /// Find device by address
    pub fn find_device(&self, address: DeviceAddress) -> Option<&device::UsbDevice> {
        self.devices.iter().find(|d| d.address() == address)
    }
}

/// Global USB manager
static USB_MANAGER: Mutex<UsbManager> = Mutex::new(UsbManager::new());

/// Initialize the USB subsystem
pub fn init() {
    USB_MANAGER.lock().init();
}

/// Get access to the USB manager
pub fn usb_manager() -> spin::MutexGuard<'static, UsbManager> {
    USB_MANAGER.lock()
}
