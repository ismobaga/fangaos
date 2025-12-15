//! IPv4 protocol implementation
//!
//! Provides IPv4 packet handling and routing

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use super::arp::Ipv4Address;
use super::ethernet::MacAddress;

/// IPv4 protocol numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpProtocol {
    ICMP = 1,
    TCP = 6,
    UDP = 17,
}

impl IpProtocol {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(IpProtocol::ICMP),
            6 => Some(IpProtocol::TCP),
            17 => Some(IpProtocol::UDP),
            _ => None,
        }
    }
}

/// IPv4 packet header
#[repr(C, packed)]
pub struct Ipv4Header {
    /// Version and header length
    pub version_ihl: u8,
    /// Type of service
    pub tos: u8,
    /// Total length
    pub total_length: u16,
    /// Identification
    pub identification: u16,
    /// Flags and fragment offset
    pub flags_fragment: u16,
    /// Time to live
    pub ttl: u8,
    /// Protocol
    pub protocol: u8,
    /// Header checksum
    pub checksum: u16,
    /// Source address
    pub src_addr: [u8; 4],
    /// Destination address
    pub dst_addr: [u8; 4],
}

impl Ipv4Header {
    /// Get IP version (should be 4)
    pub fn version(&self) -> u8 {
        self.version_ihl >> 4
    }

    /// Get header length in bytes
    pub fn header_length(&self) -> usize {
        ((self.version_ihl & 0x0F) * 4) as usize
    }
}

/// IPv4 packet parser
pub struct Ipv4Parser;

impl Ipv4Parser {
    /// Parse an IPv4 packet
    pub fn parse(data: &[u8]) -> Result<(Ipv4Header, &[u8]), &'static str> {
        if data.len() < 20 {
            return Err("IPv4 packet too short");
        }

        let version_ihl = data[0];
        let tos = data[1];
        let total_length = u16::from_be_bytes([data[2], data[3]]);
        let identification = u16::from_be_bytes([data[4], data[5]]);
        let flags_fragment = u16::from_be_bytes([data[6], data[7]]);
        let ttl = data[8];
        let protocol = data[9];
        let checksum = u16::from_be_bytes([data[10], data[11]]);

        let mut src_addr = [0u8; 4];
        let mut dst_addr = [0u8; 4];
        src_addr.copy_from_slice(&data[12..16]);
        dst_addr.copy_from_slice(&data[16..20]);

        let header = Ipv4Header {
            version_ihl,
            tos,
            total_length,
            identification,
            flags_fragment,
            ttl,
            protocol,
            checksum,
            src_addr,
            dst_addr,
        };

        // Verify version
        if header.version() != 4 {
            return Err("Invalid IP version");
        }

        let header_len = header.header_length();
        if data.len() < header_len {
            return Err("Truncated IP header");
        }

        Ok((header, &data[header_len..]))
    }

    /// Calculate IPv4 checksum
    pub fn calculate_checksum(header: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        
        for i in (0..header.len()).step_by(2) {
            if i + 1 < header.len() {
                let word = u16::from_be_bytes([header[i], header[i + 1]]);
                sum += word as u32;
            } else {
                sum += (header[i] as u32) << 8;
            }
        }

        // Add carry
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !sum as u16
    }

    /// Build an IPv4 packet
    pub fn build(
        src_addr: Ipv4Address,
        dst_addr: Ipv4Address,
        protocol: IpProtocol,
        payload: &[u8],
    ) -> Vec<u8> {
        let total_length = 20 + payload.len();
        let mut packet = Vec::with_capacity(total_length);

        // Version (4) and IHL (5 = 20 bytes)
        packet.push(0x45);
        // TOS
        packet.push(0);
        // Total length
        packet.extend_from_slice(&(total_length as u16).to_be_bytes());
        // Identification
        packet.extend_from_slice(&0u16.to_be_bytes());
        // Flags and fragment offset
        packet.extend_from_slice(&0x4000u16.to_be_bytes()); // Don't fragment
        // TTL
        packet.push(64);
        // Protocol
        packet.push(protocol as u8);
        // Checksum (placeholder)
        packet.extend_from_slice(&[0, 0]);
        // Source address
        packet.extend_from_slice(&src_addr.0);
        // Destination address
        packet.extend_from_slice(&dst_addr.0);

        // Calculate and insert checksum
        let checksum = Self::calculate_checksum(&packet[0..20]);
        packet[10..12].copy_from_slice(&checksum.to_be_bytes());

        // Add payload
        packet.extend_from_slice(payload);

        packet
    }
}

