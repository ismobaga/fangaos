//! TCP protocol implementation
//!
//! Provides reliable, ordered, connection-oriented byte stream service

use alloc::vec::Vec;
use alloc::collections::VecDeque;
use super::arp::Ipv4Address;

/// TCP connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

/// TCP flags
pub mod tcp_flags {
    pub const FIN: u8 = 0x01;
    pub const SYN: u8 = 0x02;
    pub const RST: u8 = 0x04;
    pub const PSH: u8 = 0x08;
    pub const ACK: u8 = 0x10;
    pub const URG: u8 = 0x20;
}

/// TCP header structure
#[repr(C, packed)]
pub struct TcpHeader {
    /// Source port
    pub src_port: u16,
    /// Destination port
    pub dst_port: u16,
    /// Sequence number
    pub seq_num: u32,
    /// Acknowledgment number
    pub ack_num: u32,
    /// Data offset and flags
    pub data_offset_flags: u16,
    /// Window size
    pub window_size: u16,
    /// Checksum
    pub checksum: u16,
    /// Urgent pointer
    pub urgent_ptr: u16,
}

impl TcpHeader {
    /// Get data offset (header length in 32-bit words)
    pub fn data_offset(&self) -> u8 {
        ((self.data_offset_flags >> 12) & 0x0F) as u8
    }

    /// Get header length in bytes
    pub fn header_length(&self) -> usize {
        (self.data_offset() * 4) as usize
    }

    /// Get TCP flags
    pub fn flags(&self) -> u8 {
        (self.data_offset_flags & 0x3F) as u8
    }

    /// Check if flag is set
    pub fn has_flag(&self, flag: u8) -> bool {
        self.flags() & flag != 0
    }
}

/// TCP packet parser
pub struct TcpParser;

impl TcpParser {
    /// Parse a TCP packet
    pub fn parse(data: &[u8]) -> Result<(TcpHeader, &[u8]), &'static str> {
        if data.len() < 20 {
            return Err("TCP packet too short");
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let seq_num = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ack_num = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        let data_offset_flags = u16::from_be_bytes([data[12], data[13]]);
        let window_size = u16::from_be_bytes([data[14], data[15]]);
        let checksum = u16::from_be_bytes([data[16], data[17]]);
        let urgent_ptr = u16::from_be_bytes([data[18], data[19]]);

        let header = TcpHeader {
            src_port,
            dst_port,
            seq_num,
            ack_num,
            data_offset_flags,
            window_size,
            checksum,
            urgent_ptr,
        };

        let header_len = header.header_length();
        if data.len() < header_len {
            return Err("Truncated TCP header");
        }

        Ok((header, &data[header_len..]))
    }

    /// Build a TCP packet
    pub fn build(
        src_port: u16,
        dst_port: u16,
        seq_num: u32,
        ack_num: u32,
        flags: u8,
        window_size: u16,
        payload: &[u8],
    ) -> Vec<u8> {
        let mut packet = Vec::with_capacity(20 + payload.len());

        // Source port
        packet.extend_from_slice(&src_port.to_be_bytes());
        // Destination port
        packet.extend_from_slice(&dst_port.to_be_bytes());
        // Sequence number
        packet.extend_from_slice(&seq_num.to_be_bytes());
        // Acknowledgment number
        packet.extend_from_slice(&ack_num.to_be_bytes());
        // Data offset (5 * 4 = 20 bytes) and flags
        let data_offset_flags = ((5u16 << 12) | (flags as u16));
        packet.extend_from_slice(&data_offset_flags.to_be_bytes());
        // Window size
        packet.extend_from_slice(&window_size.to_be_bytes());
        // Checksum (placeholder)
        packet.extend_from_slice(&0u16.to_be_bytes());
        // Urgent pointer
        packet.extend_from_slice(&0u16.to_be_bytes());
        // Payload
        packet.extend_from_slice(payload);

        packet
    }

    /// Calculate TCP checksum (including pseudo-header)
    pub fn calculate_checksum(
        src_ip: Ipv4Address,
        dst_ip: Ipv4Address,
        tcp_packet: &[u8],
    ) -> u16 {
        let mut sum: u32 = 0;

        // Pseudo-header: source IP
        sum += u16::from_be_bytes([src_ip.0[0], src_ip.0[1]]) as u32;
        sum += u16::from_be_bytes([src_ip.0[2], src_ip.0[3]]) as u32;

        // Pseudo-header: destination IP
        sum += u16::from_be_bytes([dst_ip.0[0], dst_ip.0[1]]) as u32;
        sum += u16::from_be_bytes([dst_ip.0[2], dst_ip.0[3]]) as u32;

        // Pseudo-header: protocol (6 for TCP)
        sum += 6u32;

        // Pseudo-header: TCP length
        sum += tcp_packet.len() as u32;

        // TCP packet
        for i in (0..tcp_packet.len()).step_by(2) {
            if i + 1 < tcp_packet.len() {
                let word = u16::from_be_bytes([tcp_packet[i], tcp_packet[i + 1]]);
                sum += word as u32;
            } else {
                sum += (tcp_packet[i] as u32) << 8;
            }
        }

        // Add carry
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !sum as u16
    }
}

