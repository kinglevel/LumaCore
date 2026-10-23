//! lumacore-runtime: Runtime-only network/session/engine state for LumaCore.
//!
//! Root.Runtime holds volatile session, engine, network, and diagnostics state
//! that is discarded on restart. This keeps the persisted show tree separate
//! from live execution state used by shells and future realtime engines.

use std::collections::{BTreeMap, HashMap};

use std::fmt;
use std::str::FromStr;

use lumacore_base::variables::VariableValue;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod health;
pub mod journal;
pub mod render_stats;
pub mod live_screens;
pub use health::{HealthBoard, HealthView, ThreadGuard, ThreadHealth, ThreadHealthView};
pub use render_stats::{
    CLOCK_STATES, ClockSync, ClockView, FRAME_RING, FrameRing, FrameSample, LineStats, LineStatsView, MemberClock,
    PtpStatus, RenderStats, RenderStatsView, STAGE_COUNT, STAGES, StageView, clock_sync,
};
pub use journal::{JOURNAL_CAPACITY, Journal, JournalEntry, JournalResult};
pub use live_screens::{
    CaptureRequest, EMPTY_WINDOW, EncoderLayer, EncoderMode, EncoderState, LayoutNode, OverdueCapture, SharedScreens, SplitDir, Screen, ScreenStore, Window,
    WindowAppearance, WindowScroll, default_layout,
};

/// Runtime-only state. Never saved to `.luma` files.
#[derive(Debug)]
pub struct RuntimeState {
    /// Logged-in users' runtime state (`Root.Runtime.Users`), keyed by username.
    pub users: HashMap<String, RuntimeUser>,
    /// Remote session links joined by this runtime.
    pub joined_sessions: BTreeMap<String, JoinedSessionRuntime>,
    /// Live network service engines keyed by engine name.
    pub network_engines: HashMap<String, EngineRuntime>,
    /// Live session authority engines keyed by engine name.
    pub session_engines: HashMap<String, EngineRuntime>,
    /// Live rendering engine status keyed by engine name.
    pub rendering_engines: HashMap<String, EngineRuntime>,
    /// Data networks of this station: interfaces, protocol lines, input
    /// sources, Art-Net nodes (`Root.Runtime.Network`).
    pub network: NetworkRuntime,
    /// Server-to-server state: listeners, discovered stations, connections,
    /// members (`Root.Runtime.LumaSessions`; decisions §11).
    pub luma_sessions: LumaSessionsRuntime,
    /// Runtime-only global variables. Lost on restart.
    pub global_variables: BTreeMap<String, VariableValue>,
    /// Claimed objects by GUID (decisions §4, §21 "6d": a move keeps the
    /// claim). Session-wide: replicated with the session runtime (§21 "6b").
    pub claims: BTreeMap<uuid::Uuid, Claim>,
    /// Claim order (newest last), for `Unclaim` without arguments.
    pub claim_order: Vec<uuid::Uuid>,
    /// Users asking for an object someone else holds (the owner answers
    /// within 10 s, or it is handed over).
    pub claim_asks: Vec<ClaimAsk>,
    /// `Root.Runtime.Devices`: reserved for what bidirectional devices report
    /// back (actual position, lamp hours, errors), keyed by fixture number
    /// (decisions §11). Empty until such a transport exists. Running
    /// playbacks live in `ShowData.Playbacks` (saved, decisions §13).
    pub devices: BTreeMap<u32, serde_json::Value>,
    /// Live screens (screens) and their windows. Shared with UI engines.
    pub live_screens: SharedScreens,
    /// The change log: the last commands that changed the show.
    pub journal: Journal,
    /// Every long-lived thread's health (shared with the threads themselves).
    pub health: std::sync::Arc<HealthBoard>,
}

