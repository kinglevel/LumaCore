//! /ws/feeds (decisions §30 "12b2 hardening"): one socket per page, one batch
//! when a feed is subscribed, then batches per period; samples carry seqs,
//! gaps and resume with `since`.

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;

use crate::HttpState;
use crate::router::build_router;
use crate::state::DiagnosticsSnapshot;
use crate::websocket::feeds::{DMX_HEADER_BYTES, DMX_UNIVERSE_BYTES, Feed, dmx_frame, envelope, shared_envelope};

type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Spawn a real HTTP server on a random port and return the address.
async fn spawn_test_server(state: HttpState) -> SocketAddr {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

fn line(seq: u64, message: &str) -> String {
    json!({"seq": seq, "time_ms": 1_760_000_000_000u64 + seq, "elapsed_ms": seq, "level": "info",
           "subsystem": "Test", "message": message})
    .to_string()
}

fn feedback(seq: u64, user: Option<&str>, title: &str) -> String {
    json!({"seq": seq, "time_ms": 1, "elapsed_ms": 1, "level": "info", "subsystem": "Show", "message": title,
           "feedback": {"kind": "message", "title": title, "user": user}})
    .to_string()
}

fn state_with_history(history: Vec<String>) -> (HttpState, tokio::sync::broadcast::Sender<String>) {
    let snap = Arc::new(DiagnosticsSnapshot { sysmon_history: history, ..Default::default() });
    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(64);
    let state = super::test_http_state(Arc::new(ArcSwap::from(snap)), tx.clone(), None);
    (state, tx)
}

fn state_with_knobs() -> (HttpState, super::Knobs) {
    let knobs = super::Knobs::default();
    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(64);
    let state = super::test_http_state_with(Arc::default(), tx, None, knobs.clone());
    (state, knobs)
}

/// A feed socket: messages in order, a JSON-array frame taken apart.
struct Client {
    ws: Ws,
    buf: VecDeque<Value>,
    /// Frames that carried more than one message.
    arrays: usize,
}

impl Client {
    async fn open(addr: SocketAddr, token: Option<&str>) -> Client {
        let url = match token {
            Some(t) => format!("ws://{addr}/ws/feeds?token={t}"),
            None => format!("ws://{addr}/ws/feeds"),
        };
        let ws = tokio_tungstenite::connect_async(&url).await.unwrap().0;
        let mut c = Client { ws, buf: VecDeque::new(), arrays: 0 };
        let hello = c.any().await;
        assert_eq!(hello["op"], "hello", "{hello}");
        c
    }

    async fn op(&mut self, v: Value) {
        self.ws.send(Message::Text(v.to_string().into())).await.unwrap();
    }

    /// The next message (ops and batches; binary frames skipped).
    async fn any(&mut self) -> Value {
        loop {
            if let Some(v) = self.buf.pop_front() {
                return v;
            }
            let msg = tokio::time::timeout(Duration::from_secs(5), self.ws.next())
                .await
                .expect("a message within 5 s")
                .unwrap()
                .unwrap();
            if let Message::Text(t) = msg {
                match serde_json::from_str::<Value>(&t).unwrap() {
                    Value::Array(items) => {
                        self.arrays += 1;
                        self.buf.extend(items);
                    }
                    v => self.buf.push_back(v),
                }
            }
        }
    }

    /// The next batch (op replies skipped, except errors).
    async fn next(&mut self) -> Value {
        loop {
            let v = self.any().await;
            match v["op"].as_str() {
                Some("subbed" | "authed") => {}
                Some(_) => return v,
                None => return v,
            }
        }
    }

    /// Nothing more within `ms`.
    async fn quiet(&mut self, ms: u64) -> bool {
        self.buf.is_empty() && tokio::time::timeout(Duration::from_millis(ms), self.ws.next()).await.is_err()
    }
}

async fn connect(addr: SocketAddr) -> Client {
    Client::open(addr, Some(super::TEST_TOKEN)).await
}

fn seqs(v: &Value) -> Vec<u64> {
    v["cols"]["seq"].as_array().unwrap().iter().map(|s| s.as_u64().unwrap()).collect()
}


#[test]
fn the_envelope_is_one_shape() {
    let v = envelope(Feed::Render, 3, 10, 12, Some((5, 9)));
    let mut keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["epoch", "feed", "from_seq", "gap", "id", "kind", "to_seq"]);
    // A shared State batch: the same shape plus its rev, framed around the
    // build's text.
    let shared: Value = serde_json::from_str(&shared_envelope(Feed::Station, 1, 1, 7, "\"data\":{}")).unwrap();
    let mut keys: Vec<&str> = shared.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["data", "epoch", "feed", "from_seq", "gap", "id", "kind", "rev", "to_seq"]);
    assert_eq!(v["feed"], "health.render");
    assert_eq!(v["kind"], "samples");
    assert_eq!(v["gap"], json!({"from": 5, "to": 9}));
    assert_eq!(v["epoch"], crate::epoch());
    assert_eq!(envelope(Feed::Pools, 1, 1, 1, None)["kind"], "state");
    assert!(envelope(Feed::Dmx, 1, 1, 1, None)["gap"].is_null());
    for f in Feed::ALL {
        assert_eq!(Feed::parse(f.name()), Some(f));
    }
}

