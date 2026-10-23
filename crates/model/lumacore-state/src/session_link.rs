//! What the core keeps of the LumaSession (decisions §23 "Step 8"): this
//! station's role, the link to its master as an **outbox** ([`LinkHandle`]:
//! frames for the link thread in `lumasession/`), the lines forwarded and
//! not answered yet, and the work commands ask for after the job
//! (`AppState::link_requests`, `session_requests`). The stream, election and
//! merge themselves are `lumasession/`; core never calls it.

use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use lumacore_tree::{LocalSnapshot, OopsKind, SubtreeSnapshot};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::DispatchResult;
use crate::defs::command::Undo;

/// What the core (and the link) hands the link thread to send to the
/// master; the link thread turns each into a stream frame.
#[derive(Debug, Clone)]
pub enum LinkOut {
    /// A line to run on the master as `user` (answered by its `id`).
    Command {
        id: u64,
        user: String,
        line: String,
        /// A gesture's line runs quietly on the master too (decisions §30 "12f").
        gesture: Option<crate::gesture::GestureLine>,
    },
    /// A member's own show change, filed on the master as `user`.
    Changes {
        user: String,
        line: String,
        undo: Option<OopsKind>,
        snapshots: Vec<SubtreeSnapshot>,
        /// Station-local state the step changed here (station settings,
        /// watches): filed in the step, put back only on this station.
        local: Vec<LocalSnapshot>,
    },
    /// A gap: ask for a snapshot (or what was missed since `resume`).
    Resync {
        resume: Option<(Uuid, u64)>,
    },
    /// The users logged in here.
    Logins {
        users: Vec<String>,
    },
    /// `Claim LumaSession`: ask the master to hand over.
    Claim,
    Pong {
        t: u64,
    },
    /// Leaving the session.
    Bye,
    /// Files of the show this station's stores lack (pictures, GDTF), by
    /// SHA-256: the master sends them (decisions §30 "12a2").
    FileWant {
        files: Vec<WantedFile>,
    },
    /// A file uploaded to this member for import (`POST /api/media`, or a
    /// fixture file, `POST /api/gdtf`, when `gdtf`): the master keeps it in
    /// its `library/media/` (`library/gdtf/`) before the forwarded `Import` /
    /// `Store` line arrives (same stream, in order). Imports always run on the
    /// master (decisions §30 "12a2", §34 round 3).
    Upload {
        name: String,
        data: String,
        gdtf: bool,
    },
}

/// A file a member asks its master for: SHA-256 and extension (`gdtf` for a
/// fixture type's file).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WantedFile {
    pub sha256: String,
    pub ext: String,
}

/// The clock liveness runs on (heartbeats, silence, round trips): this
/// station's monotonic clock in ms (`lumacore_base::clock::local_us`), never
/// the wall clock — an NTP step must neither fake a lost master nor hide a
/// dead one (R1-02).
pub fn link_ms() -> u64 {
    lumacore_base::clock::local_us() / 1000
}

/// Silence that warns ("no answer from …").
pub const WARN: Duration = Duration::from_millis(1500);
/// Silence after which the other side is lost (a 5 s session timeout).
const LOST: Duration = Duration::from_secs(5);

static TIMEOUT: OnceLock<Duration> = OnceLock::new();

/// `--session-timeout <s>` (debug builds only).
pub fn set_timeout(seconds: u64) {
    let _ = TIMEOUT.set(Duration::from_secs(seconds));
}

/// Silence after which the other side counts as lost.
pub fn lost_after() -> Duration {
    TIMEOUT.get().copied().unwrap_or(LOST)
}

/// This station's role beyond Member (a link) and Master (members).
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Role {
    #[default]
    Normal,
    /// Lost its master and nobody was reachable: its own master, output
    /// only with "output when alone". `diverged` once it changed the show.
    IdleMaster {
        diverged: bool,
        lost: String,
        quiet: bool,
    },
    /// A master that met a master with a higher epoch: no output until its
    /// operator decides.
    Fenced { by: String },
}

/// The station's `ShowFileStatus` (without the media states).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShowStatus {
    #[default]
    NoShow,
    Loaded,
    Downloading,
    Downloaded,
    Uploading,
    Saving,
    NegotiatingMaster,
    NegotiatingMember,
    WaitingForNewMaster,
}

