// ███████╗████████╗ █████╗ ████████╗███████╗
// ██╔════╝╚══██╔══╝██╔══██╗╚══██╔══╝██╔════╝
// ███████╗   ██║   ███████║   ██║   █████╗
// ╚════██║   ██║   ██╔══██║   ██║   ██╔══╝
// ███████║   ██║   ██║  ██║   ██║   ███████╗
// ╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝   ╚══════╝
// FILE: state

//! The state thread (`docs/meta/decisions.md` §26).
//!
//! One thread owns the [`ServerContext`] — show, runtime and station state —
//! and runs work on it one job at a time. Every other thread (shell clients,
//! the UDP command port, discovery, HTTP, OSC) sends it jobs through a
//! [`StateHandle`] and gets the result back; none of them holds the state, so
//! there is no global lock and a slow client can't freeze the server. The
//! state thread does no network I/O of its own.
//!
//! After each job the thread republishes the show snapshot when the show
//! changed (`AppState::show_generation`), so readers such as HTTP never wait
//! on the queue. Each job runs under a panic guard: a panic is reported and
//! the thread keeps going.
//!
//! Owns:          the state thread: its loop (jobs, the inbound events, the LumaSession link, periodic work, publishing the show and the StationView, protocols, the render and clock watches), Published, FeedSignals, the server's StateHandle instance (Host for ServerContext), HTTP_WAIT.
//! Doesn't own:   the lanes and the watchdog (lumacore-lanes), what a job does (the callers), the engines.
//! Entry points:  spawn, Published, StationView, Periodic, StateHandle, HTTP_WAIT.
//! Decisions:     §26, §31, R1-67, Q40, §34 24.2.

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use crossbeam_channel::{Receiver, select};
use lumacore_base::InboundEvent;
use lumacore_tree::ShowData;
use lumacore_tree::runtime::ThreadGuard;

use lumacore_sysmon::panic_report::{self, PanicReport};
use crate::context::ServerContext;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The state thread's context is the lanes' host (decisions §34 24.2): a job runs under the
/// station's panic guard, and a panic counts on the state thread's Health entry.
impl lumacore_lanes::Host for ServerContext {
    type Panic = PanicReport;
    fn guarded<R>(f: impl FnOnce() -> R) -> Result<R, PanicReport> {
        panic_report::guarded(f)
    }
    fn panicked(&mut self) {
        self.state.root.runtime.health.register("State").panicked();
    }
}

/// Work for the state thread.
pub type Job = lumacore_lanes::Job<ServerContext>;
/// A way to reach the state thread (`lumacore_lanes`, on this server's context).
pub type StateHandle = lumacore_lanes::StateHandle<ServerContext>;
/// A job could not deliver its result.
pub type CallError = lumacore_lanes::CallError<PanicReport>;
pub use lumacore_lanes::{Lane, MACHINE_LANE};

/// How long the loop waits for a job before running its periodic work.
const IDLE_WAKE: Duration = Duration::from_millis(20);

/// How long an HTTP request waits for the state thread before it answers
/// "busy" (R1-67, Q40).
pub const HTTP_WAIT: Duration = Duration::from_secs(10);
// The shell's own waits (`SHELL_WAIT`, `SUGGEST_WAIT`) live with it in `lumacore-shell`.

/// Work the state thread does on a timer (autosave, statuses, diagnostics).
pub struct Periodic {
    pub name: &'static str,
    pub every: Duration,
    pub run: Box<dyn FnMut(&mut ServerContext) + Send>,
    last: Option<Instant>,
}

impl Periodic {
    pub fn new(
        name: &'static str,
        every: Duration,
        run: impl FnMut(&mut ServerContext) + Send + 'static,
    ) -> Self {
        Self {
            name,
            every,
            run: Box::new(run),
            last: None,
        }
    }
}

/// The show as readers see it: replaced after every change.
pub use lumacore_engines::PublishedShow;

