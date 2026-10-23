// ██████╗ ██████╗  ██████╗ ████████╗ ██████╗  ██████╗ ██████╗ ██╗
// ██╔══██╗██╔══██╗██╔═══██╗╚══██╔══╝██╔═══██╗██╔════╝██╔═══██╗██║
// ██████╔╝██████╔╝██║   ██║   ██║   ██║   ██║██║     ██║   ██║██║
// ██╔═══╝ ██╔══██╗██║   ██║   ██║   ██║   ██║██║     ██║   ██║██║
// ██║     ██║  ██║╚██████╔╝   ██║   ╚██████╔╝╚██████╗╚██████╔╝███████╗
// ╚═╝     ╚═╝  ╚═╝ ╚═════╝    ╚═╝    ╚═════╝  ╚═════╝ ╚═════╝ ╚══════╝
// FILE: protocol

//! What a protocol is (decisions §34 22): the [`ProtocolDef`] row each protocol crate adds to the
//! catalog, and the two small traits its driver implements — [`DmxDriver`] for universes out and
//! in, [`ControlListener`] for messages that become command lines. The line runtime
//! (`lumacore-lines`) finds a protocol by its row and never names one.
//!
//! Owns:          ProtocolDef, Kind, Capability, Open, DriverIdentity, Node, ControlStart, the DmxDriver and ControlListener traits, the lookup (all, find) and its freeze check.
//! Doesn't own:   the wires (lumacore-artnet, lumacore-sacn, lumacore-osc), the line threads (lumacore-lines), the catalog (lumacore-registry).
//! Entry points:  ProtocolDef, DmxDriver, ControlListener, all, find, check.
//! Decisions:     §11, §34 22.

use std::collections::BTreeSet;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::thread::JoinHandle;

use crossbeam_channel::Sender;
use lumacore_base::InboundEvent;
use crate::Nic;
use uuid::Uuid;

use crate::errors::DmxError;
use crate::input::Packet;
use crate::line_plan::LinePlan;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a protocol's lines carry (decisions §34 22.2): it picks the protocol's group in the tree
/// (`Protocols.DMXProtocols` / `Protocols.DataProtocols`, "34f7c hardened" 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Universes out, input layers merged per channel in the tick.
    Dmx,
    /// Messages that become command lines run as the line's user.
    Data,
}

impl Kind {
    /// The tree group its protocols sit in: `DMXProtocols` / `DataProtocols`.
    pub fn group(self) -> &'static str {
        match self {
            Self::Dmx => "DMXProtocols",
            Self::Data => "DataProtocols",
        }
    }
}

/// What a protocol can do beyond sending universes; the runtime asks these, never its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// A sync packet after a line's universes (ArtSync, E1.31 sync).
    Sync,
    /// It finds nodes on its network ([`DmxDriver::nodes`]); their refresh rate caps a line.
    Discovery,
    /// It sends and receives on one shared protocol port, so an unpinned socket affects its
    /// output lines too (Art-Net on 6454; sACN sends from its own port).
    SharedPort,
}

/// How the runtime opens a protocol on a network.
#[derive(Clone, Copy)]
pub enum Open {
    Dmx(fn(&DriverIdentity) -> Result<Box<dyn DmxDriver>, DmxError>),
    Control(fn(ControlStart) -> Result<Box<dyn ControlListener>, String>),
}

/// One protocol, as its crate declares it (a catalog row, decisions §34 21, 22.1).
pub struct ProtocolDef {
    /// The name a line stores and names it by (decisions §34 "34f7b hardening" 9: CamelCase,
    /// one word — `Sacn`, `ArtNet`, `Osc`; typed in any case, written out like this).
    pub name: &'static str,
    pub kind: Kind,
    /// Whether output lines and input lines can use it.
    pub outputs: bool,
    pub inputs: bool,
    /// The UDP port it uses (a Control line's own port wins).
    pub port: u16,
    pub capabilities: &'static [Capability],
    pub open: Open,
    /// What a line without a target does (`Multicast`, `ArtPoll`), for Info and the grids.
    pub untargeted: &'static str,
    /// The universes a Dmx line takes on the wire; None for Data.
    pub universes: Option<Universes>,
    /// What only this protocol has (sACN's `Priority`, OSC's `Port`): `Line.settings`, read by
    /// `Set`, the grid columns and the reference (decisions §34 22.4, the window-settings
    /// pattern; built, since a declaration holds text). A setting in section `Output` or `Input`
    /// is one of that direction's lines only.
    pub settings: fn() -> Vec<lumacore_registry::setting::SettingDef>,
    /// What the whole protocol has (e.g. Art-Net's `ArtPollRate`): on its node in the show,
    /// `Set ArtNet "<Setting>" …` (decisions §34 "34f7c hardened" 6). None declared yet.
    pub protocol_settings: fn() -> Vec<lumacore_registry::setting::SettingDef>,
}

/// The wire universes a Dmx protocol's lines take: the property that names the first one
/// (`SacnUniverse`, `PortAddress`), their range, how one is shown and typed (Art-Net's
/// Port-Address `0.0.1`, sACN's plain number).
#[derive(Clone, Copy)]
pub struct Universes {
    pub property: &'static str,
    pub first: u16,
    pub last: u16,
    pub label: fn(u16) -> String,
    pub parse: fn(&str) -> Option<u16>,
}

