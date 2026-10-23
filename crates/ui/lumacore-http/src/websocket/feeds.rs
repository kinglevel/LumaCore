//! /ws/feeds — everything live in a page on one socket (decisions §30 "12b2
//! hardening": "Client feeds — one rule", "One /ws/feeds per page"; "12d
//! hardening": "Everything live on `/ws/feeds`"). See docs/dev/api/http-api.md.
//!
//! **Handshake.** The socket opens without a login; the station's first
//! message is `{"op":"hello","client_protocol","epoch","user","feeds":[{name,
//! kind,open,params}],"max_subs"}` (the feed list is [`FEEDS`]). Feeds marked
//! `open` (the station view: users, presence, show status — the lock screen)
//! work before a login. `{"op":"auth","token"}` logs the socket in (a cookie
//! or `?token=` on the upgrade does too) and answers `{"op":"authed","user"}`;
//! every subscription stays. `{"op":"auth","token":null}`, a logout or a
//! revoked token ends the feeds that need a login with `nosub` E-FEED-002.
//!
//! **Subscriptions.** `{"op":"sub","id":3,"feed":"pools",…params,"since"?,
//! "epoch"?,"rev"?,"period_ms"?}` / `{"op":"unsub","id":3}` (a `sub` with an
//! id in use replaces it). Every `sub` is answered `{"op":"subbed",id,feed,
//! kind}` or `{"op":"nosub",id,code,message}` with an `E-FEED-…` code
//! ([`crate::errors`]): unknown feed, not before a login, bad params, more
//! than [`MAX_SUBS`]. The send period is clamped to ≥ 16 ms; the sample rate
//! never changes with it.
//!
//! Every feed sends **one batch when it is subscribed** (the state, or the
//! history), **then changes in batches per period** — never one item per
//! message; batches due in one tick go as **one frame** (a JSON array). Two
//! kinds:
//! - **State** — latest wins: a slow client gets the next newest. Counter
//!   driven ones (`station`, `network`, `lumasessions`, `shows`, `pools`,
//!   `pool_editor`, `windows`) are shared builds ([`super::hub`]) rebuilt when
//!   the station's counters move; every batch carries a `rev` (a fingerprint
//!   of the data); a `sub` with the `rev` it has gets `unchanged: true`;
//!   keyed lists may send `diff: {fields, set, del, order?}` against `base`.
//!   Sampled ones (`health`, `dmx`) are read each period.
//! - **Samples** (`health.render`, `sysmon`, `feedback`, `screen`): every item
//!   has a `seq`; each batch carries every item since the last one sent; what
//!   the station no longer holds arrives as an explicit `gap: {from, to}`. A
//!   `since` from this run (`epoch`) resumes after it; one older than the
//!   ring gets `recovered: false` and the whole ring.
//!
//! One envelope: `{feed, kind: "state"|"samples", id, epoch, from_seq,
//! to_seq, gap, rev?, …}` with `cols` (columnar samples), `items` (lines,
//! screen events), `data` / `diff` (a state). DMX values are binary frames
//! (see [`dmx_frame`]).
//!
//! **Limits** (Home Assistant's): a send stalled for more than [`STALL`], a
//! queue of [`QUEUE_CAP`] messages, or a queue over [`QUEUE_PEAK`] for
//! [`PEAK_TIME`] closes the socket (4008, E-FEED-005); a State waits while
//! the queue is behind (only its newest goes). A ping every [`PING_EVERY`];
//! a page silent for [`SILENT`] is gone. The page reconnects (jittered) and
//! resumes with `since` / `rev`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::http::{HeaderMap, Uri};
use axum::response::IntoResponse;
use futures_util::stream::{BoxStream, SelectAll, SplitSink};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::sync::broadcast::{self, error::RecvError};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::Instant;
use tokio_stream::wrappers::WatchStream;

use lumacore_registry::{FeedDecl, FeedKind};

use super::hub::{BuildSpec, Built, SharedBuild};
use crate::errors;
use crate::{DmxUniverse, HttpState};

/// The shortest send period (one 60 Hz frame).
pub const MIN_PERIOD: Duration = Duration::from_millis(16);
/// A send that takes longer closes the socket.
pub const STALL: Duration = Duration::from_secs(2);
/// Subscriptions one socket may hold (`nosub` E-FEED-004 beyond).
pub const MAX_SUBS: usize = 64;
/// Messages queued for a page at most; one more closes it (E-FEED-005).
pub const QUEUE_CAP: usize = 256;
/// A queue this long for [`PEAK_TIME`] closes the page (E-FEED-005); State
/// batches wait while the queue holds half of it.
pub const QUEUE_PEAK: usize = 64;
pub const PEAK_TIME: Duration = Duration::from_secs(5);
/// The station pings an idle page this often …
pub const PING_EVERY: Duration = Duration::from_secs(15);
/// … and drops one it heard nothing from for this long (a half-open phone).
pub const SILENT: Duration = Duration::from_secs(45);
/// The close code for a page that can't keep up.
pub const CLOSE_SLOW: u16 = 4008;

/// The feeds a page can subscribe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feed {
    Render,
    Health,
    Sysmon,
    Feedback,
    Dmx,
    Pools,
    Windows,
    Station,
    Network,
    LumaSessions,
    Shows,
    /// A drive's library folder by kind (decisions §38 5): the files element's rows.
    Files,
    Screen,
    PoolEditor,
    Attributes,
    Detail,
    Encoder,
    Oops,
    Sequence,
    Tags,
    Patch,
    /// A Device Sheet (decisions device-sheet-window): sampled per subscription.
    Devices,
    /// A live screen's view bar (decisions 0287).
    ViewBar,
    /// A Tree window's rows (decisions 0292).
    Tree,
    /// A Tree window's inspector (decisions 0292).
    Inspector,
}

