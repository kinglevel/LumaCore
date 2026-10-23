//! lumacore-http: HTTP engine for LumaCore
//!
//! Provides:
//! - REST endpoints for diagnostics, health, metrics, views, and panel layout
//! - `/ws/feeds`: every live feed to a page on one socket (`websocket::feeds`;
//!   no SSE stream and no poll left, decisions §30 "12d hardening"), and
//!   `/ws/shell`
//! - Static file serving for the web UI

pub mod auth;
pub mod endpoints;
pub mod errors;
pub mod layout;
pub mod router;
pub mod state;
pub mod websocket;

#[cfg(test)]
mod tests;

use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;

use arc_swap::ArcSwap;
pub use layout::{LayoutEvent, ScreenEvents};
use state::DiagnosticsSnapshot;

/// A saved screen layout: its split tree and its windows by number.
pub type ScreenLayout = (
    lumacore_tree::runtime::LayoutNode,
    std::collections::BTreeMap<u32, lumacore_tree::runtime::Window>,
);

/// This server run's id: a feed's `epoch` (decisions §30 "12b2 hardening").
/// A client that reconnects to another run starts its samples over.
/// The HTTP status of a refused request: 503 when the station was busy and
/// the line didn't run (E-CMD-017, R1-67) — try again — else 400.
pub(crate) fn status_of(message: &str) -> axum::http::StatusCode {
    if message.contains("[E-CMD-017]") {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    } else {
        axum::http::StatusCode::BAD_REQUEST
    }
}

pub fn epoch() -> &'static str {
    static EPOCH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    EPOCH.get_or_init(|| {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as u64);
        format!("{ms:x}{:x}", std::process::id())
    })
}

/// One universe as the DMX window shows it (the `dmx` feed), from the playback
/// engine's published output.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DmxUniverse {
    pub universe: u32,
    /// The final output (DMX input merged in).
    pub values: Vec<u8>,
    /// Our own output, before input merges in.
    pub ours: Vec<u8>,
    /// What the input lines bring in (raw), if anything.
    pub input: Option<Vec<u8>>,
    /// Who won each channel: 0 unpatched, 1 default, 2 ours, 3 park, 4 input.
    pub winners: Vec<u8>,
    /// The input line that won each channel (0 = none).
    pub lines: Vec<u32>,
}

/// A fixture's channels in a universe (DMX window cells).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DmxPatchSpan {
    pub fixture: u32,
    pub name: String,
    /// First DMX address (1-based).
    pub start: u16,
    /// The attribute of each channel from `start` ("" = none).
    pub attributes: Vec<String>,
}

/// The station's own counters (decisions §30 "12d hardening": shared builds
/// are rebuilt when these move, never on a polling timer). Each moves when
/// what it names was published changed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Revisions {
    /// The published show (`AppState::show_generation`).
    pub show: u64,
    /// `GET /api/station` (users, presence, profiles, status line).
    pub station: u64,
    /// The loaded show's status (`ShowStatus`: name, unsaved, autosave).
    pub show_info: u64,
    /// Claimed objects (pool windows' lock).
    pub claims: u64,
    /// Menu → Network / DMX Protocols (built only while asked for).
    pub network: u64,
    /// Menu → LumaSessions (built only while asked for).
    pub lumasessions: u64,
    /// The show library on disk (a show saved, deleted, renamed).
    pub library: u64,
    /// Login tokens ended by a logout.
    pub tokens: u64,
    /// The users' own Highlight / Solo masters (the encoder bar's Masters mode, §30 "12f").
    pub modes: u64,
    /// The users' Oops lists (the Oops window, §30 "12f").
    pub undo: u64,
    /// The users' clipboards (Paste counts what it holds, §34 round 3, 18).
    pub clipboard: u64,
}

