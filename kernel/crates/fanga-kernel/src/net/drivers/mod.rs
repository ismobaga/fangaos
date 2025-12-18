//! Network card drivers

pub mod e1000;

use super::ethernet::MacAddress;
use alloc::vec::Vec;

/// Network interface trait
pub trait NetworkDevice {
    /// Get the MAC address
    fn mac_address(&self) -> MacAddress;

    /// Send a packet
    fn send_packet(&mut self, data: &[u8]) -> Result<(), &'static str>;

    /// Receive a packet
    fn receive_packet(&mut self) -> Option<Vec<u8>>;

    /// Check if a packet is available
    fn has_packet(&self) -> bool;
}

/// Network interface wrapper
pub enum NetworkInterface {
    /// E1000 network card
    E1000(e1000::E1000Driver),
}

impl NetworkInterface {
    /// Get the MAC address
    pub fn mac_address(&self) -> MacAddress {
        match self {
            NetworkInterface::E1000(driver) => driver.mac_address(),
        }
    }

    /// Send a packet
    pub fn send_packet(&mut self, data: &[u8]) -> Result<(), &'static str> {
        match self {
            NetworkInterface::E1000(driver) => driver.send_packet(data),
        }
    }

    /// Receive a packet
    pub fn receive_packet(&mut self) -> Option<Vec<u8>> {
        match self {
            NetworkInterface::E1000(driver) => driver.receive_packet(),
        }
    }

    /// Check if a packet is available
    pub fn has_packet(&self) -> bool {
        match self {
            NetworkInterface::E1000(driver) => driver.has_packet(),
        }
    }
}