/// Every feed a page can subscribe, once — the HTTP engine's rows in the catalog (decisions §34
/// 21); [`Feed::name`] binds each to its builder.
pub const FEEDS: &[FeedDecl] = &[
    FeedDecl {
        name: "health.render",
        kind: FeedKind::Samples,
        open: false,
        params: &[],
        period_ms: 50,
        doc: "Every render frame's numbers (columnar, from the engine's ring).",
    },
    FeedDecl {
        name: "health",
        kind: FeedKind::State,
        open: false,
        params: &[],
        period_ms: 250,
        doc: "Runtime.Health: lines, clock, threads, render totals (sampled).",
    },
    FeedDecl {
        name: "sysmon",
        kind: FeedKind::Samples,
        open: false,
        params: &[],
        period_ms: 50,
        doc: "System Monitor lines: the whole history, then live lines.",
    },
    FeedDecl {
        name: "feedback",
        kind: FeedKind::Samples,
        open: false,
        params: &[],
        period_ms: 50,
        doc: "The user's feedback events (and the station-wide ones).",
    },
    FeedDecl {
        name: "dmx",
        kind: FeedKind::State,
        open: false,
        params: &["universes", "universe", "follow_selection"],
        period_ms: 16,
        doc: "DMX universes: the patch as JSON, values as binary frames (sampled).",
    },
    FeedDecl {
        name: "pools",
        kind: FeedKind::State,
        open: false,
        params: &["pool", "datapool", "profile", "feature", "tags"],
        period_ms: 100,
        doc: "A pool window's cells (keyed by `no`).",
    },
    FeedDecl {
        name: "viewbar",
        kind: FeedKind::State,
        open: false,
        params: &["screen"],
        period_ms: 100,
        doc: "A live screen's view bar: each button's slot and what it holds (kind, number, name), from the user's own profile.",
    },
    FeedDecl {
        name: "windows",
        kind: FeedKind::State,
        open: false,
        params: &["screen"],
        period_ms: 100,
        doc: "A live screen's windows: live titles, bar states, chips, key LEDs (Self).",
    },
    FeedDecl {
        name: "station",
        kind: FeedKind::State,
        open: true,
        params: &[],
        period_ms: 100,
        doc: "GET /api/station live: users with presence and look, profiles, the status line.",
    },
    FeedDecl {
        name: "network",
        kind: FeedKind::State,
        open: false,
        params: &[],
        period_ms: 100,
        doc: "Menu → Network / DMX Protocols: interfaces, networks, lines (built only while watched).",
    },
    FeedDecl {
        name: "lumasessions",
        kind: FeedKind::State,
        open: false,
        params: &[],
        period_ms: 100,
        doc: "Menu → LumaSessions: this station, the stations discovery sees, members (built only while watched).",
    },
    FeedDecl {
        name: "files",
        kind: FeedKind::State,
        open: false,
        params: &["drive", "folder", "kind"],
        period_ms: 250,
        doc: "A drive's library folder (decisions §38 5): `files` keyed by `name` — folders first, then the kind's files (shows: .luma with format / saved by / versions; gdtf; mvr; media; userprofiles) with size and modified; rebuilt when the drives or the library's writes move.",
    },
    FeedDecl {
        name: "shows",
        kind: FeedKind::State,
        open: false,
        params: &["history", "drive"],
        period_ms: 100,
        doc: "Backup: the loaded show's status and facts, the show library (keyed by `name`), the history of `history` (default the loaded show), recovery copies, use and free disk.",
    },
    FeedDecl {
        name: "screen",
        kind: FeedKind::Samples,
        open: false,
        params: &["screen"],
        period_ms: 16,
        doc: "Live screen events: views / screen changed, ReloadUI, Screenshot capture. The sub is the screen's presence.",
    },
    FeedDecl {
        name: "pool_editor",
        kind: FeedKind::State,
        open: false,
        params: &["pool", "no", "datapool"],
        period_ms: 100,
        doc: "One pool object's editor content (Group: members keyed by `fixture`, grid).",
    },
    FeedDecl {
        name: "attributes",
        kind: FeedKind::State,
        open: false,
        params: &["profile"],
        period_ms: 100,
        doc: "An attribute board: the show's feature groups (no `profile`) or a profile's own groups, every attribute placed (with why, new, used, fixture types).",
    },
    FeedDecl {
        name: "encoder",
        kind: FeedKind::State,
        open: false,
        params: &["screen"],
        period_ms: 50,
        doc: "A live screen's encoder bar (decisions §30 \"12f\"): mode, layer, the strip and the four encoders (Attributes) or the masters, with the lines each gesture sends.",
    },
    FeedDecl {
        name: "oops",
        kind: FeedKind::State,
        open: false,
        params: &[],
        period_ms: 100,
        doc: "Your Oops list (decisions §30 \"12f\"): `steps` oldest first (line, category, station, time_ms, main), `undone` (next redo first), your profile's number and Oops `filters`.",
    },
    FeedDecl {
        name: "sequence",
        kind: FeedKind::State,
        open: false,
        params: &["sequence"],
        period_ms: 100,
        doc: "A sequence sheet (decisions §30 \"12f\"): `sequence` = \"selected\" or a GUID; the cues (CueZero first, OffCue last) with their timing, each feature group's own Fade / Delay / Path, the line each edit extends.",
    },
    FeedDecl {
        name: "tags",
        kind: FeedKind::State,
        open: false,
        params: &["tag", "deep"],
        period_ms: 100,
        doc: "The Tags pool window (decisions §12 \"Tags hardened\"): the tag tree (path, name, colour, count, Action badge) and the picked `tag`'s (a GUID) members, `deep` = through the tags under it too, with the lines its pane runs.",
    },
    FeedDecl {
        name: "patch",
        kind: FeedKind::State,
        open: false,
        params: &[],
        period_ms: 100,
        doc: "The Patch setup dialog (decisions §34 \"34g4 patch hardened\" 9, 12): `mode` (draft for the holder of the Devices draft, else live) and `holder`; the fixtures (no, guid, name, note, fixture type, mode, address, footprint, pos / rot, master react, tags, MVR group), the MVR tree (layers and groups with counts), classes, universes and fixture types; `clipboard` (`{object, count, cut}`, what your Paste would place).",
    },
    FeedDecl {
        name: "devices",
        kind: FeedKind::State,
        open: false,
        params: &["layer", "show", "programmer_only", "selected_only", "selection_first", "folders", "from", "count"],
        period_ms: 100,
        doc: "A Device Sheet (decisions device-sheet-window, device-grid-and-value-marks): every device in scope in tree order with its folders, the attributes in scope as columns by feature group, and for the rows `from`..`from + count` each value's readout, state, layers and source — a value from a preset carries the preset (`p`: number, name) and its text follows `show` (Value / Link / Both, decisions 0293); sampled, sent when it changes.",
    },
    FeedDecl {
        name: "tree",
        kind: FeedKind::State,
        open: false,
        params: &["path", "open", "search", "follow"],
        period_ms: 100,
        doc: "A Tree window (decisions 0292): the rows of its open branches below Root from the one row builder `ls` and NAV read (`rows` keyed by `path`: depth, slot, name, kind, meta, open, the user's bookmark letters), `path` = the selected place (`path`: a tree path or a GUID, its parents open by themselves; `missing` says why it isn't one), NAV's filter (`search`; `hidden` counts what it hid), `open` the branches opened (comma list), `follow` the object Follow = Selected jumps to (`{path, key}`).",
    },
    FeedDecl {
        name: "inspector",
        kind: FeedKind::State,
        open: false,
        params: &["path"],
        period_ms: 100,
        doc: "A Tree window's inspector (decisions 0292): NAV's inspector of `path` (a tree path or a GUID, whole or a prefix): title, target, GUID, path, the user's bookmark letters and the rows (identity, facts, properties with their settings cell `field`, links with the path they lead to); `missing` says why a place isn't one.",
    },
    FeedDecl {
        name: "detail",
        kind: FeedKind::State,
        open: false,
        params: &["kind", "no"],
        period_ms: 100,
        doc: "A Users & Profiles detail's lists: a profile's Content (keys, window presets, images, icons, screens) or a user's work and usage.",
    },
];