impl std::fmt::Display for ShowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NoShow => "No show",
            Self::Loaded => "Loaded",
            Self::Downloading => "Downloading",
            Self::Downloaded => "Downloaded",
            Self::Uploading => "Uploading",
            Self::Saving => "Saving",
            Self::NegotiatingMaster => "Negotiating (master)",
            Self::NegotiatingMember => "Negotiating (member)",
            Self::WaitingForNewMaster => "Waiting for new master",
        })
    }
}

/// The show at a split (§21 "6d"): what this station had applied of the
/// master's history (`journal`, `seq`) when it lost or replaced it.
#[derive(Debug, Clone)]
pub struct SplitBase {
    pub show: std::sync::Arc<lumacore_tree::ShowData>,
    pub journal: Option<uuid::Uuid>,
    pub seq: u64,
}

/// Where a station's split base lives on disk (R1-22): the autosaved show
/// and where it split. Station-local, never replicated.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SplitBaseFile {
    path: String,
    journal: Option<uuid::Uuid>,
    seq: u64,
    show_id: uuid::Uuid,
}

fn split_base_file() -> String {
    format!("{}/split_base.json", lumacore_luma::station_config::dir())
}

/// Note the split base kept at `path` so a restart can still Merge (R1-22).
pub fn remember_split_base(path: &str, base: &SplitBase) {
    // Unit tests leave no files behind (like `autosave_show`).
    if cfg!(test) {
        return;
    }
    let file = SplitBaseFile {
        path: path.to_string(),
        journal: base.journal,
        seq: base.seq,
        show_id: base.show.show_settings.show_id,
    };
    if let Ok(json) = serde_json::to_string_pretty(&file) {
        let _ = lumacore_luma::drive::write_atomic(split_base_file(), json);
    }
}

/// The split is resolved (merged, or another show): the note goes.
pub fn forget_split_base() {
    if cfg!(test) {
        return;
    }
    let _ = lumacore_luma::drive::remove_file(split_base_file());
}

/// At start: the split base noted before a restart, when it belongs to the
/// show that is loaded now (same show id).
pub fn restore_split_base(state: &mut AppState) {
    let Ok(text) = lumacore_luma::drive::read_to_string(split_base_file()) else {
        return;
    };
    let Ok(file) = serde_json::from_str::<SplitBaseFile>(&text) else {
        return;
    };
    if file.show_id != state.root.show_data.show_settings.show_id {
        return;
    }
    match lumacore_luma::show_file::read(std::path::Path::new(&file.path)) {
        Ok(show) => {
            state.split_base = Some(SplitBase { show: std::sync::Arc::new(show), journal: file.journal, seq: file.seq });
            lumacore_sysmon::sysmon_note("LumaSession", format!("split base restored from {} (Merge stays possible)", file.path));
        }
        Err(e) => lumacore_sysmon::sysmon_note("LumaSession", format!("[WARN] the split base {} couldn't be read ({e})", file.path)),
    }
}

/// Session work a command asked for, done after the job.
#[derive(Debug, Clone)]
pub enum SessionRequest {
    /// `Claim LumaSession` on a member: ask the master to hand over.
    HandOver,
    /// `Claim LumaSession` on a fenced master / IdleMaster: Use Mine.
    UseMine,
    /// `Claim LumaSession Station "X"` on the master: hand the master role
    /// to member X (decisions §30 "12d part 4").
    HandTo(String),
    /// `Invite Station` / `Dismiss Station`: send `request` to a station's
    /// command port (after the job, off the state thread); its answer goes
    /// to `reply_to`.
    Send {
        station: String,
        ip: String,
        port: u16,
        request: lumacore_wire::RemoteRequest,
        reply_to: ReplyTo,
    },
    /// `Set Station "Tower" "Priority" High`: another station's own setting,
    /// sent to its command port and waited for (a timeout and retries); the
    /// answer goes to `reply_to` and settles edit `id`
    /// (`stations::remote_answered`).
    EditStation {
        id: u64,
        station: String,
        ip: String,
        port: u16,
        request: lumacore_wire::RemoteRequest,
        reply_to: ReplyTo,
    },
}