/// Station facts readers need besides the show (HTTP: `/api/station`, the
/// Backup menu, token checks, station UI keys). Small; republished often.
#[derive(Debug, Clone, Default)]
pub struct StationView {
    /// `GET /api/station` body.
    pub station: serde_json::Value,
    /// The loaded show (`lumacore_cmd_showfile::show_info`).
    pub show_info: lumacore_wire::ShowStatus,
    /// Signs login tokens (station-local, never replicated).
    pub token_secret: String,
    /// Tokens ended by logout (signature → expiry).
    pub revoked_tokens: std::collections::BTreeMap<String, u64>,
    /// Tokens whose login waits for the Meow License (signatures; decisions 0290): no login yet.
    pub consent_tokens: std::collections::BTreeSet<String>,
    /// Station UI keys (Root.Resources.UiKeys).
    pub ui_keys: Vec<lumacore_tree::UiKey>,
    /// Menu → Network / DMX Protocols (`GET /api/network`, the `network`
    /// feed); built only while a feed watches it (`FeedSignals`), else null.
    pub network: serde_json::Value,
    /// Menu → LumaSessions (`GET /api/lumasessions`, the `lumasessions`
    /// feed); built only while a feed watches it, else null.
    pub lumasessions: serde_json::Value,
    /// Claimed objects: GUID → the user holding it (pool windows' lock).
    pub claims: std::collections::BTreeMap<uuid::Uuid, String>,
    /// The Backup menu's station facts (`lumacore_objects::recovery::station_facts`): the
    /// cleanup rules, when / by whom the loaded file was saved.
    pub backup: serde_json::Value,
    /// Each logged-in user's own Highlight / Solo masters (lower-case name → `{highlight: {level,
    /// mode}, solo: …}`): the encoder bar's Masters mode (decisions §30 "12f").
    pub mode_masters: std::collections::BTreeMap<String, serde_json::Value>,
    /// Each logged-in user's Oops list (lower-case name → `{steps, undone}`): the Oops window
    /// (decisions §30 "12f").
    pub undo: std::collections::BTreeMap<String, serde_json::Value>,
    /// Each user's clipboard (lower-case name → `{object, count, cut}`): Paste counts what it
    /// holds (decisions §34 round 3, 18 "slab states").
    pub clipboards: std::collections::BTreeMap<String, serde_json::Value>,
}

pub type PublishedStation = Arc<ArcSwap<StationView>>;

/// The station's own counters for the live feeds (decisions §30 "12d
/// hardening": shared builds are rebuilt when these move, never on a timer).
/// Each moves when what it names was published changed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FeedRevs {
    pub show: u64,
    pub station: u64,
    pub show_info: u64,
    pub claims: u64,
    pub network: u64,
    pub lumasessions: u64,
    /// The show library on disk (`drive::writes`).
    pub library: u64,
    pub tokens: u64,
    /// The users' own Highlight / Solo masters (`StationView::mode_masters`).
    pub modes: u64,
    /// The users' Oops lists (`StationView::undo`).
    pub undo: u64,
    /// The users' clipboards (`StationView::clipboards`).
    pub clipboard: u64,
}

/// What the live feeds ask of the state thread, and how it tells them that
/// something moved.
#[derive(Default)]
pub struct FeedSignals {
    /// Feeds watching Menu → Network / DMX Protocols: the view (and its NIC
    /// list) is built only while one does.
    pub network: std::sync::atomic::AtomicUsize,
    /// Feeds watching Menu → LumaSessions.
    pub lumasessions: std::sync::atomic::AtomicUsize,
    /// Called with the counters when they moved (the HTTP engine's watch).
    notify: std::sync::OnceLock<OnRevs>,
}

type OnRevs = Box<dyn Fn(&FeedRevs) + Send + Sync>;

impl FeedSignals {
    /// The one listener (the console bridge); later ones are ignored.
    pub fn on_change(&self, f: impl Fn(&FeedRevs) + Send + Sync + 'static) {
        let _ = self.notify.set(Box::new(f));
    }

    fn watched(counter: &std::sync::atomic::AtomicUsize) -> bool {
        counter.load(std::sync::atomic::Ordering::Relaxed) > 0
    }

    fn changed(&self, revs: &FeedRevs) {
        if let Some(f) = self.notify.get() {
            f(revs);
        }
    }
}

/// The open Devices draft for readers: its holder, the devices, and the cells someone else's live
/// edit brought in (the violet dot, decisions §34 round 3).
pub type PublishedDraft = (String, lumacore_tree::DevicesData, std::collections::BTreeMap<uuid::Uuid, std::collections::BTreeMap<String, String>>, Option<serde_json::Value>);