impl Feed {
    pub const ALL: [Feed; 25] = [
        Feed::Render,
        Feed::Health,
        Feed::Sysmon,
        Feed::Feedback,
        Feed::Dmx,
        Feed::Pools,
        Feed::Windows,
        Feed::Station,
        Feed::Network,
        Feed::LumaSessions,
        Feed::Shows,
        Feed::Files,
        Feed::Screen,
        Feed::PoolEditor,
        Feed::Attributes,
        Feed::Detail,
        Feed::Encoder,
        Feed::Oops,
        Feed::Sequence,
        Feed::Tags,
        Feed::Patch,
        Feed::Devices,
        Feed::ViewBar,
        Feed::Tree,
        Feed::Inspector,
    ];

    /// Its declaration in the catalog.
    pub fn def(self) -> &'static FeedDecl {
        lumacore_registry::get().feed(self.name()).expect("every feed is in the catalog (lumacore_http::register)")
    }

    /// Its name on the socket: the binding of a catalog row to its builder.
    pub fn name(self) -> &'static str {
        match self {
            Feed::Render => "health.render",
            Feed::Health => "health",
            Feed::Sysmon => "sysmon",
            Feed::Feedback => "feedback",
            Feed::Dmx => "dmx",
            Feed::Pools => "pools",
            Feed::Windows => "windows",
            Feed::Station => "station",
            Feed::Network => "network",
            Feed::LumaSessions => "lumasessions",
            Feed::Shows => "shows",
            Feed::Files => "files",
            Feed::Screen => "screen",
            Feed::PoolEditor => "pool_editor",
            Feed::Attributes => "attributes",
            Feed::Detail => "detail",
            Feed::Encoder => "encoder",
            Feed::Oops => "oops",
            Feed::Sequence => "sequence",
            Feed::Tags => "tags",
            Feed::Patch => "patch",
            Feed::Devices => "devices",
            Feed::ViewBar => "viewbar",
            Feed::Tree => "tree",
            Feed::Inspector => "inspector",
        }
    }

    pub fn parse(name: &str) -> Option<Feed> {
        Feed::ALL.into_iter().find(|f| f.name() == name).filter(|f| lumacore_registry::get().feed(f.name()).is_some())
    }

    pub fn kind(self) -> &'static str {
        self.def().kind.label()
    }

    /// The period when the page asks for none.
    pub fn default_period(self) -> Duration {
        Duration::from_millis(self.def().period_ms)
    }
}

/// The station's first message on a feed socket.
pub fn hello(user: Option<&str>) -> Value {
    let feeds: Vec<Value> = lumacore_registry::get()
        .feeds()
        .map(|d| json!({ "name": d.name, "kind": d.kind.label(), "open": d.open, "params": d.params }))
        .collect();
    json!({
        "op": "hello",
        "client_protocol": lumacore_wire::CLIENT_PROTOCOL,
        "epoch": crate::epoch(),
        "user": user,
        "feeds": feeds,
        "max_subs": MAX_SUBS,
    })
}

/// The upgrade needs no login; a token on it (cookie, `?token=`) logs the
/// socket in at once.
pub async fn ws_feeds(
    ws: WebSocketUpgrade,
    State(state): State<HttpState>,
    headers: HeaderMap,
    uri: Uri,
) -> impl IntoResponse {
    let token = crate::auth::token_from_request(&headers, &uri);
    let login = token
        .as_deref()
        .and_then(|t| state.console.verify_token(t))
        .zip(token);
    ws.on_upgrade(move |socket| run(socket, state, login))
}

/// What a subscription remembers between batches.
enum Track {
    Render {
        last: u64,
    },
    Health {
        last: Option<Value>,
    },
    /// Sysmon or feedback lines (parsed JSON), queued until the next batch.
    Lines {
        /// The highest seq queued or sent.
        last: u64,
        pending: Vec<Value>,
        /// The broadcast lagged after this seq: refill from the history.
        lag_from: Option<u64>,
    },
    Dmx {
        universes: Vec<u32>,
        follow: bool,
        /// The show the last patch was sent for.
        show: Option<usize>,
        last: BTreeMap<u32, DmxUniverse>,
    },
    /// A counter-driven State: a shared build.
    Shared {
        _build: Arc<SharedBuild>,
        rx: watch::Receiver<Option<Arc<Built>>>,
        /// The rev this page has.
        sent: Option<u64>,
        /// The rev the page said it has (`sub` with `rev`).
        resume: Option<u64>,
    },
    /// A Device Sheet: built each period for the window's rows, sent when it changed.
    Devices {
        /// The window's parameters as the page sent them.
        params: Value,
        /// A fingerprint of the last data sent.
        last: Option<u64>,
    },
    /// Screen events (samples); the sub counts as a viewer of `screen`.
    Screen {
        last: u64,
        _viewer: Option<Viewer>,
    },
}

struct Sub {
    feed: Feed,
    /// The `sub` as the page sent it (made again when the login changes).
    request: Value,
    period: Duration,
    due: Instant,
    /// Batches sent (a state's seq).
    sent: u64,
    /// For the first batch: `since` was given and could (not) be served.
    recovered: Option<bool>,
    /// Sends only when woken (shared builds, screen events), not each period.
    woken: bool,
    dirty: bool,
    /// Its wake stream's key; dropping `_cancel` ends that stream.
    key: u64,
    _cancel: Option<oneshot::Sender<()>>,
    track: Track,
}

/// Counts a page as a viewer of its live screen while its `screen` feed
/// lives (who a `Screenshot` asks, decisions §29 "11d").
struct Viewer {
    screens: lumacore_tree::runtime::SharedScreens,
    no: u32,
}

impl Viewer {
    fn new(screens: lumacore_tree::runtime::SharedScreens, no: u32) -> Self {
        if let Ok(mut store) = screens.write() {
            store.viewer(no, true);
        }
        Self { screens, no }
    }
}

impl Drop for Viewer {
    fn drop(&mut self) {
        if let Ok(mut store) = self.screens.write() {
            store.viewer(self.no, false);
        }
    }
}

