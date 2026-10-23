// ██╗     ██╗███╗   ██╗███████╗
// ██║     ██║████╗  ██║██╔════╝
// ██║     ██║██╔██╗ ██║█████╗
// ██║     ██║██║╚██╗██║██╔══╝
// ███████╗██║██║ ╚████║███████╗
// ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝
// FILE: line

//! Protocol lines and networks (decisions §11, §34 "34f7c hardened").
//!
//! - A **Network** is a name the show uses ("Rig", "Guest"); each station
//!   maps it to one of its NICs (station-local). A line on a network the
//!   station hasn't mapped never sends.
//! - A **line** is one DMX Protocols row of its protocol (`Sacn 1`, kept in
//!   `ShowData.Protocols.DMXProtocols.Sacn`): network, `Mode` Output | Input, a run of local
//!   universes (`LocalUniverse` + `Amount`) and the protocol universe the run starts at, the
//!   send timing, input priority, target and user. The local universes are slots by number —
//!   wiring points, never links to a Universe object.
//!
//! Owns:          the line data the show holds: Network, Line, Direction, LineSend, InputPriority, MergeMode; their identity and walk.
//! Doesn't own:   the ops over ShowData (lumacore-tree output_ops.rs), running lines (lumacore-lines).
//! Entry points:  Line, Network, MergeMode, Direction, wire_label.
//! Decisions:     §11, §21 "6d", §34 25.14, "34f7c hardened".

use lumacore_base::node::{NodeMeta, TreeNode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// How a line reaches the network: its unicast `target`, or (None) its protocol's own way (the
/// row's `untargeted`: sACN multicast, Art-Net ArtPoll).
pub fn mode_label(protocol: &str, target: Option<Ipv4Addr>) -> String {
    match target {
        Some(ip) => format!("Unicast {ip}"),
        None => crate::protocol::find(protocol).map_or("—", |p| p.untargeted).to_string(),
    }
}

/// A wire universe as its protocol shows it (Art-Net's `0.0.5`, sACN's `5`).
pub fn wire_label(protocol: &str, wire: u16) -> String {
    crate::protocol::find(protocol).and_then(|p| p.universes).map_or_else(|| wire.to_string(), |u| (u.label)(wire))
}

/// The network a line uses when none is given (`Store Sacn 1`). Like every
/// network it sends only once a station maps it.
pub const DEFAULT_NETWORK: &str = "Default";

/// A network the show uses, by name. Which NIC it is, is station-local.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub meta: NodeMeta,
}

impl TreeNode for Network {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Network"
    }
}

/// A protocol line (decisions §34 "34f7c hardened"): one DMX Protocols row, kept under its protocol in
/// `ShowData.Protocols`. Every protocol's lines share this type and every handler; a setting the
/// line's mode doesn't use (Rate on an input) is kept and unused.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub meta: NodeMeta,
    /// Its protocol, by the row's name (`Sacn`, `ArtNet`, `Osc`): the collection it lives in
    /// (`ShowData::after_load` sets it), never saved twice.
    #[serde(skip)]
    pub protocol: String,
    /// Output (local universes go out) or Input (the protocol's universes come in).
    #[serde(default = "default_direction")]
    pub mode: Direction,
    /// The first local universe (a slot by number, never a link).
    #[serde(default = "default_local")]
    pub local_universe: u32,
    /// How many local universes from `local_universe` (0 = the line carries none).
    #[serde(default)]
    pub amount: u32,
    /// The protocol universe `local_universe` is on the wire (sACN universe, Art-Net
    /// Port-Address); the rest follow on.
    #[serde(default)]
    pub universe: u16,
    /// The network it uses (GUID link).
    pub network: Uuid,
    /// Output: unicast target (None = multicast / ArtPoll). Input: the only
    /// source accepted (None = any).
    #[serde(default)]
    pub target: Option<Ipv4Addr>,
    pub enabled: bool,
    /// Input (OSC): commands run as this user, with their rights (§11)
    /// (GUID link).
    #[serde(default)]
    pub user: Option<Uuid>,
    /// Input (DMX): the band the input competes at when its universe merges
    /// in Prio mode (input priority).
    #[serde(default)]
    pub input_priority: InputPriority,
    /// Output: frames per second (1–60; the engine renders at 60, the line
    /// samples its newest frame; decisions §24 "Step 9 hardening"). Never
    /// above what an Art-Net node reports it takes.
    #[serde(default = "default_rate")]
    pub rate: u8,
    /// Output: every frame, or changes only (with repeats and a keep-alive).
    #[serde(default)]
    pub send: LineSend,
    /// Output: µs between universes, for old nodes.
    #[serde(default)]
    pub packet_gap: u32,
    /// Output: ArtSync / E1.31 sync after the line's universes.
    #[serde(default)]
    pub sync: bool,
    /// What only its protocol has (sACN's `Priority`, OSC's `Port`), declared on the protocol's
    /// row (decisions §34 22.4); a setting the protocol doesn't use stays, dormant.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub settings: BTreeMap<String, serde_json::Value>,
}

