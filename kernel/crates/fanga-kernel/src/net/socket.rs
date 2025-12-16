//! BSD-style socket API
//!
//! Provides a socket interface for network communication

use alloc::vec::Vec;
use super::arp::Ipv4Address;
use super::udp::UdpSocket;
use super::tcp::TcpConnection;

/// Socket domain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketDomain {
    /// IPv4
    Inet,
    /// IPv6 (not implemented)
    Inet6,
}

/// Socket type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    /// Stream socket (TCP)
    Stream,
    /// Datagram socket (UDP)
    Datagram,
    /// Raw socket
    Raw,
}

/// Socket protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketProtocol {
    /// TCP
    Tcp,
    /// UDP
    Udp,
    /// ICMP
    Icmp,
}

/// Socket address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocketAddr {
    /// IP address
    pub addr: Ipv4Address,
    /// Port number
    pub port: u16,
}

impl SocketAddr {
    /// Create a new socket address
    pub fn new(addr: Ipv4Address, port: u16) -> Self {
        Self { addr, port }
    }
}

/// Socket state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    /// Socket is not bound
    Unbound,
    /// Socket is bound to a local address
    Bound,
    /// Socket is listening (TCP only)
    Listening,
    /// Socket is connected
    Connected,
    /// Socket is closed
    Closed,
}

/// Socket structure
pub enum Socket {
    /// TCP socket
    Tcp(TcpSocket),
    /// UDP socket
    Udp(UdpSocketWrapper),
}

/// TCP socket wrapper
pub struct TcpSocket {
    /// Connection state
    pub connection: Option<TcpConnection>,
    /// Socket state
    pub state: SocketState,
    /// Local address
    pub local_addr: Option<SocketAddr>,
    /// Remote address
    pub remote_addr: Option<SocketAddr>,
}

impl TcpSocket {
    /// Create a new TCP socket
    pub fn new() -> Self {
        Self {
            connection: None,
            state: SocketState::Unbound,
            local_addr: None,
            remote_addr: None,
        }
    }

    /// Bind to a local address
    pub fn bind(&mut self, addr: SocketAddr) -> Result<(), &'static str> {
        if self.state != SocketState::Unbound {
            return Err("Socket already bound");
        }

        self.local_addr = Some(addr);
        self.state = SocketState::Bound;
        Ok(())
    }

    /// Listen for connections
    pub fn listen(&mut self) -> Result<(), &'static str> {
        if self.state != SocketState::Bound {
            return Err("Socket must be bound before listening");
        }

        self.state = SocketState::Listening;
        Ok(())
    }

    /// Connect to a remote address
    pub fn connect(&mut self, remote_addr: SocketAddr) -> Result<(), &'static str> {
        let local_addr = self.local_addr.ok_or("Socket must be bound")?;

        let mut connection = TcpConnection::new(
            local_addr.addr,
            local_addr.port,
            remote_addr.addr,
            remote_addr.port,
        );

        connection.connect();

        self.connection = Some(connection);
        self.remote_addr = Some(remote_addr);
        self.state = SocketState::Connected;

        Ok(())
    }

    /// Send data
    pub fn send(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        if self.state != SocketState::Connected {
            return Err("Socket not connected");
        }

        let connection = self.connection.as_mut().ok_or("No connection")?;
        connection.send_buffer.extend(data);

        Ok(data.len())
    }

    /// Receive data
    pub fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if self.state != SocketState::Connected {
            return Err("Socket not connected");
        }

        let connection = self.connection.as_mut().ok_or("No connection")?;
        let len = buffer.len().min(connection.recv_buffer.len());

        for i in 0..len {
            buffer[i] = connection.recv_buffer.pop_front().unwrap();
        }

        Ok(len)
    }

    /// Close the socket
    pub fn close(&mut self) {
        if let Some(ref mut conn) = self.connection {
            conn.close();
        }
        self.state = SocketState::Closed;
    }
}

/// UDP socket wrapper
pub struct UdpSocketWrapper {
    /// Inner UDP socket
    pub socket: UdpSocket,
    /// Socket state
    pub state: SocketState,
}

impl UdpSocketWrapper {
    /// Create a new UDP socket
    pub fn new() -> Self {
        Self {
            socket: UdpSocket::new(Ipv4Address::new(0, 0, 0, 0), 0),
            state: SocketState::Unbound,
        }
    }