/// One message to the page.
enum Out {
    Text(String),
    Binary(Vec<u8>),
}

/// One feed socket.
struct Conn {
    state: HttpState,
    /// The logged-in user and their token.
    login: Option<(String, String)>,
    subs: BTreeMap<u64, Sub>,
    lines: Option<broadcast::Receiver<String>>,
    /// Wake-ups of shared builds and screen events, by sub key.
    wakes: SelectAll<BoxStream<'static, u64>>,
    next_key: u64,
    /// Control replies (hello, subbed, nosub, authed, error), sent first.
    replies: Vec<Value>,
}

async fn next_line(rx: &mut Option<broadcast::Receiver<String>>) -> Result<String, RecvError> {
    match rx {
        Some(rx) => rx.recv().await,
        None => std::future::pending().await,
    }
}

/// The socket's writer: one queue, a send stalled for [`STALL`] ends it.
async fn write(
    mut sink: SplitSink<WebSocket, Message>,
    mut queue: mpsc::Receiver<Message>,
    mut close: oneshot::Receiver<CloseFrame>,
) {
    loop {
        tokio::select! {
            biased;
            frame = &mut close => {
                if let Ok(frame) = frame {
                    let _ = tokio::time::timeout(STALL, sink.send(Message::Close(Some(frame)))).await;
                }
                break;
            }
            msg = queue.recv() => match msg {
                Some(msg) => {
                    if !matches!(tokio::time::timeout(STALL, sink.send(msg)).await, Ok(Ok(()))) {
                        break;
                    }
                }
                None => break,
            },
        }
    }
}