/// A user's runtime state under `Root.Runtime.Users.{User}` (decisions §10,
/// §16): their mode flags, attached clients, watches. Replicated, never saved.
#[derive(Debug, Clone, Default)]
pub struct RuntimeUser {
    /// Who it is while it runs: a fresh GUID, gone at logout (decisions §12 "Tags part 1b
    /// hardened" 6).
    pub meta: lumacore_base::Identity,
    /// Visible session owner / primary user.
    pub user: String,
    /// Authoritative server or station for this session.
    pub authority: String,
    /// Whether the session currently has any attached clients.
    pub attached: bool,
    /// Network-visible stations participating in the session.
    pub stations: BTreeMap<String, SessionStationRuntime>,
    /// Attached clients using the session.
    pub clients: BTreeMap<String, SessionClientRuntime>,
    /// Active watch targets re-executed and shown in the TUI.
    pub watches: Vec<WatchTarget>,
    /// Blind / Highlight / Freeze / Solo (`Runtime.Users.{User}.Modes`).
    pub modes: Modes,
    /// The user's undo/redo history (`Runtime.Users.{User}.Undo`): filed by
    /// the master, replicated, never saved (decisions §21 "Step 6").
    pub undo: lumacore_base::undo::UndoStack,
    /// What a bare `Copy` / `Cut` put aside for `Paste` (decisions §35 9, 10): replicated, never
    /// saved, a show load clears it.
    pub clipboard: Option<Clipboard>,
}

/// A user's clipboard (decisions §35 10): the objects themselves by GUID, not a snapshot — Paste
/// takes them as they are then.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clipboard {
    /// Cut (Paste moves them) or copied (Paste makes new ones).
    pub cut: bool,
    /// Their object word (`Group`).
    pub object: String,
    /// The DataPool they were in, for pools that live in one.
    pub datapool: Option<Uuid>,
    /// Their numbers when put aside, in order (for the spacing), with their GUIDs.
    pub rows: Vec<(u32, Uuid)>,
}

/// A user's mode objects (decisions §16) and their mode masters (§29:
/// `Master 2.3` Highlight / `2.5` Solo, per user).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modes {
    pub blind: bool,
    pub highlight: bool,
    pub freeze: bool,
    pub solo: bool,
    #[serde(default)]
    pub highlight_master: ModeMaster,
    #[serde(default)]
    pub solo_master: ModeMaster,
}

/// How a mode master acts (the master modes).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MasterMode {
    /// The mode key switches the effect; the level sets its strength.
    #[default]
    Toggle,
    /// The effect is on whenever the level is above 0, key or not.
    AlwaysOn,
    /// The level is ignored: the key switches the effect at full.
    Disabled,
}

impl MasterMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "toggle" => Some(Self::Toggle),
            "alwayson" => Some(Self::AlwaysOn),
            "disabled" | "off" => Some(Self::Disabled),
            _ => None,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Toggle => "Toggle",
            Self::AlwaysOn => "AlwaysOn",
            Self::Disabled => "Disabled",
        }
    }
}

/// A user's Highlight or Solo master: a level in whole percent and a mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeMaster {
    pub level: u8,
    #[serde(default)]
    pub mode: MasterMode,
}

impl Default for ModeMaster {
    fn default() -> Self {
        Self {
            level: 100,
            mode: MasterMode::Toggle,
        }
    }
}

impl ModeMaster {
    /// How strongly the effect acts (0–1) with the mode key `on`.
    pub fn strength(&self, on: bool) -> f64 {
        let level = f64::from(self.level.min(100)) / 100.0;
        match self.mode {
            MasterMode::Toggle => if on { level } else { 0.0 },
            MasterMode::AlwaysOn => level,
            MasterMode::Disabled => if on { 1.0 } else { 0.0 },
        }
    }
}

/// One mode object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserMode {
    Blind,
    Highlight,
    Freeze,
    Solo,
}

impl UserMode {
    pub const ALL: [UserMode; 4] = [
        UserMode::Blind,
        UserMode::Highlight,
        UserMode::Freeze,
        UserMode::Solo,
    ];

    pub fn name(self) -> &'static str {
        match self {
            UserMode::Blind => "Blind",
            UserMode::Highlight => "Highlight",
            UserMode::Freeze => "Freeze",
            UserMode::Solo => "Solo",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|m| m.name().eq_ignore_ascii_case(s))
    }
}

impl Modes {
    pub fn get(&self, mode: UserMode) -> bool {
        match mode {
            UserMode::Blind => self.blind,
            UserMode::Highlight => self.highlight,
            UserMode::Freeze => self.freeze,
            UserMode::Solo => self.solo,
        }
    }