/// What the state thread publishes for readers.
#[derive(Clone)]
pub struct Published {
    pub show: PublishedShow,
    /// The open Devices draft and its holder (the Patch dialog shows its holder the draft,
    /// decisions §34 "34g4 patch hardened" 9); None while nobody holds one.
    pub draft: Arc<ArcSwap<Option<PublishedDraft>>>,
    pub station: PublishedStation,
    /// The render engine's input (the show plus every user's modes).
    pub render: Arc<lumacore_render::RenderHub>,
    /// Which interfaces serve LumaSession / Shell / HTTP (decisions §11).
    pub services: lumacore_lumasession::Services,
    /// The live feeds' demand and counters (decisions §30 "12d").
    pub feeds: Arc<FeedSignals>,
}

impl Published {
    /// `render`: `Runtime.Health.Render`, shared with the engine.
    pub fn new(show: ShowData, render: Arc<lumacore_tree::runtime::RenderStats>) -> Self {
        let show = Arc::new(show);
        Self {
            show: Arc::new(ArcSwap::new(show.clone())),
            draft: Arc::new(ArcSwap::from_pointee(None)),
            station: Arc::new(ArcSwap::from_pointee(StationView::default())),
            render: lumacore_render::RenderHub::with_stats(
                lumacore_render::EngineInput::new(show),
                render,
            ),
            services: Arc::new(ArcSwap::from_pointee(
                lumacore_lumasession::ServiceAccess::default(),
            )),
            feeds: Arc::default(),
        }
    }
}

/// How often the render engine's overload / down flags are looked at.
const RENDER_CHECK_EVERY: Duration = Duration::from_millis(500);

/// What the System Monitor was last told about the render engine.
#[derive(Default)]
struct RenderWatch {
    overload: bool,
    down: bool,
    panics: u64,
    /// Plan compiler panics seen, and whether it is failing now (an
    /// episode ends with a compile that works).
    compile_panics: u64,
    compiles: u64,
    compile_failing: bool,
    compiler_down: bool,
}

impl RenderWatch {
    fn check(&mut self, stats: &lumacore_tree::runtime::RenderStats, sysmon: &mut Vec<(String, String)>) {
        let v = stats.snapshot();
        let mut say = |msg: String| sysmon.push(("Render".to_string(), msg));
        if v.overload && !self.overload {
            say(format!(
                "[WARN] Overload: frames miss their {} µs deadline (frame {} µs, late {}, skipped {})",
                v.period_us, v.frame_max_us, v.late, v.skipped
            ));
        } else if !v.overload && self.overload {
            say(format!("Overload over (late {}, skipped {} in all)", v.late, v.skipped));
        }
        if v.panics > self.panics {
            say(if v.down {
                "[ERROR] the render engine panicked again within 10 s: it stays down, the lines hold the last frame".to_string()
            } else {
                "[WARN] the render engine panicked: holding the last frame, restarted with a fresh plan".to_string()
            });
        }
        if v.compile_panics > self.compile_panics {
            if !self.compile_failing {
                say(format!(
                    "[ERROR] the plan compiler panicked: stage output is stale since change {} (the last plan stays); the next change compiles again",
                    v.stale_since
                ));
            }
            self.compile_failing = true;
        } else if self.compile_failing && v.compiles > self.compiles {
            say("the plan compiler works again".to_string());
            self.compile_failing = false;
        }
        self.overload = v.overload;
        self.down = v.down;
        self.panics = v.panics;
        if v.compiler_down && !self.compiler_down {
            say("[ERROR] the plan compiler stopped after 3 panics within 60 s: the stage keeps the last plan; LoadShow or Restart starts it again".to_string());
        } else if !v.compiler_down && self.compiler_down {
            say("the plan compiler runs again".to_string());
        }
        self.compiler_down = v.compiler_down;
        self.compile_panics = v.compile_panics;
        self.compiles = v.compiles;
    }
}

/// Whom the session clock follows (decisions §24 "Session clock"): a
/// member its master's command port; a master or a standalone station
/// nobody (it is its own reference and runs on at its frequency).
#[derive(Default)]
struct ClockWatch {
    last: Option<(u8, Option<lumacore_clocksync::Target>)>,
}

