//  █████╗  ██████╗ ██████╗███████╗███████╗███████╗
// ██╔══██╗██╔════╝██╔════╝██╔════╝██╔════╝██╔════╝
// ███████║██║     ██║     █████╗  ███████╗███████╗
// ██╔══██║██║     ██║     ██╔══╝  ╚════██║╚════██║
// ██║  ██║╚██████╗╚██████╗███████╗███████║███████║
// ╚═╝  ╚═╝ ╚═════╝ ╚═════╝╚══════╝╚══════╝╚══════╝
// FILE: access

//! Per-NIC services (decisions §11, §30 "12d part 4"): which interfaces serve LumaSession
//! (discovery, command port, replication), Shell and HTTP; loopback always serves; LumaSession off
//! (`online`) stops everything but loopback.
//!
//! Owns:          ServiceAccess (which NICs serve LumaSession, Shell, HTTP), Services (the shared copy the listeners read).
//! Doesn't own:   the per-NIC settings (the station table, lumacore-objects), the listeners themselves (the server).
//! Entry points:  ServiceAccess::{compute, lumasession_from, shell_at, http_at}, Services.
//! Decisions:     §11, §30 "12d part 4", §34 19.2.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use lumacore_net::nics::Nic;
use lumacore_tree::StationSettings;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The access every listener reads (one shared ArcSwap, decisions §34 19.2: beside
/// `ServiceAccess`).
pub type Services = Arc<arc_swap::ArcSwap<ServiceAccess>>;

/// Which local interfaces serve what (shared with the listener threads).
#[derive(Debug, Clone)]
pub struct ServiceAccess {
    pub online: bool,
    /// NICs serving LumaSession (discovery, command port, replication).
    pub lumasession: Vec<Nic>,
    /// Addresses serving Shell / HTTP.
    pub shell: Vec<Ipv4Addr>,
    pub http: Vec<Ipv4Addr>,
    /// The discovery multicast group this station uses (Multicast Base).
    pub group: Ipv4Addr,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Default for ServiceAccess {
    /// Before the first computation: everything open (today's behaviour).
    fn default() -> Self {
        Self {
            online: true,
            lumasession: Vec::new(),
            shell: Vec::new(),
            http: Vec::new(),
            group: lumacore_wire::discovery_group(false),
        }
        .everything()
    }
}

impl ServiceAccess {
    fn everything(mut self) -> Self {
        let nics = lumacore_net::nics::list();
        self.shell = nics.iter().map(|n| n.ip).collect();
        self.http = self.shell.clone();
        self.lumasession = nics;
        self
    }

    /// From the station's settings and its interfaces.
    pub fn compute(station: &StationSettings, nics: &[Nic]) -> Self {
        let on = |f: fn(&lumacore_tree::NicServices) -> bool| -> Vec<&Nic> {
            nics.iter()
                .filter(|n| station.online && f(&station.services_of(&n.name)))
                .collect()
        };
        Self {
            online: station.online,
            lumasession: on(|s| s.lumasession).into_iter().cloned().collect(),
            shell: on(|s| s.shell).into_iter().map(|n| n.ip).collect(),
            http: on(|s| s.http).into_iter().map(|n| n.ip).collect(),
            group: lumacore_wire::discovery_group(station.multicast_alternative),
        }
    }

    /// A LumaSession request (command port, discovery) from `src`.
    pub fn lumasession_from(&self, src: IpAddr) -> bool {
        if src.is_loopback() {
            return true;
        }
        let IpAddr::V4(v4) = src else {
            return false;
        };
        self.online
            && self
                .lumasession
                .iter()
                .any(|n| n.ip == v4 || n.on_subnet(v4))
    }

    /// A Shell connection that arrived on local address `local`.
    pub fn shell_at(&self, local: IpAddr) -> bool {
        local.is_loopback() || matches!(local, IpAddr::V4(v4) if self.shell.contains(&v4))
    }

    /// An HTTP connection that arrived on local address `local`.
    pub fn http_at(&self, local: IpAddr) -> bool {
        local.is_loopback() || matches!(local, IpAddr::V4(v4) if self.http.contains(&v4))
    }
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
