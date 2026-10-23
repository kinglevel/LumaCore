//! lumacore-wire: what goes over the wire between a client (CLI, desktop app,
//! web shell bridge) and a station (decisions §23 "Step 8"). Both sides compile
//! it; clients compile nothing else of LumaCore. Station↔station stream frames
//! are not here — they carry show types and live in the station's
//! `lumasession/`.
//!
//! Versioning (a big and a small version): [`SESSION_PROTOCOL`]
//! must match for stations to join a LumaSession, [`CLIENT_PROTOCOL`] for a
//! client to attach. Bump the number on any change to what it covers — no
//! compatibility shims (forward only).

pub mod command;
pub mod dialog;
pub mod discovery;
pub mod shell;
pub mod status;
pub mod suggest;

pub use command::{
    ClockReport, RemoteReply, RemoteRequest, TimeReply, send_request, send_request_to, try_health,
};
pub use dialog::{
    ButtonKind, Countdown, Dialog, DialogAnswer, DialogButton, DialogClosed, DialogInput, DialogRow,
    DialogSelector, DialogToggle, InputKind, Pick, PickItem, Tone,
};
pub use discovery::{
    Capability, EngineInfo, EngineStatus, SessionDiscoveryWire,
    bind_discovery_listener_at,
};
pub use shell::{
    GesturePhase, ListItem, MoveDir, RemoteShellCode, RemoteShellKey, SettingField, FieldInput, ShellClientPacket, ShellServerPacket, TagNode,
    UiEvent,
    UiFocus, ViewLayers, Viewport,
};
pub use status::{
    ClaimStatus, Decision, DeviceStatus, LumaSessionStatus, ShowStatus, StartupShow, Status, StripStatus,
};
pub use suggest::{ItemAction, ItemKind, ItemStatus, SuggestItem, Suggestions, Surface};

use serde::{Deserialize, Serialize};

// ── Protocol numbers ────────────────────────────────────────────────

/// Station↔station: discovery beacon, command port, LumaSession stream.
/// Stations only join when it matches (E-LUMA-001). 10 = decisions §30
/// "12d": a user's `rights` level replaces the role in snapshots; 11 = §30
/// "12d part 4": the station table in the show, the master's line states in
/// the session runtime; 12 = §30 "Merge of bugfixes": the merge of the R1
/// fixes (forwarded commands carry ids, `commands_done`, big frames, …); 13 = §31 raw JSON
/// snapshots; 14 = §30 "12f": a forwarded line carries its gesture mark, `Playbacks.Live`;
/// 15 = §33 21: a user's `default_object` in snapshots (show format 24); 17 = §34 "34f7b": the
/// hello carries the build's version (one build per LumaSession), lines name protocol rows; 18 =
/// §34 "34f7c": lines live under their protocol, each one run of universes (show format 27); 19 =
/// §12 "Tags hardened": the tag tree (`ShowSubtree::Tags`) and tag links on every object (format 28);
/// 20 = §12 "Tags part 1b hardened": NodeMeta in levels on every show object, pool entries snapshot
/// as `[slot, object]` (format 29); 21 = §34 round 3: a member's upload carries `gdtf` (a
/// fixture file for the master's `library/gdtf/`).
pub const SESSION_PROTOCOL: u32 = 21;

/// Client↔station: shell packets, keys, UI events, status, Health, and the
/// web page's feed socket (`/ws/feeds` `hello`; 10 = decisions §30 "12d":
/// open feeds, `auth` / `subbed` / `nosub`, revisions, no SSE; 11 = the
/// station's users carry `rights`, the level, instead of `role`; 12 = §30
/// "12d part 3": `OpenMenu` carries the tab, toggle and grid states, a
/// setting field its home, unit, level and read-only reason; 13 = §30
/// "Merge of bugfixes": the R1 fixes — no `StoreView` event, busy replies;
/// 15 = §30 "12f-a": the `AttributeBoard` event and the `attributes` feed;
/// 16 = §30 "Users & Profiles follows storage": `OpenMenu.open`, the
/// `detail` feed; 17 = §30 "12f": the `gesture` shell packet; 19 = §12 "Tags hardened": the `Tags`
/// field input (a tag tree picker); 20 = §12 tags part 4b: the `tags` feed, `Pick.by_guid`;
/// 22 = device-sheet-window: the `devices` feed; 23 = top-bar-status-strip: `Status.strip`;
/// 24 = view-bar-any-object-pool-windows: the `viewbar` feed, the `pools` feed's `profile`;
/// 25 = one-preset-pool-follows-the-programmer: `Tap.window`, the `pools` feed's `feature` / `tags`).
/// A client only attaches when it matches (E-WIRE-001).
pub const CLIENT_PROTOCOL: u32 = 25;

/// The refusal a station sends a client speaking another client protocol.
pub fn client_mismatch(client: u32, station: u32) -> String {
    format!(
        "Client protocol {client}, station speaks {station} — update the client to this station's LumaCore version [E-WIRE-001]"
    )
}

// ── Ports ───────────────────────────────────────────────────────────

/// UDP command port (Health, Login, Command, LumaSession requests).
pub const REMOTE_CMD_PORT: u16 = 47653;
/// TCP shell port (interactive sessions).
pub const REMOTE_SHELL_PORT: u16 = 47654;
/// UDP discovery beacon.
pub const SESSION_DISCOVERY_PORT: u16 = 47655;
/// HTTP / WebSocket (the UI engine).
pub const REMOTE_HTTP_PORT: u16 = 47656;
/// Perfetto trace (HTTP).
pub const REMOTE_PERFETTO_PORT: u16 = 47657;
/// TCP LumaSession stream (members ↔ master).
pub const REMOTE_STREAM_PORT: u16 = 47658;
/// Discovery multicast groups: Default and Alternative (a multicast base).
pub const SESSION_DISCOVERY_GROUP: &str = "239.255.76.67";
pub const SESSION_DISCOVERY_GROUP_ALT: &str = "239.255.76.68";
/// mDNS service stations advertise (→ the HTTP port).
pub const MDNS_SERVICE: &str = "_lumacore._tcp.local.";

/// The discovery group: Default or Alternative.
pub fn discovery_group(alternative: bool) -> std::net::Ipv4Addr {
    let group = if alternative {
        SESSION_DISCOVERY_GROUP_ALT
    } else {
        SESSION_DISCOVERY_GROUP
    };
    group.parse().expect("discovery group address")
}

// ── Station kind ────────────────────────────────────────────────────

/// What a station is (its host type). Sent in the beacon and in Health.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StationKind {
    /// A full station: show, engines, terminals (desktop, rack or headless).
    #[default]
    Server,
    /// Reserved: a DMX-only node that listens to a LumaSession.
    Node,
}

impl StationKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Server => "Server",
            Self::Node => "Node",
        }
    }
}

#[cfg(test)]
mod tests;