impl ClockWatch {
    fn follow(&mut self, state: &lumacore_objects::AppState) {
        use std::net::ToSocketAddrs;
        let role = match lumacore_objects::session_link::status_word(state) {
            "Member" => 2,
            "Master" | "IdleMaster" => 1,
            _ => 0,
        };
        let target = state.link.as_ref().filter(|_| role == 2).and_then(|link| {
            let host = link
                .target
                .rsplit_once(':')
                .map_or(link.target.as_str(), |(h, _)| h);
            let addr = (host, link.master_cmd_port)
                .to_socket_addrs()
                .ok()?
                .find(|a| a.is_ipv4())?;
            (link.master_cmd_port != 0).then(|| lumacore_clocksync::Target {
                addr,
                master: link.master.clone(),
                epoch: state.session_epoch,
                from: state.root.station_settings.station_name.clone(),
            })
        });
        let now = (role, target);
        if self.last.as_ref() == Some(&now) {
            return;
        }
        let (role, target) = now.clone();
        self.last = Some(now);
        if target.is_some() {
            lumacore_clocksync::follow(target);
        } else {
            lumacore_clocksync::follow(None);
            lumacore_clocksync::set_role(if role == 2 { 0 } else { role });
        }
    }
}

/// How often protocol runtimes are reconciled without a show change.
const PROTOCOLS_EVERY: Duration = Duration::from_secs(1);

/// How often the station view is refreshed even without a show change
/// (sessions, autosave countdown).
const STATION_VIEW_EVERY: Duration = Duration::from_millis(250);

/// Where each user is logged in: this station, the master and every member
/// (decisions §30 "12a3": "A user can be logged in on multiple stations at
/// the same time").
pub(crate) fn presence(ctx: &ServerContext) -> std::collections::BTreeMap<String, Vec<String>> {
    let state = &ctx.state;
    let mut out: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    let mut add = |user: &str, station: &str| {
        let at = out.entry(user.to_lowercase()).or_default();
        if !station.is_empty() && !at.iter().any(|s| s == station) {
            at.push(station.to_string());
        }
    };
    let here = &state.root.station_settings.station_name;
    for user in ctx.sessions.users() {
        add(&user, here);
    }
    let sessions = &state.root.runtime.luma_sessions;
    for user in &sessions.master_users {
        add(user, &sessions.master);
    }
    for member in sessions.members.values() {
        for user in &member.users {
            add(user, &member.station);
        }
    }
    out
}

/// The status line of the lock surface: DMX output, LumaSession role,
/// unsaved changes.
fn surface_status(state: &lumacore_objects::AppState) -> serde_json::Value {
    let settings = &state.root.station_settings;
    let has_outputs = state.root.show_data.protocols.all_lines().any(|(_, _, l)| l.sends());
    let dmx = if !has_outputs {
        "no output"
    } else if settings.output_held {
        "output held"
    } else if state.is_authority {
        "running"
    } else {
        "sent by the master"
    };
    serde_json::json!({
        "dmx": dmx,
        "role": lumacore_objects::session_link::status_word(state),
        "lumasession": settings.lumasession,
        "unsaved": state.show_dirty,
    })
}