    pub fn set(&mut self, mode: UserMode, on: bool) {
        *match mode {
            UserMode::Blind => &mut self.blind,
            UserMode::Highlight => &mut self.highlight,
            UserMode::Freeze => &mut self.freeze,
            UserMode::Solo => &mut self.solo,
        } = on;
    }

    /// The modes that are on, in display order.
    pub fn on(&self) -> impl Iterator<Item = UserMode> + '_ {
        UserMode::ALL.into_iter().filter(|m| self.get(*m))
    }
}


/// An active watch target (serializable for Oops steps, decisions §29 "11c").
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WatchTarget {
    /// The original command string to re-execute.
    pub command: String,
    /// Requested refresh interval in ms.
    pub interval_ms: u32,
}

/// Lightweight runtime status for an engine consumer.
#[derive(Debug, Clone, Default)]
pub struct EngineRuntime {
    pub running: bool,
    pub revision: u64,
    pub last_error: Option<String>,
}

/// The station's data networks at runtime (never saved).
#[derive(Debug, Clone, Default)]
pub struct NetworkRuntime {
    pub interfaces: BTreeMap<String, NetworkInterfaceRuntime>,
    pub active_output_lines: u32,
    /// Every protocol line's live state (`Root.Runtime.Network.Lines`).
    pub lines: Vec<LineRuntime>,
    /// Nodes a protocol with `Discovery` found (`Root.Runtime.Network.Nodes`: Art-Net's ArtPoll).
    pub nodes: Vec<NodeRuntime>,
    /// DMX input sources heard now (`Root.Runtime.Network.Sources`).
    pub sources: Vec<SourceRuntime>,
}

/// A DMX input source heard on a network (decisions §11 "Step 4c").
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SourceRuntime {
    pub protocol: String,
    pub network: String,
    /// sACN source name, or the sender's IP (Art-Net).
    pub name: String,
    /// sACN CID or IP.
    pub id: String,
    pub ip: String,
    /// sACN priority (Art-Net: 100).
    pub priority: u8,
    /// Protocol universes it sends.
    pub universes: Vec<u16>,
}

/// A protocol line's live state (decisions §11).
#[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
pub struct LineRuntime {
    /// `Out` or `In`.
    pub direction: String,
    pub line: u32,
    pub name: String,
    pub protocol: String,
    pub network: String,
    /// Sending N universes / listening / not mapped / socket error / …
    pub state: String,
    /// Something keeps the line from working (shown red).
    pub problem: bool,
    /// The line works (sending or listening).
    pub active: bool,
    /// Its socket shares the protocol port unpinned to the NIC: unicast may
    /// be lost if another program uses the port (R1-54).
    #[serde(default)]
    pub unpinned: bool,
}

/// A node a protocol found on a network (an Art-Net node that answered ArtPoll).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NodeRuntime {
    /// The protocol row that found it (`ArtNet`).
    pub protocol: String,
    pub ip: String,
    pub name: String,
    /// The wire universes its output ports take.
    pub outputs: Vec<u16>,
    /// The network it was found on.
    pub network: String,
}

/// Server-to-server runtime state (`Root.Runtime.LumaSessions`).
#[derive(Debug, Clone, Default)]
pub struct LumaSessionsRuntime {
    pub listeners: BTreeMap<String, NetworkListenerRuntime>,
    /// Stations seen by discovery (and this station), by `ip:cmd_port`.
    pub discovery: BTreeMap<String, DiscoveryPeerRuntime>,
    pub connections: BTreeMap<String, NetworkConnectionRuntime>,
    pub health: NetworkHealthRuntime,
    pub active_cmd_clients: u32,
    pub active_shell_clients: u32,
    /// Stations that joined this master, by station name.
    pub members: BTreeMap<String, MemberRuntime>,
    /// On a member: the master's station name and the users logged in there
    /// (who is where, for the lock surface — decisions §30 "12a3").
    pub master: String,
    pub master_users: Vec<String>,
    /// On a member: the master's protocol lines as it runs them (DMX
    /// Protocols shows "sent by FOH", decisions §30 "12d part 4").
    pub master_lines: Vec<LineRuntime>,
    /// The newest forwarded command the master ran for each member, by
    /// station id (replicated with the session runtime, never saved): a
    /// command resent after a reconnect or to a newly elected master is
    /// recognised and never runs twice (decisions §10 "Stations never re-run
    /// commands"; R1-20, R1-21).
    pub commands_done: BTreeMap<String, CommandsDone>,
}

