// ██████╗ ██╗███████╗ ██████╗ ██████╗ ██╗   ██╗███████╗██████╗ ██╗   ██╗
// ██╔══██╗██║██╔════╝██╔════╝██╔═══██╗██║   ██║██╔════╝██╔══██╗╚██╗ ██╔╝
// ██║  ██║██║███████╗██║     ██║   ██║██║   ██║█████╗  ██████╔╝ ╚████╔╝
// ██║  ██║██║╚════██║██║     ██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗  ╚██╔╝
// ██████╔╝██║███████║╚██████╗╚██████╔╝ ╚████╔╝ ███████╗██║  ██║   ██║
// ╚═════╝ ╚═╝╚══════╝ ╚═════╝ ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝   ╚═╝
// FILE: discovery

//! What the station knows of a peer from its discovery beacons (the beacon itself is
//! `lumacore_wire`).
//!
//! Owns:          DiscoveryPeerState, one peer as its beacons describe it.
//! Doesn't own:   the beacon on the wire (lumacore-wire), the discovery cache and election (the server, lumasession).
//! Entry points:  DiscoveryPeerState.
//! Decisions:     §21 "6c", §34 20.6.

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

#[derive(Debug, Clone)]
pub struct DiscoveryPeerState {
    pub key: String,
    pub station_name: String,
    pub authority: String,
    pub endpoint: String,
    pub cmd_port: u16,
    pub stream_port: u16,
    pub session_count: u32,
    pub reachable: bool,
    pub last_seen_unix_ms: u64,
    pub ip: std::net::IpAddr,
    pub lumasession: String,
    pub location: String,
    pub priority: String,
    pub status: String,
    pub version: String,
    pub invite: bool,
    pub station_id: String,
    pub epoch: u64,
    pub applied: u64,
    pub online_since_ms: u64,
    pub show_status: String,
}
