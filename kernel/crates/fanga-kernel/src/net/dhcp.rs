//! DHCP (Dynamic Host Configuration Protocol) client
//!
//! Provides automatic network configuration

use alloc::vec::Vec;
use super::arp::Ipv4Address;
use super::ethernet::MacAddress;

/// DHCP message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpMessageType {
    Discover = 1,
    Offer = 2,
    Request = 3,
    Decline = 4,
    Ack = 5,
    Nak = 6,
    Release = 7,
    Inform = 8,
}

impl DhcpMessageType {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(DhcpMessageType::Discover),
            2 => Some(DhcpMessageType::Offer),
            3 => Some(DhcpMessageType::Request),
            4 => Some(DhcpMessageType::Decline),
            5 => Some(DhcpMessageType::Ack),
            6 => Some(DhcpMessageType::Nak),
            7 => Some(DhcpMessageType::Release),
            8 => Some(DhcpMessageType::Inform),
            _ => None,
        }
    }
}

/// DHCP packet structure
#[repr(C, packed)]
pub struct DhcpPacket {
    /// Operation (1 = request, 2 = reply)
    pub op: u8,
    /// Hardware type (1 = Ethernet)
    pub htype: u8,
    /// Hardware address length
    pub hlen: u8,
    /// Hops
    pub hops: u8,
    /// Transaction ID
    pub xid: u32,
    /// Seconds elapsed
    pub secs: u16,
    /// Flags
    pub flags: u16,
    /// Client IP address
    pub ciaddr: [u8; 4],
    /// Your IP address
    pub yiaddr: [u8; 4],
    /// Server IP address
    pub siaddr: [u8; 4],
    /// Gateway IP address
    pub giaddr: [u8; 4],
    /// Client hardware address
    pub chaddr: [u8; 16],
    /// Server name
    pub sname: [u8; 64],
    /// Boot file name
    pub file: [u8; 128],
}

/// DHCP option codes
pub mod dhcp_options {
    pub const SUBNET_MASK: u8 = 1;
    pub const ROUTER: u8 = 3;
    pub const DNS_SERVER: u8 = 6;
    pub const REQUESTED_IP: u8 = 50;
    pub const LEASE_TIME: u8 = 51;
    pub const MESSAGE_TYPE: u8 = 53;
    pub const SERVER_ID: u8 = 54;
    pub const PARAM_REQUEST: u8 = 55;
    pub const END: u8 = 255;
}

/// DHCP configuration result
#[derive(Debug, Clone, Copy)]
pub struct DhcpConfig {
    /// Assigned IP address
    pub ip_address: Ipv4Address,
    /// Subnet mask
    pub subnet_mask: Ipv4Address,
    /// Gateway address
    pub gateway: Option<Ipv4Address>,
    /// DNS server
    pub dns_server: Option<Ipv4Address>,
    /// Lease time in seconds
    pub lease_time: u32,
}

/// DHCP client state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpState {
    Init,
    Selecting,
    Requesting,
    Bound,
    Renewing,
    Rebinding,
}

/// DHCP client
pub struct DhcpClient {
    /// Client state
    pub state: DhcpState,
    /// Client MAC address
    pub mac_address: MacAddress,
    /// Transaction ID
    pub transaction_id: u32,
    /// Current configuration
    pub config: Option<DhcpConfig>,
}

impl DhcpClient {
    /// Create a new DHCP client
    pub fn new(mac_address: MacAddress) -> Self {
        Self {
            state: DhcpState::Init,
            mac_address,
            transaction_id: 0x12345678, // TODO: Generate random ID
            config: None,
        }
    }

    /// Build a DHCP DISCOVER packet
    pub fn build_discover(&self) -> Vec<u8> {
        let mut packet = Vec::with_capacity(300);

        // Operation: Request
        packet.push(1);
        // Hardware type: Ethernet
        packet.push(1);
        // Hardware address length
        packet.push(6);
        // Hops
        packet.push(0);
        // Transaction ID
        packet.extend_from_slice(&self.transaction_id.to_be_bytes());
        // Seconds elapsed
        packet.extend_from_slice(&0u16.to_be_bytes());
        // Flags (broadcast)
        packet.extend_from_slice(&0x8000u16.to_be_bytes());
        // Client IP (0.0.0.0)
        packet.extend_from_slice(&[0; 4]);
        // Your IP (0.0.0.0)
        packet.extend_from_slice(&[0; 4]);
        // Server IP (0.0.0.0)
        packet.extend_from_slice(&[0; 4]);
        // Gateway IP (0.0.0.0)
        packet.extend_from_slice(&[0; 4]);
        // Client hardware address
        packet.extend_from_slice(&self.mac_address.0);
        packet.extend_from_slice(&[0; 10]); // Padding
        // Server name (empty)
        packet.extend_from_slice(&[0; 64]);
        // Boot file (empty)
        packet.extend_from_slice(&[0; 128]);

        // Magic cookie
        packet.extend_from_slice(&[0x63, 0x82, 0x53, 0x63]);

        // DHCP Message Type option
        packet.push(dhcp_options::MESSAGE_TYPE);
        packet.push(1);
        packet.push(DhcpMessageType::Discover as u8);

        // Parameter Request List option
        packet.push(dhcp_options::PARAM_REQUEST);
        packet.push(3);
        packet.push(dhcp_options::SUBNET_MASK);
        packet.push(dhcp_options::ROUTER);
        packet.push(dhcp_options::DNS_SERVER);

        // End option
        packet.push(dhcp_options::END);

        packet
    }

