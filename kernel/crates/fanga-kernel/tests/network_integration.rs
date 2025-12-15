//! Integration tests for networking subsystem
//!
//! These tests verify the interaction between different networking components.

#![cfg(test)]

use fanga_kernel::net::{
    ethernet::{MacAddress, EthernetParser, EtherType},
    arp::{Ipv4Address, ArpCache, ArpParser},
    ipv4::{Ipv4Parser, IpProtocol, RoutingTable, RouteEntry},
    udp::UdpParser,
    tcp::{TcpParser, TcpConnection, TcpState, tcp_flags},
    socket::{SocketAddr, TcpSocket, UdpSocketWrapper, SocketState, SocketManager, SocketDomain, SocketType, SocketProtocol},
    dhcp::{DhcpClient, DhcpState},
};

#[test]
fn test_ethernet_frame_lifecycle() {
    // Test creating and parsing an Ethernet frame
    let src_mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    let dst_mac = MacAddress::new([0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB]);
    let payload = vec![0x01, 0x02, 0x03, 0x04];

    let frame = EthernetParser::build(dst_mac, src_mac, EtherType::IPv4, &payload);
    
    assert_eq!(frame.len(), 14 + payload.len());

    let (parsed_dst, parsed_src, ethertype, parsed_payload) = EthernetParser::parse(&frame).unwrap();
    
    assert_eq!(parsed_dst, dst_mac);
    assert_eq!(parsed_src, src_mac);
    assert_eq!(ethertype, EtherType::IPv4);
    assert_eq!(parsed_payload, &payload[..]);
}

#[test]
fn test_arp_cache_operations() {
    let mut cache = ArpCache::new();
    
    let ip1 = Ipv4Address::new(192, 168, 1, 1);
    let mac1 = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    
    let ip2 = Ipv4Address::new(192, 168, 1, 2);
    let mac2 = MacAddress::new([0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB]);

    // Initially empty
    assert!(cache.lookup(&ip1).is_none());
    assert!(cache.lookup(&ip2).is_none());

    // Add entries
    cache.insert(ip1, mac1);
    cache.insert(ip2, mac2);

    // Verify lookups
    assert_eq!(cache.lookup(&ip1), Some(mac1));
    assert_eq!(cache.lookup(&ip2), Some(mac2));

    // Clear and verify
    cache.clear();
    assert!(cache.lookup(&ip1).is_none());
    assert!(cache.lookup(&ip2).is_none());
}