/// Only /ws/feeds and the shell socket exist (decisions §30 "12b2 hardening":
/// one feed rule) and no SSE stream is left ("12d hardening": everything live
/// on /ws/feeds) — a new per-feed socket or event stream fails here.
#[test]
fn the_router_has_only_the_feeds_and_shell_sockets_and_no_sse() {
    let src = include_str!("../router.rs");
    let routes: Vec<&str> = src
        .match_indices("\"/ws/")
        .map(|(i, _)| src[i + 1..].split('"').next().unwrap())
        .collect();
    assert_eq!(routes, ["/ws/feeds", "/ws/shell"]);
    let all: Vec<&str> = src
        .match_indices(".route(\"")
        .map(|(i, _)| src[i + 8..].split('"').next().unwrap())
        .collect();
    assert!(all.len() > 15, "{all:?}");
    for r in &all {
        assert!(!r.contains("events"), "an event stream route: {r}");
    }
    for file in [include_str!("../router.rs"), include_str!("../endpoints/station.rs"), include_str!("../endpoints/mod.rs")] {
        assert!(!file.contains("Sse") && !file.contains("sse::"), "no SSE responder");
    }
}

#[tokio::test]
async fn sysmon_sends_the_whole_history_in_one_batch_then_live_batches() {
    let history: Vec<String> = (1..=300).map(|n| line(n, &format!("boot {n}"))).collect();
    let (state, tx) = state_with_history(history);
    let addr = spawn_test_server(state).await;
    let mut ws = connect(addr).await;
    ws.op(json!({"op": "sub", "id": 1, "feed": "sysmon"})).await;
    let first = ws.next().await;
    assert_eq!(first["feed"], "sysmon");
    assert_eq!(first["kind"], "samples");
    assert_eq!(first["id"], 1);
    assert_eq!((first["from_seq"].as_u64(), first["to_seq"].as_u64()), (Some(1), Some(300)));
    assert_eq!(seqs(&first), (1..=300).collect::<Vec<_>>());
    assert_eq!(first["cols"]["message"][299], "boot 300");
    assert!(first.get("recovered").is_none(), "no since, no recovered");

    for n in 301..=310 {
        tx.send(line(n, "live")).unwrap();
    }
    // A line the history already had is not sent twice.
    tx.send(line(300, "boot 300")).unwrap();
    let mut got = Vec::new();
    let mut messages = 0;
    while got.len() < 10 {
        got.extend(seqs(&ws.next().await));
        messages += 1;
    }
    assert_eq!(got, (301..=310).collect::<Vec<_>>());
    assert!(messages < 10, "lines come in batches, not one per message");
}