    /// Build a DHCP REQUEST packet
    pub fn build_request(&self, offered_ip: Ipv4Address, server_ip: Ipv4Address) -> Vec<u8> {
        let mut packet = Vec::with_capacity(300);

        // Operation: Request
        packet.push(1);
        // Hardware type: Ethernet
        packet.push(1);
        // Hardware address length
        packet.push(6);
        // Hops
        packet.push(0);
        // Transaction ID
        packet.extend_from_slice(&self.transaction_id.to_be_bytes());
        // Seconds elapsed
        packet.extend_from_slice(&0u16.to_be_bytes());
        // Flags (broadcast)
        packet.extend_from_slice(&0x8000u16.to_be_bytes());
        // Client IP (0.0.0.0)
        packet.extend_from_slice(&[0; 4]);
        // Your IP (0.0.0.0)
        packet.extend_from_slice(&[0; 4]);
        // Server IP
        packet.extend_from_slice(&server_ip.0);
        // Gateway IP (0.0.0.0)
        packet.extend_from_slice(&[0; 4]);
        // Client hardware address
        packet.extend_from_slice(&self.mac_address.0);
        packet.extend_from_slice(&[0; 10]); // Padding
        // Server name (empty)
        packet.extend_from_slice(&[0; 64]);
        // Boot file (empty)
        packet.extend_from_slice(&[0; 128]);

        // Magic cookie
        packet.extend_from_slice(&[0x63, 0x82, 0x53, 0x63]);

        // DHCP Message Type option
        packet.push(dhcp_options::MESSAGE_TYPE);
        packet.push(1);
        packet.push(DhcpMessageType::Request as u8);

        // Requested IP Address option
        packet.push(dhcp_options::REQUESTED_IP);
        packet.push(4);
        packet.extend_from_slice(&offered_ip.0);

        // Server Identifier option
        packet.push(dhcp_options::SERVER_ID);
        packet.push(4);
        packet.extend_from_slice(&server_ip.0);

        // End option
        packet.push(dhcp_options::END);

        packet
    }

    /// Start DHCP discovery process
    pub fn start_discovery(&mut self) {
        self.state = DhcpState::Selecting;
        // Build and send DISCOVER packet
        let _discover = self.build_discover();
        // TODO: Send through network stack
    }

    /// Handle received DHCP packet
    pub fn handle_packet(&mut self, _packet: &[u8]) -> Result<(), &'static str> {
        // TODO: Parse packet and handle based on state
        Err("DHCP packet handling not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dhcp_message_type() {
        assert_eq!(DhcpMessageType::from_u8(1), Some(DhcpMessageType::Discover));
        assert_eq!(DhcpMessageType::from_u8(5), Some(DhcpMessageType::Ack));
        assert_eq!(DhcpMessageType::from_u8(99), None);
    }

    #[test]
    fn test_dhcp_client_creation() {
        let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let client = DhcpClient::new(mac);

        assert_eq!(client.state, DhcpState::Init);
        assert_eq!(client.mac_address, mac);
        assert!(client.config.is_none());
    }

    #[test]
    fn test_dhcp_discover_packet() {
        let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let client = DhcpClient::new(mac);

        let discover = client.build_discover();

        // Check minimum packet length
        assert!(discover.len() >= 236);

        // Check operation (request)
        assert_eq!(discover[0], 1);

        // Check hardware type (Ethernet)
        assert_eq!(discover[1], 1);

        // Check hardware address length
        assert_eq!(discover[2], 6);

        // Check MAC address
        assert_eq!(&discover[28..34], &mac.0);
    }

    #[test]
    fn test_dhcp_request_packet() {
        let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let client = DhcpClient::new(mac);

        let offered_ip = Ipv4Address::new(192, 168, 1, 100);
        let server_ip = Ipv4Address::new(192, 168, 1, 1);

        let request = client.build_request(offered_ip, server_ip);

        // Check minimum packet length
        assert!(request.len() >= 236);

        // Check operation (request)
        assert_eq!(request[0], 1);

        // Check MAC address
        assert_eq!(&request[28..34], &mac.0);
    }
}
