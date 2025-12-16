//! ARP (Address Resolution Protocol) implementation
//!
//! Provides address resolution between IP addresses and MAC addresses

use alloc::collections::BTreeMap;
use super::ethernet::MacAddress;

/// IPv4 address structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    /// Create a new IPv4 address
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    /// Convert to u32 in network byte order
    pub fn to_be_u32(&self) -> u32 {
        u32::from_be_bytes(self.0)
    }

    /// Create from u32 in network byte order
    pub fn from_be_u32(value: u32) -> Self {
        Self(value.to_be_bytes())
    }
}

/// ARP operation codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpOperation {
    Request = 1,
    Reply = 2,
}

impl ArpOperation {
    /// Convert from u16
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(ArpOperation::Request),
            2 => Some(ArpOperation::Reply),
            _ => None,
        }
    }
}

/// ARP packet structure
#[repr(C, packed)]
pub struct ArpPacket {
    /// Hardware type (Ethernet = 1)
    pub hw_type: u16,
    /// Protocol type (IPv4 = 0x0800)
    pub proto_type: u16,
    /// Hardware address length
    pub hw_addr_len: u8,
    /// Protocol address length
    pub proto_addr_len: u8,
    /// Operation
    pub operation: u16,
    /// Sender hardware address
    pub sender_hw_addr: [u8; 6],
    /// Sender protocol address
    pub sender_proto_addr: [u8; 4],
    /// Target hardware address
    pub target_hw_addr: [u8; 6],
    /// Target protocol address
    pub target_proto_addr: [u8; 4],
}

/// ARP cache entry
#[derive(Debug, Clone, Copy)]
struct ArpCacheEntry {
    mac_address: MacAddress,
    // Future: add timestamp for expiration
}

/// ARP cache
pub struct ArpCache {
    cache: BTreeMap<Ipv4Address, ArpCacheEntry>,
}

impl ArpCache {
    /// Create a new ARP cache
    pub fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
        }
    }

    /// Insert an entry into the cache
    pub fn insert(&mut self, ip: Ipv4Address, mac: MacAddress) {
        self.cache.insert(ip, ArpCacheEntry { mac_address: mac });
    }

    /// Lookup an IP address in the cache
    pub fn lookup(&self, ip: &Ipv4Address) -> Option<MacAddress> {
        self.cache.get(ip).map(|entry| entry.mac_address)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// ARP packet parser
pub struct ArpParser;

impl ArpParser {
    /// Parse an ARP packet
    pub fn parse(data: &[u8]) -> Result<ArpPacket, &'static str> {
        if data.len() < 28 {
            return Err("ARP packet too short");
        }

        let hw_type = u16::from_be_bytes([data[0], data[1]]);
        let proto_type = u16::from_be_bytes([data[2], data[3]]);
        let hw_addr_len = data[4];
        let proto_addr_len = data[5];
        let operation = u16::from_be_bytes([data[6], data[7]]);

        let mut sender_hw_addr = [0u8; 6];
        let mut sender_proto_addr = [0u8; 4];
        let mut target_hw_addr = [0u8; 6];
        let mut target_proto_addr = [0u8; 4];

        sender_hw_addr.copy_from_slice(&data[8..14]);
        sender_proto_addr.copy_from_slice(&data[14..18]);
        target_hw_addr.copy_from_slice(&data[18..24]);
        target_proto_addr.copy_from_slice(&data[24..28]);

        Ok(ArpPacket {
            hw_type,
            proto_type,
            hw_addr_len,
            proto_addr_len,
            operation,
            sender_hw_addr,
            sender_proto_addr,
            target_hw_addr,
            target_proto_addr,
        })
    }

    /// Build an ARP request packet
    pub fn build_request(
        sender_mac: MacAddress,
        sender_ip: Ipv4Address,
        target_ip: Ipv4Address,
    ) -> [u8; 28] {
        let mut packet = [0u8; 28];
        
        // Hardware type (Ethernet)
        packet[0..2].copy_from_slice(&1u16.to_be_bytes());
        // Protocol type (IPv4)
        packet[2..4].copy_from_slice(&0x0800u16.to_be_bytes());
        // Hardware address length
        packet[4] = 6;
        // Protocol address length
        packet[5] = 4;
        // Operation (Request)
        packet[6..8].copy_from_slice(&1u16.to_be_bytes());
        // Sender hardware address
        packet[8..14].copy_from_slice(&sender_mac.0);
        // Sender protocol address
        packet[14..18].copy_from_slice(&sender_ip.0);
        // Target hardware address (unknown, set to zero)
        packet[18..24].copy_from_slice(&[0; 6]);
        // Target protocol address
        packet[24..28].copy_from_slice(&target_ip.0);

        packet
    }

    /// Build an ARP reply packet
    pub fn build_reply(
        sender_mac: MacAddress,
        sender_ip: Ipv4Address,
        target_mac: MacAddress,
        target_ip: Ipv4Address,
    ) -> [u8; 28] {
        let mut packet = [0u8; 28];
        
        // Hardware type (Ethernet)
        packet[0..2].copy_from_slice(&1u16.to_be_bytes());
        // Protocol type (IPv4)
        packet[2..4].copy_from_slice(&0x0800u16.to_be_bytes());
        // Hardware address length
        packet[4] = 6;
        // Protocol address length
        packet[5] = 4;
        // Operation (Reply)
        packet[6..8].copy_from_slice(&2u16.to_be_bytes());
        // Sender hardware address
        packet[8..14].copy_from_slice(&sender_mac.0);
        // Sender protocol address
        packet[14..18].copy_from_slice(&sender_ip.0);
        // Target hardware address
        packet[18..24].copy_from_slice(&target_mac.0);
        // Target protocol address
        packet[24..28].copy_from_slice(&target_ip.0);

        packet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_address() {
        let ip = Ipv4Address::new(192, 168, 1, 1);
        assert_eq!(ip.0, [192, 168, 1, 1]);
        
        let ip2 = Ipv4Address::from_be_u32(ip.to_be_u32());
        assert_eq!(ip, ip2);
    }

    #[test]
    fn test_arp_cache() {
        let mut cache = ArpCache::new();
        let ip = Ipv4Address::new(192, 168, 1, 1);
        let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);

        assert!(cache.lookup(&ip).is_none());

        cache.insert(ip, mac);
        assert_eq!(cache.lookup(&ip), Some(mac));

        cache.clear();
        assert!(cache.lookup(&ip).is_none());
    }

    #[test]
    fn test_arp_request_build() {
        let sender_mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let sender_ip = Ipv4Address::new(192, 168, 1, 100);
        let target_ip = Ipv4Address::new(192, 168, 1, 1);

        let packet = ArpParser::build_request(sender_mac, sender_ip, target_ip);
        
        // Verify operation is request (1)
        assert_eq!(u16::from_be_bytes([packet[6], packet[7]]), 1);
        
        // Verify sender IP
        assert_eq!(&packet[14..18], &[192, 168, 1, 100]);
        
        // Verify target IP
        assert_eq!(&packet[24..28], &[192, 168, 1, 1]);
    }
}
