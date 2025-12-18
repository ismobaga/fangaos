# Networking Subsystem

## Overview

FangaOS implements a comprehensive networking stack with support for basic network operations, including network card drivers, protocol implementations, and a BSD-style socket API.

## Architecture

The networking subsystem is organized into several layers following the OSI model:

```
Application Layer:   Socket API
Transport Layer:     TCP, UDP
Network Layer:       IPv4, ICMP
Data Link Layer:     Ethernet, ARP
Physical Layer:      E1000 Driver
```

## Components

### Network Card Drivers

#### E1000 Driver (`net/drivers/e1000.rs`)

Intel 82540EM/82545EM/82545GM Gigabit Ethernet Controller driver.

**Features:**
- PCI device detection structure
- Memory-mapped I/O register access
- Receive and transmit descriptor rings
- MAC address reading from EEPROM
- Packet transmission and reception

**Register Offsets:**
- CTRL (0x0000): Device Control
- STATUS (0x0008): Device Status
- RCTL (0x0100): Receive Control
- TCTL (0x0400): Transmit Control
- RX/TX Descriptor Base Addresses

**Initialization:**
```rust
let driver = E1000Driver::probe()?;
// Driver handles device reset, descriptor ring setup, and enable
```

### Data Link Layer

#### Ethernet (`net/ethernet.rs`)

Ethernet frame handling with support for:

**Frame Structure:**
```
+----------------+----------------+----------+----------+
| Dst MAC (6)    | Src MAC (6)    | Type (2) | Payload  |
+----------------+----------------+----------+----------+
```

**Supported EtherTypes:**
- IPv4 (0x0800)
- ARP (0x0806)
- IPv6 (0x86DD)

**API:**
```rust
// Build an Ethernet frame
let frame = EthernetParser::build(dst_mac, src_mac, EtherType::IPv4, &payload);

// Parse an Ethernet frame
let (dst_mac, src_mac, ethertype, payload) = EthernetParser::parse(&frame)?;
```

#### ARP (`net/arp.rs`)

Address Resolution Protocol for mapping IPv4 addresses to MAC addresses.

**Features:**
- ARP cache with IP-to-MAC mapping
- ARP request/reply packet building and parsing
- Cache operations (insert, lookup, clear)

**API:**
```rust
// Create and use ARP cache
let mut cache = ArpCache::new();
cache.insert(ip_addr, mac_addr);
let mac = cache.lookup(&ip_addr);

// Build ARP request
let request = ArpParser::build_request(sender_mac, sender_ip, target_ip);

// Build ARP reply
let reply = ArpParser::build_reply(sender_mac, sender_ip, target_mac, target_ip);
```

### Network Layer

#### IPv4 (`net/ipv4.rs`)

Internet Protocol version 4 implementation.

**Features:**
- IPv4 packet parsing and construction
- Header checksum calculation
- Routing table with longest-prefix matching
- Support for ICMP, TCP, and UDP protocols

**IPv4 Header:**
```
+-------+-------+---------------+---------------+
| Ver/  | TOS   | Total Length                  |
| IHL   |       |                               |
+-------+-------+---------------+---------------+
| Identification | Flags/Fragment Offset        |
+---------------+---------------+---------------+
| TTL   | Proto | Header Checksum               |
+-------+-------+---------------+---------------+
| Source Address                                |
+-----------------------------------------------+
| Destination Address                           |
+-----------------------------------------------+
```

**API:**
```rust
// Build IPv4 packet
let packet = Ipv4Parser::build(src_ip, dst_ip, IpProtocol::UDP, &payload);

// Parse IPv4 packet
let (header, payload) = Ipv4Parser::parse(&packet)?;

// Routing table
let mut table = RoutingTable::new();
table.add_route(RouteEntry { ... });
let route = table.lookup(&dst_ip);
```

### Transport Layer

#### UDP (`net/udp.rs`)

User Datagram Protocol for connectionless communication.

**Features:**
- UDP packet building and parsing
- Checksum calculation (with pseudo-header)
- UDP socket structure

**UDP Header:**
```
+---------------+---------------+
| Source Port   | Dest Port     |
+---------------+---------------+
| Length        | Checksum      |
+---------------+---------------+
```

**API:**
```rust
// Build UDP packet
let packet = UdpParser::build(src_port, dst_port, &payload);

// Parse UDP packet
let (header, payload) = UdpParser::parse(&packet)?;

// Create UDP socket
let socket = UdpSocket::new(local_addr, local_port);
```

#### TCP (`net/tcp.rs`)

Transmission Control Protocol for reliable, ordered communication.

**Features:**
- Full TCP state machine implementation
- TCP connection management
- Send and receive buffers
- Flow control with window management

**TCP States:**
- Closed
- Listen
- SynSent
- SynReceived
- Established
- FinWait1/FinWait2
- CloseWait
- Closing
- LastAck
- TimeWait

**TCP Header:**
```
+---------------+---------------+
| Source Port   | Dest Port     |
+---------------+---------------+
| Sequence Number               |
+---------------+---------------+
| Acknowledgment Number         |
+---------------+---------------+
| Offset| Flags | Window        |
+---------------+---------------+
| Checksum      | Urgent Ptr    |
+---------------+---------------+
```

