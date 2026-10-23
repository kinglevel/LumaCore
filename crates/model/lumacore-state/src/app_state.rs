//! Core application state: `AppState` struct and show-data operations.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use lumacore_render::{EngineInput, UserLayer};
use lumacore_tree::{PendingReplicationOp, Root, ShowData, ShowSubtree, Verbosity};

use super::{FixtureTypeSource, INTERNAL_ROOT_USER, ReplContext};

/// Shortcut input mode — determines how keypresses are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutMode {
    /// Normal typing with full ghosting/autocompletion, no single-key interception.
    Keyboard,
    /// Console-style single-key shortcuts for programmer operations.
    Programmer,
    /// Tree browser mode: arrow-key navigation, view/copy/move objects.
    Navigator,
}

impl ShortcutMode {
    /// Cycle to the next mode: KBD → PRG → NAV → KBD.
    pub fn next(self) -> Self {
        match self {
            Self::Keyboard => Self::Programmer,
            Self::Programmer => Self::Navigator,
            Self::Navigator => Self::Keyboard,
        }
    }

    /// Short label for status bar display.
    pub fn label(self) -> &'static str {
        match self {
            Self::Keyboard => "KBD",
            Self::Programmer => "PRG",
            Self::Navigator => "NAV",
        }
    }
}

impl std::fmt::Display for ShortcutMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// How long a command waits for the engine to render what was published
/// (one frame is 16.7 ms; a stopped engine is not waited for longer).
const OUTPUT_WAIT: std::time::Duration = std::time::Duration::from_millis(50);

/// Who a command line belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineOwner {
    /// A window: live screen `screen`, window `no`.
    Window { screen: u32, no: u32 },
    /// A UiKey pressed on the keyboard (`Self` = the focused window).
    Key(String),
    /// A macro's line (its runtime isn't built: E-CMD-016).
    Macro(u32),
    /// A plugin's line (its runtime isn't built: E-CMD-016).
    Plugin(String),
}