    /// Bind to a local address
    pub fn bind(&mut self, addr: SocketAddr) -> Result<(), &'static str> {
        if self.state != SocketState::Unbound {
            return Err("Socket already bound");
        }

        self.socket.bind(addr.addr, addr.port);
        self.state = SocketState::Bound;
        Ok(())
    }

    /// Connect to a remote address
    pub fn connect(&mut self, remote_addr: SocketAddr) -> Result<(), &'static str> {
        self.socket.connect(remote_addr.addr, remote_addr.port);
        self.state = SocketState::Connected;
        Ok(())
    }

    /// Send data to a specific address
    pub fn sendto(&mut self, _data: &[u8], _addr: SocketAddr) -> Result<usize, &'static str> {
        if self.state == SocketState::Unbound {
            return Err("Socket not bound");
        }

        // TODO: Implement actual sending through network stack
        Ok(0)
    }

    /// Receive data
    pub fn recvfrom(&mut self, _buffer: &mut [u8]) -> Result<(usize, SocketAddr), &'static str> {
        if self.state == SocketState::Unbound {
            return Err("Socket not bound");
        }

        // TODO: Implement actual receiving from network stack
        Err("Not implemented")
    }

    /// Close the socket
    pub fn close(&mut self) {
        self.state = SocketState::Closed;
    }
}

/// Socket manager
pub struct SocketManager {
    sockets: Vec<Socket>,
}

impl SocketManager {
    /// Create a new socket manager
    pub fn new() -> Self {
        Self {
            sockets: Vec::new(),
        }
    }

    /// Create a new socket
    pub fn socket(&mut self, domain: SocketDomain, socket_type: SocketType, _protocol: SocketProtocol) -> Result<usize, &'static str> {
        if domain != SocketDomain::Inet {
            return Err("Only IPv4 supported");
        }

        let socket = match socket_type {
            SocketType::Stream => Socket::Tcp(TcpSocket::new()),
            SocketType::Datagram => Socket::Udp(UdpSocketWrapper::new()),
            SocketType::Raw => return Err("Raw sockets not implemented"),
        };

        let fd = self.sockets.len();
        self.sockets.push(socket);
        Ok(fd)
    }

    /// Get a socket by file descriptor
    pub fn get_socket(&mut self, fd: usize) -> Option<&mut Socket> {
        self.sockets.get_mut(fd)
    }

    /// Close a socket
    pub fn close(&mut self, fd: usize) -> Result<(), &'static str> {
        if fd >= self.sockets.len() {
            return Err("Invalid socket descriptor");
        }

        match &mut self.sockets[fd] {
            Socket::Tcp(tcp) => tcp.close(),
            Socket::Udp(udp) => udp.close(),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_addr() {
        let addr = SocketAddr::new(Ipv4Address::new(192, 168, 1, 1), 80);
        assert_eq!(addr.addr.0, [192, 168, 1, 1]);
        assert_eq!(addr.port, 80);
    }

    #[test]
    fn test_tcp_socket() {
        let mut socket = TcpSocket::new();
        assert_eq!(socket.state, SocketState::Unbound);

        let local_addr = SocketAddr::new(Ipv4Address::new(192, 168, 1, 100), 1024);
        assert!(socket.bind(local_addr).is_ok());
        assert_eq!(socket.state, SocketState::Bound);

        assert!(socket.listen().is_ok());
        assert_eq!(socket.state, SocketState::Listening);
    }

    #[test]
    fn test_udp_socket() {
        let mut socket = UdpSocketWrapper::new();
        assert_eq!(socket.state, SocketState::Unbound);

        let local_addr = SocketAddr::new(Ipv4Address::new(192, 168, 1, 100), 1024);
        assert!(socket.bind(local_addr).is_ok());
        assert_eq!(socket.state, SocketState::Bound);
    }

    #[test]
    fn test_socket_manager() {
        let mut manager = SocketManager::new();

        let tcp_fd = manager.socket(SocketDomain::Inet, SocketType::Stream, SocketProtocol::Tcp);
        assert!(tcp_fd.is_ok());
        assert_eq!(tcp_fd.unwrap(), 0);

        let udp_fd = manager.socket(SocketDomain::Inet, SocketType::Datagram, SocketProtocol::Udp);
        assert!(udp_fd.is_ok());
        assert_eq!(udp_fd.unwrap(), 1);

        assert!(manager.close(0).is_ok());
    }
}