/// The known stations with their state, for `List Station`.
pub fn known_lines(state: &AppState) -> Vec<String> {
    let members = &state.root.runtime.luma_sessions.members;
    state
        .root
        .show_data
        .session
        .stations
        .values()
        .map(|s| {
            let what = match members.get(s.meta.name().unwrap_or_default()) {
                Some(m) if m.connected => "In Session",
                Some(_) => "Lost",
                None if s.meta.name() == Some(state.root.station_settings.station_name.as_str()) => "This station",
                None => "Known",
            };
            format!("  {:<16} {:<12} {:<16} {}", s.meta.name().unwrap_or_default(), what, s.ip, s.priority)
        })
        .collect()
}

/// LumaSession health for the status bar: role, master, health (ok / late
/// / lost), latency, pending commands, and a decision to make.
pub fn status_json(state: &AppState) -> lumacore_wire::LumaSessionStatus {
    let status = status_word(state);
    let (health, latency_ms, master) = match &state.link {
        Some(link) => {
            let silent = link.stats.silent_for();
            let health = if silent >= lost_after() {
                "lost"
            } else if silent >= WARN {
                "late"
            } else {
                "ok"
            };
            (
                health,
                Some(link.stats.rtt().as_millis() as u64),
                Some(link.master.clone()),
            )
        }
        None => ("ok", None, None),
    };
    lumacore_wire::LumaSessionStatus {
        status: status.to_string(),
        health: health.to_string(),
        latency_ms,
        master,
        pending: state.forwards.len(),
        show_status: state.show_status.to_string(),
        decision: state.decision.clone(),
    }
}

/// The status a station shows in discovery and menus.
pub fn status_word(state: &AppState) -> &'static str {
    match state.session_role {
        Role::IdleMaster { .. } | Role::Fenced { .. } => "IdleMaster",
        Role::Normal if state.link.is_some() => "Member",
        Role::Normal if !state.root.runtime.luma_sessions.members.is_empty() => "Master",
        Role::Normal => "Standalone",
    }
}

/// Heartbeat and health of one stream connection (§21 "6c").
#[derive(Debug, Default)]
pub struct LinkStats {
    /// When the last frame (or piece of one) arrived ([`link_ms`]).
    pub last_rx_ms: std::sync::atomic::AtomicU64,
    /// Round trip of the last heartbeat (µs).
    pub rtt_us: std::sync::atomic::AtomicU64,
    /// Commands sent again after a reconnect.
    pub resends: std::sync::atomic::AtomicU64,
}

impl LinkStats {
    pub fn heard(&self) {
        self.heard_at(link_ms());
    }

    fn heard_at(&self, now_ms: u64) {
        self.last_rx_ms.store(now_ms, Ordering::Relaxed);
    }

    /// How long nothing arrived.
    pub fn silent_for(&self) -> Duration {
        self.silent_at(link_ms())
    }

    fn silent_at(&self, now_ms: u64) -> Duration {
        let last = self.last_rx_ms.load(Ordering::Relaxed);
        Duration::from_millis(now_ms.saturating_sub(last))
    }

    pub fn rtt(&self) -> Duration {
        Duration::from_micros(self.rtt_us.load(Ordering::Relaxed))
    }
}

/// A member's way to its master (`AppState::link`).
#[derive(Clone)]
pub struct LinkHandle {
    /// Frames for the master, in order.
    pub tx: Sender<LinkOut>,
    /// Set by Leave: the link thread ends.
    pub stop: Arc<AtomicBool>,
    /// The connected socket, so Leave can close it.
    pub socket: Arc<Mutex<Option<TcpStream>>>,
    /// `host:port` of the master's stream.
    pub target: String,
    /// The master's station name (from its Welcome).
    pub master: String,
    /// A gap was found and a snapshot asked for: ops wait for it.
    pub resyncing: Arc<AtomicBool>,
    /// Heartbeat and health.
    pub stats: Arc<LinkStats>,
    /// The master's station priority (a joiner that outranks it may Claim).
    pub master_priority: Option<lumacore_tree::StationPriority>,
    /// The master's command port (session clock pings; 0 = unknown).
    pub master_cmd_port: u16,
    /// This join's id (`Hello.link`): fresh per join, the same across
    /// reconnects and a new master, so (station id, link, command id) names
    /// a forwarded command everywhere (R1-20, R1-21).
    pub link: Uuid,
    next_id: Arc<std::sync::atomic::AtomicU64>,
}