/// Routing table entry
#[derive(Debug, Clone, Copy)]
pub struct RouteEntry {
    /// Destination network
    pub network: Ipv4Address,
    /// Network mask
    pub netmask: Ipv4Address,
    /// Gateway
    pub gateway: Option<Ipv4Address>,
    /// Interface MAC address
    pub interface_mac: MacAddress,
}

/// Routing table
pub struct RoutingTable {
    routes: Vec<RouteEntry>,
}

impl RoutingTable {
    /// Create a new routing table
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
        }
    }

    /// Add a route
    pub fn add_route(&mut self, entry: RouteEntry) {
        self.routes.push(entry);
    }

    /// Lookup a route for a destination address
    pub fn lookup(&self, dst: &Ipv4Address) -> Option<&RouteEntry> {
        // Find the most specific matching route
        let dst_u32 = dst.to_be_u32();
        
        self.routes.iter()
            .filter(|route| {
                let network = route.network.to_be_u32();
                let netmask = route.netmask.to_be_u32();
                (dst_u32 & netmask) == (network & netmask)
            })
            .max_by_key(|route| {
                // More specific routes have more 1s in netmask
                route.netmask.to_be_u32().count_ones()
            })
    }

    /// Clear all routes
    pub fn clear(&mut self) {
        self.routes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_header_parsing() {
        // Simple IPv4 header (no options)
        let header_data = [
            0x45, 0x00, // Version, IHL, TOS
            0x00, 0x28, // Total length (40 bytes)
            0x00, 0x00, // Identification
            0x40, 0x00, // Flags, fragment offset
            0x40, 0x06, // TTL (64), Protocol (TCP = 6)
            0x00, 0x00, // Checksum (placeholder)
            192, 168, 1, 100, // Source IP
            192, 168, 1, 1,   // Dest IP
        ];

        let result = Ipv4Parser::parse(&header_data);
        assert!(result.is_ok());

        let (header, _) = result.unwrap();
        assert_eq!(header.version(), 4);
        assert_eq!(header.header_length(), 20);
        assert_eq!(header.protocol, 6);
        assert_eq!(header.src_addr, [192, 168, 1, 100]);
        assert_eq!(header.dst_addr, [192, 168, 1, 1]);
    }

    #[test]
    fn test_checksum_calculation() {
        let header = [
            0x45, 0x00, 0x00, 0x3c, 0x1c, 0x46, 0x40, 0x00,
            0x40, 0x06, 0x00, 0x00, 0xac, 0x10, 0x0a, 0x63,
            0xac, 0x10, 0x0a, 0x0c,
        ];

        let checksum = Ipv4Parser::calculate_checksum(&header);
        assert_ne!(checksum, 0); // Should produce a non-zero checksum
    }

    #[test]
    fn test_routing_table() {
        let mut table = RoutingTable::new();
        
        let default_route = RouteEntry {
            network: Ipv4Address::new(0, 0, 0, 0),
            netmask: Ipv4Address::new(0, 0, 0, 0),
            gateway: Some(Ipv4Address::new(192, 168, 1, 1)),
            interface_mac: MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]),
        };

        let local_route = RouteEntry {
            network: Ipv4Address::new(192, 168, 1, 0),
            netmask: Ipv4Address::new(255, 255, 255, 0),
            gateway: None,
            interface_mac: MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]),
        };

        table.add_route(default_route);
        table.add_route(local_route);

        // Local address should match local route (more specific)
        let local_dst = Ipv4Address::new(192, 168, 1, 100);
        let route = table.lookup(&local_dst);
        assert!(route.is_some());
        assert_eq!(route.unwrap().netmask.0, [255, 255, 255, 0]);

        // Remote address should match default route
        let remote_dst = Ipv4Address::new(8, 8, 8, 8);
        let route = table.lookup(&remote_dst);
        assert!(route.is_some());
    }
}