fn station_view(ctx: &ServerContext, feeds: &FeedSignals) -> StationView {
    let state = &ctx.state;
    let presence = presence(ctx);
    let show = &state.root.show_data;
    let users: Vec<serde_json::Value> = show
        .users
        .iter()
        .map(|(no, u)| {
            let profile_no = show.profile_of(u).map(|(n, _)| n);
            let profile = show.profile_of(u).map(|(_, p)| p);
            let here = presence.get(&u.login.to_lowercase()).cloned().unwrap_or_default();
            let has_password = lumacore_objects::passwords::has_password(state, &u.login);
            serde_json::json!({
                "guid": u.meta.identity.guid,
                "object": "User",
                "no": no,
                "name": u.login,
                "rights": u.rights.label(),
                // The Users grid's cells (decisions §30 "12d part 4").
                "password": if has_password { "set" } else { "—" },
                "online": if here.is_empty() { "—".to_string() } else { here.join(", ") },
                "color": u.color,
                "profile": profile.and_then(|p| p.meta.identity.name.clone()),
                "profile_no": profile_no,
                // Programmer values / selected fixtures (the delete ask's counts).
                "values": u.programmer.values.len(),
                "selected": u.programmer.selected_fixtures.len(),
                "has_password": has_password,
                "remote_login": u.remote_login,
                "logged_in": here,
                "look": profile.map(lumacore_objects::commands::media::look_of),
            })
        })
        .collect();
    // The Users & Profiles window's list (decisions §30 "12b2").
    let profiles: Vec<serde_json::Value> = show
        .user_profiles
        .iter()
        .map(|(no, p)| {
            let used_by: Vec<&str> = show
                .users
                .values()
                .filter(|u| show.profile_of(u).is_some_and(|(n, _)| n == *no))
                .map(|u| u.login.as_str())
                .collect();
            serde_json::json!({
                "guid": p.meta.identity.guid,
                "object": "UserProfile",
                "no": no,
                "name": p.meta.identity.name,
                // The template Default (decisions §30 "12c2"): no edits,
                // no users, never deleted.
                "locked": p.locked,
                "used_by_text": if used_by.is_empty() { "—".to_string() } else { used_by.join(", ") },
                "used_by": used_by,
                "screens": p.content.screen_configs.values().map(|c| c.screens.len()).sum::<usize>(),
                "views": p.content.views.len(),
                "window_presets": p.content.window_presets.values().map(|t| t.len()).sum::<usize>(),
                "preset": p.settings.theme.preset,
                "look": lumacore_objects::commands::media::look_of(p),
            })
        })
        .collect();
    StationView {
        station: serde_json::json!({
            "name": state.root.station_settings.station_name,
            "id": state.local_settings.station_id,
            "version": env!("CARGO_PKG_VERSION"),
            "build": state.root.diagnostics.build_info.build_id,
            // The app checks these before it attaches to a server on its own
            // machine (decisions §23 "App ↔ local server hardening").
            "client_protocol": lumacore_wire::CLIENT_PROTOCOL,
            "pid": std::process::id(),
            "exe": std::env::current_exe().ok(),
            "started_ms": started_ms(),
            "http_port": ctx.http_port,
            "shell_port": ctx.shell_port,
            "show": state.current_show,
            "users": users,
            "profiles": profiles,
            // This station's drives (decisions §38): the Drives menu's rows, every file dialog's rail.
            "drives": state.drives.iter().enumerate().map(|(i, d)| serde_json::json!({
                "no": i + 1, "id": d.id, "object": "Drive", "name": d.id, "label": d.label,
                "kind": d.kind.word(), "path": d.mount.to_string_lossy(),
                "library": if d.has_library() { "Yes" } else { "No" },
                "library_tone": if d.has_library() { "ok" } else { "warn" },
                "state": d.state_word(), "tone": if d.present { "ok" } else { "off" },
                "free": if d.present { lumacore_objects::drives::gb(d.free) } else { String::new() },
                "total": if d.present { lumacore_objects::drives::gb(d.total) } else { String::new() },
                "free_bytes": d.free, "total_bytes": d.total, "since_ms": d.since_ms,
            })).collect::<Vec<_>>(),
            "status": surface_status(state),
        }),
        mode_masters: state
            .root
            .runtime
            .users
            .iter()
            .map(|(name, u)| {
                let m = &u.modes;
                (
                    name.to_lowercase(),
                    serde_json::json!({
                        "highlight": { "level": m.highlight_master.level, "mode": m.highlight_master.mode.word() },
                        "solo": { "level": m.solo_master.level, "mode": m.solo_master.mode.word() },
                    }),
                )
            })
            .collect(),
        undo: state
            .root
            .runtime
            .users
            .keys()
            .filter_map(|name| {
                let stack = state.undo_stack(name)?;
                let row = |f: &lumacore_tree::undo::UndoFrame| {
                    serde_json::json!({
                        "line": f.description,
                        "category": f.category.label(),
                        "station": f.station,
                        "time_ms": f.time_ms,
                        "main": f.main,
                    })
                };
                // The newest 200 (oldest first); undone: the next redo first.
                let skip = stack.frames.len().saturating_sub(200);
                Some((
                    name.to_lowercase(),
                    serde_json::json!({
                        "steps": stack.frames.iter().skip(skip).map(row).collect::<Vec<_>>(),
                        "undone": stack.redo_frames.iter().rev().take(200).map(row).collect::<Vec<_>>(),
                    }),
                ))
            })
            .collect(),
        clipboards: state
            .root
            .runtime
            .users
            .iter()
            .filter_map(|(name, u)| {
                let c = u.clipboard.as_ref()?;
                Some((name.to_lowercase(), serde_json::json!({ "object": c.object, "count": c.rows.len(), "cut": c.cut })))
            })
            .collect(),
        show_info: lumacore_cmd_showfile::show_info(state),
        token_secret: state.local_settings.token_secret.clone(),
        revoked_tokens: state.local_settings.revoked_tokens.clone(),
        consent_tokens: state.consent.token_signatures(),
        ui_keys: state.root.resources.ui_keys.values().cloned().collect(),
        // Only while a page shows the menu (decisions §30 "12d": no NIC list
        // and no menu view every 250 ms for nobody).
        network: if FeedSignals::watched(&feeds.network) {
            lumacore_views::menu_rows::network(state)
        } else {
            serde_json::Value::Null
        },
        lumasessions: if FeedSignals::watched(&feeds.lumasessions) {
            lumacore_views::menu_rows::lumasessions(state)
        } else {
            serde_json::Value::Null
        },
        claims: state
            .root
            .runtime
            .claims
            .iter()
            .map(|(g, c)| (*g, c.user.clone()))
            .collect(),
        backup: lumacore_objects::recovery::station_facts(state),
    }
}