**API:**
```rust
// Create TCP connection
let mut conn = TcpConnection::new(local_addr, local_port, remote_addr, remote_port);

// Connect
conn.connect();

// Handle incoming packet
conn.handle_packet(&header, &payload);

// Close connection
conn.close();
```

### Application Layer

#### Socket API (`net/socket.rs`)

BSD-style socket interface for network programming.

**Socket Types:**
- Stream (TCP)
- Datagram (UDP)
- Raw

**Socket States:**
- Unbound
- Bound
- Listening (TCP only)
- Connected
- Closed

**API:**
```rust
// Create socket manager
let mut manager = SocketManager::new();

// Create TCP socket
let tcp_fd = manager.socket(SocketDomain::Inet, SocketType::Stream, SocketProtocol::Tcp)?;

// Create UDP socket
let udp_fd = manager.socket(SocketDomain::Inet, SocketType::Datagram, SocketProtocol::Udp)?;

// TCP operations
if let Some(Socket::Tcp(tcp_socket)) = manager.get_socket(tcp_fd) {
    tcp_socket.bind(SocketAddr::new(local_ip, local_port))?;
    tcp_socket.listen()?;
    tcp_socket.connect(remote_addr)?;
    tcp_socket.send(&data)?;
    tcp_socket.recv(&mut buffer)?;
}

// UDP operations
if let Some(Socket::Udp(udp_socket)) = manager.get_socket(udp_fd) {
    udp_socket.bind(SocketAddr::new(local_ip, local_port))?;
    udp_socket.sendto(&data, remote_addr)?;
    udp_socket.recvfrom(&mut buffer)?;
}
```

#### DHCP Client (`net/dhcp.rs`)

Dynamic Host Configuration Protocol client for automatic network configuration.

**Features:**
- DHCP discovery process
- DHCP message types (Discover, Offer, Request, Ack)
- Network configuration structure

**DHCP States:**
- Init
- Selecting
- Requesting
- Bound
- Renewing
- Rebinding

**API:**
```rust
// Create DHCP client
let mut client = DhcpClient::new(mac_address);

// Start discovery
client.start_discovery();

// Build DHCP Discover packet
let discover = client.build_discover();

// Build DHCP Request packet
let request = client.build_request(offered_ip, server_ip);
```

## Network Stack

The main network stack structure (`NetworkStack`) integrates all components:

```rust
pub struct NetworkStack {
    interface: Option<NetworkInterface>,
    arp_cache: ArpCache,
    routing_table: RoutingTable,
    sockets: Vec<Socket>,
}
```

**Initialization:**
```rust
// Initialize networking subsystem
net::init()?;

// Access network stack
let stack = NetworkStack::get().lock();
```

## Testing

The networking subsystem includes comprehensive unit and integration tests:

**Unit Tests:**
- Individual protocol parsers
- Packet building and parsing
- State machine transitions
- Cache operations

**Integration Tests:**
- Complete network stack integration
- Multi-layer packet processing
- Socket API operations
- DHCP client functionality

**Run Tests:**
```bash
# Run all tests
make test

# Run networking tests only
cd kernel/crates/fanga-kernel
cargo test --test network_integration --target x86_64-unknown-linux-gnu
```

## Future Enhancements

Planned improvements for the networking subsystem:

1. **PCI Bus Scanning**: Implement actual PCI device enumeration for E1000 detection
2. **DMA Support**: Add DMA buffer management for efficient packet transfers
3. **Interrupt Handling**: Implement interrupt-driven packet reception
4. **IPv6 Support**: Add IPv6 protocol stack
5. **ICMP**: Implement ICMP for ping and error reporting
6. **DNS Client**: Add DNS resolver for hostname resolution
7. **Advanced TCP Features**: 
   - Retransmission and timeout handling
   - Congestion control
   - Selective acknowledgment (SACK)
8. **Socket Options**: Implement socket configuration options (SO_REUSEADDR, etc.)
9. **Multicast Support**: Add multicast group management
10. **Network Statistics**: Implement network statistics and monitoring

## Performance Considerations

- **Zero-Copy**: Where possible, avoid copying packet data
- **Ring Buffers**: Use circular buffers for efficient descriptor management
- **Interrupt Coalescing**: Reduce interrupt overhead with batching
- **Checksum Offloading**: Utilize hardware checksum calculation when available

## Security Notes

The current implementation provides basic networking functionality. For production use, consider:

- Packet filtering and firewalling
- Rate limiting and DoS protection
- Secure protocol implementations (TLS/SSL)
- Input validation and bounds checking
- Memory safety in DMA operations

## References

- [Intel 82540EM Datasheet](https://www.intel.com/content/www/us/en/products/docs/network-io/ethernet/gigabit-adapters/82540em-datasheet.html)
- [RFC 791: Internet Protocol](https://tools.ietf.org/html/rfc791)
- [RFC 792: ICMP](https://tools.ietf.org/html/rfc792)
- [RFC 793: TCP](https://tools.ietf.org/html/rfc793)
- [RFC 768: UDP](https://tools.ietf.org/html/rfc768)
- [RFC 826: ARP](https://tools.ietf.org/html/rfc826)
- [RFC 2131: DHCP](https://tools.ietf.org/html/rfc2131)
- [OSDev Wiki: Network Stack](https://wiki.osdev.org/Network_Stack)
