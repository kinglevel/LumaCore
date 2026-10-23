//! The status a station sends each shell client for its command line bar
//! (the status icons): what differs per user / client. A field change bumps
//! [`crate::CLIENT_PROTOCOL`].

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub user: String,
    /// Shortcut mode: `KBD`, `PRG`, `NAV`.
    pub mode: String,
    /// Walking a selection: [position, total].
    pub walk: Option<[usize; 2]>,
    /// This user's mode objects that are on (decisions §16).
    pub modes: Vec<String>,
    /// Fixtures in the user's selection.
    pub selected: usize,
    pub claims: Vec<ClaimStatus>,
    pub show: ShowStatus,
    /// LumaSession health and a decision to make (§21 "6c").
    pub lumasession: LumaSessionStatus,
    /// The client's device: locked, normal theme (decisions §30 "12a3").
    #[serde(default)]
    pub device: DeviceStatus,
    /// The station's open dialogs for this client (decisions §30 "12b"):
    /// confirms of its lines, claim asks to its user, the merge decision.
    #[serde(default)]
    pub dialogs: Vec<crate::dialog::Dialog>,
    /// Dialogs this client had that were just answered (shown a moment).
    #[serde(default)]
    pub dialogs_closed: Vec<crate::dialog::DialogClosed>,
    /// The user's profile setting for the command line's suggestion list
    /// (decisions §30 "12b2"): `Auto` (opens after a short pause while
    /// typing), `TabOnly` (opens on Tab) or `Off`.
    #[serde(default)]
    pub suggestions: String,
    /// The user's profile setting `HoldFollows` (decisions §30 "12c2"): a
    /// pane held full screen on this device maximises it on the shared live
    /// screen too (`Set Window <s>.<n> Maximized On`); false = only here.
    #[serde(default)]
    pub hold_follows: bool,
    /// New attributes waiting in the user's profile for their answer
    /// (decisions §30 "12f-a": "n waiting" in the sidebar; tap = the board).
    #[serde(default)]
    pub attributes_waiting: u32,
    /// The top bar's facts (decisions top-bar-status-strip).
    #[serde(default)]
    pub strip: StripStatus,
}

/// What the top bar's status strip shows besides the show and the modes (decisions
/// top-bar-status-strip): the station beside the mark, the user's profile, DataPool and selected
/// sequence, and how many steps Oops can undo.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StripStatus {
    /// This station's name (`--station`).
    pub station: String,
    /// The user's profile.
    pub profile: String,
    /// The user's DataPool as shown: `1` or `1 Main`.
    pub datapool: String,
    /// The user's selected sequence as shown (`1 Main`); empty: none.
    pub sequence: String,
    /// Steps the user's Oops can undo.
    pub oops: u32,
}

/// A device's state on this station (every window of an app, or a browser
/// tab). Station-local, never replicated.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub id: String,
    /// DeskLock is on: the device shows the lock surface.
    pub locked: bool,
    /// The normal theme (the factory preset) instead of the profile's look:
    /// the station's `ThemeOverride` preset when set, else Dark.
    pub normal_theme: bool,
    /// `Set Station ThemeOverride <preset>`: every device of this station
    /// starts in the normal theme with that preset (None = off).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme_override: Option<String>,
    /// The screen config this device shows (`Set Device ScreenConfig <n>`,
    /// decisions §30 "12c2"); None = Follow user (the user's own choice).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen_config: Option<u32>,
}

/// An object someone holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimStatus {
    pub object: String,
    pub mine: bool,
    pub owner: String,
    /// Held by an open editor (released when it closes).
    pub editor: bool,
}

/// The loaded show (`Info Show`, the status bar's show menu).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ShowStatus {
    pub name: Option<String>,
    /// Unsaved changes.
    pub dirty: bool,
    pub description: String,
    pub autosave_minutes: u32,
    /// Seconds until the next autosave.
    pub autosave_in: Option<u64>,
    /// What the station loads at start.
    pub startup: StartupShow,
    pub format_version: u32,
    pub saved_with_commit: String,
    pub show_id: String,
    /// Merge on rejoin: the default answer and its timeout (seconds).
    pub merge_default: String,
    pub merge_timeout: Option<u32>,
    /// The reference check's summary.
    pub references: String,
}

/// What a station loads when it starts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "mode", content = "name")]
pub enum StartupShow {
    /// The last show loaded or saved (default) — also after a restart or a
    /// rebuild.
    #[default]
    Last,
    /// Start with an empty show.
    None,
    /// Always this show.
    Named(String),
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LumaSessionStatus {
    /// Standalone, Master, Member, IdleMaster.
    pub status: String,
    /// The link to the master: `ok`, `late`, `lost`.
    pub health: String,
    pub latency_ms: Option<u64>,
    pub master: Option<String>,
    /// Lines forwarded to the master and not answered yet.
    pub pending: usize,
    /// Its show status (e.g. "Loaded").
    pub show_status: String,
    /// A decision this station's operator has to make.
    pub decision: Option<Decision>,
}

/// A question for the operator when two masters meet or a diverged station
/// rejoins (the session data merge pop-up, §21 "6c" / "6d").
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub id: u64,
    pub title: String,
    pub message: String,
    /// (label, command), the negotiation words. Merge's command is empty
    /// while it is blocked (`merge_blocked`).
    pub options: Vec<(String, String)>,
    /// What differs between the two shows (the merge preview), one line
    /// each; empty until the session's show has been read.
    pub details: Vec<String>,
    /// Why Merge isn't possible (greyed out), if so.
    pub merge_blocked: Option<String>,
    /// The answer taken when nobody answers (ShowSettings MergeDefault) and
    /// when (unix ms; None = waits for the operator).
    pub default: String,
    pub deadline_ms: Option<u64>,
    /// `Pause LumaSession` (clicking the pop-up) stopped the countdown.
    pub paused: bool,
}

impl Decision {
    /// The option that runs when the countdown ends: the default, or
    /// Cancel when the default (Merge) is blocked.
    pub fn default_command(&self) -> Option<&str> {
        let pick = |label: &str| {
            self.options
                .iter()
                .find(|(l, c)| l == label && !c.is_empty())
                .map(|(_, c)| c.as_str())
        };
        pick(&self.default).or_else(|| pick("Cancel"))
    }
}