impl std::fmt::Debug for LinkHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LinkHandle({} → {})", self.target, self.master)
    }
}

impl LinkHandle {
    pub fn new(target: String) -> (Self, Receiver<LinkOut>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (
            Self {
                tx,
                stop: Arc::default(),
                socket: Arc::default(),
                target,
                master: String::new(),
                resyncing: Arc::default(),
                stats: Arc::default(),
                master_priority: None,
                master_cmd_port: 0,
                link: Uuid::new_v4(),
                next_id: Arc::default(),
            },
            rx,
        )
    }

    /// A handle for tests (no thread behind it) — the station's tests use it too.
    #[doc(hidden)]
    pub fn test_handle(target: &str) -> (Self, Receiver<LinkOut>) {
        Self::new(target.to_string())
    }

    /// End the link: say Bye, stop the thread and close the socket.
    pub fn close(&self) {
        let _ = self.tx.send(LinkOut::Bye);
        self.stop.store(true, Ordering::Relaxed);
        // Give the writer a moment to send the Bye before the socket goes.
        std::thread::sleep(Duration::from_millis(50));
        if let Ok(mut socket) = self.socket.lock()
            && let Some(socket) = socket.take()
        {
            let _ = socket.shutdown(std::net::Shutdown::Both);
        }
    }
}

/// Where the answer to a forwarded line goes.
#[derive(Debug, Clone, Default)]
pub enum ReplyTo {
    /// Nobody waits (OSC, keys without a shell): the System Monitor.
    #[default]
    Log,
    /// The output of this user's Shell.
    Shell(String),
    /// A UDP / HTTP request waiting for it.
    Waiter(Sender<String>),
}

/// A command sent to the master and not answered yet.
#[derive(Debug, Clone)]
pub struct Forward {
    pub user: String,
    /// The line as shown (passwords masked).
    pub line: String,
    /// The line as typed (run here if this station becomes master).
    pub raw: String,
    pub reply_to: ReplyTo,
    pub sent: Instant,
    /// Its client was told it is pending.
    pub noted: bool,
    /// The master ran it (its result arrived: `commands_done` in the session
    /// runtime); only the answer may still be missing. Never run here.
    pub ran: bool,
}

/// What a client hears when its command ran on the master but the answer
/// was lost on the way (a reconnect, a new master).
pub const ANSWER_LOST: &str = "Ran on the master (its answer was lost).";

/// The master's results say which of this member's forwarded commands ran
/// (`Runtime.LumaSessions.commands_done`): mark them, so a new master or an
/// IdleMaster never runs them again (R1-21). Their answers may still come.
pub fn mark_ran(state: &mut AppState) {
    let Some(link) = state.link.as_ref().map(|l| l.link) else {
        return;
    };
    let Some(done) = state
        .root
        .runtime
        .luma_sessions
        .commands_done
        .get(&state.local_settings.station_id)
        .copied()
    else {
        return;
    };
    for (id, forward) in state.forwards.iter_mut() {
        if done.ran(link, *id) {
            forward.ran = true;
        }
    }
}

/// A join a command asked for (started after the job: no I/O in a job).
#[derive(Debug, Clone)]
pub struct LinkRequest {
    pub host: String,
    pub port: u16,
    /// The user who joins (notes, journal).
    pub user: String,
    /// Who hears how the join went.
    pub reply_to: ReplyTo,
    /// The station that invited this one, if any.
    pub invited_by: Option<String>,
    /// Keep trying for the loss timeout (the master may still be taking
    /// over: a handover, a rejoin); else the first failure ends the join.
    pub patient: bool,
    /// Join with Merge (§21 "6d"): the master merges this show into its own.
    pub merge: bool,
}

// ── Member: sending ─────────────────────────────────────────────────────

