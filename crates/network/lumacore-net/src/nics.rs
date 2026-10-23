// ███╗   ██╗██╗ ██████╗███████╗
// ████╗  ██║██║██╔════╝██╔════╝
// ██╔██╗ ██║██║██║     ███████╗
// ██║╚██╗██║██║██║     ╚════██║
// ██║ ╚████║██║╚██████╗███████║
// ╚═╝  ╚═══╝╚═╝ ╚═════╝╚══════╝
// FILE: nics

//! The station's network interfaces (IPv4) and how a network mapping
//! (`Set Network "Rig" Interface en1`) finds one (decisions §11).
//!
//! Owns:          Nic, the station's IPv4 interfaces, how a network mapping finds one, a socket listening on one.
//! Doesn't own:   which NIC a network maps to (the station table, lumacore-objects).
//! Entry points:  Nic, list, resolve, listen.
//! Decisions:     §11.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};

use socket2::{Domain, Protocol, Socket, Type};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One IPv4 address of a network interface: plain data, the protocols' (decisions §34 25.14).
pub use lumacore_protocols::Nic;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC


/// Every IPv4 address of every interface (loopback included).
pub fn list() -> Vec<Nic> {
    let Ok(ifaces) = if_addrs::get_if_addrs() else {
        return Vec::new();
    };
    ifaces
        .into_iter()
        .filter_map(|i| match i.addr {
            if_addrs::IfAddr::V4(v4) => Some(Nic {
                index: i.index,
                name: i.name,
                ip: v4.ip,
                netmask: v4.netmask,
                broadcast: v4
                    .broadcast
                    .unwrap_or_else(|| directed_broadcast(v4.ip, v4.netmask)),
            }),
            _ => None,
        })
        .collect()
}

fn directed_broadcast(ip: Ipv4Addr, mask: Ipv4Addr) -> Ipv4Addr {
    Ipv4Addr::from(u32::from(ip) | !u32::from(mask))
}

/// The interface a mapping names: an interface name (`en1`), one of its
/// addresses (`10.0.0.5`) or a subnet (`10.0.0.0/24`, a preferred IP).
pub fn resolve<'a>(spec: &str, nics: &'a [Nic]) -> Option<&'a Nic> {
    let spec = spec.trim();
    if let Some((net, len)) = spec.split_once('/') {
        let net: Ipv4Addr = net.parse().ok()?;
        let len: u32 = len.parse().ok().filter(|l| *l <= 32)?;
        let mask = if len == 0 { 0 } else { u32::MAX << (32 - len) };
        return nics
            .iter()
            .find(|n| u32::from(n.ip) & mask == u32::from(net) & mask);
    }
    if let Ok(ip) = spec.parse::<Ipv4Addr>() {
        return nics.iter().find(|n| n.ip == ip);
    }
    nics.iter().find(|n| n.name.eq_ignore_ascii_case(spec))
}

/// A non-blocking UDP socket that receives what arrives on `nic` at
/// `port` (unicast, broadcast and joined multicast groups). It is bound to
/// the wildcard address — a socket bound to the NIC's own address misses
/// broadcasts — and pinned to the interface where the OS allows it. The
/// bool says whether it is pinned; when not, the caller filters packets by
/// the NIC's subnet (decisions §11 "Step 4c hardening").
pub fn listen(nic: &Nic, port: u16) -> std::io::Result<(UdpSocket, bool)> {
    let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    s.set_reuse_address(true)?;
    #[cfg(unix)]
    s.set_reuse_port(true)?;
    s.set_broadcast(true)?;
    s.bind(&SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).into())?;
    s.set_nonblocking(true)?;
    let pinned = pin(&s, nic);
    Ok((s.into(), pinned))
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "linux", target_os = "android"))]
fn pin(s: &Socket, nic: &Nic) -> bool {
    nic.index
        .and_then(std::num::NonZeroU32::new)
        .is_some_and(|i| s.bind_device_by_index_v4(Some(i)).is_ok())
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "linux", target_os = "android")))]
fn pin(_: &Socket, _: &Nic) -> bool {
    false
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