/// What ran of one member's forwarded commands: its link (a fresh id per
/// join) and the newest command id of that link.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandsDone {
    pub link: uuid::Uuid,
    pub last: u64,
}

impl CommandsDone {
    /// Whether command `id` of `link` already ran.
    pub fn ran(&self, link: uuid::Uuid, id: u64) -> bool {
        self.link == link && id <= self.last
    }
}

/// A station that joined this master's LumaSession (replicated with the
/// session runtime, so every station knows who is in the session).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MemberRuntime {
    pub station: String,
    pub ip: String,
    pub cmd_port: u16,
    pub joined_unix_ms: u64,
    /// Its station stream is connected (false while it reconnects).
    #[serde(default)]
    pub connected: bool,
    /// Users logged in on that station.
    #[serde(default)]
    pub users: Vec<String>,
    /// Its stable station id and LumaSession stream port (election, handover).
    #[serde(default)]
    pub station_id: String,
    #[serde(default)]
    pub stream_port: u16,
}

/// A station-local interface exposed by the network engine.
#[derive(Debug, Clone, Default)]
pub struct NetworkInterfaceRuntime {
    pub name: String,
    pub address: String,
    pub active: bool,
    /// Services it serves (loopback always serves Shell and HTTP).
    pub lumasession: bool,
    pub shell: bool,
    pub http: bool,
}

/// A bound runtime listener owned by the network engine.
#[derive(Debug, Clone, Default)]
pub struct NetworkListenerRuntime {
    pub name: String,
    pub transport: String,
    pub bind: String,
    pub port: u16,
    pub active: bool,
}

/// A station seen by discovery (the LumaSessions grid).
#[derive(Debug, Clone, Default)]
pub struct DiscoveryPeerRuntime {
    pub name: String,
    pub role: String,
    pub endpoint: String,
    pub reachable: bool,
    pub session_count: u32,
    pub last_seen_unix_ms: Option<u64>,
    /// Its LumaSession, location, priority, status (Standalone, Master,
    /// Member), version, whether it accepts invites, its IP and ports.
    pub lumasession: String,
    pub location: String,
    pub priority: String,
    pub status: String,
    pub version: String,
    pub invite: bool,
    pub ip: String,
    pub cmd_port: u16,
    /// Its LumaSession stream port (where members join it).
    pub stream_port: u16,
    pub station_id: String,
    /// Its show status (ShowFileStatus) and LumaSession epoch.
    pub show_status: String,
    pub epoch: u64,
}

/// A live connection owned by the network engine.
#[derive(Debug, Clone, Default)]
pub struct NetworkConnectionRuntime {
    pub id: String,
    pub transport: String,
    pub peer: String,
    pub session_id: Option<Uuid>,
    pub client_id: Option<Uuid>,
    pub state: String,
}

/// Simple health counters for the shared network engine.
#[derive(Debug, Clone, Default)]
pub struct NetworkHealthRuntime {
    pub state: String,
    pub last_event: Option<String>,
    pub dropped_connections: u64,
    pub failed_requests: u64,
}

/// Priority level for a station within a multi-station session.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StationPriority {
    Never,
    VeryLow,
    Low,
    #[default]
    Normal,
    High,
}

impl fmt::Display for StationPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StationPriority::Never => write!(f, "Never"),
            StationPriority::VeryLow => write!(f, "VeryLow"),
            StationPriority::Low => write!(f, "Low"),
            StationPriority::Normal => write!(f, "Normal"),
            StationPriority::High => write!(f, "High"),
        }
    }
}

impl FromStr for StationPriority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "never" => Ok(StationPriority::Never),
            "verylow" | "very_low" => Ok(StationPriority::VeryLow),
            "low" => Ok(StationPriority::Low),
            "normal" => Ok(StationPriority::Normal),
            "high" => Ok(StationPriority::High),
            _ => Err(format!(
                "Unknown priority '{s}'. Use: Never, VeryLow, Low, Normal, High"
            )),
        }
    }
}