#[tokio::test]
async fn sysmon_resumes_after_since_or_starts_over() {
    let history: Vec<String> = (10..=20).map(|n| line(n, "x")).collect();
    let (state, _tx) = state_with_history(history);
    let addr = spawn_test_server(state).await;
    let mut ws = connect(addr).await;

    ws.op(json!({"op": "sub", "id": 1, "feed": "sysmon", "since": 15, "epoch": crate::epoch()})).await;
    let v = ws.next().await;
    assert_eq!(v["recovered"], true);
    assert_eq!(seqs(&v), (16..=20).collect::<Vec<_>>());

    // Older than the ring: the whole ring, recovered false.
    ws.op(json!({"op": "sub", "id": 2, "feed": "sysmon", "since": 3})).await;
    let v = ws.next().await;
    assert_eq!(v["id"], 2);
    assert_eq!(v["recovered"], false);
    assert_eq!(seqs(&v), (10..=20).collect::<Vec<_>>());

    // Another run's since means nothing: a fresh start.
    ws.op(json!({"op": "sub", "id": 3, "feed": "sysmon", "since": 15, "epoch": "another-run"})).await;
    let v = ws.next().await;
    assert!(v.get("recovered").is_none());
    assert_eq!(seqs(&v).len(), 11);
}

#[tokio::test]
async fn render_frames_come_every_one_with_a_gap_when_the_ring_moved_on() {
    use lumacore_tree::runtime::{FRAME_RING, FrameSample};
    let (state, _tx) = state_with_history(vec![]);
    let stats = state.console.render_stats().unwrap();
    let push = |from: u64, to: u64| {
        for seq in from..=to {
            stats.ring.push(&FrameSample { seq, frame_us: seq * 10, late: seq == 3, ..Default::default() });
        }
    };
    push(1, 5);
    let addr = spawn_test_server(state.clone()).await;
    let mut ws = connect(addr).await;
    ws.op(json!({"op": "sub", "id": 7, "feed": "health.render", "period_ms": 1})).await;
    let v = ws.next().await;
    assert_eq!(v["feed"], "health.render");
    assert_eq!(seqs(&v), [1, 2, 3, 4, 5]);
    assert_eq!(v["cols"]["frame_us"], json!([10, 20, 30, 40, 50]));
    assert_eq!(v["cols"]["late"], json!([0, 0, 1, 0, 0]));
    assert_eq!(v["stages"].as_array().unwrap().len(), v["cols"]["stage_us"].as_array().unwrap().len());
    assert!(v["gap"].is_null());

    // Every frame, none twice.
    push(6, 8);
    let v = ws.next().await;
    assert_eq!(seqs(&v), [6, 7, 8]);

    // The ring moved on past what the page had: an explicit gap.
    let end = 8 + 3 * FRAME_RING as u64;
    push(9, end);
    let v = ws.next().await;
    let gap = &v["gap"];
    assert_eq!(gap["from"], 9);
    assert_eq!(gap["to"].as_u64().unwrap() + 1, v["from_seq"].as_u64().unwrap());
    assert_eq!(v["to_seq"].as_u64(), Some(end));
}

#[tokio::test]
async fn feedback_is_the_users_own_and_station_wide_lines() {
    let (state, tx) = state_with_history(vec![]);
    let addr = spawn_test_server(state).await;
    let mut ws = connect(addr).await;
    ws.op(json!({"op": "sub", "id": 4, "feed": "feedback"})).await;
    let v = ws.next().await;
    assert_eq!(v["items"], json!([]), "the first batch comes even when empty");
    tx.send(feedback(1, None, "Show saved")).unwrap();
    tx.send(feedback(2, Some("LD"), "not mine")).unwrap();
    tx.send(line(3, "no feedback")).unwrap();
    tx.send(feedback(4, Some("admin"), "mine")).unwrap();
    let mut titles = Vec::new();
    while titles.len() < 2 {
        let v = ws.next().await;
        titles.extend(v["items"].as_array().unwrap().iter().map(|i| i["feedback"]["title"].as_str().unwrap().to_string()));
    }
    assert_eq!(titles, ["Show saved", "mine"]);
}

