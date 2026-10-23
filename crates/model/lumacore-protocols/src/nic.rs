// ███╗   ██╗██╗ ██████╗
// ████╗  ██║██║██╔════╝
// ██╔██╗ ██║██║██║
// ██║╚██╗██║██║██║
// ██║ ╚████║██║╚██████╗
// ╚═╝  ╚═══╝╚═╝ ╚═════╝
// FILE: nic

//! A network interface address as plain data (decisions §34 25.14): what a driver is bound to.
//! Listing the station's interfaces from the OS is `lumacore-net`'s.
//!
//! Owns:          Nic: one IPv4 address of an interface, as plain data.
//! Doesn't own:   listing the OS's interfaces, sockets (lumacore-net nics.rs).
//! Entry points:  Nic, Nic::on_subnet.
//! Decisions:     §34 25.14.

use std::net::Ipv4Addr;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One IPv4 address of a network interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nic {
    pub name: String,
    pub ip: Ipv4Addr,
    pub netmask: Ipv4Addr,
    /// The directed broadcast (e.g. 10.0.0.255).
    pub broadcast: Ipv4Addr,
    /// The OS interface index (binds receive sockets to this NIC).
    pub index: Option<u32>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Nic {
    /// Whether `ip` is on this NIC's subnet.
    pub fn on_subnet(&self, ip: Ipv4Addr) -> bool {
        let mask = u32::from(self.netmask);
        u32::from(ip) & mask == u32::from(self.ip) & mask
    }
}