/// When this process started (unix ms).
/// When the server started (wall clock), worked out once: recomputing it from the uptime wobbles
/// by a millisecond, and every wobble would rebuild the `station` feed.
fn started_ms() -> u64 {
    static STARTED: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *STARTED.get_or_init(|| {
        let up = lumacore_sysmon::SERVER_START.get().map_or(0, |s| s.elapsed().as_millis() as u64);
        lumacore_tree::now_ms().saturating_sub(up)
    })
}

/// A copy of the show for readers (undo stacks live in Runtime, not here).
fn publishable(show: &ShowData) -> ShowData {
    show.clone()
}

/// Start the state thread. It owns `ctx` until it stops, then hands it back
/// through the join handle (for the shutdown report and Restart).
pub fn spawn(
    ctx: ServerContext,
    inbound: Receiver<InboundEvent>,
    periodic: Vec<Periodic>,
    published: Published,
) -> (StateHandle, JoinHandle<ServerContext>) {
    let health = ctx.state.root.runtime.health.register("State");
    let (state, lanes) = lumacore_lanes::lanes(move || health.refused());
    let alive = Arc::new(());
    let watched = Arc::downgrade(&alive);
    let handle = std::thread::Builder::new()
        .name("state".to_string())
        .spawn(move || {
            let _alive = alive;
            run(ctx, lanes, inbound, periodic, published)
        })
        .expect("failed to spawn the state thread");
    // The watchdog (R1-67, Q40): one System Monitor line per busy episode.
    lumacore_lanes::spawn_watchdog(&state, watched);
    (state, handle)
}