#[tokio::test]
async fn unsub_stops_a_feed_and_unknown_feeds_are_refused() {
    let (state, tx) = state_with_history(vec![line(1, "a")]);
    let addr = spawn_test_server(state).await;
    let mut ws = connect(addr).await;
    ws.op(json!({"op": "sub", "id": 9, "feed": "nope"})).await;
    let v = ws.next().await;
    assert_eq!(v["op"], "nosub");
    assert_eq!(v["id"], 9);
    assert_eq!(v["code"], "E-FEED-001");

    ws.op(json!({"op": "sub", "id": 1, "feed": "sysmon"})).await;
    assert_eq!(seqs(&ws.next().await), [1]);
    ws.op(json!({"op": "unsub", "id": 1})).await;
    ws.op(json!({"op": "sub", "id": 2, "feed": "health"})).await;
    tx.send(line(2, "after unsub")).unwrap();
    // Only the health state arrives (once: it doesn't change in tests), no
    // sysmon batch.
    let v = ws.next().await;
    assert_eq!(v["feed"], "health", "{v}");
    assert!(ws.quiet(400).await, "nothing else");
}

#[test]
fn a_dmx_frame_is_a_header_then_fixed_universe_blocks() {
    let u = crate::DmxUniverse {
        universe: 3,
        values: vec![255; 512],
        ours: vec![7; 512],
        input: Some(vec![9; 512]),
        winners: vec![2; 512],
        lines: vec![70_000; 512],
    };
    let b = dmx_frame(5, 6, &[u]);
    assert_eq!(b.len(), DMX_HEADER_BYTES + DMX_UNIVERSE_BYTES);
    assert_eq!(u32::from_le_bytes(b[0..4].try_into().unwrap()), 5);
    assert_eq!(u32::from_le_bytes(b[4..8].try_into().unwrap()), 6);
    assert!(f64::from_le_bytes(b[8..16].try_into().unwrap()) > 1.7e12);
    assert_eq!(u16::from_le_bytes(b[16..18].try_into().unwrap()), 1);
    let u0 = DMX_HEADER_BYTES;
    assert_eq!(u32::from_le_bytes(b[u0..u0 + 4].try_into().unwrap()), 3);
    assert_eq!(b[u0 + 4], 1, "has input");
    let arrays = u0 + 8;
    assert_eq!(&b[arrays..arrays + 2], &[255, 255]);
    assert_eq!(b[arrays + 512], 7);
    assert_eq!(b[arrays + 1024], 2);
    assert_eq!(b[arrays + 1536], 9);
    let lines = arrays + 2048;
    assert_eq!(lines % 2, 0);
    assert_eq!(u16::from_le_bytes(b[lines..lines + 2].try_into().unwrap()), u16::MAX);
}
/// The `windows` feed (decisions §30 "12c"): a State feed for one of the
/// user's live screens — one batch on sub, then only when it changes;
/// another user's screen is `missing`.
#[tokio::test]
async fn windows_is_a_state_feed_of_the_users_own_screen() {
    let (state, _tx) = state_with_history(vec![]);
    let (mine, theirs) = {
        let mut store = state.live_screens.write().unwrap();
        (store.join_or_create("Admin", None), store.join_or_create("Bob", None))
    };
    let addr = spawn_test_server(state.clone()).await;
    let mut ws = connect(addr).await;
    ws.op(json!({"op": "sub", "id": 4, "feed": "windows", "screen": mine, "period_ms": 16})).await;
    let first = ws.next().await;
    assert_eq!((first["feed"].as_str(), first["kind"].as_str()), (Some("windows"), Some("state")));
    assert_eq!(first["data"]["screen"], mine);
    // Nothing changed: no second batch (a sub on another screen answers next).
    ws.op(json!({"op": "sub", "id": 5, "feed": "windows", "screen": theirs})).await;
    let other = ws.next().await;
    assert_eq!(other["id"], 5, "the unchanged feed 4 sent nothing more");
    assert_eq!(other["data"]["missing"], theirs);
}

