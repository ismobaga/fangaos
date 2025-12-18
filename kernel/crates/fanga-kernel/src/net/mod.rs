//! Networking subsystem for FangaOS
//!
//! This module provides basic networking capabilities including:
//! - Network card drivers (E1000)
//! - Ethernet frame handling
//! - ARP protocol
//! - IPv4 stack
//! - UDP and TCP protocols
//! - BSD-style socket API
//! - DHCP client

#![allow(dead_code)]

pub mod drivers;
pub mod ethernet;
pub mod arp;
pub mod ipv4;
pub mod udp;
pub mod tcp;
pub mod socket;
pub mod dhcp;

use spin::Mutex;
use alloc::vec::Vec;

/// Global network stack instance
static NETWORK_STACK: Mutex<Option<NetworkStack>> = Mutex::new(None);

/// Main network stack structure
pub struct NetworkStack {
    /// Network interface
    interface: Option<drivers::NetworkInterface>,
    /// ARP cache
    arp_cache: arp::ArpCache,
    /// IPv4 routing table
    routing_table: ipv4::RoutingTable,
    /// Active sockets
    sockets: Vec<socket::Socket>,
}

impl NetworkStack {
    /// Create a new network stack
    pub fn new() -> Self {
        Self {
            interface: None,
            arp_cache: arp::ArpCache::new(),
            routing_table: ipv4::RoutingTable::new(),
            sockets: Vec::new(),
        }
    }

    /// Initialize the network stack
    pub fn init(&mut self) -> Result<(), &'static str> {
        // Initialize network driver (E1000)
        match drivers::e1000::E1000Driver::probe() {
            Ok(driver) => {
                self.interface = Some(drivers::NetworkInterface::E1000(driver));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Get the global network stack instance
    pub fn get() -> &'static Mutex<Option<NetworkStack>> {
        &NETWORK_STACK
    }
}

/// Initialize the networking subsystem
pub fn init() -> Result<(), &'static str> {
    let mut stack = NetworkStack::new();
    stack.init()?;
    *NETWORK_STACK.lock() = Some(stack);
    Ok(())
}