/// Shared application state passed to every command handler.
pub struct AppState {
    /// The root object tree (show data + station settings).
    pub root: Root,
    /// Current sticky prompt context (console-style control scoping: Fixture, Universe, Group, etc.).
    ///
    /// This is the *control* context — it determines what bare commands like `At 50` apply to.
    /// It is separate from the navigation path and is NOT affected by `cd` or `clear`.
    pub context: ReplContext,
    /// Current tree navigation path (Linux filesystem-style, separate from control context).
    ///
    /// Each element is a display-friendly node name like "ShowData", "Devices", "DataPools".
    /// `cd ..` pops the last element. `cd showdata` pushes "ShowData". `cd /` clears all.
    /// This is displayed in the path bar and changed by `cd` only — never by `clear`.
    pub navigation_path: Vec<String>,
    /// The screen (screen) the executing shell session is attached to,
    /// if any. View commands without `/Screen` act on it (the screen with
    /// focus). Swapped per remote session like `navigation_path`.
    pub current_live_screen: Option<u32>,
    /// The executing client is a UI (web / app window: it registered terminal
    /// viewports), even before it has joined a screen. UI events (Logout,
    /// password prompt, confirmations) go to it instead of text answers.
    pub ui_client: bool,
    /// The current command comes from another machine (a Shell client whose
    /// real address isn't loopback): `Login` checks Remote Login.
    pub remote_client: bool,
    /// Set by `Clear Programmer User …` for its line: each cleared
    /// programmer's undo step goes to that user (decisions §10).
    pub undo_to_owners: bool,
    /// `/NoOops` on the current line: it runs, replicates and is journaled,
    /// but files no Oops step (decisions §29 "11c").
    pub no_oops: bool,
    /// Station-local state the current step changed (station settings,
    /// watches), filed with it (`dispatch::commit`).
    pub pending_local: Vec<lumacore_tree::LocalSnapshot>,
    /// The focused window in `current_live_screen`; `Store Window` without a
    /// placement adds beside it.
    pub current_window: Option<u32>,
    /// Who the running line belongs to when its transport says so (a window
    /// bar line names its window, decisions §30 "12c"); None = the focused
    /// window. Set around one line by the shell, never kept.
    pub line_owner: Option<LineOwner>,
    /// The running line with `Self` resolved (`Window 1.4`), when it had
    /// one: the System Monitor's `Cmd` line shows it.
    pub resolved_line: Option<String>,
    /// UI events queued by commands for the executing session's client
    /// (drained by the remote shell session after each command).
    pub ui_requests: Vec<lumacore_wire::UiEvent>,
    /// The device the executing client belongs to (decisions §30 "12a3");
    /// swapped per client like `current_live_screen`.
    pub current_device: Option<String>,
    /// The shell client executing (None = the station's console, a one-shot
    /// path); swapped per client like `current_device`.
    pub current_client: Option<uuid::Uuid>,
    /// The station's open dialogs (decisions §30 "12b"; station-local).
    pub dialogs: crate::dialogs::Board,
    /// The logins waiting for the Meow License (decisions 0290; station-local).
    pub consent: crate::consent::Consents,
    /// Claim asks answered with "Remember for this session" (decisions §30
    /// "12b"): (owner, asker) → hand over (true) or keep. Kept by the
    /// master, which decides claims; dropped when the owner logs out.
    pub claim_remember: std::collections::BTreeMap<(String, String), bool>,
    /// A question a handler that returns text asks (`dispatch` turns the
    /// line's result into `DispatchResult::Ask`).
    pub asking: Option<crate::dialogs::Ask>,
    /// Every device's lock and normal theme (station-local, never replicated).
    pub devices: crate::device_table::Devices,
    /// Who switched `On DeskLock Station` on (R1-42): while set, a device
    /// that connects starts locked. Cleared when no device is locked any more.
    pub station_lock: Option<String>,
    /// The startup show this start skipped because the last start died
    /// while loading it (the `.starting` marker, E-SHOW-006): the recovery
    /// ask offers its newer copy (decisions §30 "Merge of bugfixes").
    pub skipped_startup_show: Option<String>,
    /// The name each AssetRoot / Network had when this station last looked
    /// (by GUID): a rename moves this station's own mapping (R1-69, Q41).
    pub seen_names: std::collections::HashMap<uuid::Uuid, String>,
    /// The DataPool an owner-first line names (`<verb> DataPool 2 Group 3`,
    /// R1-55) while it runs; None = the user's selected one.
    pub datapool_override: Option<u32>,
    /// A stored line (a UI key's) is running: Delete doesn't ask.
    pub running_stored_line: bool,
    /// The running line came through the gesture channel (decisions §30 "12f"): quiet.
    pub gesture: Option<crate::gesture::GestureLine>,
    /// Where each gesture under way began: (user, gesture id) → every subtree its moves changed,
    /// as it was before the first move (station-local; `gesture::remember` / `before_line`).
    pub gesture_starts: std::collections::HashMap<(String, String), Vec<lumacore_tree::undo::SubtreeSnapshot>>,
    /// Name of the loaded show file (`library/shows/<name>.luma`); None =
    /// untitled.
    pub current_show: Option<String>,
    /// The show changed since it was last loaded or saved (set when an undo
    /// frame is pushed).
    /// What upgrading the last loaded show did (an older format, decisions 0289); the load's
    /// reply takes it.
    pub upgraded: Option<lumacore_luma::show_file::Upgraded>,
    pub show_dirty: bool,
    /// This station's drives (decisions §38; `drives::reconcile`).
    pub drives: Vec<crate::drives::Drive>,
    /// When the show was last saved or loaded (autosave counts from here).
    pub last_save_at: std::time::Instant,
    /// The loaded show file's facts: its history rows and GUID index (the
    /// next save's Changes), when and by whom it was saved (decisions §30
    /// "12d part 5"; station-local, never replicated).
    pub saved: lumacore_luma::backups::Saved,
    /// `Login` in a remote shell: the client moves to this user's session.
    pub pending_user_switch: Option<String>,
    /// `Restart` was issued: the server re-executes itself after shutting down.
    pub restart_requested: bool,
    /// Station local settings (startup show, last show).
    pub local_settings: lumacore_luma::station_config::LocalSettings,
    /// The Dmx protocols whose drivers run, with their active lines (`Sacn` → 2; set by the
    /// line runtimes; decisions §34 "34f7b hardening").
    pub protocols_running: std::collections::BTreeMap<String, u32>,
    /// The render engine's hub (set by the server): its latest frame is
    /// the real output. None in tests and tools — the plan is then compiled
    /// and rendered here, once, with the same code.
    pub playback: Option<Arc<lumacore_render::RenderHub>>,
    /// Source metadata for imported fixture types (keyed by the numeric ID used in show_data).
    pub gdtf_sources: HashMap<u32, FixtureTypeSource>,
    /// The live patch while the Devices claim's draft is edited (§11).
    pub patch_draft: Option<crate::patch_draft::PatchDraft>,
    /// What each user identifies, by login (station output only, §34 round 3).
    pub identify: std::collections::BTreeMap<String, crate::identifying::Identifying>,
    /// Current user name (for claim ownership).
    pub user: String,
    /// Temporary acting user used when the hidden root executes a command on behalf of a show user.
    pub acting_user: Option<String>,
    /// Whether debug mode is enabled (unlocks dangerous commands).
    pub debug_mode: bool,
    /// Direct DMX overrides: (universe_id, address) -> value.
    /// Set by the `dmx` command. Cleared by `clear`. Not output yet (the DMX
    /// tester comes later).

    /// Per-command verbosity override (transient, reset after each command).
    ///
    /// Set by `/v` (Verbose) or `/d` (Debug) flags in the command line.
    /// Falls back to the user profile's `verbosity` setting when `None`.
    pub current_verbosity: Option<Verbosity>,
    /// Current shortcut mode: Keyboard (normal typing), Programmer (single-key shortcuts),
    /// or Navigator (tree browser). Cycled by Shift-F10 or the `shortcuts` command.
    pub shortcut_mode: ShortcutMode,
    /// Pending sysmon messages queued by the command dispatch layer.
    /// Drained by the server after each key/command handling cycle.
    pub sysmon_pending: Vec<(String, String)>,
    /// Tracks which user last mutated each subtree: (username, sequence_id).
    pub subtree_owners: HashMap<ShowSubtree, (String, u64)>,
    /// Monotonic undo sequence counter.
    pub undo_sequence: u64,
    /// The change path's images of the show (`terminal::change`).
    pub changes: super::change::ChangeTracker,
    /// Bumped whenever the show changed (a command, a load, an undo); the
    /// server republishes its show snapshot when it moves.
    pub show_generation: u64,
    /// Bumped whenever the patch may have changed (the change path saw
    /// `Devices` move, a load, an undo or replicated restore of it): the
    /// plan compiler rebuilds its channel tier only then (§24).
    pub patch_generation: u64,
    /// Counts what the state thread published to the engine; frames carry
    /// it, so a command can wait for its output (§24).
    pub engine_generation: u64,
    /// Nesting depth of `dispatch` (macros, confirmations): only the outermost
    /// call tracks changes and guards panics.
    pub dispatch_depth: u32,