#[test]
fn test_arp_request_response_cycle() {
    let sender_mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    let sender_ip = Ipv4Address::new(192, 168, 1, 100);
    let target_ip = Ipv4Address::new(192, 168, 1, 1);

    // Create ARP request
    let request = ArpParser::build_request(sender_mac, sender_ip, target_ip);
    let parsed_request = ArpParser::parse(&request).unwrap();

    // Copy values from packed struct to avoid unaligned reference errors
    let operation = parsed_request.operation;
    assert_eq!(operation, 1); // Request
    assert_eq!(parsed_request.sender_hw_addr, sender_mac.0);
    assert_eq!(parsed_request.sender_proto_addr, sender_ip.0);
    assert_eq!(parsed_request.target_proto_addr, target_ip.0);

    // Create ARP reply
    let target_mac = MacAddress::new([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    let reply = ArpParser::build_reply(target_mac, target_ip, sender_mac, sender_ip);
    let parsed_reply = ArpParser::parse(&reply).unwrap();

    // Copy values from packed struct to avoid unaligned reference errors
    let reply_operation = parsed_reply.operation;
    assert_eq!(reply_operation, 2); // Reply
    assert_eq!(parsed_reply.sender_hw_addr, target_mac.0);
    assert_eq!(parsed_reply.target_hw_addr, sender_mac.0);
}

#[test]
fn test_ipv4_packet_building_and_parsing() {
    let src_ip = Ipv4Address::new(192, 168, 1, 100);
    let dst_ip = Ipv4Address::new(192, 168, 1, 1);
    let payload = vec![0x01, 0x02, 0x03, 0x04];

    // Build IPv4 packet
    let packet = Ipv4Parser::build(src_ip, dst_ip, IpProtocol::UDP, &payload);

    // Parse it back
    let (header, parsed_payload) = Ipv4Parser::parse(&packet).unwrap();

    assert_eq!(header.version(), 4);
    assert_eq!(header.protocol, IpProtocol::UDP as u8);
    assert_eq!(header.src_addr, src_ip.0);
    assert_eq!(header.dst_addr, dst_ip.0);
    assert_eq!(parsed_payload, &payload[..]);
}

#[test]
fn test_routing_table_lookup() {
    let mut table = RoutingTable::new();
    
    let interface_mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);

    // Add default route
    table.add_route(RouteEntry {
        network: Ipv4Address::new(0, 0, 0, 0),
        netmask: Ipv4Address::new(0, 0, 0, 0),
        gateway: Some(Ipv4Address::new(192, 168, 1, 1)),
        interface_mac,
    });

    // Add specific route
    table.add_route(RouteEntry {
        network: Ipv4Address::new(192, 168, 1, 0),
        netmask: Ipv4Address::new(255, 255, 255, 0),
        gateway: None,
        interface_mac,
    });

    // Test local network address
    let local = Ipv4Address::new(192, 168, 1, 50);
    let route = table.lookup(&local).unwrap();
    assert_eq!(route.network.0, [192, 168, 1, 0]);

    // Test remote address
    let remote = Ipv4Address::new(8, 8, 8, 8);
    let route = table.lookup(&remote).unwrap();
    assert_eq!(route.network.0, [0, 0, 0, 0]);
}

#[test]
fn test_udp_packet_lifecycle() {
    let payload = vec![0x01, 0x02, 0x03, 0x04];
    
    // Build UDP packet
    let packet = UdpParser::build(1024, 53, &payload);

    // Parse it back
    let (header, parsed_payload) = UdpParser::parse(&packet).unwrap();

    // Copy values from packed struct to avoid unaligned reference errors
    let src_port = header.src_port;
    let dst_port = header.dst_port;
    let length = header.length;
    
    assert_eq!(src_port, 1024);
    assert_eq!(dst_port, 53);
    assert_eq!(length, 12); // 8 byte header + 4 byte payload
    assert_eq!(parsed_payload, &payload[..]);
}

#[test]
fn test_tcp_connection_establishment() {
    let local_addr = Ipv4Address::new(192, 168, 1, 100);
    let remote_addr = Ipv4Address::new(192, 168, 1, 1);
    
    let mut conn = TcpConnection::new(local_addr, 1024, remote_addr, 80);

    // Initial state
    assert_eq!(conn.state, TcpState::Closed);

    // Initiate connection
    conn.connect();
    assert_eq!(conn.state, TcpState::SynSent);

    // Build SYN packet
    let syn = TcpParser::build(1024, 80, 1000, 0, tcp_flags::SYN, 8192, &[]);
    let (syn_header, _) = TcpParser::parse(&syn).unwrap();
    
    assert!(syn_header.has_flag(tcp_flags::SYN));
    assert!(!syn_header.has_flag(tcp_flags::ACK));
}

#[test]
fn test_socket_api() {
    // Create socket manager
    let mut manager = SocketManager::new();

    // Create TCP socket
    let tcp_fd = manager.socket(SocketDomain::Inet, SocketType::Stream, SocketProtocol::Tcp).unwrap();
    assert_eq!(tcp_fd, 0);

    // Create UDP socket
    let udp_fd = manager.socket(SocketDomain::Inet, SocketType::Datagram, SocketProtocol::Udp).unwrap();
    assert_eq!(udp_fd, 1);

    // Close sockets
    assert!(manager.close(tcp_fd).is_ok());
    assert!(manager.close(udp_fd).is_ok());
}

#[test]
fn test_tcp_socket_bind_and_listen() {
    let mut socket = TcpSocket::new();
    let addr = SocketAddr::new(Ipv4Address::new(0, 0, 0, 0), 8080);

    // Bind socket
    assert!(socket.bind(addr).is_ok());
    assert_eq!(socket.state, SocketState::Bound);

    // Cannot bind again
    assert!(socket.bind(addr).is_err());

    // Listen on socket
    assert!(socket.listen().is_ok());
    assert_eq!(socket.state, SocketState::Listening);
}

#[test]
fn test_udp_socket_bind() {
    let mut socket = UdpSocketWrapper::new();
    let addr = SocketAddr::new(Ipv4Address::new(0, 0, 0, 0), 5353);

    // Initial state
    assert_eq!(socket.state, SocketState::Unbound);

    // Bind socket
    assert!(socket.bind(addr).is_ok());
    assert_eq!(socket.state, SocketState::Bound);

    // Cannot bind again
    assert!(socket.bind(addr).is_err());
}

#[test]
fn test_dhcp_client_initialization() {
    let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    let client = DhcpClient::new(mac);

    assert_eq!(client.state, DhcpState::Init);
    assert_eq!(client.mac_address, mac);
    assert!(client.config.is_none());
}

#[test]
fn test_dhcp_discover_packet_structure() {
    let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    let client = DhcpClient::new(mac);

    let discover = client.build_discover();

    // Verify DHCP packet structure
    assert!(discover.len() >= 236); // Minimum DHCP packet size

    // Verify operation (1 = request)
    assert_eq!(discover[0], 1);

    // Verify hardware type (1 = Ethernet)
    assert_eq!(discover[1], 1);

    // Verify hardware address length (6 for Ethernet)
    assert_eq!(discover[2], 6);

    // Verify client hardware address
    assert_eq!(&discover[28..34], &mac.0);

    // Verify magic cookie (starts at offset 236)
    assert_eq!(&discover[236..240], &[0x63, 0x82, 0x53, 0x63]);
}

#[test]
fn test_complete_network_stack_integration() {
    // Simulate a complete network communication scenario

    // 1. Ethernet layer
    let src_mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    let dst_mac = MacAddress::new([0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB]);

    // 2. IP layer
    let src_ip = Ipv4Address::new(192, 168, 1, 100);
    let dst_ip = Ipv4Address::new(192, 168, 1, 1);

    // 3. UDP layer
    let udp_payload = vec![0x01, 0x02, 0x03, 0x04];
    let udp_packet = UdpParser::build(1024, 53, &udp_payload);

    // 4. Build IP packet with UDP payload
    let ip_packet = Ipv4Parser::build(src_ip, dst_ip, IpProtocol::UDP, &udp_packet);

    // 5. Build Ethernet frame with IP packet
    let eth_frame = EthernetParser::build(dst_mac, src_mac, EtherType::IPv4, &ip_packet);

    // Now parse it all back to verify
    
    // Parse Ethernet
    let (parsed_dst, parsed_src, ethertype, ip_payload) = EthernetParser::parse(&eth_frame).unwrap();
    assert_eq!(parsed_dst, dst_mac);
    assert_eq!(parsed_src, src_mac);
    assert_eq!(ethertype, EtherType::IPv4);

    // Parse IP
    let (ip_header, udp_payload) = Ipv4Parser::parse(ip_payload).unwrap();
    assert_eq!(ip_header.src_addr, src_ip.0);
    assert_eq!(ip_header.dst_addr, dst_ip.0);
    assert_eq!(ip_header.protocol, IpProtocol::UDP as u8);

    // Parse UDP
    let (udp_header, final_payload) = UdpParser::parse(udp_payload).unwrap();
    
    // Copy values from packed struct to avoid unaligned reference errors
    let udp_src_port = udp_header.src_port;
    let udp_dst_port = udp_header.dst_port;
    
    assert_eq!(udp_src_port, 1024);
    assert_eq!(udp_dst_port, 53);
    assert_eq!(final_payload, &[0x01, 0x02, 0x03, 0x04]);
}