/// TCP connection structure
pub struct TcpConnection {
    /// Connection state
    pub state: TcpState,
    /// Local address
    pub local_addr: Ipv4Address,
    /// Local port
    pub local_port: u16,
    /// Remote address
    pub remote_addr: Ipv4Address,
    /// Remote port
    pub remote_port: u16,
    /// Send sequence number
    pub send_seq: u32,
    /// Receive sequence number
    pub recv_seq: u32,
    /// Send window
    pub send_window: u16,
    /// Receive window
    pub recv_window: u16,
    /// Receive buffer
    pub recv_buffer: VecDeque<u8>,
    /// Send buffer
    pub send_buffer: VecDeque<u8>,
}

impl TcpConnection {
    /// Create a new TCP connection
    pub fn new(
        local_addr: Ipv4Address,
        local_port: u16,
        remote_addr: Ipv4Address,
        remote_port: u16,
    ) -> Self {
        Self {
            state: TcpState::Closed,
            local_addr,
            local_port,
            remote_addr,
            remote_port,
            send_seq: 0,
            recv_seq: 0,
            send_window: 8192,
            recv_window: 8192,
            recv_buffer: VecDeque::new(),
            send_buffer: VecDeque::new(),
        }
    }

    /// Handle state transitions
    pub fn handle_packet(&mut self, header: &TcpHeader, _payload: &[u8]) {
        match self.state {
            TcpState::Closed => {
                // Nothing to do
            }
            TcpState::Listen => {
                if header.has_flag(tcp_flags::SYN) {
                    self.state = TcpState::SynReceived;
                    self.recv_seq = header.seq_num + 1;
                }
            }
            TcpState::SynSent => {
                if header.has_flag(tcp_flags::SYN | tcp_flags::ACK) {
                    self.state = TcpState::Established;
                    self.recv_seq = header.seq_num + 1;
                }
            }
            TcpState::SynReceived => {
                if header.has_flag(tcp_flags::ACK) {
                    self.state = TcpState::Established;
                }
            }
            TcpState::Established => {
                if header.has_flag(tcp_flags::FIN) {
                    self.state = TcpState::CloseWait;
                    self.recv_seq = header.seq_num + 1;
                }
            }
            TcpState::FinWait1 => {
                if header.has_flag(tcp_flags::FIN | tcp_flags::ACK) {
                    self.state = TcpState::TimeWait;
                } else if header.has_flag(tcp_flags::ACK) {
                    self.state = TcpState::FinWait2;
                }
            }
            TcpState::FinWait2 => {
                if header.has_flag(tcp_flags::FIN) {
                    self.state = TcpState::TimeWait;
                }
            }
            _ => {
                // Other states not fully implemented
            }
        }
    }

    /// Initiate connection (send SYN)
    pub fn connect(&mut self) {
        self.state = TcpState::SynSent;
        self.send_seq = 1000; // Initial sequence number
    }

    /// Close connection
    pub fn close(&mut self) {
        match self.state {
            TcpState::Established => {
                self.state = TcpState::FinWait1;
            }
            TcpState::CloseWait => {
                self.state = TcpState::LastAck;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_parsing() {
        let tcp_data = [
            0x04, 0x00, // Source port (1024)
            0x00, 0x50, // Dest port (80 - HTTP)
            0x00, 0x00, 0x00, 0x01, // Sequence number
            0x00, 0x00, 0x00, 0x00, // Ack number
            0x50, 0x02, // Data offset (5) and flags (SYN)
            0x20, 0x00, // Window size (8192)
            0x00, 0x00, // Checksum
            0x00, 0x00, // Urgent pointer
        ];

        let result = TcpParser::parse(&tcp_data);
        assert!(result.is_ok());

        let (header, _) = result.unwrap();
        // Copy values from packed struct to avoid unaligned reference errors
        let src_port = header.src_port;
        let dst_port = header.dst_port;
        let seq_num = header.seq_num;
        
        assert_eq!(src_port, 1024);
        assert_eq!(dst_port, 80);
        assert_eq!(seq_num, 1);
        assert_eq!(header.data_offset(), 5);
        assert!(header.has_flag(tcp_flags::SYN));
    }

    #[test]
    fn test_tcp_build() {
        let packet = TcpParser::build(
            1024,
            80,
            1000,
            0,
            tcp_flags::SYN,
            8192,
            &[],
        );

        assert_eq!(packet.len(), 20); // Minimum TCP header
        
        // Verify source port
        assert_eq!(u16::from_be_bytes([packet[0], packet[1]]), 1024);
        
        // Verify destination port
        assert_eq!(u16::from_be_bytes([packet[2], packet[3]]), 80);
        
        // Verify sequence number
        assert_eq!(u32::from_be_bytes([packet[4], packet[5], packet[6], packet[7]]), 1000);
    }

    #[test]
    fn test_tcp_state_machine() {
        let local_addr = Ipv4Address::new(192, 168, 1, 100);
        let remote_addr = Ipv4Address::new(192, 168, 1, 1);
        let mut conn = TcpConnection::new(local_addr, 1024, remote_addr, 80);

        assert_eq!(conn.state, TcpState::Closed);

        conn.connect();
        assert_eq!(conn.state, TcpState::SynSent);

        // Simulate receiving SYN-ACK
        let syn_ack = TcpHeader {
            src_port: 80,
            dst_port: 1024,
            seq_num: 2000,
            ack_num: 1001,
            data_offset_flags: ((5u16 << 12) | (tcp_flags::SYN | tcp_flags::ACK) as u16),
            window_size: 8192,
            checksum: 0,
            urgent_ptr: 0,
        };

        conn.handle_packet(&syn_ack, &[]);
        assert_eq!(conn.state, TcpState::Established);
    }
}
