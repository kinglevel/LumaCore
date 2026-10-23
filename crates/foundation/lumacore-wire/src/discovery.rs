//! The discovery beacon every station multicasts (and the engine info it
//! carries). The app's server list and other stations read it.

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, Socket, Type};

use crate::StationKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDiscoveryWire {
    pub station_name: String,
    pub authority: String,
    pub cmd_port: u16,
    pub shell_port: u16,
    /// The LumaSession stream port (members join here).
    #[serde(default)]
    pub stream_port: u16,
    pub session_count: u32,
    pub active_shell_clients: u32,
    pub build_id: String,
    pub sent_unix_ms: u64,
    /// Engine info advertised for remote visibility.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub engines: Vec<EngineInfo>,
    /// Station priority for authority negotiation.
    #[serde(default)]
    pub station_priority: String,
    /// HTTP / UI engine port — thin clients connect here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_port: Option<u16>,
    /// Stable station id (thin clients remember servers by it).
    #[serde(default)]
    pub station_id: String,
    #[serde(default)]
    pub version: String,
    /// Name of the loaded show.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show: Option<String>,
    /// Its LumaSession, location, whether it accepts invites, and status
    /// (Standalone, Master, Member) — the LumaSessions grid (§11).
    #[serde(default)]
    pub lumasession: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub invite: bool,
    #[serde(default)]
    pub status: String,
    /// The discovery group it beacons on (Multicast Base, §21 "6c"):
    /// beacons from another group are ignored.
    #[serde(default)]
    pub group: String,
    /// Its LumaSession epoch (a master's own, a member's master's).
    #[serde(default)]
    pub epoch: u64,
    /// Results it has applied (a master: its journal tip) — the election's
    /// "most up to date".
    #[serde(default)]
    pub applied: u64,
    /// When its LumaSession went online (unix ms) — the election's online time.
    #[serde(default)]
    pub online_since_ms: u64,
    /// Its show status.
    #[serde(default)]
    pub show_status: String,
    /// What the station is (its host type) and the protocols it speaks: the
    /// app greys out a station whose client protocol differs ("Wrong
    /// version"); 0 = a station older than the field.
    #[serde(default)]
    pub kind: StationKind,
    #[serde(default)]
    pub client_protocol: u32,
    #[serde(default)]
    pub session_protocol: u32,
    /// The station's last beacon on a clean shutdown (§30 "12b2 hardening",
    /// Servers list): listeners drop it at once instead of waiting for it to
    /// go quiet. Never set on a running station's beacons.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub leaving: bool,
}

/// Platform capabilities that engines can require.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Bind HTTP ports (axum / tokio).
    Http,
    /// DMX frame buffer rendering.
    DmxFramebuffer,
    /// Shared precision clock.
    Clock,
    /// Network I/O (sACN, Art-Net, multicast).
    Network,
    /// GPU compute / rendering (future).
    Gpu,
    /// Show data tree access.
    ShowData,
    /// Inbound event dispatch (for input engines: OSC, MIDI, etc.).
    CommandDispatch,
}

/// Runtime status of an engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineStatus {
    Stopped,
    Running,
    Error(String),
}

/// Serialisable info about an engine — included in discovery beacons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub name: String,
    pub status: EngineStatus,
    pub requires: Vec<Capability>,
    /// Port the engine is listening on, if any.
    pub port: Option<u16>,
}

/// A socket that hears beacons of both discovery groups (Default /
/// Alternative); several listeners on one machine share the port.
pub fn bind_discovery_listener_at(port: u16) -> std::io::Result<UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.bind(&SocketAddr::from(([0, 0, 0, 0], port)).into())?;
    for alternative in [false, true] {
        socket.join_multicast_v4(&crate::discovery_group(alternative), &Ipv4Addr::UNSPECIFIED)?;
    }
    // Blocking with a read timeout: the listener loop waits here instead of spinning.
    let socket: UdpSocket = socket.into();
    socket.set_read_timeout(Some(Duration::from_millis(250)))?;
    Ok(socket)
}