/// A station's LumaSession state (decisions §21 "Step 6 hardening"; the
/// LumaSessions menu shows the same words).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StationStatus {
    /// Not in a LumaSession.
    #[default]
    Standalone,
    /// The LumaSession's master: runs every show change.
    Master,
    /// Joined to a master.
    Member,
    /// A member that lost its master and works alone (6c).
    IdleMaster,
}

impl fmt::Display for StationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A station participating in a session.
#[derive(Debug, Clone, Default)]
pub struct SessionStationRuntime {
    pub name: String,
    pub role: String,
    pub endpoint: String,
    pub connected: bool,
    /// Priority of this station for authority negotiation.
    pub priority: StationPriority,
    /// Current status of this station within the session.
    pub status: StationStatus,
    /// How long the station has been online (milliseconds).
    pub online_time_ms: u64,
    /// Optional slot identifier when assigned to a numbered slot.
    pub slot_id: Option<String>,
    /// Current authority state (e.g. "standby", "primary", "secondary").
    pub authority_state: String,
    /// Current sync state (e.g. "synced", "catching_up", "offline").
    pub sync_state: String,
}

/// A client attached to a session.
#[derive(Debug, Clone, Default)]
pub struct SessionClientRuntime {
    pub id: Uuid,
    pub kind: String,
    pub remote_addr: String,
    pub width: u16,
    pub height: u16,
    pub connected: bool,
}

/// A remote authority this runtime has explicitly joined.
#[derive(Debug, Clone, Default)]
pub struct JoinedSessionRuntime {
    pub target: String,
    pub user: String,
    pub authority: String,
    pub cmd_port: u16,
    pub shell_port: u16,
    pub connected: bool,
    pub last_error: Option<String>,
}

impl RuntimeState {
    pub fn new() -> Self {
        let mut network_engines = HashMap::new();
        network_engines.insert("SessionNetwork".to_string(), EngineRuntime::default());

        let mut session_engines = HashMap::new();
        session_engines.insert("SessionAuthority".to_string(), EngineRuntime::default());

        Self {
            users: HashMap::new(),
            joined_sessions: BTreeMap::new(),
            network_engines,
            session_engines,
            rendering_engines: HashMap::new(),
            network: NetworkRuntime::default(),
            luma_sessions: LumaSessionsRuntime::default(),
            global_variables: BTreeMap::new(),
            claims: BTreeMap::new(),
            claim_order: Vec::new(),
            claim_asks: Vec::new(),
            devices: BTreeMap::new(),
            live_screens: ScreenStore::shared(),
            journal: Journal::default(),
            health: std::sync::Arc::new(HealthBoard::default()),
        }
    }

    /// Get or create a user's runtime state.
    pub fn user(&mut self, username: &str) -> &mut RuntimeUser {
        self.users
            .entry(username.to_string())
            .or_insert_with(|| RuntimeUser {
                user: username.to_string(),
                ..Default::default()
            })
    }

    /// A user's runtime state (read-only), if they have any.
    pub fn user_ref(&self, username: &str) -> Option<&RuntimeUser> {
        self.users.get(username)
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

/// One claimed object (`Root.Runtime.Claims`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    /// The claim itself, while it stands: a fresh GUID.
    #[serde(default)]
    pub meta: lumacore_base::Identity,
    /// How it is named (`Group 3`, `Patch`).
    pub label: String,
    /// The user holding it.
    pub user: String,
    pub since_ms: u64,
    /// Taken by opening an editor (released when the editor closes).
    #[serde(default)]
    pub editor: bool,
}

/// A user asking for an object another user holds (§21 "6d").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimAsk {
    pub object: uuid::Uuid,
    pub label: String,
    /// Who asks, and who holds it.
    pub from: String,
    pub owner: String,
    /// When it is handed over unless the owner refuses (unix ms): only what
    /// the owner's countdown shows.
    pub deadline_ms: u64,
    /// The same on the session clock (`clock::session_ms`), which decides:
    /// continuous across a master handover and never moved by a wall-clock
    /// step.
    pub due_session_ms: u64,
}