/// Access to the console for a UI engine: run commands as a user and read
/// per-user show data. The HTTP engine holds only a snapshot of ShowData, so
/// anything that must land in the show (stored views, view buttons) goes
/// through real commands — the same ones the terminal uses.
pub trait ConsoleBridge: Send + Sync {
    /// Execute a command line as `user` from `device` (the page's device id,
    /// if it named one: a locked device's lines are refused, E-LOCK-001).
    /// `Ok` carries the (plain-text) output.
    fn execute(&self, user: &str, device: Option<&str>, cmd: &str) -> Result<String, String>;
    /// The user's stored views and view buttons:
    /// `{"views": [{no, name, screens, windows}], "buttons": {"<slot>": <view no>}}`.
    fn user_views(&self, user: &str) -> serde_json::Value;
    /// The show library for the Backup menu:
    /// `{"current": {name, dirty, description, …}, "shows": [{name, size, modified}]}`.
    fn shows(&self) -> serde_json::Value;
    /// The `shows` feed (decisions §30 "12d part 5"): the library, the
    /// history of `history` (default the loaded show), recovery copies, use
    /// and free disk.
    fn shows_for(&self, history: Option<&str>, drive: Option<&str>) -> serde_json::Value {
        let _ = (history, drive);
        self.shows()
    }
    /// The Load show dialog's facts and parts of a show file, a version or
    /// a recovery copy (`GET /api/show-parts`), from its manifest only.
    fn show_parts(&self, show: &str, version: Option<&str>, recovery: Option<&str>, _drive: Option<&str>) -> serde_json::Value {
        let _ = (show, version, recovery);
        serde_json::Value::Null
    }
    /// Check a login (`remote` = not from this machine) and issue a token:
    /// `(user, token)`.
    /// `from` = the client's address, for sysmon.
    fn login(
        &self,
        user: &str,
        password: Option<&str>,
        remote: bool,
        from: &str,
    ) -> Result<(String, String), String>;
    /// The user a login token belongs to, if valid (a login still waiting for the Meow License is
    /// none yet, decisions 0290).
    fn verify_token(&self, token: &str) -> Option<String>;
    /// The user of a valid token whose login waits for the Meow License (decisions 0290): the
    /// page asks its consent; nothing else takes it.
    fn pending_consent(&self, token: &str) -> Option<String> {
        let _ = token;
        None
    }
    /// End a login token (logout); `from` = the client's address, for sysmon.
    fn logout(&self, token: &str, from: &str);
    /// Station info and login users for `GET /api/station`.
    fn station(&self) -> serde_json::Value;
    /// Unlock `device` for the logged-in `user` with their password (a user
    /// without one needs none; Admin's is `admin`) — decisions §30 "12a3".
    /// Why a request from `device` may change nothing (it is DeskLocked:
    /// the coded E-LOCK-001 reply), or None (decisions §30 "12a3").
    /// Err: the station couldn't tell (busy) — the request is refused, never
    /// let through unchecked.
    fn device_lock_refusal(&self, device: &str) -> Result<Option<String>, String> {
        let _ = device;
        Ok(None)
    }
    fn unlock(&self, user: &str, device: &str, password: Option<&str>, from: &str) -> Result<(), String> {
        let _ = (user, device, password, from);
        Err("not supported".into())
    }
    /// A picture from the station's store by SHA-256: (bytes, MIME type).
    fn media(&self, sha256: &str) -> Option<(Vec<u8>, String)> {
        let _ = sha256;
        None
    }
    /// Put an upload into `library/media/` (as `user`); returns its name.
    fn save_media(&self, user: &str, name: &str, bytes: Vec<u8>) -> Result<String, String> {
        let _ = (user, name, bytes);
        Err("not supported".into())
    }
    /// The look of the user's profile for `GET /api/me`: `{theme, lock_screen}`.
    fn look(&self, user: &str) -> serde_json::Value {
        let _ = user;
        serde_json::Value::Null
    }
    /// The logged-in user's screen configs for `GET /api/me`:
    /// `{"default": n, "configs": [{no, name, screens: [n…]}]}`.
    fn screen_configs(&self, user: &str) -> serde_json::Value;
    /// System Monitor history, oldest first, one JSON object per line
    /// (live lines arrive on `HttpState::sysmon_tx` in the same form).
    fn sysmon_history(&self) -> Vec<String>;
    /// UI keys this user can use and their gesture timing:
    /// `{double_click_ms, hold_ms, keys: [{id, label, icon, tooltip, owner,
    /// press, double_press, hold, double_replaces_press, state, locked}]}`.
    fn ui_keys(&self, user: &str) -> serde_json::Value;
    /// Running progress and statuses that are on, for `user` (JSON sysmon
    /// lines, as the `feedback` feed sends them).
    fn feedback_active(&self, user: &str) -> Vec<String>;
    /// The user's notification settings: `{mode: "timed"|"permanent",
    /// timed_ms, surfaces: {"<category>.<level>": "none"|"card"|"popup"}}`.
    fn feedback_settings(&self, user: &str) -> serde_json::Value;
    /// The screen config to use (`config`, else the user's default) and the
    /// saved layout of `screen` in it, if the show has one.
    fn screen_layout(
        &self,
        user: &str,
        config: Option<u32>,
        screen: u32,
    ) -> (u32, Option<ScreenLayout>);
    /// A pool window's content for `user` (decisions §22): `{pool, window,
    /// actions, datapool, cells}`; `pool` is an object keyword or window
    /// type, `datapool` None = the user's. None when there is no such pool.
    /// A live screen's windows as their clients see them (the `windows`
    /// feed, decisions §30 "12c"): `{screen, windows: {no: {type, title,
    /// bar, items, states, chips, values, keys, preset}}}`.
    fn windows(&self, _user: &str, screen: &lumacore_tree::runtime::Screen) -> serde_json::Value {
        serde_json::json!({ "screen": screen.no, "windows": {} })
    }