/// The socket opens without a login (decisions §30 "12d hardening"): `hello`
/// lists the feed registry; only `open` feeds work before `auth`; `auth`
/// keeps every subscription and the ones that waited are taken again.
#[tokio::test]
async fn the_socket_opens_without_login_and_auth_keeps_the_subscriptions() {
    let (state, knobs) = state_with_knobs();
    knobs.set_station(json!({"name": "T13", "users": [{"name": "Admin"}]}));
    let addr = spawn_test_server(state).await;
    let url = format!("ws://{addr}/ws/feeds");
    let ws = tokio_tungstenite::connect_async(&url).await.unwrap().0;
    let mut c = Client { ws, buf: VecDeque::new(), arrays: 0 };
    let hello = c.any().await;
    assert_eq!(hello["op"], "hello");
    assert_eq!(hello["client_protocol"], lumacore_wire::CLIENT_PROTOCOL);
    assert_eq!(hello["epoch"], crate::epoch());
    assert!(hello["user"].is_null());
    assert_eq!(hello["max_subs"], 64);
    let feeds = hello["feeds"].as_array().unwrap();
    assert_eq!(feeds.len(), Feed::ALL.len());
    let open: Vec<&str> = feeds.iter().filter(|f| f["open"] == true).map(|f| f["name"].as_str().unwrap()).collect();
    assert_eq!(open, ["station"]);
    for name in ["network", "lumasessions", "shows", "station", "screen", "pool_editor"] {
        assert!(feeds.iter().any(|f| f["name"] == name && f["kind"].is_string()), "{name} in the hello");
    }

    // The open feed works; one that needs a login is refused with its code.
    c.op(json!({"op": "sub", "id": 1, "feed": "station"})).await;
    assert_eq!(c.any().await, json!({"op": "subbed", "id": 1, "feed": "station", "kind": "state"}));
    let st = c.any().await;
    assert_eq!((st["id"].as_u64(), st["data"]["name"].as_str()), (Some(1), Some("T13")));
    assert!(st["rev"].as_u64().is_some());
    c.op(json!({"op": "sub", "id": 2, "feed": "sysmon"})).await;
    let no = c.any().await;
    assert_eq!((no["op"].as_str(), no["id"].as_u64(), no["code"].as_str()), (Some("nosub"), Some(2), Some("E-FEED-002")));

    // A wrong token: refused, nothing changes.
    c.op(json!({"op": "auth", "token": "nope"})).await;
    let e = c.any().await;
    assert_eq!((e["op"].as_str(), e["code"].as_str()), (Some("error"), Some("E-FEED-006")));

    // auth: authed, then the page sends the waiting sub again; the station
    // feed stays (no second batch: nothing changed).
    c.op(json!({"op": "auth", "token": super::TEST_TOKEN})).await;
    assert_eq!(c.any().await, json!({"op": "authed", "user": "Admin"}));
    c.op(json!({"op": "sub", "id": 2, "feed": "sysmon"})).await;
    assert_eq!(c.any().await["op"], "subbed");
    assert_eq!(c.any().await["feed"], "sysmon");

    // A change of the station view reaches the open feed.
    knobs.set_station(json!({"name": "T13b"}));
    let st = c.next().await;
    assert_eq!((st["id"].as_u64(), st["data"]["name"].as_str()), (Some(1), Some("T13b")));

    // Logged out: the feeds that need a login end with nosub, the open one stays.
    c.op(json!({"op": "auth", "token": null})).await;
    let no = c.any().await;
    assert_eq!((no["op"].as_str(), no["id"].as_u64(), no["code"].as_str()), (Some("nosub"), Some(2), Some("E-FEED-002")));
    assert_eq!(c.any().await, json!({"op": "authed", "user": null}));
    knobs.set_station(json!({"name": "T13c"}));
    assert_eq!(c.next().await["data"]["name"], "T13c");
}