    // ── Replication state ────────────────────────────────────────────────────
    /// Outbox of pending replication operations awaiting authority sequencing.
    pub replication_outbox: Vec<PendingReplicationOp>,
    /// True while applying inbound replicated ops from remote stations.
    /// Suppresses local undo frame creation and outbox emission.
    pub replaying_remote_op: bool,
    /// Last op applied from the master: its journal and sequence (member).
    pub last_applied: Option<(uuid::Uuid, u64)>,
    /// Files already asked of the master (`<sha256>.<ext>`), so a member asks
    /// once (decisions §30 "12a2").
    pub files_wanted: std::collections::BTreeSet<String>,
    /// The station stream to this station's master (member only, §21 "6b").
    pub link: Option<crate::session_link::LinkHandle>,
    /// Commands sent to the master and not answered yet: the pending outbox.
    pub forwards: std::collections::BTreeMap<u64, crate::session_link::Forward>,
    /// Where the reply to a forwarded command goes; the transport running a
    /// line sets it (a Shell, a waiting UDP/HTTP request, else the log).
    pub reply_to: crate::session_link::ReplyTo,
    /// Replies the running job waits for (UDP / HTTP command requests).
    pub forward_waits: Option<Vec<crossbeam_channel::Receiver<String>>>,
    /// Joins a command asked for, started after the job (no I/O in a job).
    pub link_requests: Vec<crate::session_link::LinkRequest>,
    /// This master has members: commands record their undo steps for them.
    pub has_members: bool,
    /// What commands did to undo stacks since the last flush (master).
    pub undo_log: Vec<lumacore_tree::UserUndoStep>,
    /// The show was replaced (load, new show): members take a full snapshot.
    pub show_replaced: bool,
    /// Counts whole-show replacements (never reset): dispatch tells a line
    /// that brought a new show (and its own Default) from one that edited.
    pub show_loads: u64,
    /// The attribute sync (not the line) wrote to the locked Default
    /// profile in this pass: dispatch lets it through (decisions §30 "12f-a").
    pub synced_default: bool,
    /// The station whose client sent the command running now (master).
    pub command_station: Option<String>,
    /// Edits of another station's own settings sent from here and not yet
    /// reported back (decisions §30 "Remote station edits"): the Stations
    /// grid's pending / refused cells. Station-local, never saved.
    pub remote_edits: Vec<crate::stations::RemoteEdit>,
    /// While another station's edit of this station runs here: who sent it
    /// (`Anna@FOH`) — the log line and the reply's reverse line.
    pub remote_from: Option<String>,
    /// IdleMaster or fenced, beyond Member / Master (§21 "6c").
    pub session_role: crate::session_link::Role,
    /// The ShowFileStatus of this station.
    pub show_status: crate::session_link::ShowStatus,
    /// The show as it was when this station split from its master (an
    /// IdleMaster) or took over from it (an elected master): the base of a
    /// merge on rejoin (§21 "6d").
    pub split_base: Option<crate::session_link::SplitBase>,
    /// Notices for users about their claims (takeovers), sent to their
    /// sessions after the job: (user, text).
    pub claim_notices: Vec<(String, String)>,
    /// The last reference check: when it ran and what it found (§21 "6d").
    pub references: Option<(String, lumacore_base::IdentityReport)>,
    /// When this station's LumaSession went online (unix ms): online time
    /// in the election.
    pub online_since_ms: u64,
    /// The session epoch this station follows (a master's is its journal's).
    pub session_epoch: u64,
    /// A decision its operator has to make (two masters, a diverged rejoin).
    pub decision: Option<lumacore_wire::Decision>,
    /// Session work commands asked for (`Claim LumaSession`), after the job.
    pub session_requests: Vec<crate::session_link::SessionRequest>,
    /// What this station sends, in words (output distribution log).
    pub output_plan: String,
    /// Whether this station is the session authority.
    pub is_authority: bool,
    /// This server's LumaSession ports (command, shell, stream), for invites.
    pub lumasession_ports: (u16, u16, u16),
    /// Station output settings changed (`Set Network … "Interface"`): the
    /// state thread reconciles protocol runtimes now.
    pub protocols_dirty: bool,
    /// Profiles the state put back (Oops, Redo, a replicated step): the layer above refreshes
    /// their view bars (`commands::views::follow_touched_profiles`, decisions §34 15).
    pub profiles_touched: Vec<uuid::Uuid>,
}