fn run(
    mut ctx: ServerContext,
    lanes: lumacore_lanes::Lanes<ServerContext>,
    mut inbound: Receiver<InboundEvent>,
    mut periodic: Vec<Periodic>,
    published: Published,
) -> ServerContext {
    panic_report::install();
    lumacore_lanes::enter_state_thread();
    let lumacore_lanes::Lanes { shell, mut machine, busy } = lanes;
    let health = ctx.state.root.runtime.health.register("State");
    let _alive = ThreadGuard(health.clone());
    let mut published_generation = None;
    let mut station_published = Instant::now() - STATION_VIEW_EVERY;
    let mut protocols_checked = Instant::now() - PROTOCOLS_EVERY;
    let mut render_checked = Instant::now();
    let mut render_watch = RenderWatch::default();
    let mut clock_watch = ClockWatch::default();
    let mut revs = FeedRevs::default();
    let mut told = revs;
    let mut library_writes = lumacore_luma::drive::writes();

    let run_job = |ctx: &mut ServerContext, job: Job| {
        let started = Instant::now();
        busy.start();
        lumacore_trace::trace_zone!("state_job");
        if panic_report::guarded(|| job(ctx)).is_err() {
            health.panicked();
        }
        // Joins/leaves an invite or dismissal asked for.
        if !ctx.lumasession.deferred.is_empty() && panic_report::guarded(|| lumacore_lumasession::run_deferred(ctx)).is_err() {
            health.panicked();
        }
        health.tick(started.elapsed().as_micros() as u64);
    };
    loop {
        // The shell lane goes first: a flood on the machine lane never holds a person's key back.
        if let Ok(job) = shell.try_recv() {
            run_job(&mut ctx, job);
        } else { select! {
            recv(shell) -> job => match job {
                Ok(job) => run_job(&mut ctx, job),
                // Every handle is gone: nobody can send work any more.
                Err(_) => break,
            },
            recv(machine) -> job => match job {
                Ok(job) => run_job(&mut ctx, job),
                Err(_) => machine = crossbeam_channel::never(),
            },
            recv(inbound) -> event => match event {
                Ok(event) => {
                    let started = Instant::now();
                    busy.start();
                    if panic_report::guarded(|| handle_inbound(&mut ctx, event)).is_err() {
                        health.panicked();
                    }
                    health.tick(started.elapsed().as_micros() as u64);
                }
                // No input engine left: stop listening for one.
                Err(_) => inbound = crossbeam_channel::never(),
            },
            default(IDLE_WAKE) => health.heartbeat(),
        } }
        health.set_queue_depth(shell.len() + machine.len() + inbound.len());
        // The watchdog covers the whole turn, not only jobs (R1 review):
        // the link, periodic tasks, publishing and protocols can hang too.
        busy.start_if_idle();

        // The LumaSession link: the master sends what changed, a member
        // starts joins and reports pending commands (decisions §21 "6b").
        if panic_report::guarded(|| lumacore_lumasession::link::after_work(&mut ctx)).is_err() {
            health.panicked();
        }

        let now = Instant::now();
        for task in &mut periodic {
            if task
                .last
                .is_none_or(|last| now.duration_since(last) >= task.every)
            {
                task.last = Some(now);
                if panic_report::guarded(|| (task.run)(&mut ctx)).is_err() {
                    health.panicked();
                    log::error!("periodic task '{}' panicked", task.name);
                }
            }
        }

        let show_moved = published_generation != Some(ctx.state.show_generation);
        if show_moved {
            published_generation = Some(ctx.state.show_generation);
            // This station applies its row of the show's station table (a
            // load, a replication, an Oops — decisions §30 "12d part 4").
            lumacore_objects::stations::apply_own_row(&mut ctx.state);
            // A renamed AssetRoot / Network moves this station's mapping.
            ctx.state.follow_renames();
            // While a Devices draft is open the rig and readers see the live
            // patch (decisions §11 "staged patch").
            let copy = publishable(&ctx.state.root.show_data);
            published.show.store(Arc::new(
                lumacore_objects::patch_draft::live_view(&ctx.state, copy),
            ));
            published.draft.store(Arc::new(
                ctx.state.patch_draft.as_ref().map(|d| (d.owner.clone(), ctx.state.root.show_data.devices.clone(), d.marks.clone(), d.review.as_ref().and_then(|r| serde_json::to_value(r).ok()))),
            ));
            revs.show += 1;
        }
        // Protocol runtimes follow the show, the station's network mapping
        // and its session role (decisions §11); NICs are rechecked each second.
        if show_moved
            || ctx.state.protocols_dirty
            || now.duration_since(protocols_checked) >= PROTOCOLS_EVERY
        {
            protocols_checked = now;
            // A station setting changed (a NIC's services, the network map):
            // the runtime view (Menu → Network) and the station view follow
            // at once, not at the next discovery beacon (decisions §30 "12d").
            if std::mem::take(&mut ctx.state.protocols_dirty) {
                ctx.refresh_runtime();
                ctx.publish_station_now = true;
            }
            ctx.services.store(Arc::new(lumacore_lumasession::ServiceAccess::compute(
                &ctx.state.root.station_settings,
                &lumacore_net::nics::list(),
            )));
            if let Some(mut p) = ctx.protocols.take() {
                if panic_report::guarded(|| p.reconcile(&mut ctx.state)).is_err() {
                    health.panicked();
                }
                ctx.protocols = Some(p);
            }
        }
        // The engine renders the published show with the users' modes; a
        // mode change alone (Freeze, Blind, Solo, login) republishes too.
        // The compiler thread turns it into the next render plan (§24).
        let input = ctx.state.engine_input(published.show.load_full());
        let last = published.render.input.load();
        if show_moved
            || last.users != input.users
            || last.identify != input.identify
            || last.patch_generation != input.patch_generation
        {
            ctx.state.engine_generation += 1;
            published.render.publish(lumacore_render::EngineInput {
                generation: ctx.state.engine_generation,
                ..input
            });
        }
        // Overload and a stopped engine go to the System Monitor once per
        // episode (the engine itself never logs).
        if now.duration_since(render_checked) >= RENDER_CHECK_EVERY {
            render_checked = now;
            render_watch.check(&published.render.stats, &mut ctx.state.sysmon_pending);
            clock_watch.follow(&ctx.state);
        }
        if show_moved
            || std::mem::take(&mut ctx.publish_station_now)
            // A login consented (or a token began to wait): HTTP sees it at once (decisions 0290).
            || ctx.state.consent.take_changed()
            || now.duration_since(station_published) >= STATION_VIEW_EVERY
        {
            station_published = now;
            let view = station_view(&ctx, &published.feeds);
            let before = published.station.load();
            let mv = |moved: bool, rev: &mut u64| *rev += u64::from(moved);
            mv(view.station != before.station, &mut revs.station);
            mv(view.show_info != before.show_info || view.backup != before.backup, &mut revs.show_info);
            mv(view.claims != before.claims, &mut revs.claims);
            mv(view.network != before.network, &mut revs.network);
            mv(view.lumasessions != before.lumasessions, &mut revs.lumasessions);
            mv(
                view.revoked_tokens != before.revoked_tokens || view.consent_tokens != before.consent_tokens,
                &mut revs.tokens,
            );
            mv(view.mode_masters != before.mode_masters, &mut revs.modes);
            mv(view.undo != before.undo, &mut revs.undo);
            mv(view.clipboards != before.clipboards, &mut revs.clipboard);
            drop(before);
            published.station.store(Arc::new(view));
        }
        // A show saved, deleted or renamed: the Backup list follows.
        let writes = lumacore_luma::drive::writes();
        if writes != library_writes {
            library_writes = writes;
            revs.library += 1;
        }
        if revs != told {
            told = revs;
            published.feeds.changed(&revs);
        }
        busy.end();

        if ctx.shutdown_requested {
            break;
        }
    }
    ctx
}

