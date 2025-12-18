//! UDP protocol implementation
//!
//! Provides connectionless datagram service

use alloc::vec::Vec;
use super::arp::Ipv4Address;

/// UDP header structure
#[repr(C, packed)]
pub struct UdpHeader {
    /// Source port
    pub src_port: u16,
    /// Destination port
    pub dst_port: u16,
    /// Length (header + data)
    pub length: u16,
    /// Checksum
    pub checksum: u16,
}

/// UDP packet parser
pub struct UdpParser;

impl UdpParser {
    /// Parse a UDP packet
    pub fn parse(data: &[u8]) -> Result<(UdpHeader, &[u8]), &'static str> {
        if data.len() < 8 {
            return Err("UDP packet too short");
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let length = u16::from_be_bytes([data[4], data[5]]);
        let checksum = u16::from_be_bytes([data[6], data[7]]);

        let header = UdpHeader {
            src_port,
            dst_port,
            length,
            checksum,
        };

        Ok((header, &data[8..]))
    }

    /// Build a UDP packet
    pub fn build(
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) -> Vec<u8> {
        let length = 8 + payload.len();
        let mut packet = Vec::with_capacity(length);

        // Source port
        packet.extend_from_slice(&src_port.to_be_bytes());
        // Destination port
        packet.extend_from_slice(&dst_port.to_be_bytes());
        // Length
        packet.extend_from_slice(&(length as u16).to_be_bytes());
        // Checksum (0 = disabled for now)
        packet.extend_from_slice(&0u16.to_be_bytes());
        // Payload
        packet.extend_from_slice(payload);

        packet
    }

    /// Calculate UDP checksum (including pseudo-header)
    pub fn calculate_checksum(
        src_ip: Ipv4Address,
        dst_ip: Ipv4Address,
        udp_packet: &[u8],
    ) -> u16 {
        let mut sum: u32 = 0;

        // Pseudo-header: source IP
        sum += u16::from_be_bytes([src_ip.0[0], src_ip.0[1]]) as u32;
        sum += u16::from_be_bytes([src_ip.0[2], src_ip.0[3]]) as u32;

        // Pseudo-header: destination IP
        sum += u16::from_be_bytes([dst_ip.0[0], dst_ip.0[1]]) as u32;
        sum += u16::from_be_bytes([dst_ip.0[2], dst_ip.0[3]]) as u32;

        // Pseudo-header: protocol (17 for UDP)
        sum += 17u32;

        // Pseudo-header: UDP length
        sum += udp_packet.len() as u32;

        // UDP packet
        for i in (0..udp_packet.len()).step_by(2) {
            if i + 1 < udp_packet.len() {
                let word = u16::from_be_bytes([udp_packet[i], udp_packet[i + 1]]);
                sum += word as u32;
            } else {
                sum += (udp_packet[i] as u32) << 8;
            }
        }

        // Add carry
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !sum as u16
    }
}

/// UDP socket structure
pub struct UdpSocket {
    /// Local address
    pub local_addr: Ipv4Address,
    /// Local port
    pub local_port: u16,
    /// Remote address (if connected)
    pub remote_addr: Option<Ipv4Address>,
    /// Remote port (if connected)
    pub remote_port: Option<u16>,
}

impl UdpSocket {
    /// Create a new UDP socket
    pub fn new(local_addr: Ipv4Address, local_port: u16) -> Self {
        Self {
            local_addr,
            local_port,
            remote_addr: None,
            remote_port: None,
        }
    }

    /// Bind to a local address and port
    pub fn bind(&mut self, addr: Ipv4Address, port: u16) {
        self.local_addr = addr;
        self.local_port = port;
    }

    /// Connect to a remote address and port
    pub fn connect(&mut self, addr: Ipv4Address, port: u16) {
        self.remote_addr = Some(addr);
        self.remote_port = Some(port);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_parsing() {
        let udp_data = [
            0x04, 0x00, // Source port (1024)
            0x00, 0x35, // Dest port (53 - DNS)
            0x00, 0x10, // Length (16 bytes)
            0x00, 0x00, // Checksum
            0x01, 0x02, 0x03, 0x04, // Payload
            0x05, 0x06, 0x07, 0x08,
        ];

        let result = UdpParser::parse(&udp_data);
        assert!(result.is_ok());

        let (header, payload) = result.unwrap();
        // Copy values from packed struct to avoid unaligned reference errors
        let src_port = header.src_port;
        let dst_port = header.dst_port;
        let length = header.length;
        
        assert_eq!(src_port, 1024);
        assert_eq!(dst_port, 53);
        assert_eq!(length, 16);
        assert_eq!(payload.len(), 8);
    }

    #[test]
    fn test_udp_build() {
        let payload = [0x01, 0x02, 0x03, 0x04];
        let packet = UdpParser::build(1024, 53, &payload);

        assert_eq!(packet.len(), 12); // 8 byte header + 4 byte payload
        
        // Verify source port
        assert_eq!(u16::from_be_bytes([packet[0], packet[1]]), 1024);
        
        // Verify destination port
        assert_eq!(u16::from_be_bytes([packet[2], packet[3]]), 53);
        
        // Verify length
        assert_eq!(u16::from_be_bytes([packet[4], packet[5]]), 12);
        
        // Verify payload
        assert_eq!(&packet[8..], &payload);
    }

    #[test]
    fn test_udp_socket() {
        let local_addr = Ipv4Address::new(192, 168, 1, 100);
        let mut socket = UdpSocket::new(local_addr, 1024);

        assert_eq!(socket.local_addr, local_addr);
        assert_eq!(socket.local_port, 1024);
        assert!(socket.remote_addr.is_none());

        let remote_addr = Ipv4Address::new(192, 168, 1, 1);
        socket.connect(remote_addr, 53);

        assert_eq!(socket.remote_addr, Some(remote_addr));
        assert_eq!(socket.remote_port, Some(53));
    }
}