async fn run(socket: WebSocket, state: HttpState, login: Option<(String, String)>) {
    let (sink, mut stream) = socket.split();
    let (out, queue) = mpsc::channel::<Message>(QUEUE_CAP);
    let (close_tx, close_rx) = oneshot::channel::<CloseFrame>();
    tokio::spawn(write(sink, queue, close_rx));
    let mut close_tx = Some(close_tx);
    let mut close = |code: u16, reason: String| {
        if let Some(tx) = close_tx.take() {
            let _ = tx.send(CloseFrame { code, reason: reason.into() });
        }
    };

    let mut stopping = state.stopping.clone();
    let mut revs = state.console.revisions();
    let mut revs_alive = true;
    let mut tokens = revs.borrow_and_update().tokens;
    let mut conn = Conn {
        replies: vec![hello(login.as_ref().map(|l| l.0.as_str()))],
        state,
        login,
        subs: BTreeMap::new(),
        lines: None,
        wakes: SelectAll::new(),
        next_key: 0,
    };
    let mut over_peak: Option<Instant> = None;
    let mut heard = Instant::now();
    let mut ping_at = Instant::now() + PING_EVERY;
    loop {
        let backlog = QUEUE_CAP - out.capacity();
        // Replies first, then every batch due now: JSON ones as one frame.
        let now = Instant::now();
        let mut texts: Vec<String> = conn.replies.drain(..).map(|r| r.to_string()).collect();
        let mut binary = Vec::new();
        for o in conn.due(now, backlog) {
            match o {
                Out::Text(t) => texts.push(t),
                Out::Binary(b) => binary.push(b),
            }
        }
        let mut frames: Vec<Message> = match texts.len() {
            0 => Vec::new(),
            1 => vec![Message::Text(texts.remove(0).into())],
            _ => vec![Message::Text(format!("[{}]", texts.join(",")).into())],
        };
        frames.extend(binary.into_iter().map(|b| Message::Binary(b.into())));
        if now >= ping_at {
            ping_at = now + PING_EVERY;
            if now.duration_since(heard) > SILENT {
                break;
            }
            frames.push(Message::Ping(Vec::new().into()));
        }
        let mut full = false;
        for f in frames {
            if out.try_send(f).is_err() {
                full = true;
                break;
            }
        }
        let backlog = QUEUE_CAP - out.capacity();
        if backlog >= QUEUE_PEAK {
            let since = *over_peak.get_or_insert(now);
            full |= now.duration_since(since) > PEAK_TIME;
        } else {
            over_peak = None;
        }
        if full {
            close(CLOSE_SLOW, format!("[{}] the page doesn't keep up with its feeds", errors::TOO_SLOW));
            break;
        }

        let next = conn
            .next_due()
            .min(ping_at)
            .min(if over_peak.is_some() { now + MIN_PERIOD } else { ping_at });
        tokio::select! {
            // The engine shuts down: close, so the graceful shutdown can end.
            _ = stopping.changed() => {
                close(1001, "the station stops".into());
                break;
            }
            msg = stream.next() => {
                let Some(Ok(msg)) = msg else { break };
                heard = Instant::now();
                match msg {
                    Message::Text(text) => conn.op(&text),
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            line = next_line(&mut conn.lines) => match line {
                Ok(line) => conn.queue_line(&line),
                Err(RecvError::Lagged(_)) => {
                    for sub in conn.subs.values_mut() {
                        if let Track::Lines { last, lag_from, .. } = &mut sub.track {
                            lag_from.get_or_insert(*last);
                        }
                    }
                }
                Err(RecvError::Closed) => conn.lines = None,
            },
            Some(key) = conn.wakes.next(), if !conn.wakes.is_empty() => conn.wake(key),
            changed = revs.changed(), if revs_alive => match changed {
                // A logout revokes tokens: a socket on one is logged out.
                Ok(()) => {
                    let now = revs.borrow_and_update().tokens;
                    if now != tokens {
                        tokens = now;
                        conn.recheck_login();
                    }
                }
                Err(_) => revs_alive = false,
            },
            _ = tokio::time::sleep_until(next) => {}
            // The writer stopped (a send stalled): the page is gone.
            _ = out.closed() => break,
        }
        if !conn.subs.values().any(|s| matches!(s.track, Track::Lines { .. })) {
            conn.lines = None;
        }
    }
}

/// The envelope every batch carries.
pub fn envelope(feed: Feed, id: u64, from: u64, to: u64, gap: Option<(u64, u64)>) -> Value {
    json!({
        "feed": feed.name(),
        "kind": feed.kind(),
        "id": id,
        "epoch": crate::epoch(),
        "from_seq": from,
        "to_seq": to,
        "gap": gap.map(|(from, to)| json!({"from": from, "to": to})),
    })
}

/// A shared State batch, framed around the build's text (serialized once
/// for every subscriber): `rest` is `"data":…`, `"base":…,"diff":…` or
/// `"unchanged":true`.
pub(crate) fn shared_envelope(feed: Feed, id: u64, seq: u64, rev: u64, rest: &str) -> String {
    format!(
        "{{\"feed\":\"{}\",\"kind\":\"state\",\"id\":{id},\"epoch\":\"{}\",\"from_seq\":{seq},\"to_seq\":{seq},\"gap\":null,\"rev\":{rev},{rest}}}",
        feed.name(),
        crate::epoch()
    )
}

fn seq_of(v: &Value) -> u64 {
    v["seq"].as_u64().unwrap_or(0)
}

/// Feedback lines for `user`: station-wide (no `user`) or theirs.
fn for_user(line: &Value, user: &str) -> bool {
    let f = &line["feedback"];
    !f.is_null() && f["user"].as_str().is_none_or(|u| u.eq_ignore_ascii_case(user))
}

fn history(state: &HttpState) -> Vec<Value> {
    state
        .console
        .sysmon_history()
        .iter()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

impl Conn {
    fn user(&self) -> &str {
        self.login.as_ref().map_or("", |l| l.0.as_str())
    }

    fn nosub(&mut self, id: u64, code: &str, message: impl Into<String>) {
        self.replies
            .push(json!({ "op": "nosub", "id": id, "code": code, "message": message.into() }));
    }

    fn error(&mut self, id: Option<u64>, code: &str, message: &str) {
        self.replies
            .push(json!({ "op": "error", "id": id, "code": code, "message": message }));
    }

    /// A page's op.
    fn op(&mut self, text: &str) {
        let Ok(v) = serde_json::from_str::<Value>(text) else {
            return self.error(None, errors::BAD_MESSAGE, "not JSON");
        };
        match v["op"].as_str() {
            Some("auth") => self.auth(v["token"].as_str()),
            Some("sub") => match v["id"].as_u64() {
                Some(id) => self.sub(id, v),
                None => self.error(None, errors::BAD_PARAMS, "a sub needs an id"),
            },
            Some("unsub") => {
                if let Some(id) = v["id"].as_u64() {
                    self.subs.remove(&id);
                }
            }
            _ => self.error(v["id"].as_u64(), errors::BAD_MESSAGE, "op is auth, sub or unsub"),
        }
    }

    /// Log the socket in (`Some` token) or out (`None`). Subscriptions stay:
    /// another user's feeds start over as theirs.
    fn auth(&mut self, token: Option<&str>) {
        let Some(token) = token else {
            return self.logout("logged out");
        };
        let Some(user) = self.state.console.verify_token(token) else {
            return self.error(None, errors::AUTH_REFUSED, "the login token isn't valid");
        };
        let changed = self
            .login
            .as_ref()
            .is_none_or(|(u, _)| !u.eq_ignore_ascii_case(&user));
        self.login = Some((user.clone(), token.to_string()));
        self.replies.push(json!({ "op": "authed", "user": user }));
        if changed {
            let again: Vec<(u64, Value)> = self
                .subs
                .iter()
                .filter(|(_, s)| !s.feed.def().open)
                .map(|(id, s)| (*id, s.request.clone()))
                .collect();
            for (id, mut request) in again {
                if let Some(o) = request.as_object_mut() {
                    for k in ["since", "epoch", "rev"] {
                        o.remove(k);
                    }
                }
                self.sub(id, request);
            }
        }
    }

    fn logout(&mut self, why: &str) {
        self.login = None;
        let ended: Vec<u64> = self
            .subs
            .iter()
            .filter(|(_, s)| !s.feed.def().open)
            .map(|(id, _)| *id)
            .collect();
        for id in ended {
            self.subs.remove(&id);
            self.nosub(id, errors::LOGIN_REQUIRED, format!("{why}: this feed needs a login"));
        }
        self.replies.push(json!({ "op": "authed", "user": null }));
    }

    /// The token was revoked (a logout elsewhere): log the socket out.
    fn recheck_login(&mut self) {
        if let Some((_, token)) = &self.login
            && self.state.console.verify_token(token).is_none()
        {
            self.logout("the login ended");
        }
    }

    fn sub(&mut self, id: u64, v: Value) {
        // A sub with an id in use replaces it.
        self.subs.remove(&id);
        let Some(feed) = v["feed"].as_str().and_then(Feed::parse) else {
            return self.nosub(id, errors::UNKNOWN_FEED, format!("no feed {}", v["feed"]));
        };
        let def = feed.def();
        if self.login.is_none() && !def.open {
            return self.nosub(id, errors::LOGIN_REQUIRED, format!("the {} feed needs a login", def.name));
        }
        if self.subs.len() >= MAX_SUBS {
            return self.nosub(id, errors::TOO_MANY, format!("at most {MAX_SUBS} subscriptions on one socket"));
        }
        let period = v["period_ms"]
            .as_u64()
            .map_or(feed.default_period(), Duration::from_millis)
            .max(MIN_PERIOD);
        // A `since` from another run means nothing here.
        let since = v["since"]
            .as_u64()
            .filter(|_| v["epoch"].as_str().is_none_or(|e| e == crate::epoch()));
        let (track, recovered, wake) = match self.start(feed, &v, since) {
            Ok(started) => started,
            Err(message) => return self.nosub(id, errors::BAD_PARAMS, message),
        };
        let woken = wake.is_some();
        let dirty = match &track {
            Track::Shared { rx, .. } => rx.borrow().is_some(),
            _ => true,
        };
        self.next_key += 1;
        let key = self.next_key;
        let cancel = wake.map(|stream| {
            let (tx, rx) = oneshot::channel::<()>();
            self.wakes.push(stream.map(move |()| key).take_until(rx).boxed());
            tx
        });
        self.replies
            .push(json!({ "op": "subbed", "id": id, "feed": def.name, "kind": feed.kind() }));
        self.subs.insert(
            id,
            Sub {
                feed,
                request: v,
                period,
                due: Instant::now(),
                sent: 0,
                recovered,
                woken,
                dirty,
                key,
                _cancel: cancel,
                track,
            },
        );
    }

    fn wake(&mut self, key: u64) {
        if let Some(sub) = self.subs.values_mut().find(|s| s.key == key) {
            sub.dirty = true;
        }
    }

    /// When the next batch may be due.
    fn next_due(&self) -> Instant {
        self.subs
            .values()
            .filter(|s| !s.woken || s.dirty)
            .map(|s| s.due)
            .min()
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(3600))
    }

    /// A new subscription: what it remembers, whether `since` was served,
    /// and what wakes it (none: it sends each period).
    #[allow(clippy::type_complexity, reason = "one constructor per feed")]
    fn start(
        &mut self,
        feed: Feed,
        v: &Value,
        since: Option<u64>,
    ) -> Result<(Track, Option<bool>, Option<BoxStream<'static, ()>>), String> {
        let user = self.user().to_string();
        let state = self.state.clone();
        let text = |k: &str| v[k].as_str().filter(|s| !s.is_empty()).map(str::to_string);
        let spec = match feed {
            Feed::Station => Some(BuildSpec::Station),
            Feed::Network => Some(BuildSpec::Network),
            Feed::LumaSessions => Some(BuildSpec::LumaSessions),
            Feed::Shows => Some(BuildSpec::Shows { history: text("history").unwrap_or_default(), drive: text("drive").unwrap_or_default() }),
            Feed::Files => Some(BuildSpec::Files {
                drive: text("drive").unwrap_or_else(|| "local-library".to_string()),
                folder: text("folder").unwrap_or_default(),
                kind: text("kind").unwrap_or_default(),
            }),
            Feed::Pools => Some(BuildSpec::Pools {
                user: user.clone(),
                pool: text("pool").ok_or("pools needs a pool (an object keyword or window type)")?,
                datapool: v["datapool"].clone(),
                profile: v["profile"].clone(),
                filter: serde_json::json!({ "feature": v["feature"].clone(), "tags": v["tags"].clone() }),
            }),
            Feed::PoolEditor => Some(BuildSpec::PoolEditor {
                user: user.clone(),
                pool: text("pool").ok_or("pool_editor needs a pool")?,
                datapool: v["datapool"].clone(),
                no: v["no"]
                    .as_u64()
                    .and_then(|n| u32::try_from(n).ok())
                    .ok_or("pool_editor needs the object's no")?,
            }),
            Feed::Attributes => Some(BuildSpec::Attributes { user: user.clone(), profile: v["profile"].clone() }),
            Feed::Detail => Some(BuildSpec::Detail {
                kind: text("kind").ok_or("detail needs a kind (profile | user)")?,
                no: v["no"].as_u64().and_then(|n| u32::try_from(n).ok()).ok_or("detail needs the object's no")?,
            }),
            Feed::Windows => Some(BuildSpec::Windows {
                user: user.clone(),
                screen: v["screen"].as_u64().unwrap_or(0) as u32,
            }),
            Feed::Oops => Some(BuildSpec::Oops { user: user.clone() }),
            Feed::Sequence => Some(BuildSpec::Sequence { user: user.clone(), sequence: v["sequence"].clone() }),
            Feed::Tags => Some(BuildSpec::Tags { tag: v["tag"].clone(), deep: v["deep"].as_bool().unwrap_or(false) }),
            Feed::Patch => Some(BuildSpec::Patch { user: user.clone() }),
            Feed::Tree => Some(BuildSpec::Tree {
                user: user.clone(),
                path: text("path").unwrap_or_default(),
                open: text("open").unwrap_or_default(),
                search: text("search").unwrap_or_default(),
                follow: v["follow"].as_bool().unwrap_or(false),
            }),
            Feed::Inspector => Some(BuildSpec::Inspector { user: user.clone(), path: text("path").unwrap_or_default() }),
            Feed::ViewBar => Some(BuildSpec::ViewBar {
                user: user.clone(),
                screen: v["screen"].as_u64().unwrap_or(0) as u32,
            }),
            Feed::Encoder => Some(BuildSpec::Encoder {
                user: user.clone(),
                screen: v["screen"].as_u64().unwrap_or(0) as u32,
            }),
            _ => None,
        };
        if let Some(spec) = spec {
            let build = state.hub.subscribe(&state, spec);
            let rx = build.receiver();
            let wake = WatchStream::from_changes(rx.clone()).map(|_| ()).boxed();
            let resume = v["rev"].as_u64();
            return Ok((Track::Shared { _build: build, rx, sent: None, resume }, None, Some(wake)));
        }
        Ok(match feed {
            Feed::Render => {
                let ring = state.console.render_stats();
                let held = |s: u64| ring.as_ref().is_some_and(|r| s >= r.ring.head() || s + 1 >= r.ring.oldest());
                match since {
                    Some(s) if held(s) => (Track::Render { last: s }, Some(true), None),
                    Some(_) => (Track::Render { last: 0 }, Some(false), None),
                    None => (Track::Render { last: 0 }, None, None),
                }
            }
            Feed::Health => (Track::Health { last: None }, None, None),
            Feed::Sysmon | Feed::Feedback => {
                // Live lines first, then the history: nothing falls between.
                if self.lines.is_none() {
                    self.lines = Some(state.sysmon_tx.subscribe());
                }
                let hist = history(&state);
                let newest = hist.last().map_or(0, seq_of);
                let oldest = hist.first().map_or(0, seq_of);
                let resumes = since.filter(|s| *s >= newest || *s + 1 >= oldest);
                let recovered = since.map(|_| resumes.is_some());
                let mut pending: Vec<Value> = match resumes {
                    Some(s) => hist.into_iter().filter(|l| seq_of(l) > s).collect(),
                    None => hist,
                };
                if feed == Feed::Feedback {
                    pending.retain(|l| for_user(l, &user));
                    let mut active: Vec<Value> = state
                        .console
                        .feedback_active(&user)
                        .iter()
                        .filter_map(|l| serde_json::from_str(l).ok())
                        .collect();
                    if resumes.is_none() {
                        // A fresh page: what is on now, not every old message.
                        pending.clear();
                    }
                    active.retain(|a| !pending.iter().any(|p| seq_of(p) == seq_of(a)));
                    pending.extend(active);
                    pending.sort_by_key(seq_of);
                }
                (Track::Lines { last: newest, pending, lag_from: None }, recovered, None)
            }
            Feed::Dmx => {
                let mut universes: Vec<u32> = v["universes"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|u| u.as_u64()).map(|u| u as u32).collect())
                    .unwrap_or_default();
                if let Some(u) = v["universe"].as_u64() {
                    universes.push(u as u32);
                }
                universes.sort_unstable();
                universes.dedup();
                let follow = v["follow_selection"].as_bool().unwrap_or(false);
                (Track::Dmx { universes, follow, show: None, last: BTreeMap::new() }, None, None)
            }
            Feed::Devices => (Track::Devices { params: v.clone(), last: None }, None, None),
            Feed::Screen => {
                let events = &state.screen_events;
                // A fresh page starts at the newest event: an old ReloadUI or
                // capture is never run again.
                let (last, recovered) = match since {
                    Some(s) if events.holds(s) => (s, Some(true)),
                    Some(_) => (events.newest(), Some(false)),
                    None => (events.newest(), None),
                };
                let viewer = v["screen"]
                    .as_u64()
                    .and_then(|n| u32::try_from(n).ok())
                    .filter(|n| *n > 0)
                    .map(|n| Viewer::new(state.live_screens.clone(), n));
                let wake = WatchStream::from_changes(events.watch()).map(|_| ()).boxed();
                (Track::Screen { last, _viewer: viewer }, recovered, Some(wake))
            }
            _ => unreachable!("shared feeds return above"),
        })
    }

    /// A live line from the System Monitor: queue it for the subscriptions
    /// that want it.
    fn queue_line(&mut self, line: &str) {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            return;
        };
        let seq = seq_of(&v);
        let user = self.user().to_string();
        for sub in self.subs.values_mut() {
            let feed = sub.feed;
            if let Track::Lines { last, pending, .. } = &mut sub.track {
                if seq <= *last || (feed == Feed::Feedback && !for_user(&v, &user)) {
                    continue;
                }
                *last = seq;
                pending.push(v.clone());
            }
        }
    }

    /// Every batch due at `now`. While the page's queue is behind
    /// (`backlog`), States wait — only their newest goes later.
    fn due(&mut self, now: Instant, backlog: usize) -> Vec<Out> {
        let mut out = Vec::new();
        let user = self.user().to_string();
        for (id, sub) in self.subs.iter_mut() {
            if sub.due > now || (sub.woken && !sub.dirty) {
                continue;
            }
            if sub.feed.def().kind == FeedKind::State && backlog > QUEUE_PEAK / 2 {
                sub.due = now + if sub.woken { MIN_PERIOD } else { sub.period };
                continue;
            }
            sub.due = now + sub.period;
            sub.dirty = false;
            out.extend(batch(*id, sub, &self.state, &user));
        }
        out
    }
}

