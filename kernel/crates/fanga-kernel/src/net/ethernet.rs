//! Ethernet layer implementation
//!
//! Handles Ethernet frame parsing and construction

use alloc::vec::Vec;

/// Ethernet frame structure
#[repr(C, packed)]
pub struct EthernetFrame {
    /// Destination MAC address
    pub dst_mac: [u8; 6],
    /// Source MAC address
    pub src_mac: [u8; 6],
    /// EtherType
    pub ethertype: u16,
}

/// EtherType values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtherType {
    /// IPv4
    IPv4 = 0x0800,
    /// ARP
    ARP = 0x0806,
    /// IPv6
    IPv6 = 0x86DD,
}

impl EtherType {
    /// Convert from u16
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0800 => Some(EtherType::IPv4),
            0x0806 => Some(EtherType::ARP),
            0x86DD => Some(EtherType::IPv6),
            _ => None,
        }
    }

    /// Convert to u16 in network byte order
    pub fn to_be_u16(self) -> u16 {
        self as u16
    }
}

/// MAC address structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    /// Create a new MAC address
    pub fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Broadcast MAC address
    pub fn broadcast() -> Self {
        Self([0xff; 6])
    }

    /// Check if this is a broadcast address
    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xff; 6]
    }

    /// Check if this is a multicast address
    pub fn is_multicast(&self) -> bool {
        self.0[0] & 0x01 != 0
    }
}

/// Ethernet frame parser
pub struct EthernetParser;

impl EthernetParser {
    /// Parse an Ethernet frame from raw bytes
    pub fn parse(data: &[u8]) -> Result<(MacAddress, MacAddress, EtherType, &[u8]), &'static str> {
        if data.len() < 14 {
            return Err("Frame too short");
        }

        let mut dst_mac = [0u8; 6];
        let mut src_mac = [0u8; 6];
        dst_mac.copy_from_slice(&data[0..6]);
        src_mac.copy_from_slice(&data[6..12]);

        let ethertype = u16::from_be_bytes([data[12], data[13]]);
        let ethertype = EtherType::from_u16(ethertype)
            .ok_or("Unknown EtherType")?;

        Ok((MacAddress(dst_mac), MacAddress(src_mac), ethertype, &data[14..]))
    }

    /// Build an Ethernet frame
    pub fn build(dst_mac: MacAddress, src_mac: MacAddress, ethertype: EtherType, payload: &[u8]) -> Vec<u8> {
        let mut frame = Vec::with_capacity(14 + payload.len());
        frame.extend_from_slice(&dst_mac.0);
        frame.extend_from_slice(&src_mac.0);
        frame.extend_from_slice(&ethertype.to_be_u16().to_be_bytes());
        frame.extend_from_slice(payload);
        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_address() {
        let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert!(!mac.is_broadcast());
        assert!(!mac.is_multicast());

        let broadcast = MacAddress::broadcast();
        assert!(broadcast.is_broadcast());

        let multicast = MacAddress::new([0x01, 0x00, 0x5e, 0x00, 0x00, 0x01]);
        assert!(multicast.is_multicast());
    }

    #[test]
    fn test_ethertype() {
        assert_eq!(EtherType::from_u16(0x0800), Some(EtherType::IPv4));
        assert_eq!(EtherType::from_u16(0x0806), Some(EtherType::ARP));
        assert_eq!(EtherType::from_u16(0x86DD), Some(EtherType::IPv6));
        assert_eq!(EtherType::from_u16(0x1234), None);
    }

    #[test]
    fn test_frame_parsing() {
        let frame_data = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst MAC
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src MAC
            0x08, 0x00, // EtherType (IPv4)
            0x45, 0x00, // IP header start
        ];

        let result = EthernetParser::parse(&frame_data);
        assert!(result.is_ok());

        let (dst, src, ethertype, payload) = result.unwrap();
        assert!(dst.is_broadcast());
        assert_eq!(src.0, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert_eq!(ethertype, EtherType::IPv4);
        assert_eq!(payload, &[0x45, 0x00]);
    }
}