/// A new output line's rate: every second render frame (30 Hz).
pub const DEFAULT_LINE_RATE: u8 = 30;

fn default_rate() -> u8 {
    DEFAULT_LINE_RATE
}

/// What an output line sends each time it is due.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineSend {
    /// Every universe, every time.
    #[default]
    Full,
    /// A changed universe at once and 3 repeats, then a keep-alive every
    /// second (Art-Net 4 / E1.31: 800–1000 ms).
    Changes,
}

impl LineSend {
    pub const ALL: [Self; 2] = [Self::Full, Self::Changes];
    /// The labels (= stored names), for `Set`.
    pub const LABELS: [&'static str; 2] = ["Full", "Changes"];

    pub fn label(self) -> &'static str {
        match self {
            Self::Full => "Full",
            Self::Changes => "Changes",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|m| m.label().eq_ignore_ascii_case(s.trim_matches('"')))
    }
}

fn default_direction() -> Direction {
    Direction::Out
}

fn default_local() -> u32 {
    1
}

impl Line {
    pub fn new(
        name: Option<String>,
        direction: Direction,
        protocol: &str,
        network: Uuid,
    ) -> Self {
        Self {
            meta: NodeMeta::new(name),
            protocol: protocol.to_string(),
            mode: direction,
            local_universe: 1,
            amount: 0,
            universe: crate::protocol::find(protocol).and_then(|p| p.universes).map_or(0, |u| u.first),
            network,
            target: None,
            enabled: true,
            user: None,
            input_priority: InputPriority::default(),
            rate: DEFAULT_LINE_RATE,
            send: LineSend::Full,
            packet_gap: 0,
            sync: false,
            settings: BTreeMap::new(),
        }
    }

    /// A setting of its protocol: the line's own value, else the row's declared default.
    pub fn setting(&self, key: &str) -> Option<serde_json::Value> {
        if let Some(v) = self.settings.get(key) {
            return Some(v.clone());
        }
        crate::protocol::find(&self.protocol)
            .and_then(|p| (p.settings)().into_iter().find(|d| d.key == key))
            .map(|d| d.default)
    }

    /// Its protocol's declared settings with the line's values (defaults where none is set), for
    /// Info and the grids: `[("Priority", 150)]`.
    pub fn declared(&self) -> Vec<(String, serde_json::Value)> {
        let Some(p) = crate::protocol::find(&self.protocol) else { return Vec::new() };
        (p.settings)()
            .into_iter()
            .map(|d| {
                let v = self.settings.get(&d.key).cloned().unwrap_or(d.default);
                (d.key, v)
            })
            .collect()
    }

    /// A number setting of its protocol (`Priority`, `Port`), the default where none is set.
    pub fn setting_u64(&self, key: &str) -> Option<u64> {
        self.setting(key).and_then(|v| v.as_u64())
    }

    /// An output line (Mode Output).
    pub fn sends(&self) -> bool {
        self.mode == Direction::Out
    }

    /// Its local universes, first to last (empty when `amount` is 0).
    pub fn slots(&self) -> std::ops::Range<u32> {
        self.local_universe..self.local_universe.saturating_add(self.amount)
    }

    /// Its protocol universes, first to last (wider than u16 when the run leaves the range).
    pub fn wires(&self) -> std::ops::Range<u32> {
        let first = u32::from(self.universe);
        first..first + self.amount
    }

    /// (local universe, protocol universe) of every slot of the run.
    pub fn pairs(&self) -> impl Iterator<Item = (u32, u16)> + '_ {
        self.slots().zip(self.wires()).filter_map(|(s, w)| u16::try_from(w).ok().map(|w| (s, w)))
    }

    /// `1–32 → 101–132` (or `40 ← 9`), the protocol's own spelling of its universes.
    pub fn run_label(&self) -> String {
        if self.amount == 0 {
            return "—".to_string();
        }
        let arrow = if self.mode == Direction::Out { "→" } else { "←" };
        let wire = |w: u32| wire_label(&self.protocol, u16::try_from(w).unwrap_or(u16::MAX));
        let (last_slot, last_wire) = (self.local_universe + self.amount - 1, u32::from(self.universe) + self.amount - 1);
        if self.amount == 1 {
            format!("{} {arrow} {}", self.local_universe, wire(u32::from(self.universe)))
        } else {
            format!("{}–{last_slot} {arrow} {}–{}", self.local_universe, wire(u32::from(self.universe)), wire(last_wire))
        }
    }
}