/// The messages due for one subscription (none when nothing changed).
fn batch(id: u64, sub: &mut Sub, state: &HttpState, user: &str) -> Vec<Out> {
    let first = sub.sent == 0;
    let feed = sub.feed;
    let mut out = match &mut sub.track {
        Track::Render { last } => render_batch(id, last, first, state),
        Track::Health { last } => {
            let now = state.console.health();
            if !first && last.as_ref() == Some(&now) {
                Vec::new()
            } else {
                let mut env = envelope(feed, id, sub.sent + 1, sub.sent + 1, None);
                env["data"] = now.clone();
                *last = Some(now);
                vec![env]
            }
        }
        Track::Lines { last, pending, lag_from } => {
            let mut gap = None;
            if let Some(from) = lag_from.take() {
                // Refill what the broadcast dropped from the history.
                let have: BTreeSet<u64> = pending.iter().map(seq_of).collect();
                let hist = history(state);
                if let Some(oldest) = hist.first().map(seq_of)
                    && oldest > from + 1
                {
                    gap = Some((from + 1, oldest - 1));
                }
                pending.extend(hist.into_iter().filter(|l| {
                    let s = seq_of(l);
                    s > from && !have.contains(&s) && (feed == Feed::Sysmon || for_user(l, user))
                }));
                pending.sort_by_key(seq_of);
                *last = (*last).max(pending.last().map_or(0, seq_of));
            }
            if pending.is_empty() && !first && gap.is_none() {
                Vec::new()
            } else {
                let items = std::mem::take(pending);
                let from = items.first().map_or(*last, seq_of);
                let to = items.last().map_or(*last, seq_of);
                let mut env = envelope(feed, id, from, to, gap);
                if feed == Feed::Sysmon {
                    env["cols"] = sysmon_cols(&items);
                } else {
                    env["items"] = Value::Array(items);
                }
                vec![env]
            }
        }
        Track::Screen { last, .. } => {
            let (items, gap) = state.screen_events.after(*last);
            if items.is_empty() && gap.is_none() && !first {
                Vec::new()
            } else {
                let from = items.first().map_or(*last, seq_of);
                let to = items.last().map_or(*last, seq_of);
                *last = to;
                let mut env = envelope(feed, id, from, to, gap);
                env["items"] = Value::Array(items);
                vec![env]
            }
        }
        Track::Shared { rx, sent, resume, .. } => {
            let Some(built) = rx.borrow().clone() else {
                return Vec::new();
            };
            if *sent == Some(built.rev) {
                return Vec::new();
            }
            let seq = sub.sent + 1;
            let text = if first && *resume == Some(built.rev) {
                shared_envelope(feed, id, seq, built.rev, "\"unchanged\":true")
            } else {
                match (&built.diff, *sent) {
                    (Some((base, diff)), Some(had)) if *base == had => {
                        shared_envelope(feed, id, seq, built.rev, &format!("\"base\":{base},\"diff\":{diff}"))
                    }
                    _ => shared_envelope(feed, id, seq, built.rev, &format!("\"data\":{}", built.text)),
                }
            };
            *sent = Some(built.rev);
            sub.sent = seq;
            return vec![Out::Text(text)];
        }
        Track::Dmx { .. } => return dmx_batch(id, sub, state, user),
        Track::Devices { .. } => return devices_batch(id, sub, state, user),
    };
    if let (true, Some(env), Some(r)) = (first, out.first_mut(), sub.recovered) {
        env["recovered"] = r.into();
    }
    sub.sent += out.len() as u64;
    out.into_iter().map(|v| Out::Text(v.to_string())).collect()
}