    /// A profile screen's view bar for this user (the `viewbar` feed, decisions 0287).
    fn view_bar(&self, _user: &str, screen: u32) -> serde_json::Value {
        serde_json::json!({ "screen": screen, "buttons": [] })
    }

    /// This user's Oops list and the profile's Oops filters (the `oops` feed, §30 "12f").
    fn oops(&self, _user: &str) -> serde_json::Value {
        serde_json::json!({ "steps": [], "undone": [] })
    }

    /// A sequence sheet (the `sequence` feed, decisions §30 "12f").
    fn sequence_sheet(&self, _user: &str, _sequence: &serde_json::Value) -> serde_json::Value {
        serde_json::json!({ "missing": true })
    }

    /// The Patch setup dialog's rows for a user: the draft for its holder, else the live devices
    /// (the `patch` feed, decisions §34 "34g4 patch hardened" 9).
    fn patch(&self, _user: &str) -> serde_json::Value {
        serde_json::json!({ "mode": "live", "fixtures": [], "tree": [] })
    }

    /// The Tags pool window (the `tags` feed, decisions §12 "Tags hardened").
    fn tags(&self, _user: &str, _tag: &serde_json::Value, _deep: bool) -> serde_json::Value {
        serde_json::json!({ "nodes": [], "members": [] })
    }

    /// A Tree window's rows for a user (the `tree` feed, decisions 0292): `params` are the
    /// window's `path`, `open` (a comma list), `search` and `follow`.
    fn tree_window(&self, _user: &str, _params: &serde_json::Value) -> serde_json::Value {
        serde_json::json!({ "rows": [], "path": null })
    }

    /// A Tree window's inspector of one place for a user (the `inspector` feed, decisions 0292).
    fn inspector(&self, _user: &str, path: &str) -> serde_json::Value {
        serde_json::json!({ "missing": "no station", "asked": path })
    }

    /// The paint picker's palette, a theme's slots and the user's recent colours (the `paint`
    /// feed, decisions 0294): `profile` = whose theme (a number; none = the user's).
    fn paint(&self, _user: &str, _profile: &serde_json::Value) -> serde_json::Value {
        serde_json::json!({ "slots": [], "recent": [] })
    }

    /// The Rules window's rows (the `rules` feed, decisions 0294): `profile` = whose layer (none
    /// = the show's, `mine`, a number), `which` an attribute name to look up.
    fn rules(&self, _user: &str, _profile: &serde_json::Value, _which: &str) -> serde_json::Value {
        serde_json::json!({ "rows": [] })
    }

    /// A Device Sheet for a user (the `devices` feed, decisions device-sheet-window): `params` are
    /// the window's (`layer`, `show`, `programmer_only`, `selected_only`, `selection_first`, `folders`,
    /// `from`, `count`).
    fn devices(&self, _user: &str, _params: &serde_json::Value) -> serde_json::Value {
        serde_json::json!({ "rows": [], "columns": [], "cells": {}, "total": 0 })
    }

    /// A live screen's encoder bar (the `encoder` feed, decisions §30 "12f").
    fn encoder(&self, _user: &str, screen: &lumacore_tree::runtime::Screen) -> serde_json::Value {
        serde_json::json!({ "screen": screen.no, "on": false })
    }