/// Send a line to the master as the acting user and keep it pending until
/// the master answers. Nothing runs here.
pub fn forward(state: &mut AppState, line: &str) -> DispatchResult {
    let Some(link) = &state.link else {
        return DispatchResult::Output((String::new()).into());
    };
    let id = link
        .next_id
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        + 1;
    let user = state.effective_user_name().to_string();
    let sent = link.tx.send(LinkOut::Command {
        id,
        user: user.clone(),
        line: line.to_string(),
        gesture: state.gesture.clone(),
    });
    if sent.is_err() {
        return DispatchResult::Output(crate::Reply::error(
            "The link to the master is closed: Leave LumaSession and join again.",
            "E-LUMA-004",
        ));
    }
    let mut reply_to = state.reply_to.clone();
    if let Some(waits) = &mut state.forward_waits {
        let (tx, rx) = crossbeam_channel::bounded(1);
        waits.push(rx);
        reply_to = ReplyTo::Waiter(tx);
    }
    state.forwards.insert(
        id,
        Forward {
            user,
            line: crate::passwords::mask_secrets(line),
            raw: line.to_string(),
            reply_to,
            sent: Instant::now(),
            noted: false,
            ran: false,
        },
    );
    DispatchResult::Output((String::new()).into())
}

/// A member's own show change (a station-local action wrote to the show):
/// the master files it as the acting user; this station keeps it meanwhile.
pub fn submit_changes(
    state: &mut AppState,
    line: &str,
    undo: Undo,
    changes: &[SubtreeSnapshot],
    local: Vec<LocalSnapshot>,
) {
    let Some(link) = &state.link else {
        return;
    };
    let _ = link.tx.send(LinkOut::Changes {
        user: state.effective_user_name().to_string(),
        line: line.to_string(),
        undo: undo.kind(),
        snapshots: changes.to_vec(),
        local,
    });
}

/// Leave the master: Bye, close, drop what was pending.
pub fn leave(state: &mut AppState) -> Option<String> {
    let link = state.link.take()?;
    link.close();
    let pending = state.forwards.len();
    state.forwards.clear();
    state.last_applied = None;
    state.root.runtime.joined_sessions.clear();
    state.root.runtime.luma_sessions.members.clear();
    state.root.runtime.luma_sessions.commands_done.clear();
    let mut out = format!("Left the LumaSession of {}.", link.master);
    if pending > 0 {
        out.push_str(&format!(
            " {pending} pending command(s) were dropped (the master never confirmed them)."
        ));
    }
    Some(out)
}

/// Keep unsaved changes before another show replaces this one: a copy in
/// `library/shows/.autosave/` named for why (`invite`, `use-master`,
/// `use-mine`, `before-merge`, `signal`; decisions §21 "6d").
/// `Ok(Some(path))` = kept there, `Ok(None)` = nothing unsaved, `Err` = the
/// copy couldn't be written — then the caller must not replace the show
/// (R1-16).
pub fn autosave_copy(state: &AppState, reason: &str) -> Result<Option<String>, String> {
    if !state.show_dirty {
        return Ok(None);
    }
    autosave_show(state, &state.root.show_data, reason).map(Some)
}

/// Write `show` as a recovery copy (`.autosave/{show}_{reason}_{station}_{stamp}.luma`,
/// `lumacore_luma::backups`), whether or not it changed: a split base is kept
/// as it was. Returns the path, or why it couldn't be written (R1-16); in
/// unit tests without a test library nothing is written (an empty path).
pub fn autosave_show(
    state: &AppState,
    show: &lumacore_tree::ShowData,
    reason: &str,
) -> Result<String, String> {
    let path = crate::recovery::try_recovery_copy(state, show, reason)?.unwrap_or_default();
    if !path.is_empty() {
        lumacore_sysmon::sysmon_note("LumaSession", format!("Show kept ({reason}): {path}"));
    }
    Ok(path)
}

/// The refusal when unsaved changes couldn't be kept: nothing is replaced.
pub fn not_kept(why: &str) -> String {
    crate::errors::render_code(
        format!("Cannot keep your unsaved changes ({why}); nothing was replaced."),
        "E-CMD-012",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liveness_counts_on_the_monotonic_clock() {
        let stats = LinkStats::default();
        stats.heard_at(10_000);
        assert_eq!(stats.silent_at(16_000), Duration::from_secs(6));
        // A clock behind the last frame never counts as silence…
        assert_eq!(stats.silent_at(9_000), Duration::ZERO);
        // …and the clock used is the station's monotonic one.
        stats.heard();
        let heard = stats.last_rx_ms.load(Ordering::Relaxed);
        let mono = lumacore_base::clock::local_us() / 1000;
        assert!(heard <= mono && mono - heard < 1000);
        assert!(stats.silent_for() < WARN);
    }
}