impl Universes {
    /// `1–63999`, `0.0.0–127.15.15`.
    pub fn range_text(&self) -> String {
        format!("{}–{}", (self.label)(self.first), (self.label)(self.last))
    }
}

/// No settings of its own.
pub fn no_settings() -> Vec<lumacore_registry::setting::SettingDef> {
    Vec::new()
}

/// Who a driver is on its network.
#[derive(Debug, Clone)]
pub struct DriverIdentity {
    pub nic: Nic,
    /// Station name (sACN source name, ArtPollReply name).
    pub name: String,
    /// Our sACN CID (our own packets are ignored).
    pub cid: Uuid,
}

/// A node a driver found on its network ([`Capability::Discovery`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub ip: Ipv4Addr,
    pub name: String,
    /// The universes (Port-Addresses) its outputs take.
    pub outputs: Vec<u16>,
    /// The most frames per second it takes; None = not reported.
    pub refresh_hz: Option<u16>,
}

/// What a Control listener starts with.
pub struct ControlStart {
    pub bind: Ipv4Addr,
    pub port: u16,
    /// The user its commands run as (decisions §31 "OSC line users").
    pub user: String,
    pub inbound: Sender<InboundEvent>,
    pub health: Option<Arc<lumacore_runtime::ThreadHealth>>,
}

/// A DMX protocol on one NIC, driven by one line thread (`lumacore_lines`). `send` and `sync`
/// run in the per-frame path: they allocate, lock and log nothing (decisions §24).
pub trait DmxDriver: Send {
    /// The output plan changed (sACN: its streams). Allocates; never inside the send.
    fn plan(&mut self, _plan: &[LinePlan]) {}
    /// Where one route of a line sends (Art-Net: `unicast`, else the nodes on it, else the
    /// broadcast); empty = the driver decides in `send`. Worked out when the plan or the nodes
    /// change, never while sending.
    fn targets(&mut self, _wire: u16, _unicast: Option<Ipv4Addr>) -> Vec<Ipv4Addr> {
        Vec::new()
    }
    /// Get ready for these (target, universe) pairs of a line, so the send allocates nothing.
    fn prepare(&mut self, _streams: &[(Ipv4Addr, u16)]) {}
    /// Send one universe of `line`; returns the packets sent.
    fn send(&mut self, wire: u16, data: &[u8; 512], line: &LinePlan, targets: &[Ipv4Addr]) -> std::io::Result<u64>;
    /// The sync packet after a line's universes ([`Capability::Sync`]).
    fn sync(&mut self, _line: &LinePlan) -> std::io::Result<()> {
        Ok(())
    }
    /// After a round of sends (sACN: universe discovery when it is due).
    fn after_send(&mut self) {}
    /// Take in input for `wires` into `packets` (opening or closing an input socket as needed);
    /// returns the first problem.
    fn receive(&mut self, wires: &BTreeSet<u16>, packets: &mut Vec<Packet>) -> Option<String>;
    /// Whether its socket on the shared protocol port is pinned to the NIC (when not, unicast
    /// may go to another program on the port); None = no such socket now.
    fn pinned(&self) -> Option<bool>;
    /// The nodes answering now ([`Capability::Discovery`]).
    fn nodes(&self) -> Vec<Node> {
        Vec::new()
    }
}

/// A running Data line.
pub trait ControlListener: Send {
    /// Ask the thread to end; returns it to join.
    fn stop(&mut self) -> Option<JoinHandle<()>>;
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl ProtocolDef {
    pub fn can(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }
}

/// Every protocol in this build, in registration order.
pub fn all() -> impl Iterator<Item = &'static ProtocolDef> {
    lumacore_registry::get().rows::<ProtocolDef>()
}

/// The protocol named `name` (any case), if this build has it.
pub fn find(name: &str) -> Option<&'static ProtocolDef> {
    all().find(|p| p.name.eq_ignore_ascii_case(name))
}

/// The freeze check of the protocol rows: one name, one protocol; the opener fits the kind.
pub fn check(catalog: &lumacore_registry::Catalog) -> Vec<String> {
    let rows: Vec<&ProtocolDef> = catalog.rows::<ProtocolDef>().collect();
    let mut wrong = Vec::new();
    for (i, p) in rows.iter().enumerate() {
        if rows[..i].iter().any(|o| o.name.eq_ignore_ascii_case(p.name)) {
            wrong.push(format!("protocol {}: declared twice", p.name));
        }
        let fits = matches!((p.kind, p.open), (Kind::Dmx, Open::Dmx(_)) | (Kind::Data, Open::Control(_)));
        if !fits {
            wrong.push(format!("protocol {}: its opener doesn't fit its kind", p.name));
        }
        if !p.outputs && !p.inputs {
            wrong.push(format!("protocol {}: takes neither output nor input lines", p.name));
        }
        if (p.kind == Kind::Dmx) != p.universes.is_some() {
            wrong.push(format!("protocol {}: a Dmx protocol declares its universes, a Data one none", p.name));
        }
    }
    wrong
}
