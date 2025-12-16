/// PS/2 Mouse driver for x86_64
///
/// This module provides a PS/2 mouse driver that handles mouse movement
/// and button events.

use crate::port::{inb, outb};

const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
const PS2_COMMAND_PORT: u16 = 0x64;

/// Mouse buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

impl MouseButtons {
    pub const fn new() -> Self {
        Self {
            left: false,
            right: false,
            middle: false,
        }
    }
}

/// Mouse packet data
#[derive(Debug, Clone, Copy)]
pub struct MousePacket {
    pub buttons: MouseButtons,
    pub x_movement: i16,
    pub y_movement: i16,
    pub z_movement: i8, // Scroll wheel
}

/// Mouse state tracker
pub struct Mouse {
    // Packet assembly
    packet_bytes: [u8; 4],
    packet_index: usize,
    
    // Current state
    x: i32,
    y: i32,
    buttons: MouseButtons,
    
    // Accumulated movement since last read
    delta_x: i32,
    delta_y: i32,
    scroll: i32,
    
    // Mouse enabled
    enabled: bool,
}

impl Mouse {
    pub const fn new() -> Self {
        Self {
            packet_bytes: [0; 4],
            packet_index: 0,
            x: 0,
            y: 0,
            buttons: MouseButtons::new(),
            delta_x: 0,
            delta_y: 0,
            scroll: 0,
            enabled: false,
        }
    }
    
    /// Initialize the PS/2 mouse
    pub fn init(&mut self) -> Result<(), &'static str> {
        // Wait for controller to be ready
        self.wait_for_write();
        
        // Enable auxiliary device (mouse)
        unsafe {
            outb(PS2_COMMAND_PORT, 0xA8);
        }
        
        // Wait for controller
        self.wait_for_write();
        
        // Enable interrupts for mouse
        unsafe {
            outb(PS2_COMMAND_PORT, 0x20); // Read command byte
        }
        
        self.wait_for_read();
        let status = unsafe { inb(PS2_DATA_PORT) };
        
        self.wait_for_write();
        unsafe {
            outb(PS2_COMMAND_PORT, 0x60); // Write command byte
        }
        
        self.wait_for_write();
        unsafe {
            outb(PS2_DATA_PORT, status | 0x02); // Enable mouse IRQ
        }
        
        // Set default settings
        self.write_mouse(0xF6)?;
        self.read_ack()?;
        
        // Enable data reporting
        self.write_mouse(0xF4)?;
        self.read_ack()?;
        
        self.enabled = true;
        Ok(())
    }
    
    /// Process a byte from the mouse
    pub fn process_byte(&mut self, byte: u8) -> Option<MousePacket> {
        // First byte has bit 3 set
        if self.packet_index == 0 && (byte & 0x08) == 0 {
            // Invalid packet, discard
            return None;
        }
        
        self.packet_bytes[self.packet_index] = byte;
        self.packet_index += 1;
        
        // Standard PS/2 mouse uses 3-byte packets
        if self.packet_index >= 3 {
            self.packet_index = 0;
            
            let packet = self.parse_packet();
            
            // Update state
            self.buttons = packet.buttons;
            self.delta_x += packet.x_movement as i32;
            self.delta_y += packet.y_movement as i32;
            self.x += packet.x_movement as i32;
            self.y += packet.y_movement as i32;
            
            return Some(packet);
        }
        
        None
    }
    
    /// Parse a 3-byte mouse packet
    fn parse_packet(&self) -> MousePacket {
        let byte0 = self.packet_bytes[0];
        let byte1 = self.packet_bytes[1];
        let byte2 = self.packet_bytes[2];
        
        // Extract button states
        let buttons = MouseButtons {
            left: (byte0 & 0x01) != 0,
            right: (byte0 & 0x02) != 0,
            middle: (byte0 & 0x04) != 0,
        };
        
        // Extract movement (9-bit signed values)
        let mut x_movement = byte1 as i16;
        if (byte0 & 0x10) != 0 {
            x_movement |= 0xFF00u16 as i16; // Sign extend
        }
        
        let mut y_movement = byte2 as i16;
        if (byte0 & 0x20) != 0 {
            y_movement |= 0xFF00u16 as i16; // Sign extend
        }
        
        // Y is inverted in PS/2 protocol
        y_movement = -y_movement;
        
        MousePacket {
            buttons,
            x_movement,
            y_movement,
            z_movement: 0, // Standard PS/2 mouse doesn't have scroll wheel
        }
    }
    
    /// Get current mouse position
    pub fn position(&self) -> (i32, i32) {
        (self.x, self.y)
    }
    
    /// Set mouse position (for bounds enforcement)
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
    
    /// Get current button state
    pub fn buttons(&self) -> MouseButtons {
        self.buttons
    }
    
    /// Get and clear accumulated movement
    pub fn take_movement(&mut self) -> (i32, i32, i32) {
        let delta_x = self.delta_x;
        let delta_y = self.delta_y;
        let scroll = self.scroll;
        self.delta_x = 0;
        self.delta_y = 0;
        self.scroll = 0;
        (delta_x, delta_y, scroll)
    }
    
    /// Check if mouse is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Write a command to the mouse
    fn write_mouse(&self, cmd: u8) -> Result<(), &'static str> {
        self.wait_for_write();
        
        unsafe {
            outb(PS2_COMMAND_PORT, 0xD4); // Tell controller we're writing to mouse
        }
        
        self.wait_for_write();
        
        unsafe {
            outb(PS2_DATA_PORT, cmd);
        }
        
        Ok(())
    }
    
    /// Read acknowledgment from mouse
    fn read_ack(&self) -> Result<(), &'static str> {
        self.wait_for_read();
        
        let response = unsafe { inb(PS2_DATA_PORT) };
        
        if response == 0xFA {
            Ok(())
        } else {
            Err("Mouse did not acknowledge")
        }
    }
    
    /// Wait for controller to be ready for writing
    fn wait_for_write(&self) {
        for _ in 0..1000 {
            if (unsafe { inb(PS2_STATUS_PORT) } & 0x02) == 0 {
                return;
            }
        }
    }
    
    /// Wait for controller to have data ready for reading
    fn wait_for_read(&self) {
        for _ in 0..1000 {
            if (unsafe { inb(PS2_STATUS_PORT) } & 0x01) != 0 {
                return;
            }
        }
    }
}

/// Global mouse state
/// Note: This is accessed from interrupt context (IRQ12 handler).
/// The mouse is initialized once at boot and then only read/updated from the
/// interrupt handler, making the single-threaded access safe.
static mut MOUSE: Mouse = Mouse::new();

/// Type for mouse event callback
pub type MouseCallback = fn(MousePacket);

/// Global mouse event callback
static mut MOUSE_CALLBACK: Option<MouseCallback> = None;

/// Initialize the PS/2 mouse
pub fn init() -> Result<(), &'static str> {
    #[cfg(not(test))]
    unsafe { MOUSE.init() }
    
    #[cfg(test)]
    Ok(())
}

/// Set the mouse event callback
///
/// # Safety
/// Must be called before enabling mouse interrupts
pub unsafe fn set_mouse_callback(callback: MouseCallback) {
    MOUSE_CALLBACK = Some(callback);
}

/// Get a mutable reference to the global mouse
pub fn mouse() -> &'static mut Mouse {
    unsafe { &mut MOUSE }
}

/// Dispatch a mouse packet to the registered callback
pub(crate) fn dispatch_packet(packet: MousePacket) {
    unsafe {
        if let Some(callback) = MOUSE_CALLBACK {
            callback(packet);
        }
    }
}
