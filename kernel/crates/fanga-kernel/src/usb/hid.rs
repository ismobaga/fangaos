/// USB Human Interface Device (HID) support
///
/// This module implements USB HID protocol for keyboards, mice, and other
/// input devices.

use alloc::vec::Vec;
use super::{DeviceAddress, EndpointNum};

/// HID subclass codes
pub mod subclass {
    pub const NONE: u8 = 0x00;
    pub const BOOT_INTERFACE: u8 = 0x01;
}

/// HID protocol codes
pub mod protocol {
    pub const NONE: u8 = 0x00;
    pub const KEYBOARD: u8 = 0x01;
    pub const MOUSE: u8 = 0x02;
}

/// HID request types
pub mod request {
    pub const GET_REPORT: u8 = 0x01;
    pub const GET_IDLE: u8 = 0x02;
    pub const GET_PROTOCOL: u8 = 0x03;
    pub const SET_REPORT: u8 = 0x09;
    pub const SET_IDLE: u8 = 0x0A;
    pub const SET_PROTOCOL: u8 = 0x0B;
}

/// HID report types
pub mod report_type {
    pub const INPUT: u8 = 0x01;
    pub const OUTPUT: u8 = 0x02;
    pub const FEATURE: u8 = 0x03;
}

/// USB HID keyboard report (boot protocol)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HidKeyboardReport {
    pub modifiers: u8,  // Modifier keys bitmap
    pub reserved: u8,
    pub keycodes: [u8; 6], // Up to 6 simultaneous key presses
}

impl HidKeyboardReport {
    /// Check if a modifier key is pressed
    pub fn is_ctrl(&self) -> bool {
        (self.modifiers & 0x11) != 0 // Left or right Ctrl
    }
    
    pub fn is_shift(&self) -> bool {
        (self.modifiers & 0x22) != 0 // Left or right Shift
    }
    
    pub fn is_alt(&self) -> bool {
        (self.modifiers & 0x44) != 0 // Left or right Alt
    }
    
    pub fn is_gui(&self) -> bool {
        (self.modifiers & 0x88) != 0 // Left or right GUI (Windows/Super key)
    }
}

/// USB HID mouse report (boot protocol)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HidMouseReport {
    pub buttons: u8,    // Button bitmap
    pub x_movement: i8, // Relative X movement
    pub y_movement: i8, // Relative Y movement
    pub wheel: i8,      // Scroll wheel (optional)
}

impl HidMouseReport {
    /// Check if left button is pressed
    pub fn is_left_button(&self) -> bool {
        (self.buttons & 0x01) != 0
    }
    
    /// Check if right button is pressed
    pub fn is_right_button(&self) -> bool {
        (self.buttons & 0x02) != 0
    }
    
    /// Check if middle button is pressed
    pub fn is_middle_button(&self) -> bool {
        (self.buttons & 0x04) != 0
    }
}

/// HID device interface
pub struct HidDevice {
    address: DeviceAddress,
    endpoint_in: EndpointNum,
    endpoint_out: Option<EndpointNum>,
    protocol: u8, // Keyboard, mouse, etc.
    report_size: usize,
}

impl HidDevice {
    /// Create a new HID device
    pub fn new(
        address: DeviceAddress,
        endpoint_in: EndpointNum,
        endpoint_out: Option<EndpointNum>,
        protocol: u8,
    ) -> Self {
        let report_size = match protocol {
            protocol::KEYBOARD => core::mem::size_of::<HidKeyboardReport>(),
            protocol::MOUSE => core::mem::size_of::<HidMouseReport>(),
            _ => 8, // Default size
        };
        
        Self {
            address,
            endpoint_in,
            endpoint_out,
            protocol,
            report_size,
        }
    }
    
    /// Get device address
    pub fn address(&self) -> DeviceAddress {
        self.address
    }
    
    /// Get input endpoint
    pub fn endpoint_in(&self) -> EndpointNum {
        self.endpoint_in
    }
    
    /// Get protocol
    pub fn protocol(&self) -> u8 {
        self.protocol
    }
    
    /// Check if device is a keyboard
    pub fn is_keyboard(&self) -> bool {
        self.protocol == protocol::KEYBOARD
    }
    
    /// Check if device is a mouse
    pub fn is_mouse(&self) -> bool {
        self.protocol == protocol::MOUSE
    }
    
    /// Get report size
    pub fn report_size(&self) -> usize {
        self.report_size
    }
}

/// HID keyboard event
#[derive(Debug, Clone)]
pub struct HidKeyboardEvent {
    pub report: HidKeyboardReport,
}

/// HID mouse event
#[derive(Debug, Clone)]
pub struct HidMouseEvent {
    pub report: HidMouseReport,
}

/// Callback type for HID keyboard events
pub type HidKeyboardCallback = fn(HidKeyboardEvent);

/// Callback type for HID mouse events
pub type HidMouseCallback = fn(HidMouseEvent);

/// Global HID event callbacks
static mut KEYBOARD_CALLBACK: Option<HidKeyboardCallback> = None;
static mut MOUSE_CALLBACK: Option<HidMouseCallback> = None;

/// Set HID keyboard callback
pub unsafe fn set_keyboard_callback(callback: HidKeyboardCallback) {
    KEYBOARD_CALLBACK = Some(callback);
}

/// Set HID mouse callback
pub unsafe fn set_mouse_callback(callback: HidMouseCallback) {
    MOUSE_CALLBACK = Some(callback);
}

/// Dispatch keyboard event
pub(crate) fn dispatch_keyboard_event(event: HidKeyboardEvent) {
    unsafe {
        if let Some(callback) = KEYBOARD_CALLBACK {
            callback(event);
        }
    }
}

/// Dispatch mouse event
pub(crate) fn dispatch_mouse_event(event: HidMouseEvent) {
    unsafe {
        if let Some(callback) = MOUSE_CALLBACK {
            callback(event);
        }
    }
}