impl TreeNode for Line {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Line"
    }
}

/// The band DMX input competes at in a Prio universe (input priority,
/// highest first). Until the Priority-pool bands exist they map onto
/// today's merge layers (decisions §11 "Step 4c hardening").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum InputPriority {
    /// Above everything but Park.
    Super,
    /// LTP with frozen programmers, above playbacks.
    Prog,
    /// Above playbacks and programmers (below frozen programmers).
    Highest,
    High,
    /// The latest change wins against playbacks and programmers.
    #[default]
    #[serde(rename = "LTP")]
    Ltp,
    /// Only where nothing of ours gives a value (defaults, unpatched).
    Low,
    Lowest,
}

impl InputPriority {
    /// The labels (= stored names), for `Set`.
    pub const LABELS: [&'static str; 7] = ["Super", "Prog", "Highest", "High", "LTP", "Low", "Lowest"];

    pub const ALL: [Self; 7] = [
        Self::Super,
        Self::Prog,
        Self::Highest,
        Self::High,
        Self::Ltp,
        Self::Low,
        Self::Lowest,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Super => "Super",
            Self::Prog => "Prog",
            Self::Highest => "Highest",
            Self::High => "High",
            Self::Ltp => "LTP",
            Self::Low => "Low",
            Self::Lowest => "Lowest",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|p| p.label().eq_ignore_ascii_case(s.trim_matches('"')))
    }
}

/// Which way a line goes: its `Mode` (Output / Input).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Direction {
    #[serde(rename = "Output")]
    Out,
    #[serde(rename = "Input")]
    In,
}

impl Direction {
    /// The labels (= stored names), for `Set … "Mode"`.
    pub const LABELS: [&'static str; 2] = ["Output", "Input"];

    /// `Output` / `Input`.
    pub fn label(self) -> &'static str {
        match self {
            Self::Out => "Output",
            Self::In => "Input",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [Self::Out, Self::In].into_iter().find(|d| d.label().eq_ignore_ascii_case(s.trim_matches('"')))
    }
}

/// How DMX input merges into a local universe (`DMXMergeMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MergeMode {
    /// Input is ignored.
    Off,
    /// Input competes at its line's input priority.
    #[default]
    Prio,
    /// Highest value wins.
    #[serde(rename = "HTP", alias = "Htp")]
    Htp,
    /// Lowest value wins.
    #[serde(rename = "LowTP", alias = "LowTp")]
    LowTp,
}

impl MergeMode {
    pub const ALL: [Self; 4] = [Self::Off, Self::Prio, Self::Htp, Self::LowTp];
    /// The labels (= stored names), for `Set`.
    pub const LABELS: [&'static str; 4] = ["Off", "Prio", "HTP", "LowTP"];

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Prio => "Prio",
            Self::Htp => "HTP",
            Self::LowTp => "LowTP",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|m| m.label().eq_ignore_ascii_case(s.trim_matches('"')))
    }
}

// *************************************************************************************************
// Identity
// Every Network and Line has its GUID and walks its links (decisions §21 "6d"); the impls live
// with the types (decisions §34 25.14).
// *************************************************************************************************

impl lumacore_base::Identified for Network {
    fn meta(&self) -> &lumacore_base::NodeMeta {
        &self.meta
    }
    fn kind(&self) -> &'static str {
        "Network"
    }
}

impl lumacore_base::Identified for Line {
    fn meta(&self) -> &lumacore_base::NodeMeta {
        &self.meta
    }
    fn kind(&self) -> &'static str {
        "Line"
    }
}

impl lumacore_base::Walk for Network {
    fn walk(&self, w: &mut lumacore_base::Walker) {
        let Self { meta } = self;
        meta.walk_links(w);
    }
}

impl lumacore_base::Walk for Line {
    fn walk(&self, w: &mut lumacore_base::Walker) {
        let Self {
            meta,
            protocol,
            mode,
            local_universe,
            amount,
            universe,
            network,
            target: _,
            enabled: _,
            user,
            input_priority,
            rate: _,
            send: _,
            packet_gap: _,
            sync: _,
            settings,
        } = self;
        meta.walk_links(w);
        w.data(protocol);
        w.data(mode);
        // Slots by number: wiring points, never links (decisions §34 "34f7c hardened" 2).
        w.data(local_universe);
        w.data(amount);
        w.data(universe);
        w.link("Network", "Network", *network);
        w.link_opt("User", "User", *user);
        w.data(input_priority);
        // Values of the protocol's declared settings (numbers, words), never links.
        w.data(settings);
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