    /// `profile`: the Views pool's Profile setting (a GUID, else the user's own); `filter`:
    /// `{feature, tags}` the window narrows the pool to (the Presets window).
    fn pool(&self, user: &str, pool: &str, datapool: Option<u32>, profile: &serde_json::Value, filter: &serde_json::Value) -> Option<serde_json::Value> {
        let _ = (user, pool, datapool, profile, filter);
        None
    }
    /// A pool window's DataPool setting as a datapool number: null, 0 or
    /// "Selected" = the user's selected one (None here), a number, or a
    /// datapool's GUID string (decisions §30 "12b2").
    fn resolve_datapool(&self, user: &str, datapool: &serde_json::Value) -> Option<u32> {
        let _ = user;
        datapool.as_u64().map(|n| n as u32).filter(|n| *n > 0)
    }
    /// One pool object's editor content (Group: members and grid).
    fn pool_object(
        &self,
        user: &str,
        pool: &str,
        datapool: Option<u32>,
        no: u32,
    ) -> Option<serde_json::Value> {
        let _ = (user, pool, datapool, no);
        None
    }
    /// An attribute board (decisions §30 "12f-a"): the show's layer
    /// (`profile` 0 / absent) or a profile's; None when it is gone.
    fn attribute_board(&self, user: &str, profile: &serde_json::Value) -> Option<serde_json::Value> {
        let _ = (user, profile);
        None
    }
    /// A Users & Profiles detail's lists (`kind` profile / user, its number).
    fn detail_view(&self, kind: &str, no: u32) -> Option<serde_json::Value> {
        let _ = (kind, no);
        None
    }
    /// A universe's live output for the DMX window; None when the show has
    /// no such universe. Reads only what the engine publishes.
    fn dmx_universe(&self, universe: u32) -> Option<DmxUniverse> {
        let _ = universe;
        None
    }
    /// `Runtime.Health` for the Realtime window (the `health` feed): render
    /// stages, lines, the session clock, threads. Atomics only.
    fn health(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
    /// The render engine's numbers, with its per-frame ring (the
    /// `health.render` feed reads it off the render thread: atomics only).
    fn render_stats(&self) -> Option<Arc<lumacore_tree::runtime::RenderStats>> {
        None
    }
    /// What each channel of `universe` is patched to (from the published
    /// show).
    fn dmx_patch(&self, universe: u32) -> Vec<DmxPatchSpan> {
        let _ = universe;
        Vec::new()
    }
    /// The universe of `user`'s first selected fixture (Only Selection).
    fn dmx_selected_universe(&self, user: &str) -> Option<u32> {
        let _ = user;
        None
    }
    /// Whether a connection that arrived on local address `local` may be
    /// served (Menu → Network → Interfaces: HTTP per NIC; loopback always).
    fn http_allows(&self, local: std::net::IpAddr) -> bool {
        let _ = local;
        true
    }
    /// `GET /api/network`: interfaces and services, the show's networks and
    /// this station's map, lines with live state, sources, nodes.
    fn network(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
    /// `GET /api/lumasessions`: this station's settings and every station
    /// discovery sees, members.
    fn lumasessions(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
    /// `GET /api/menu-types` (decisions §30 "12d"): the menu registry —
    /// every menu's tabs, grids, function block and power toggle. Static
    /// (the registry); a profile's last tabs and columns ride on `OpenMenu`.
    fn menu_types(&self) -> serde_json::Value {
        serde_json::Value::Array(Vec::new())
    }
    /// `GET /api/object-menus` (decisions §34 round 3, 18): the one context menu of every object,
    /// by keyword — its common actions, Set part and own lines. Static (the registry).
    fn object_menus(&self) -> serde_json::Value {
        serde_json::Value::Object(serde_json::Map::new())
    }
    /// `GET /api/fixture-types` (decisions §34 round 3): the show's fixture types (the user's
    /// draft's when they hold it) and the library's files.
    fn fixture_types(&self, _user: &str, _drive: Option<&str>) -> serde_json::Value {
        serde_json::json!({ "show": [], "library": [] })
    }
    /// `GET /api/fixture-type`: one mode's channel map and the file's details.
    fn fixture_type(&self, _user: &str, _which: &crate::endpoints::gdtf::Which) -> Option<serde_json::Value> {
        None
    }
    /// `GET /api/fixture-type/thumbnail`: the file's picture and its media type.
    fn fixture_thumbnail(&self, _user: &str, _which: &crate::endpoints::gdtf::Which) -> Option<(Vec<u8>, String)> {
        None
    }
    /// `POST /api/gdtf`: Upload a fixture file into `library/gdtf/` (Setup and up).
    fn save_gdtf(&self, _user: &str, _name: &str, _bytes: Vec<u8>) -> Result<String, String> {
        Err("Fixture files can't be uploaded here".into())
    }
    /// The `files` feed (decisions §38 5): a drive's library folder by kind —
    /// `{"drive", "folder", "files": [{name, size, modified, dir, …}]}`; `files` keyed by `name`.
    fn files(&self, drive: &str, folder: &str, kind: &str) -> serde_json::Value {
        serde_json::json!({ "drive": drive, "folder": folder, "kind": kind, "files": [] })
    }
    /// `POST /api/upload?drive=&folder=&name=` (decisions §38 5, This device): a file from the
    /// browser into a drive's library folder (Setup and up; the first write makes the tree).
    fn save_upload(&self, _user: &str, _drive: &str, _folder: &str, _name: &str, _bytes: Vec<u8>) -> Result<String, String> {
        Err("Files can't be uploaded here".into())
    }
    /// `GET /api/tool-types` (decisions §30 "12e"): the editor tools —
    /// groups, each tool's icon, question and status. Static (the registry).
    fn tool_types(&self) -> serde_json::Value {
        serde_json::json!({ "groups": [], "tools": [] })
    }
    /// The station's counters, waking every shared feed build when they move
    /// (decisions §30 "12d hardening"). The default never moves.
    fn revisions(&self) -> tokio::sync::watch::Receiver<Revisions> {
        static NEVER: std::sync::OnceLock<tokio::sync::watch::Sender<Revisions>> = std::sync::OnceLock::new();
        NEVER.get_or_init(|| tokio::sync::watch::channel(Revisions::default()).0).subscribe()
    }
    /// A feed whose data the station builds only while someone watches it
    /// (`"network"`, `"lumasessions"`) gained (`true`) or lost a watcher.
    fn feed_demand(&self, feed: &str, on: bool) {
        let _ = (feed, on);
    }
    /// `POST /api/screenshots`: `user`'s client uploads its capture `id` of
    /// live screen `screen` (decisions §29 "11d"). Ok = the saved file's path.
    fn save_screenshot(&self, user: &str, id: u64, screen: u32, png: Vec<u8>) -> Result<String, String> {
        let _ = (user, id, screen, png);
        Err("screenshots aren't supported here".into())
    }
}

/// The HTTP listener: connections that arrive on an interface that doesn't
/// serve HTTP are dropped (decisions §11).
struct FilteredListener {
    inner: tokio::net::TcpListener,
    console: Arc<dyn ConsoleBridge>,
}

impl axum::serve::Listener for FilteredListener {
    type Io = tokio::net::TcpStream;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let (io, addr) = axum::serve::Listener::accept(&mut self.inner).await;
            match io.local_addr() {
                Ok(local) if self.console.http_allows(local.ip()) => return (io, addr),
                _ => drop(io),
            }
        }
    }

    fn local_addr(&self) -> std::io::Result<Self::Addr> {
        self.inner.local_addr()
    }
}

/// Shared state passed to all HTTP handlers.
#[derive(Clone)]
pub struct HttpState {
    pub diagnostics: Arc<ArcSwap<DiagnosticsSnapshot>>,
    pub sysmon_tx: tokio::sync::broadcast::Sender<String>,
    /// `--web-root`: serve the WebUI from this folder (live edits); `None` = the embedded files
    /// (decisions §36 2).
    pub web_root: Option<std::path::PathBuf>,
    /// TCP port of the shell listener (for /ws/shell proxy).
    pub shell_port: u16,
    /// The show as the server last published it (replaced after every
    /// change). Read-only: anything that must change the show goes through
    /// `console`, as a command.
    pub show: Arc<ArcSwap<lumacore_tree::ShowData>>,
    /// The live screens' events (views, layouts, ReloadUI, capture) with
    /// their seq: the `screen` feed.
    pub screen_events: Arc<ScreenEvents>,
    /// Shared builds of the counter-driven State feeds.
    pub hub: Arc<websocket::hub::FeedHub>,
    /// Live screens (Root.Runtime) — the canonical windows this engine renders.
    pub live_screens: lumacore_tree::runtime::SharedScreens,
    /// Command execution / per-user data (stored views, view buttons).
    pub console: Arc<dyn ConsoleBridge>,
    /// Turns true when the engine shuts down: long-lived sockets (`/ws/feeds`)
    /// close then, so the graceful shutdown isn't held by an open page.
    pub stopping: tokio::sync::watch::Receiver<bool>,
}

/// Handle returned by `HttpEngine::start()` for control and shutdown.
pub struct HttpHandle {
    thread: Option<thread::JoinHandle<()>>,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

impl HttpHandle {
    /// Stop accepting connections and finish open requests.
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }

    /// The engine thread, for joining after `stop`.
    pub fn take_thread(&mut self) -> Option<thread::JoinHandle<()>> {
        self.thread.take()
    }
}

/// HTTP engine that runs axum on a dedicated tokio runtime in its own OS thread.
pub struct HttpEngine;

impl HttpEngine {
    /// Start the HTTP engine on a new OS thread with its own tokio runtime.
    ///
    /// Returns an `HttpHandle` that keeps the thread alive.
    #[allow(
        clippy::too_many_arguments,
        reason = "replaced by the ServerBuilder (decisions.md §26)"
    )]
    pub fn start(
        addr: SocketAddr,
        diagnostics: Arc<ArcSwap<DiagnosticsSnapshot>>,
        sysmon_tx: tokio::sync::broadcast::Sender<String>,
        web_root: Option<std::path::PathBuf>,
        shell_port: u16,
        show: Arc<ArcSwap<lumacore_tree::ShowData>>,
        live_screens: lumacore_tree::runtime::SharedScreens,
        console: Arc<dyn ConsoleBridge>,
    ) -> HttpHandle {
        // Every screen change (UI edits, `set` from the command line),
        // ReloadUI and Screenshot go to the pages as the `screen` feed. Runs
        // under the store's write lock, so the listener must not touch the
        // store.
        let screen_events = Arc::new(ScreenEvents::default());
        if let Ok(mut store) = live_screens.write() {
            let events = screen_events.clone();
            store.subscribe(move |vs| {
                events.push(&LayoutEvent::ScreenChanged { no: vs.no, revision: vs.revision });
            });
            let events = screen_events.clone();
            store.subscribe_views(move |_user| events.push(&LayoutEvent::ViewsChanged));
            let events = screen_events.clone();
            store.subscribe_reload(move |screens| {
                events.push(&LayoutEvent::ReloadUi { screens: screens.map(<[u32]>::to_vec) });
            });
            let events = screen_events.clone();
            store.subscribe_capture(move |request| {
                events.push(&LayoutEvent::Capture { id: request.id, screens: request.screens.clone() });
            });
        }

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let handle = thread::Builder::new()
            .name("http-engine".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .thread_name("http-worker")
                    .enable_all()
                    .build()
                    .expect("failed to build tokio runtime for HTTP engine");

                rt.block_on(async move {
                    let filter_console = console.clone();
                    let (stop_tx, stopping) = tokio::sync::watch::channel(false);
                    let http_state = HttpState {
                        stopping,
                        diagnostics,
                        sysmon_tx,
                        web_root: web_root.clone(),
                        shell_port,
                        show,
                        screen_events,
                        hub: Arc::default(),
                        live_screens,
                        console,
                    };

                    let app = router::build_router(http_state);

                    let listener = tokio::net::TcpListener::bind(addr)
                        .await
                        .unwrap_or_else(|e| {
                            panic!("HTTP engine failed to bind {addr}: {e}");
                        });

                    log::info!("HTTP engine listening on {addr}");

                    // Connect info: logins need to know whether the client is local.
                    use axum::serve::ListenerExt;
                    let listener = FilteredListener {
                        inner: listener,
                        console: filter_console,
                    }
                    .tap_io(|_| {});
                    axum::serve(
                        listener,
                        app.into_make_service_with_connect_info::<SocketAddr>(),
                    )
                    .with_graceful_shutdown(async move {
                        let _ = shutdown_rx.await;
                        let _ = stop_tx.send(true);
                    })
                    .await
                    .expect("HTTP engine serve error");
                });
            })
            .expect("failed to spawn HTTP engine thread");

        HttpHandle {
            thread: Some(handle),
            shutdown: Some(shutdown_tx),
        }
    }
}

/// This crate's rows in the catalog, after the crates it builds on (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_tree::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.errors(errors::ERRORS);
    r.feeds(websocket::feeds::FEEDS);
}

/// This crate's tests read the catalog: it is installed before the first one (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