/// A revoked token (a logout elsewhere) logs the socket out at the next
/// counter change.
#[tokio::test]
async fn a_revoked_token_ends_the_feeds_that_need_a_login() {
    let (state, knobs) = state_with_knobs();
    knobs.tokens.lock().unwrap().push(("bob-token".into(), "Bob".into()));
    let addr = spawn_test_server(state).await;
    let mut c = Client::open(addr, Some("bob-token")).await;
    c.op(json!({"op": "sub", "id": 4, "feed": "health"})).await;
    assert_eq!(c.next().await["feed"], "health");
    knobs.tokens.lock().unwrap().clear();
    knobs.revisions.send_modify(|r| r.tokens += 1);
    let no = c.any().await;
    assert_eq!((no["op"].as_str(), no["code"].as_str()), (Some("nosub"), Some("E-FEED-002")));
}

/// Shared builds (decisions §30 "12d hardening"): one build per (feed,
/// params) while subscribed, fanned out; batches due in one tick go as one
/// frame; a resubscribe with the same rev gets `unchanged`.
#[tokio::test]
async fn state_feeds_share_one_build_and_resume_by_rev() {
    let (state, knobs) = state_with_knobs();
    knobs.set_station(json!({"name": "A"}));
    let addr = spawn_test_server(state.clone()).await;
    let mut a = connect(addr).await;
    let mut b = connect(addr).await;
    a.op(json!({"op": "sub", "id": 1, "feed": "station"})).await;
    a.op(json!({"op": "sub", "id": 2, "feed": "station"})).await;
    b.op(json!({"op": "sub", "id": 7, "feed": "station"})).await;
    let first = a.next().await;
    let rev = first["rev"].as_u64().unwrap();
    assert_eq!(a.next().await["rev"], rev);
    assert_eq!(b.next().await["rev"], rev);
    assert_eq!(state.hub.running(), 1, "one build for three subscriptions");

    // One change: both of a's subscriptions in one frame.
    let arrays = a.arrays;
    knobs.set_station(json!({"name": "B"}));
    let x = a.next().await;
    let y = a.next().await;
    assert_eq!((x["data"]["name"].as_str(), y["data"]["name"].as_str()), (Some("B"), Some("B")));
    assert_ne!(x["id"], y["id"]);
    assert_eq!(a.arrays, arrays + 1, "batches due in one tick are one frame");
    let rev_b = x["rev"].as_u64().unwrap();
    assert_ne!(rev_b, rev);
    assert_eq!(b.next().await["rev"], rev_b);

    // The same counters move but the data didn't change: nothing is sent.
    knobs.revisions.send_modify(|r| r.station += 1);
    assert!(b.quiet(300).await);

    // A page that comes back with the rev it has: unchanged, no data.
    b.op(json!({"op": "sub", "id": 8, "feed": "station", "rev": rev_b})).await;
    let u = b.next().await;
    assert_eq!((u["id"].as_u64(), u["unchanged"].as_bool(), u["rev"].as_u64()), (Some(8), Some(true), Some(rev_b)));
    assert!(u.get("data").is_none());
    // An old rev: the full state.
    b.op(json!({"op": "sub", "id": 9, "feed": "station", "rev": rev})).await;
    assert_eq!(b.next().await["data"]["name"], "B");

    // Every subscriber gone: the build stops.
    drop(a);
    drop(b);
    for _ in 0..50 {
        if state.hub.running() == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(state.hub.running(), 0);
}

/// The `screen` feed: screen events with a seq (none lost, a fresh page
/// never runs an old ReloadUI); the subscription is the screen's presence.
#[tokio::test]
async fn the_screen_feed_carries_events_and_counts_as_a_viewer() {
    use crate::LayoutEvent;
    let (state, _tx) = state_with_history(vec![]);
    state.screen_events.push(&LayoutEvent::ReloadUi { screens: None });
    let addr = spawn_test_server(state.clone()).await;
    let mut c = connect(addr).await;
    c.op(json!({"op": "sub", "id": 3, "feed": "screen", "screen": 5})).await;
    assert_eq!(c.any().await["op"], "subbed");
    assert_eq!(state.live_screens.read().unwrap().viewers(5), 1, "subbed = registered");
    let first = c.next().await;
    assert_eq!((first["feed"].as_str(), first["kind"].as_str()), (Some("screen"), Some("samples")));
    assert_eq!(first["items"], json!([]), "the old ReloadUI isn't sent to a fresh page");
    assert_eq!(first["to_seq"], 1);

    state.screen_events.push(&LayoutEvent::Capture { id: 9, screens: vec![5] });
    state.screen_events.push(&LayoutEvent::ViewsChanged);
    let mut items = Vec::new();
    while items.len() < 2 {
        items.extend(c.next().await["items"].as_array().unwrap().clone());
    }
    assert_eq!(items[0]["event"], "capture");
    assert_eq!(items[0]["id"], 9);
    assert_eq!(items[1], json!({"event": "views_changed", "seq": 3}));

    // A page that missed some resumes after its seq.
    c.op(json!({"op": "sub", "id": 4, "feed": "screen", "since": 1, "epoch": crate::epoch()})).await;
    let v = c.next().await;
    assert_eq!(v["recovered"], true);
    assert_eq!(v["items"].as_array().unwrap().len(), 2);

    c.op(json!({"op": "unsub", "id": 3})).await;
    for _ in 0..50 {
        if state.live_screens.read().unwrap().viewers(5) == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(state.live_screens.read().unwrap().viewers(5), 0);
}

/// At most 64 subscriptions per socket; bad params are refused by code.
#[tokio::test]
async fn subscriptions_are_limited_and_params_checked() {
    let (state, _tx) = state_with_history(vec![]);
    let addr = spawn_test_server(state).await;
    let mut c = connect(addr).await;
    c.op(json!({"op": "sub", "id": 1, "feed": "pools"})).await;
    let no = c.any().await;
    assert_eq!((no["op"].as_str(), no["code"].as_str()), (Some("nosub"), Some("E-FEED-003")));
    c.op(json!({"op": "sub", "id": 2, "feed": "pool_editor", "pool": "Group"})).await;
    assert_eq!(c.any().await["code"], "E-FEED-003");
    c.op(json!({"op": "nope"})).await;
    assert_eq!(c.any().await["code"], "E-FEED-007");
    for id in 1..=64 {
        c.op(json!({"op": "sub", "id": id, "feed": "screen"})).await;
    }
    c.op(json!({"op": "sub", "id": 65, "feed": "screen"})).await;
    // Reusing an id replaces: still allowed.
    c.op(json!({"op": "sub", "id": 64, "feed": "screen"})).await;
    let mut refused = Vec::new();
    let mut subbed = 0;
    while subbed < 65 {
        let v = c.any().await;
        match v["op"].as_str() {
            Some("subbed") => subbed += 1,
            Some("nosub") => refused.push((v["id"].as_u64().unwrap(), v["code"].as_str().unwrap().to_string())),
            _ => {}
        }
        if refused.len() == 1 && subbed == 65 {
            break;
        }
    }
    assert_eq!(refused, [(65, "E-FEED-004".to_string())]);
}