/// What happened to an undo stack ([`AppState::note_undo_step`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoKind {
    /// A frame was filed.
    Push,
    /// Oops took the newest frame.
    Undo,
    /// Redo put one back.
    Redo,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// A UI client runs the current command (see `ui_client`).
    pub fn has_ui(&self) -> bool {
        self.ui_client || self.current_live_screen.is_some()
    }

    /// Create a new AppState with an empty show.
    pub fn new() -> Self {
        Self {
            root: Root::new(),
            context: ReplContext::Fixture,
            navigation_path: Vec::new(),
            current_live_screen: None,
            ui_client: false,
            remote_client: false,
            undo_to_owners: false,
            no_oops: false,
            pending_local: Vec::new(),
            current_window: None,
            line_owner: None,
            resolved_line: None,
            ui_requests: Vec::new(),
            current_device: None,
            current_client: None,
            dialogs: Default::default(),
            consent: Default::default(),
            claim_remember: Default::default(),
            asking: None,
            devices: Default::default(),
            station_lock: None,
            skipped_startup_show: None,
            seen_names: Default::default(),
            datapool_override: None,
            running_stored_line: false,
            gesture: None,
            gesture_starts: std::collections::HashMap::new(),
            current_show: None,
            upgraded: None,
            show_dirty: false,
            drives: Vec::new(),
            last_save_at: std::time::Instant::now(),
            saved: Default::default(),
            pending_user_switch: None,
            restart_requested: false,
            local_settings: lumacore_luma::station_config::LocalSettings::default(),
            protocols_running: Default::default(),
            playback: None,
            gdtf_sources: HashMap::new(),
            patch_draft: None,
            identify: Default::default(),
            user: "Admin".to_string(),
            acting_user: None,
            debug_mode: false,
            current_verbosity: None,
            shortcut_mode: ShortcutMode::Programmer,
            sysmon_pending: Vec::new(),
            subtree_owners: HashMap::new(),
            undo_sequence: 0,
            changes: Default::default(),
            show_generation: 0,
            patch_generation: 0,
            engine_generation: 0,
            references: None,
            claim_notices: Vec::new(),
            split_base: None,
            dispatch_depth: 0,
            replication_outbox: Vec::new(),
            replaying_remote_op: false,
            last_applied: None,
            files_wanted: Default::default(),
            link: None,
            forwards: std::collections::BTreeMap::new(),
            reply_to: crate::session_link::ReplyTo::Log,
            forward_waits: None,
            link_requests: Vec::new(),
            has_members: false,
            undo_log: Vec::new(),
            show_replaced: false,
            show_loads: 0,
            synced_default: false,
            command_station: None,
            remote_edits: Vec::new(),
            remote_from: None,
            session_role: crate::session_link::Role::Normal,
            show_status: crate::session_link::ShowStatus::NoShow,
            online_since_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_millis() as u64),
            session_epoch: 1,
            decision: None,
            session_requests: Vec::new(),
            output_plan: String::new(),
            is_authority: true,
            lumasession_ports: (
                lumacore_wire::REMOTE_CMD_PORT,
                lumacore_wire::REMOTE_SHELL_PORT,
                lumacore_wire::REMOTE_STREAM_PORT,
            ),
            protocols_dirty: false,
            profiles_touched: Vec::new(),
        }
    }

    /// Clear all undo stacks and subtree ownership.
    ///
    /// Called when Patch is unclaimed or a show file is loaded —
    /// the usual console rule: every Oops entry is cleared when the Patch changes or a show loads.
    pub fn clear_all_undo_stacks(&mut self) {
        for user in self.root.runtime.users.values_mut() {
            user.undo.clear_all();
        }
        self.subtree_owners.clear();
        self.undo_sequence = 0;
    }

    /// The runtime key of a user: their name as the show spells it.
    fn runtime_key(&self, user: &str) -> String {
        self.root
            .show_data
            .user(user)
            .map_or_else(|| user.to_string(), |u| u.login.clone())
    }

    /// A user's undo history (`Runtime.Users.{User}.Undo`), if they have one.
    pub fn undo_stack(&self, user: &str) -> Option<&lumacore_tree::UndoStack> {
        self.root
            .runtime
            .user_ref(&self.runtime_key(user))
            .map(|u| &u.undo)
    }

    /// A user's undo history, created on first use.
    pub fn undo_stack_mut(&mut self, user: &str) -> &mut lumacore_tree::UndoStack {
        let key = self.runtime_key(user);
        &mut self.root.runtime.user(&key).undo
    }

    /// Record what a command did to `user`'s undo stack, for the members
    /// (decisions §21 "6b": they replay the master's steps). Call after the
    /// stack changed; nothing is kept without members.
    pub fn note_undo_step(&mut self, user: &str, step: UndoKind) {
        if !self.has_members || self.replaying_remote_op {
            return;
        }
        let key = self.runtime_key(user);
        let Some(stack) = self.undo_stack(&key) else {
            return;
        };
        let step = match step {
            UndoKind::Push => stack.frames.back().cloned().map(lumacore_tree::UndoStep::Push),
            UndoKind::Undo => stack
                .redo_frames
                .back()
                .cloned()
                .map(|redo| lumacore_tree::UndoStep::Undo { redo }),
            UndoKind::Redo => stack
                .frames
                .back()
                .cloned()
                .map(|undo| lumacore_tree::UndoStep::Redo { undo }),
        };
        if let Some(step) = step {
            self.undo_log.push(lumacore_tree::UserUndoStep { user: key, step });
        }
    }

    /// Load a .luma show file (the container, `lumacore_luma::show_file`) and
    /// replace the current show.
    ///
    /// `ShowData::after_load` rebuilds all ID pools so future allocations are safe.
    pub fn load_show(&mut self, path: &str) -> Result<(), String> {
        let (mut show_data, upgraded) = lumacore_luma::show_file::read_upgraded(Path::new(path))?;
        // An older show upgraded step by step (decisions 0289): kept for whoever loaded it to say.
        // A saved group-master fade started on the saving station's session
        // clock, which means nothing here: every master stands at its target
        // (R1-03). Only a file load — a sync or Oops keeps the running fade.
        // A load starts with no echoes (decisions §32: never saved).
        show_data.playbacks.echoes = Default::default();
        for level in show_data.playbacks.live.masters.groups.values_mut() {
            *level = level.settled();
        }

        self.replace_show_data(show_data);
        // The file's history and index (the next save compares with it).
        self.saved = lumacore_luma::show_file::manifest_of(Path::new(path))
            .map(|m| lumacore_luma::backups::Saved::from_manifest(&m))
            .unwrap_or_default();
        // Every Oops entry is cleared when a show file loads.
        self.clear_all_undo_stacks();
        // Another file: its names are what it says, not renames of the
        // last show's (a snapshot from the master keeps following them).
        self.seen_names.clear();
        self.upgraded = upgraded;

        Ok(())
    }

    /// Callers outside the state go through `objects::replace_show`, which runs the checks after.
    pub fn replace_show_data(&mut self, show_data: ShowData) {
        // Members start over from a full snapshot (decisions §21 "6b").
        self.show_replaced = true;
        self.show_loads += 1;
        self.show_status = crate::session_link::ShowStatus::Loaded;
        self.root.show_data = show_data;
        self.root.show_data.after_load();
        // Windows of a type this build doesn't know are dropped, and said
        // so (R1-57, Q32).
        for note in self.root.show_data.drop_unknown_windows() {
            lumacore_sysmon::sysmon_note("Show", format!("[WARN] {note}"));
        }
        // A fixture type whose footprint changed: fixtures follow where
        // the slots are free; the rest stay listed (R1-52, Q20).
        for note in lumacore_patch::check::follow_footprints(&mut self.root.show_data) {
            lumacore_sysmon::sysmon_note("Patch", note);
        }
        // Another show: every Oops list starts over (decisions §29 "11c").
        self.clear_all_undo_stacks();
        // … and every clipboard: its GUIDs name the old show's objects (decisions §35 10).
        for user in self.root.runtime.users.values_mut() {
            user.clipboard = None;
        }
        // Another show: the old split base means nothing any more — on disk
        // too, so a restart never brings back a split already resolved (Use
        // Master's snapshot, LoadShow, CleanStart). None at startup: the
        // startup show loads before `restore_split_base` reads the file.
        if self.split_base.take().is_some() {
            crate::session_link::forget_split_base();
        }
        // An open Patch draft starts again from the new show's patch.
        if let Some(d) = &mut self.patch_draft {
            d.live = self.root.show_data.devices.clone();
        }
        self.show_generation += 1;
        self.patch_generation += 1;
        // Another show: a compiler that stopped after panics tries again (Q39).
        if let Some(hub) = &self.playback {
            hub.restart_compiler();
        }
        // The checks that read the registry run above the state (`objects::after_show_replaced`).
    }

    /// What the render engine renders from: `show` plus every logged-in
    /// user's modes (decisions §27), stamped with the publish and patch
    /// generations (§24).
    pub fn engine_input(&self, show: Arc<ShowData>) -> EngineInput {
        let mut users: Vec<UserLayer> = Vec::new();
        // Master 2.2 / 2.4 cap everybody's Highlight / Solo (decisions §30 "12f").
        let grand = self.root.show_data.playbacks.live.masters.grand;
        for (name, user) in &self.root.runtime.users {
            let Some(user_no) = self.root.show_data.user_no(name) else {
                continue;
            };
            let m = &user.modes;
            users.push(UserLayer {
                user_no,
                freeze: m.freeze,
                blind: m.blind,
                // The modes times their per-user masters (decisions §29).
                solo: m.solo_master.strength(m.solo) * grand.solo,
                highlight: m.highlight_master.strength(m.highlight) * grand.highlight,
            });
        }
        users.sort_by_key(|u| u.user_no);
        EngineInput {
            show,
            users,
            identify: self.identify.values().flat_map(|i| i.targets.iter().cloned()).collect(),
            generation: self.engine_generation,
            patch_generation: self.patch_generation,
        }
    }

    /// The engine's frame once it carries everything published so far (at
    /// most one frame, ≤ 17 ms); without an engine, rendered here once.
    pub fn output_frame(&self) -> Arc<lumacore_render::RenderedFrame> {
        match &self.playback {
            Some(hub) => {
                hub.wait_rendered(self.engine_generation, OUTPUT_WAIT);
                hub.frame.load_full()
            }
            None => Arc::new(lumacore_render::render_now(
                &self.engine_input(Arc::new(self.root.show_data.clone())),
                lumacore_base::clock::session_ms(),
            )),
        }
    }

    /// The current output of one universe (see [`Self::output_frame`]).
    pub fn published_frame(&self, universe: u32) -> Option<[u8; 512]> {
        self.output_frame().get(universe).copied()
    }

    /// What arrives for a universe from the DMX input lines (HTP over
    /// lines), before it merges with our output. None without input.
    pub fn published_input(&self, universe: u32) -> Option<[u8; 512]> {
        self.playback.as_ref()?.input_universe(universe)
    }

    /// The plan the engine renders (the winning value of every fixture
    /// attribute, before masters), once it carries everything published so
    /// far; without an engine, compiled here. Commands read output only
    /// through this and [`Self::output_frame`]: the merge runs on the
    /// compiler and the engine, never twice (§24).
    pub fn output_plan(&self) -> Arc<lumacore_render::Plan> {
        match &self.playback {
            Some(hub) => {
                hub.wait_rendered(self.engine_generation, OUTPUT_WAIT);
                hub.plan.load_full()
            }
            None => Arc::new(lumacore_render::Plan::compile(
                &self.engine_input(Arc::new(self.root.show_data.clone())),
                None,
            )),
        }
    }

    /// Whether the current user has show_descriptions enabled.
    pub fn show_descriptions(&self) -> bool {
        let user = self.effective_user_name();
        if user == INTERNAL_ROOT_USER {
            return true;
        }

        self.root
            .show_data
            .profile_for_user(user)
            .map(|p| p.settings.show_descriptions)
            .unwrap_or(true)
    }

    /// Whether clear_buffer is active (user setting OR watches are active).
    pub fn clear_buffer_active(&self) -> bool {
        let user = self.effective_user_name();
        if user == INTERNAL_ROOT_USER {
            return false;
        }
        if !self.watches().is_empty() {
            return true;
        }

        self.root
            .show_data
            .profile_for_user(user)
            .map(|p| p.settings.clear_buffer)
            .unwrap_or(false)
    }

    /// Get the current user's watches (read-only).
    pub fn watches(&self) -> &[lumacore_tree::WatchTarget] {
        self.root
            .runtime
            .user_ref(self.effective_user_name())
            .map(|s| s.watches.as_slice())
            .unwrap_or(&[])
    }

    /// Get the current user's watches (mutable).
    pub fn watches_mut(&mut self) -> &mut Vec<lumacore_tree::WatchTarget> {
        let user = self.effective_user_name().to_string();
        &mut self.root.runtime.user(&user).watches
    }

    /// Get a mutable reference to the current user's profile.
    pub fn current_user_profile_mut(&mut self) -> Option<&mut lumacore_tree::UserProfile> {
        let user = self.effective_user_name().to_string();
        if user == INTERNAL_ROOT_USER {
            return None;
        }

        self.root.show_data.profile_for_user_mut(user.as_str())
    }

    /// The acting user's selected datapool (decisions §29), else the first;
    /// while an owner-first line (`Delete DataPool 2 Group 3`) runs, that one.
    pub fn selected_datapool(&self) -> u32 {
        self.datapool_override
            .unwrap_or_else(|| crate::pools::user_datapool(&self.root.show_data, self.effective_user_name()))
    }

    /// The acting user's selected sequence, if it (still) exists.
    /// As (datapool, number): it may be in any DataPool (decisions §30 "12f").
    pub fn selected_sequence(&self) -> Option<(u32, u32)> {
        let guid = self.root.show_data.user(self.effective_user_name())?.selected.sequence?;
        self.root.show_data.sequence_by_guid(guid).map(|(dp, n, _)| (dp, n))
    }

    /// A selection whose object was deleted is cleared (decisions §29).
    /// A fixture that left the patch leaves no programmer value, selection
    /// or grid spot behind, in the same step (R1-38, Q9: Oops brings both
    /// back). Not while a Devices draft is open: the rig still runs the live
    /// patch; keeping the draft cleans up.
    fn clear_gone_fixtures(&mut self) {
        if self.patch_draft.is_some() {
            return;
        }
        let show = &self.root.show_data;
        let held = show.users.values().any(|u| {
            let p = &u.programmer;
            !p.values.is_empty() || !p.selected_fixtures.is_empty() || !p.grid.is_empty()
        });
        if !held {
            return;
        }
        let patched: std::collections::HashSet<uuid::Uuid> =
            show.devices.dmx.patch.fixtures.values().map(|f| f.meta.identity.guid).collect();
        let gone = |g: &uuid::Uuid| !patched.contains(g);
        let any_gone = show.users.values().any(|u| {
            let p = &u.programmer;
            p.values.keys().any(|(g, _)| gone(g)) || p.selected_fixtures.iter().any(gone) || p.grid.keys().any(gone)
        });
        if !any_gone {
            return;
        }
        for user in self.root.show_data.users.values_mut() {
            let p = &mut user.programmer;
            p.values.retain(|(g, _), _| !gone(g));
            p.selected_fixtures.retain(|g| !gone(g));
            p.grid.retain(|g, _| !gone(g));
        }
    }

    /// Station maps stay keyed by name (they work across shows), so when an
    /// AssetRoot or Network is renamed — here or on another station — this
    /// station moves its own folder / NIC mapping to the new name (R1-69,
    /// Q41: the one exception to "links are GUIDs", decisions §8 / §11).
    /// Returns whether a mapping moved.
    pub fn follow_renames(&mut self) -> bool {
        let show = &self.root.show_data;
        let now: Vec<(uuid::Uuid, String, bool)> = show
            .assets
            .roots
            .values()
            .map(|r| (r.meta.identity.guid, r.name.to_lowercase(), true))
            .chain(
                show.protocols
                    .networks
                    .values()
                    .filter_map(|n| Some((n.meta.identity.guid, n.meta.identity.name.clone()?.to_lowercase(), false))),
            )
            .collect();
        let mut moved = false;
        for (guid, name, asset) in now {
            if let Some(old) = self.seen_names.insert(guid, name.clone())
                && old != name
            {
                let map = if asset {
                    &mut self.root.station_settings.asset_roots
                } else {
                    &mut self.root.station_settings.networks
                };
                if !map.contains_key(&name)
                    && let Some(value) = map.remove(&old)
                {
                    map.insert(name, value);
                    moved = true;
                }
            }
        }
        if moved {
            self.local_settings.store_from(&self.root.station_settings);
            self.protocols_dirty = true;
        }
        moved
    }

    pub fn clear_dangling_selections(&mut self) {
        self.clear_gone_fixtures();
        let show = &self.root.show_data;
        let datapools: Vec<uuid::Uuid> = show.datapools.values().map(|d| d.meta.identity.guid).collect();
        let sequences: Vec<uuid::Uuid> = show.sequences().map(|(_, _, s)| s.meta.identity.guid).collect();
        let timecodes: Vec<uuid::Uuid> = show
            .datapools
            .values()
            .flat_map(|d| d.timecodes.values().map(|t| t.meta.identity.guid))
            .collect();
        let dangling = |sel: &lumacore_tree::Selected| {
            sel.datapool.is_some_and(|g| !datapools.contains(&g))
                || sel.sequence.is_some_and(|g| !sequences.contains(&g))
                || sel.timecode.is_some_and(|g| !timecodes.contains(&g))
        };
        if !show.users.values().any(|u| dangling(&u.selected)) {
            return;
        }
        for user in self.root.show_data.users.values_mut() {
            let sel = &mut user.selected;
            if sel.datapool.is_some_and(|g| !datapools.contains(&g)) {
                sel.datapool = None;
            }
            if sel.sequence.is_some_and(|g| !sequences.contains(&g)) {
                sel.sequence = None;
            }
            if sel.timecode.is_some_and(|g| !timecodes.contains(&g)) {
                sel.timecode = None;
            }
        }
    }

    /// Get the effective verbosity for the current command.
    ///
    /// Per-command override (`/v`, `/d` flags) takes priority over user profile default.
    pub fn effective_verbosity(&self) -> Verbosity {
        let user = self.effective_user_name();
        if user == INTERNAL_ROOT_USER {
            return self.current_verbosity.clone().unwrap_or_default();
        }
        if let Some(v) = &self.current_verbosity {
            return v.clone();
        }

        self.root
            .show_data
            .profile_for_user(user)
            .map(|p| p.settings.verbosity.clone())
            .unwrap_or_default()
    }

    /// The acting user's programmer (and selection), mutable.
    pub fn current_programmer_mut(&mut self) -> Option<&mut lumacore_tree::programmer::Programmer> {
        let user = self.effective_user_name().to_string();
        if user == INTERNAL_ROOT_USER {
            return None;
        }

        self.root.show_data.programmer_of_mut(&user)
    }

    /// Get an immutable reference to the current user's programmer.
    pub fn current_programmer(&self) -> Option<&lumacore_tree::programmer::Programmer> {
        let user = self.effective_user_name();
        if user == INTERNAL_ROOT_USER {
            return None;
        }

        self.root.show_data.programmer_of(user)
    }

    /// Replace the current user's selection. Selecting claims nothing; only
    /// fixtures another user has claimed (`Claim Fixture`) are refused.
    ///
    /// Accepts user-facing integer IDs and converts to GUIDs internally.
    pub fn replace_current_selection(&mut self, ids: Vec<u32>) -> Result<usize, String> {
        super::programmers::ensure_unclaimed(self, &ids)?;
        let count = ids.len();
        let guids = self.root.show_data.fixture_guids(&ids);
        let prog = self
            .current_programmer_mut()
            .ok_or_else(|| "No programmer available.".to_string())?;
        prog.selected_fixtures = guids;
        prog.prune_grid();
        prog.clear_state = lumacore_tree::programmer::ClearState::Fresh;
        prog.exit_walk();
        Ok(count)
    }

    /// Add one fixture to the current user's selection (refused if another
    /// user claimed it).
    pub fn add_fixture_to_selection(&mut self, fixture_id: u32) -> Result<usize, String> {
        self.select_fixtures(&[fixture_id], lumacore_tree::programmer::SelectMode::Add)
    }

    /// Run `f` with `dp` as the DataPool a DataPool object means (a line's owner, decisions §30
    /// "12a2 / 12a3 hardening"; None keeps the current one), then put the scope back. The shared
    /// door for the DataPool scope (tidy `shared_doors_have_no_copies`).
    pub fn in_datapool<R>(&mut self, dp: Option<u32>, f: impl FnOnce(&mut Self) -> R) -> R {
        let before = self.datapool_override;
        if dp.is_some() {
            self.datapool_override = dp;
        }
        let out = f(self);
        self.datapool_override = before;
        out
    }

    /// Select fixtures (in order) by the selection rule (decisions §22): a
    /// plain select adds until a value is set, then replaces. They land in
    /// the grid from the cursor in a row. Returns the selection's size.
    pub fn select_fixtures(
        &mut self,
        ids: &[u32],
        how: lumacore_tree::programmer::SelectMode,
    ) -> Result<usize, String> {
        super::programmers::ensure_unclaimed(self, ids)?;
        let guids = self.root.show_data.fixture_guids(ids);
        self.select_members(&guids, &[], how, true, lumacore_tree::GridCursorMove::Linked)
    }

    /// Select fixtures with their grid spots (a group's members; `cells`
    /// empty = a row). `cursor` Linked follows the user's profile.
    pub fn select_members(
        &mut self,
        fixtures: &[uuid::Uuid],
        cells: &[lumacore_tree::GridCell],
        how: lumacore_tree::programmer::SelectMode,
        preserve: bool,
        cursor: lumacore_tree::GridCursorMove,
    ) -> Result<usize, String> {
        let cursor = match cursor {
            lumacore_tree::GridCursorMove::Linked => self
                .root
                .show_data
                .profile_for_user(self.effective_user_name())
                .map_or(lumacore_tree::GridCursorMove::AppendX, |p| {
                    p.settings.preferences.grid_cursor_move
                }),
            other => other,
        };
        let prog = self
            .current_programmer_mut()
            .ok_or_else(|| "No programmer available.".to_string())?;
        Ok(prog.apply_selection(fixtures, cells, how, preserve, cursor))
    }

    /// Groups of the user's selected datapool (decisions §22: `Group 3` is
    /// pool-relative).
    pub fn user_groups(&self) -> &std::collections::BTreeMap<u32, lumacore_tree::Group> {
        static NONE: std::sync::OnceLock<std::collections::BTreeMap<u32, lumacore_tree::Group>> =
            std::sync::OnceLock::new();
        self.root
            .show_data
            .datapools
            .get(&self.selected_datapool())
            .map_or_else(|| NONE.get_or_init(Default::default), |dp| &dp.groups)
    }

    /// Find an existing user profile by name and switch the session to it.
    pub fn switch_to_existing_user(&mut self, user: &str) -> Option<String> {
        use lumacore_tree::TreeNode;

        let trimmed = user.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case(INTERNAL_ROOT_USER) {
            return None;
        }

        if let Some(name) = self
            .root
            .show_data
            .user_profiles
            .values()
            .find(|p| {
                p.name()
                    .map(|n| n.eq_ignore_ascii_case(trimmed))
                    .unwrap_or(false)
            })
            .and_then(|p| p.name().map(|s| s.to_string()))
        {
            self.user = name.clone();
            return Some(name);
        }

        None
    }

    /// Switch the runtime into the hidden internal superuser.
    pub fn switch_to_internal_root(&mut self) {
        self.user = INTERNAL_ROOT_USER.to_string();
        self.acting_user = None;
    }

    /// Return the user identity that command logic should operate on.
    pub fn effective_user_name(&self) -> &str {
        self.acting_user.as_deref().unwrap_or(&self.user)
    }

    /// Enter a scoped acting-user context without changing the internal runtime identity.
    pub fn begin_acting_as_existing_user(&mut self, user: &str) -> Option<String> {
        use lumacore_tree::TreeNode;

        let trimmed = user.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case(INTERNAL_ROOT_USER) {
            return None;
        }

        let show = &self.root.show_data;
        let name = show.user(trimmed).map(|u| u.login.clone()).or_else(|| {
            show.user_profiles
                .values()
                .find(|p| p.name().is_some_and(|n| n.eq_ignore_ascii_case(trimmed)))
                .and_then(|p| p.name().map(|s| s.to_string()))
        })?;

        self.acting_user = Some(name.clone());
        Some(name)
    }

    /// Clear any scoped acting-user context and return to the true runtime identity.
    pub fn clear_acting_user(&mut self) {
        self.acting_user = None;
    }

    /// Collect all known object names for tab completion / hints.
    pub fn collect_names(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();

        // Fixture type names
        for ft in self.root.show_data.devices.dmx.patch.fixture_types.values() {
            if let Some(name) = lumacore_tree::TreeNode::name(ft) {
                names.push(name.to_string());
            }
        }

        // Fixture names
        for f in self.root.show_data.devices.dmx.patch.fixtures.values() {
            if let Some(name) = lumacore_tree::TreeNode::name(f) {
                names.push(name.to_string());
            }
        }

        // Universe names
        for u in self.root.show_data.devices.dmx.patch.universes.values() {
            if let Some(name) = lumacore_tree::TreeNode::name(u) {
                names.push(name.to_string());
            }
        }

        names
    }
}