/// The `windows` feed's data for one of the user's live screens (another
/// user's screen, or none, is `{"missing": n}`).
/// A live screen's encoder bar (decisions §30 "12f").
pub fn encoder_data(state: &HttpState, user: &str, screen: u32) -> Value {
    let found = state
        .live_screens
        .read()
        .ok()
        .and_then(|store| store.get(screen).filter(|s| s.user.eq_ignore_ascii_case(user)).cloned());
    match found {
        Some(s) => state.console.encoder(user, &s),
        None => json!({ "missing": screen }),
    }
}

/// A live screen's view bar (the `viewbar` feed): the bar of the profile screen it is.
pub fn viewbar_data(state: &HttpState, user: &str, screen: u32) -> Value {
    let found = state
        .live_screens
        .read()
        .ok()
        .and_then(|store| store.get(screen).filter(|s| s.user.eq_ignore_ascii_case(user)).map(|s| s.screen));
    match found {
        Some(s) => state.console.view_bar(user, s),
        None => json!({ "missing": screen }),
    }
}

pub fn windows_data(state: &HttpState, user: &str, screen: u32) -> Value {
    let found = state
        .live_screens
        .read()
        .ok()
        .and_then(|store| store.get(screen).filter(|s| s.user.eq_ignore_ascii_case(user)).cloned());
    match found {
        Some(s) => state.console.windows(user, &s),
        None => json!({ "missing": screen }),
    }
}

