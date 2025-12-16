/// USB device representation and management

use super::{DeviceAddress, UsbSpeed};
use super::descriptor::DeviceDescriptor;
use alloc::vec::Vec;

/// USB device state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Attached,
    Powered,
    Default,
    Addressed,
    Configured,
    Suspended,
}

/// USB device
pub struct UsbDevice {
    address: DeviceAddress,
    speed: UsbSpeed,
    state: DeviceState,
    descriptor: Option<DeviceDescriptor>,
    configurations: Vec<u8>, // Configuration descriptors
}

impl UsbDevice {
    /// Create a new USB device
    pub fn new(address: DeviceAddress, speed: UsbSpeed) -> Self {
        Self {
            address,
            speed,
            state: DeviceState::Default,
            descriptor: None,
            configurations: Vec::new(),
        }
    }
    
    /// Get device address
    pub fn address(&self) -> DeviceAddress {
        self.address
    }
    
    /// Get device speed
    pub fn speed(&self) -> UsbSpeed {
        self.speed
    }
    
    /// Get device state
    pub fn state(&self) -> DeviceState {
        self.state
    }
    
    /// Set device state
    pub fn set_state(&mut self, state: DeviceState) {
        self.state = state;
    }
    
    /// Set device descriptor
    pub fn set_descriptor(&mut self, descriptor: DeviceDescriptor) {
        self.descriptor = Some(descriptor);
    }
    
    /// Get device descriptor
    pub fn descriptor(&self) -> Option<&DeviceDescriptor> {
        self.descriptor.as_ref()
    }
    
    /// Check if device is a HID device
    pub fn is_hid_device(&self) -> bool {
        if let Some(desc) = &self.descriptor {
            desc.device_class == 0x03 // HID class
        } else {
            false
        }
    }
    
    /// Check if device is a keyboard
    pub fn is_keyboard(&self) -> bool {
        if let Some(desc) = &self.descriptor {
            desc.device_class == 0x03 && desc.device_subclass == 0x01 // HID Boot Interface
        } else {
            false
        }
    }
    
    /// Check if device is a mouse
    pub fn is_mouse(&self) -> bool {
        if let Some(desc) = &self.descriptor {
            desc.device_class == 0x03 && desc.device_subclass == 0x01 // HID Boot Interface
        } else {
            false
        }
    }
}