/// An event from an input engine (OSC, …) runs like any other command, as
/// the input's user.
fn handle_inbound(ctx: &mut ServerContext, event: InboundEvent) {
    match event {
        // A command that names no user never runs (it would run as nobody —
        // never as Admin, decisions §30 "12d"); input lines tag theirs.
        InboundEvent::Command(cmd) => {
            ctx.state.sysmon_pending.push((
                "Command".to_string(),
                format!(
                    "refused an input command with no user: '{}' [E-RIGHTS-003]",
                    lumacore_objects::passwords::mask_secrets(&cmd)
                ),
            ));
        }
        InboundEvent::UserCommand { user, command } => {
            log::info!(
                "inbound event: '{}' as {user}",
                lumacore_objects::passwords::mask_secrets(&command)
            );
            let (_reply, should_exit) = crate::net::handler::execute_remote_command(ctx, &user, &command);
            if should_exit {
                ctx.shutdown_requested = true;
            }
        }
        // Not built yet (decisions §29): said once in the System Monitor.
        InboundEvent::AttributeSet { .. } => planned_once(
            &ATTRIBUTE_SET_SAID,
            "OSC /lumacore/fixture/<id>/<attribute> isn't built yet (planned) — use /lumacore/cmd",
        ),
        InboundEvent::TransportControl(_) => planned_once(
            &TRANSPORT_SAID,
            "OSC transport without a line user isn't built yet (planned)",
        ),
    }
}

static ATTRIBUTE_SET_SAID: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static TRANSPORT_SAID: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Note a planned inbound event in the System Monitor the first time only.
fn planned_once(said: &std::sync::atomic::AtomicBool, message: &str) {
    if !said.swap(true, std::sync::atomic::Ordering::Relaxed) {
        lumacore_sysmon::sysmon_note("OSC", format!("[WARN] {message}"));
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