/// Render frames since the last batch, columnar.
fn render_batch(id: u64, last: &mut u64, first: bool, state: &HttpState) -> Vec<Value> {
    use lumacore_tree::runtime::STAGES;
    let mut frames = Vec::new();
    if let Some(stats) = state.console.render_stats() {
        stats.ring.read_after(*last, &mut frames);
    }
    if frames.is_empty() && !first {
        return Vec::new();
    }
    let gap = match frames.first() {
        Some(f) if *last > 0 && f.seq > *last + 1 => Some((*last + 1, f.seq - 1)),
        _ => None,
    };
    let from = frames.first().map_or(*last, |f| f.seq);
    let to = frames.last().map_or(*last, |f| f.seq);
    let col = |f: fn(&lumacore_tree::runtime::FrameSample) -> u64| -> Vec<u64> { frames.iter().map(f).collect() };
    let stage_us: Vec<Vec<u64>> = (0..STAGES.len())
        .map(|i| frames.iter().map(|f| f.stages[i]).collect())
        .collect();
    let mut env = envelope(Feed::Render, id, from, to, gap);
    env["stages"] = json!(STAGES);
    env["cols"] = json!({
        "seq": col(|f| f.seq),
        "t_ms": col(|f| f.t_ms),
        "frame_us": col(|f| f.frame_us),
        "headroom_us": col(|f| f.headroom_us),
        "jitter_us": col(|f| f.jitter_us),
        "late": col(|f| u64::from(f.late)),
        "stage_us": stage_us,
    });
    *last = to;
    vec![env]
}

/// Sysmon lines, columnar.
fn sysmon_cols(items: &[Value]) -> Value {
    let num = |k: &str| -> Vec<u64> { items.iter().map(|l| l[k].as_u64().unwrap_or(0)).collect() };
    let text = |k: &str| -> Vec<&str> { items.iter().map(|l| l[k].as_str().unwrap_or("")).collect() };
    json!({
        "seq": num("seq"),
        "time_ms": num("time_ms"),
        "elapsed_ms": num("elapsed_ms"),
        "level": text("level"),
        "subsystem": text("subsystem"),
        "message": text("message"),
    })
}

/// The `devices` feed: the sheet built for the window's rows, sent when it differs from the last.
fn devices_batch(id: u64, sub: &mut Sub, state: &HttpState, user: &str) -> Vec<Out> {
    use std::hash::{Hash, Hasher};
    let Track::Devices { params, last } = &mut sub.track else {
        return Vec::new();
    };
    let data = state.console.devices(user, params);
    let text = data.to_string();
    let mut h = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut h);
    let print = h.finish();
    if *last == Some(print) {
        return Vec::new();
    }
    *last = Some(print);
    sub.sent += 1;
    let mut env = envelope(Feed::Devices, id, sub.sent, sub.sent, None);
    env["data"] = data;
    vec![Out::Text(env.to_string())]
}

/// The DMX feed: the patch (JSON) when the show changes, the universes'
/// values (binary) when they change.
fn dmx_batch(id: u64, sub: &mut Sub, state: &HttpState, user: &str) -> Vec<Out> {
    let Track::Dmx { universes, follow, show, last } = &mut sub.track else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut text = |sent: &mut u64, body: (&str, Value)| {
        *sent += 1;
        let mut env = envelope(Feed::Dmx, id, *sent, *sent, None);
        env[body.0] = body.1;
        out.push(Out::Text(env.to_string()));
    };
    if *follow
        && let Some(u) = state.console.dmx_selected_universe(user)
        && universes.as_slice() != [u]
    {
        *universes = vec![u];
        *show = None;
        last.clear();
        text(&mut sub.sent, ("follow", json!(u)));
    }
    let shown = state.show.load_full();
    let show_id = std::sync::Arc::as_ptr(&shown) as usize;
    if *show != Some(show_id) {
        *show = Some(show_id);
        let all: Vec<u32> = shown.devices.dmx.patch.universes.keys().copied().collect();
        let patches: Vec<Value> = universes
            .iter()
            .map(|u| json!({"universe": u, "exists": all.contains(u), "spans": state.console.dmx_patch(*u)}))
            .collect();
        text(&mut sub.sent, ("data", json!({"universes": all, "patches": patches})));
    }
    let mut changed = Vec::new();
    for u in universes.iter() {
        if let Some(now) = state.console.dmx_universe(*u)
            && last.get(u) != Some(&now)
        {
            last.insert(*u, now.clone());
            changed.push(now);
        }
    }
    if !changed.is_empty() {
        sub.sent += 1;
        out.push(Out::Binary(dmx_frame(id, sub.sent, &changed)));
    }
    out
}

/// Bytes per universe in a DMX frame: an 8-byte head, then values, ours,
/// winners and input (512 each) and the winning input line (u16 × 512).
pub const DMX_UNIVERSE_BYTES: usize = 8 + 4 * 512 + 2 * 512;
/// The DMX frame's header.
pub const DMX_HEADER_BYTES: usize = 20;

/// One binary DMX frame, little-endian: `u32 id`, `u32 seq`, `f64 time_ms`,
/// `u16 count`, `u16 0`; then per universe `u32 universe`, `u8 flags` (1 =
/// has input), 3 × `u8 0`, `values[512]`, `ours[512]`, `winners[512]`
/// (0 unpatched, 1 default, 2 ours, 3 park, 4 input), `input[512]`,
/// `lines: u16[512]`. Every array starts on an even offset.
pub fn dmx_frame(id: u64, seq: u64, universes: &[DmxUniverse]) -> Vec<u8> {
    let time_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |d| d.as_millis() as f64);
    let mut b = Vec::with_capacity(DMX_HEADER_BYTES + universes.len() * DMX_UNIVERSE_BYTES);
    b.extend_from_slice(&(id as u32).to_le_bytes());
    b.extend_from_slice(&(seq as u32).to_le_bytes());
    b.extend_from_slice(&time_ms.to_le_bytes());
    b.extend_from_slice(&(universes.len() as u16).to_le_bytes());
    b.extend_from_slice(&0u16.to_le_bytes());
    let fixed = |v: &[u8], b: &mut Vec<u8>| {
        let mut a = [0u8; 512];
        let n = v.len().min(512);
        a[..n].copy_from_slice(&v[..n]);
        b.extend_from_slice(&a);
    };
    for u in universes {
        b.extend_from_slice(&u.universe.to_le_bytes());
        b.extend_from_slice(&[u8::from(u.input.is_some()), 0, 0, 0]);
        fixed(&u.values, &mut b);
        fixed(&u.ours, &mut b);
        fixed(&u.winners, &mut b);
        fixed(u.input.as_deref().unwrap_or(&[]), &mut b);
        for i in 0..512 {
            let line = u.lines.get(i).copied().unwrap_or(0).min(u32::from(u16::MAX)) as u16;
            b.extend_from_slice(&line.to_le_bytes());
        }
    }
    b
}
