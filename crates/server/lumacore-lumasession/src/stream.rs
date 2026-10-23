// ███████╗████████╗██████╗ ███████╗ █████╗ ███╗   ███╗
// ██╔════╝╚══██╔══╝██╔══██╗██╔════╝██╔══██╗████╗ ████║
// ███████╗   ██║   ██████╔╝█████╗  ███████║██╔████╔██║
// ╚════██║   ██║   ██╔══██╗██╔══╝  ██╔══██║██║╚██╔╝██║
// ███████║   ██║   ██║  ██║███████╗██║  ██║██║ ╚═╝ ██║
// ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝
// FILE: stream

//! The LumaSession stream (decisions §21 "6b hardening"): one TCP
//! connection from each member to its master on port 47658, carrying the
//! join, forwarded commands, results, gap fill and snapshots.
//!
//! Frames are length-prefixed (u32, big endian) zstd-compressed JSON
//! [`StreamMsg`]s. The first frame is the member's [`Hello`] with the stream
//! protocol version; the master answers [`Welcome`] or `Refused`.
//!
//! Threads here only do I/O: everything they learn goes to the state thread
//! as a job (`StateHandle::send` / `call`), where `crate::link` handles it.
//!
//! Owns:          the LumaSession stream: zstd JSON frames (kind + body), Hello / Welcome, the stream server and a member's link thread, heartbeats and loss, files a member lacks.
//! Doesn't own:   what the frames mean on either side (link, election), the state thread (lumacore-lanes).
//! Entry points:  spawn_stream_server, spawn_link, read_frame, write_frame, StreamMsg, Hello.
//! Decisions:     §21 "6b", "6c", §34 19.3.

use std::collections::{BTreeMap, VecDeque};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError};
use lumacore_tree::runtime::{HealthBoard, ThreadGuard};
use lumacore_tree::{LocalSnapshot, OopsKind, ReplicationOp, SubtreeSnapshot, UndoStack};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::election::BEAT;
use super::link::{self, SessionRuntime};
use lumacore_objects::session_link::{LinkHandle, LinkOut, LinkRequest, LinkStats};
use crate::access::Services;
use lumacore_lanes::StateHandle;

use crate::host::SessionHost;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The core's outbox frames (`lumacore_objects::session_link::LinkOut`) on the wire.
impl From<LinkOut> for StreamMsg {
    fn from(out: LinkOut) -> Self {
        match out {
            LinkOut::Command { id, user, line, gesture } => Self::Command { id, user, line, gesture },
            LinkOut::Changes {
                user,
                line,
                undo,
                snapshots,
                local,
            } => Self::Changes {
                user,
                line,
                undo,
                snapshots,
                local,
            },
            LinkOut::Resync { resume } => Self::Resync { resume },
            LinkOut::Logins { users } => Self::Logins { users },
            LinkOut::Claim => Self::Claim,
            LinkOut::Pong { t } => Self::Pong { t },
            LinkOut::Bye => Self::Bye,
            LinkOut::FileWant { files } => Self::FileWant { files },
            LinkOut::Upload { name, data, gdtf } => Self::Upload { name, data, gdtf },
        }
    }
}

/// Frames larger than this are refused (a broken or hostile peer).
const MAX_FRAME: usize = 1 << 30;

/// How long a station waits to connect to its master.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

/// How long the answer to a Hello may take (the master serializes the show).
const WELCOME_TIMEOUT: Duration = Duration::from_secs(30);

/// The same when reconnecting: a master that accepts but doesn't answer
/// (frozen, overloaded) must not hold the election up.
const REWELCOME_TIMEOUT: Duration = Duration::from_secs(3);

/// Wait between connection attempts.
const RETRY_FIRST: Duration = Duration::from_millis(500);

/// A member's first frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hello {
    pub protocol: u32,
    /// The show format this build reads and writes (`FORMAT_VERSION`).
    pub format_version: u32,
    /// The version and commit this build was made from (`-dirty` = uncommitted): a LumaSession
    /// runs one build (decisions §34 "34f7b hardening" 3, E-LUMA-006).
    pub version: String,
    pub commit: String,
    pub station: String,
    /// Its stable id and priority (known stations, election).
    #[serde(default)]
    pub station_id: String,
    #[serde(default)]
    pub priority: lumacore_tree::StationPriority,
    /// The user who asked to join (journal, notes).
    pub user: String,
    /// The member's UDP command port (Dismiss reaches it there) and its own
    /// stream port (a handover makes it master).
    pub cmd_port: u16,
    #[serde(default)]
    pub stream_port: u16,
    /// This join's link (`LinkHandle::link`: fresh per join, kept across
    /// reconnects and a new master). A forwarded command is (station id,
    /// link, command id): what ran is recognised when it is resent.
    pub link: Uuid,
    /// Reconnecting: the journal and sequence it has applied.
    pub resume: Option<(Uuid, u64)>,
    /// Only look: the master answers with a [`Preview`] of its show and
    /// closes (the merge prompt lists what differs, §21 "6d").
    #[serde(default)]
    pub peek: bool,
    /// Join with Merge: the master merges this station's show into its own
    /// before the Welcome.
    #[serde(default)]
    pub merge: Option<MergeOffer>,
}

/// What a station rejoining with Merge brings (§21 "6d").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeOffer {
    /// Its show (`.luma` JSON).
    #[serde(with = "lumacore_base::embedded_json::text")]
    pub show: String,
    /// Its show at the split, and the journal it came from.
    #[serde(with = "lumacore_base::embedded_json::text_opt", default)]
    pub base: Option<String>,
    pub base_journal: Option<Uuid>,
    /// Its own journal (a fenced old master: the session's history started
    /// from it).
    pub journal: Option<Uuid>,
}

/// The master's show for a merge prompt, with its own split base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preview {
    pub master: String,
    #[serde(with = "lumacore_base::embedded_json::text")]
    pub show: String,
    pub journal: Uuid,
    #[serde(with = "lumacore_base::embedded_json::text_opt", default)]
    pub base: Option<String>,
    pub base_journal: Option<Uuid>,
}

/// The master's answer to a Hello, and the full snapshot it sends after a
/// show load or a resync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Welcome {
    /// The master's station name, priority and LumaSession.
    pub master: String,
    #[serde(default)]
    pub master_priority: Option<lumacore_tree::StationPriority>,
    pub lumasession: String,
    pub commit: String,
    pub journal: Uuid,
    pub epoch: u64,
    /// The last sequence the snapshot (or `ops`) covers.
    pub seq: u64,
    /// The whole show as `.luma` JSON, unless the member only needs `ops`.
    #[serde(with = "lumacore_base::embedded_json::text_opt", default)]
    pub show: Option<String>,
    /// Every user's undo history (members mirror it).
    pub undo: BTreeMap<String, UndoStack>,
    /// What a resuming member missed, oldest first.
    pub ops: Vec<ReplicationOp>,
    /// The session-wide runtime (modes, claims, variables, members).
    pub runtime: SessionRuntime,
    /// A join with Merge: what the merge did (one line per item).
    #[serde(default)]
    pub merged: Vec<String>,
    /// The master's command port: members ping it for the session clock
    /// (decisions §24 "Session clock").
    #[serde(default)]
    pub cmd_port: u16,
}

/// Everything that travels on the stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
// Tag, then body: an internally tagged enum buffers its fields and can't carry raw JSON
// (`embedded_json`); written tag first, an adjacently tagged one reads the body straight.
#[serde(tag = "kind", content = "body", rename_all = "snake_case")]
pub enum StreamMsg {
    /// A frame already encoded (length + zstd JSON): the master encodes a broadcast once and
    /// hands every member the same bytes (decisions §31). Local only — never on the wire as
    /// itself.
    #[serde(skip)]
    Encoded(std::sync::Arc<[u8]>),
    Hello(Hello),
    Welcome(Box<Welcome>),
    /// A full snapshot (show load, resync) on a running link.
    Snapshot(Box<Welcome>),
    /// The answer to a peeking Hello.
    Preview(Box<Preview>),
    Refused {
        code: String,
        message: String,
    },
    /// Member → master: run this line as `user` (quietly when it is a gesture's, §30 "12f").
    Command {
        id: u64,
        user: String,
        line: String,
        #[serde(default)]
        gesture: Option<lumacore_objects::gesture::GestureLine>,
    },
    /// Master → member: how a forwarded command ended.
    Reply {
        id: u64,
        ok: bool,
        message: String,
    },
    /// Master → members: a result, in `(epoch, seq)` order.
    Op(Box<ReplicationOp>),
    /// Member → master: the users logged in on the member.
    Logins {
        users: Vec<String>,
    },
    /// Member → master: show changes a station-local action made there
    /// (live screens writing through to the profile, `Store View`, …): the
    /// master files them as `user`. `undo`: None = not undoable.
    Changes {
        user: String,
        line: String,
        undo: Option<OopsKind>,
        snapshots: Vec<SubtreeSnapshot>,
        /// Station settings / watches the step changed on the member.
        #[serde(default)]
        local: Vec<LocalSnapshot>,
    },
    /// Member → master: a gap; send a snapshot (or what it missed).
    Resync {
        resume: Option<(Uuid, u64)>,
    },
    /// Member → master: leaving the session.
    Bye,
    /// Heartbeat (both ways, every 500 ms; `t` = the sender's monotonic ms,
    /// echoed back in the Pong: only the sender reads it).
    Ping { t: u64 },
    Pong { t: u64 },
    /// Member → master: `Claim LumaSession` — hand the master role over.
    Claim,
    /// Master → members: the session's master is now `station` at
    /// `host:port` with `epoch` (a handover, a fenced master, Use Mine);
    /// `autosave`: keep a copy of this station's show first.
    NewMaster {
        station: String,
        host: String,
        port: u16,
        epoch: u64,
        autosave: bool,
    },
    /// Member → master: send these files of the show (decisions §30 "12a2").
    FileWant {
        files: Vec<lumacore_objects::session_link::WantedFile>,
    },
    /// Member → master: a file uploaded on the member for import, base64;
    /// the master puts it into its `library/media/` (§30 "12a2"), a fixture
    /// file (`gdtf`) into its `library/gdtf/` (§34 round 3).
    Upload {
        name: String,
        data: String,
        #[serde(default)]
        gdtf: bool,
    },
    /// Master → member: one file, base64; the member checks its SHA-256 and
    /// keeps it in its store.
    File {
        sha256: String,
        ext: String,
        data: String,
    },
}

/// The file a member asked for, from this station's stores.
fn file_frame(f: &lumacore_objects::session_link::WantedFile) -> Option<StreamMsg> {
    use base64::Engine;
    let bytes = if f.ext == "gdtf" {
        lumacore_luma::show_file::stored_gdtf(&f.sha256)?
    } else {
        lumacore_luma::media::stored(&f.sha256, &f.ext)?
    };
    Some(StreamMsg::File {
        sha256: f.sha256.clone(),
        ext: f.ext.clone(),
        data: base64::engine::general_purpose::STANDARD.encode(bytes),
    })
}

/// Keep a file the master sent, when it is what was asked for.
fn keep_file(sha256: &str, ext: &str, data: &str) -> Result<(), String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| e.to_string())?;
    if lumacore_luma::media::sha256(&bytes) != sha256 {
        return Err(format!("{sha256}.{ext} arrived damaged"));
    }
    if ext == "gdtf" {
        lumacore_luma::show_file::store_gdtf(&bytes).map(|_| ())
    } else if lumacore_tree::media::IMAGE_EXTENSIONS.contains(&ext) {
        lumacore_luma::media::store(&bytes, ext).map(|_| ())
    } else {
        Err(format!("{ext} is not a show file type"))
    }
}

/// One frame's bytes: its length, then the zstd-packed JSON.
pub fn encode_frame(msg: &StreamMsg) -> std::io::Result<std::sync::Arc<[u8]>> {
    if let StreamMsg::Encoded(bytes) = msg {
        return Ok(bytes.clone());
    }
    let json = serde_json::to_vec(msg).map_err(std::io::Error::other)?;
    let packed = zstd::encode_all(&json[..], 3)?;
    let len = u32::try_from(packed.len()).map_err(std::io::Error::other)?;
    let mut frame = Vec::with_capacity(4 + packed.len());
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(&packed);
    Ok(frame.into())
}

/// Write one frame.
pub fn write_frame(w: &mut impl Write, msg: &StreamMsg) -> std::io::Result<()> {
    w.write_all(&encode_frame(msg)?)?;
    w.flush()
}

impl StreamMsg {
    /// An [`StreamMsg::Encoded`] frame read back (tests, local readers); anything else as it is.
    pub fn decoded(self) -> Self {
        match self {
            StreamMsg::Encoded(bytes) => read_frame(&mut &bytes[..]).unwrap_or(StreamMsg::Bye),
            other => other,
        }
    }
}

/// A running link reads a frame in pieces of at most this size and counts
/// each piece as heard, so a big frame (a snapshot, a file) on a slow link
/// doesn't look like silence (R1-23).
const CHUNK: usize = 64 * 1024;

/// The frame's length from its 4-byte header (checked against the cap).
fn frame_len(r: &mut impl Read) -> std::io::Result<usize> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len)?;
    let len = u32::from_be_bytes(len) as usize;
    if len > MAX_FRAME {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("frame of {len} bytes"),
        ));
    }
    Ok(len)
}

fn decode(packed: &[u8]) -> std::io::Result<StreamMsg> {
    let json = zstd::decode_all(packed)?;
    serde_json::from_slice(&json)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn short(got: usize, len: usize) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::UnexpectedEof,
        format!("frame ended after {got} of {len} bytes"),
    )
}

/// Read one frame (blocks). Memory grows with the bytes that arrive, never
/// with what the header claims.
pub fn read_frame(r: &mut impl Read) -> std::io::Result<StreamMsg> {
    let len = frame_len(r)?;
    let mut packed = Vec::new();
    r.take(len as u64).read_to_end(&mut packed)?;
    if packed.len() != len {
        return Err(short(packed.len(), len));
    }
    decode(&packed)
}

/// Read one frame on a running link: in pieces of at most [`CHUNK`], each
/// counted as heard (`stats`), so the heartbeat holds while a big frame is
/// still arriving.
pub fn read_frame_heard(r: &mut impl Read, stats: &LinkStats) -> std::io::Result<StreamMsg> {
    let len = frame_len(r)?;
    stats.heard();
    let mut packed = Vec::new();
    let mut chunk = vec![0u8; CHUNK.min(len)];
    while packed.len() < len {
        let want = (len - packed.len()).min(CHUNK);
        let n = match r.read(&mut chunk[..want]) {
            Ok(0) => return Err(short(packed.len(), len)),
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        packed.extend_from_slice(&chunk[..n]);
        stats.heard();
    }
    decode(&packed)
}

/// A named thread, so it shows by name in profilers, panics and the shutdown log.
pub(super) fn spawn_named(name: &str, f: impl FnOnce() + Send + 'static) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name(name.to_string())
        .spawn(f)
        .unwrap_or_else(|e| panic!("failed to spawn thread '{name}': {e}"))
}

pub fn bind_stream_listener_at(port: u16) -> std::io::Result<TcpListener> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    listener.set_nonblocking(true)?;
    Ok(listener)
}

// *************************************************************************************************
// Master side
// *************************************************************************************************
/// Accept member connections on interfaces that serve LumaSession.
pub fn spawn_stream_server<H: SessionHost>(
    listener: TcpListener,
    state: StateHandle<H>,
    health: Arc<HealthBoard>,
    stop: Arc<AtomicBool>,
    services: Services,
) -> thread::JoinHandle<()> {
    spawn_named("stream-listener", move || {
        let health = health.register("Stream");
        let _alive = ThreadGuard(health.clone());
        while !stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, addr)) => {
                    health.tick(0);
                    if !services.load().lumasession_from(addr.ip()) {
                        continue;
                    }
                    let state = state.clone();
                    spawn_named("stream-peer", move || serve_peer(stream, addr, state));
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    health.heartbeat();
                    thread::sleep(Duration::from_millis(50));
                }
                Err(err) => {
                    log::error!("LumaSession stream listener: {err}");
                    thread::sleep(Duration::from_millis(250));
                }
            }
        }
    })
}

/// One member's connection on the master.
fn serve_peer<H: SessionHost>(mut stream: TcpStream, addr: SocketAddr, state: StateHandle<H>) {
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_nodelay(true);
    let _ = stream.set_read_timeout(Some(WELCOME_TIMEOUT));
    let Ok(StreamMsg::Hello(hello)) = read_frame(&mut stream) else {
        return;
    };
    let station = hello.station.clone();
    if hello.peek {
        if let Ok(preview) = state.call(link::preview) {
            let _ = write_frame(&mut stream, &StreamMsg::Preview(Box::new(preview)));
        }
        return;
    }
    let (tx, rx) = crossbeam_channel::unbounded::<StreamMsg>();
    let stats = Arc::new(LinkStats::default());
    stats.heard();
    let answer = {
        let (tx, stats) = (tx.clone(), stats.clone());
        state.call(move |ctx| link::admit(ctx, hello, addr, tx, stats))
    };
    let Ok((answer, conn)) = answer else {
        return;
    };
    let refused = matches!(answer, StreamMsg::Refused { .. });
    if write_frame(&mut stream, &answer).is_err() || refused {
        if !refused {
            state.send(move |ctx| link::peer_gone(ctx, &station, conn, false));
        }
        return;
    }
    let _ = stream.set_read_timeout(None);

    // Results and replies go out in the order the state thread queued
    // them, with a heartbeat; a silent member is cut off after the timeout.
    let Ok(mut out) = stream.try_clone() else {
        return;
    };
    // A member that stops reading can't hold the writer forever.
    let _ = out.set_write_timeout(Some(lumacore_objects::session_link::lost_after()));
    let writer = {
        let stats = stats.clone();
        spawn_named("stream-peer-out", move || {
            beat_and_send(&mut out, &rx, &stats, None, |_| {});
        })
    };

    let mut bye = false;
    while let Ok(msg) = read_frame_heard(&mut stream, &stats) {
        let from = station.clone();
        match msg {
            StreamMsg::Ping { t } => {
                let _ = tx.send(StreamMsg::Pong { t });
            }
            StreamMsg::Pong { t } => stats.rtt_us.store(
                link_rtt_us(t),
                std::sync::atomic::Ordering::Relaxed,
            ),
            StreamMsg::Command { id, user, line, gesture } => {
                state.send(move |ctx| link::run_forwarded(ctx, &from, conn, id, &user, &line, gesture));
            }
            StreamMsg::Logins { users } => {
                state.send(move |ctx| link::logins(ctx, &from, users));
            }
            StreamMsg::Changes {
                user,
                line,
                undo,
                snapshots,
                local,
            } => {
                state.send(move |ctx| {
                    link::apply_changes(ctx, &from, &user, &line, undo, snapshots, local)
                });
            }
            StreamMsg::Resync { resume } => {
                state.send(move |ctx| link::resync(ctx, &from, conn, resume));
            }
            StreamMsg::Claim => {
                state.send(move |ctx| super::election::hand_over(ctx, &from));
            }
            // Read here, off the state thread; sent in the queue's order.
            StreamMsg::FileWant { files } => {
                for f in &files {
                    if let Some(frame) = file_frame(f) {
                        let _ = tx.send(frame);
                    }
                }
            }
            // Written here, before the forwarded Import line (later on this
            // stream) reaches the state thread.
            StreamMsg::Upload { name, data, gdtf } => {
                use base64::Engine;
                let saved = base64::engine::general_purpose::STANDARD
                    .decode(&data)
                    .map_err(|e| e.to_string())
                    .and_then(|bytes| {
                        if gdtf {
                            lumacore_objects::commands::gdtf_library::save_upload(&name, &bytes)
                        } else {
                            lumacore_luma::media::save_upload(&name, &bytes)
                        }
                    });
                if let Err(e) = saved {
                    let note = format!("upload {name} from {from}: {e}");
                    state.send(move |ctx| link::note_link(ctx, &note));
                }
            }
            StreamMsg::Bye => {
                bye = true;
                break;
            }
            _ => {}
        }
    }
    let _ = stream.shutdown(std::net::Shutdown::Both);
    drop(tx);
    state.send(move |ctx| link::peer_gone(ctx, &station, conn, bye));
    let _ = writer.join();
}

/// Round trip of a heartbeat sent at `t` (this station's monotonic ms,
/// `session_link::link_ms`), in µs.
fn link_rtt_us(t: u64) -> u64 {
    lumacore_objects::session_link::link_ms().saturating_sub(t) * 1000
}

/// Write queued messages and a heartbeat every [`BEAT`]; stop when the
/// other side has been silent for the loss timeout (the socket is shut, so
/// the reader ends too), when the channel closes, or `stop` / `broken` is
/// set. `on_send` sees each message before it is written.
fn beat_and_send<M: Into<StreamMsg>>(
    out: &mut TcpStream,
    rx: &Receiver<M>,
    stats: &LinkStats,
    stop: Option<(&AtomicBool, &AtomicBool)>,
    mut on_send: impl FnMut(&StreamMsg),
) {
    let mut last_beat = std::time::Instant::now() - BEAT;
    loop {
        if stop.is_some_and(|(stop, broken)| {
            stop.load(Ordering::Relaxed) || broken.load(Ordering::Relaxed)
        }) {
            break;
        }
        if stats.silent_for() >= lumacore_objects::session_link::lost_after() {
            let _ = out.shutdown(std::net::Shutdown::Both);
            break;
        }
        if last_beat.elapsed() >= BEAT {
            last_beat = std::time::Instant::now();
            if write_frame(out, &StreamMsg::Ping { t: lumacore_objects::session_link::link_ms() }).is_err() {
                let _ = out.shutdown(std::net::Shutdown::Both);
                break;
            }
        }
        let msg = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(msg) => msg.into(),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        on_send(&msg);
        if write_frame(out, &msg).is_err() {
            let _ = out.shutdown(std::net::Shutdown::Both);
            break;
        }
        if matches!(msg, StreamMsg::Bye) {
            break;
        }
    }
}

// *************************************************************************************************
// Member side
// *************************************************************************************************
/// Start the link to a master: connect, Hello, take the show, then carry
/// commands out and results in until `Leave LumaSession`. A lost connection
/// is retried with back-off; unanswered commands are resent (the pending
/// outbox). The first connection failing ends the join.
pub fn spawn_link<H: SessionHost>(
    state: StateHandle<H>,
    request: LinkRequest,
    hello: Hello,
    handle: LinkHandle,
    rx: Receiver<LinkOut>,
) {
    spawn_named("lumasession-link", move || {
        run_link(state, request, hello, handle, rx);
    });
}

/// Read a master's show without joining (the merge prompt, §21 "6d").
pub fn peek(target: &str, hello: Hello) -> Result<Preview, String> {
    let mut stream = connect(target).map_err(|e| format!("{target}: {e}"))?;
    let _ = stream.set_read_timeout(Some(WELCOME_TIMEOUT));
    write_frame(&mut stream, &StreamMsg::Hello(hello)).map_err(|e| e.to_string())?;
    match read_frame(&mut stream).map_err(|e| e.to_string())? {
        StreamMsg::Preview(p) => Ok(*p),
        StreamMsg::Refused { message, .. } => Err(message),
        _ => Err(format!("{target} answered something else")),
    }
}

fn connect(target: &str) -> std::io::Result<TcpStream> {
    let mut last = std::io::Error::other(format!("can't resolve {target}"));
    for addr in target.to_socket_addrs()? {
        match TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT) {
            Ok(stream) => return Ok(stream),
            Err(e) => last = e,
        }
    }
    Err(last)
}

/// What a member does after its connection to the master broke.
enum AfterLoss {
    /// Try the same master again.
    Retry,
    /// Connect to this master instead (the election's winner).
    Follow { target: String, master: String },
    /// This station is master or IdleMaster now: the link ends.
    Done,
}

/// The connection broke: within the loss timeout keep trying the master;
/// after it, elect (§21 "6c"). A winner that can't be reached for the loss
/// timeout is dropped and the election runs again without it.
fn after_loss<H: SessionHost>(
    state: &StateHandle<H>,
    handle: &LinkHandle,
    excluded: &mut Vec<String>,
    chasing: &mut Option<(String, std::time::Instant)>,
) -> AfterLoss {
    if handle.stats.silent_for() < lumacore_objects::session_link::lost_after() {
        return AfterLoss::Retry;
    }
    if let Some((winner, since)) = chasing.as_ref()
        && since.elapsed() >= lumacore_objects::session_link::lost_after()
    {
        excluded.push(winner.clone());
        *chasing = None;
    }
    let ex = excluded.clone();
    let lost = handle.master.clone();
    let Ok(elected) = state.call(move |ctx| {
        ctx.state().show_status = lumacore_objects::session_link::ShowStatus::WaitingForNewMaster;
        super::election::elect(ctx, &ex)
    }) else {
        return AfterLoss::Done;
    };
    match elected {
        super::election::Elected::Me => {
            let why = format!("{lost} was lost and this station won the election");
            state.send(move |ctx| super::election::become_master(ctx, None, &why));
            AfterLoss::Done
        }
        super::election::Elected::Alone => {
            state.send(move |ctx| super::election::become_idle(ctx, &lost));
            AfterLoss::Done
        }
        super::election::Elected::Other { name, target } => {
            if chasing.as_ref().is_none_or(|(w, _)| *w != name) {
                *chasing = Some((name.clone(), std::time::Instant::now()));
                let note = format!("{lost} was lost: {name} won the election, following it.");
                state.send(move |ctx| super::link::note_link(ctx, &note));
            }
            if target == handle.target {
                AfterLoss::Retry
            } else {
                AfterLoss::Follow {
                    target,
                    master: name,
                }
            }
        }
    }
}

fn run_link<H: SessionHost>(
    state: StateHandle<H>,
    request: LinkRequest,
    hello: Hello,
    mut handle: LinkHandle,
    rx: Receiver<LinkOut>,
) {
    // Commands written but not answered yet, oldest first: resent after a
    // reconnect (to the same master or the next one).
    let unconfirmed: Arc<Mutex<VecDeque<StreamMsg>>> = Arc::default();
    let mut first = true;
    let started = std::time::Instant::now();
    let mut excluded: Vec<String> = Vec::new();
    let mut chasing: Option<(String, std::time::Instant)> = None;
    handle.stats.heard();
    while !handle.stop.load(Ordering::Relaxed) {
        // A first join gives up at once, unless it is patient (a handover
        // or rejoin: the master may still be taking over).
        let give_up = first
            && (!request.patient || started.elapsed() >= lumacore_objects::session_link::lost_after());
        let fail = |message: String| {
            let request = request.clone();
            state.send(move |ctx| link::join_failed(ctx, &request, &message));
        };
        let lost = |handle: &mut LinkHandle,
                    excluded: &mut Vec<String>,
                    chasing: &mut Option<(String, std::time::Instant)>|
         -> bool {
            match after_loss(&state, handle, excluded, chasing) {
                AfterLoss::Retry => {
                    thread::sleep(RETRY_FIRST);
                    true
                }
                AfterLoss::Follow { target, master } => {
                    handle.target = target;
                    handle.master = master;
                    true
                }
                AfterLoss::Done => false,
            }
        };
        let mut stream = match connect(&handle.target) {
            Ok(stream) => stream,
            Err(e) if give_up => {
                fail(format!("can't reach {}: {e}", handle.target));
                return;
            }
            Err(_) if first => {
                thread::sleep(RETRY_FIRST);
                continue;
            }
            Err(_) => {
                if lost(&mut handle, &mut excluded, &mut chasing) {
                    continue;
                }
                return;
            }
        };
        let _ = stream.set_nodelay(true);
        let _ = stream.set_read_timeout(Some(if first {
            WELCOME_TIMEOUT
        } else {
            REWELCOME_TIMEOUT
        }));
        let resume = if first {
            None
        } else {
            state.call(|ctx| ctx.state().last_applied).ok().flatten()
        };
        // Merge happens once: a reconnect resumes the merged session.
        let greeting = StreamMsg::Hello(Hello {
            resume,
            merge: if first { hello.merge.clone() } else { None },
            ..hello.clone()
        });
        let answer = write_frame(&mut stream, &greeting).and_then(|_| read_frame(&mut stream));
        let welcome = match answer {
            Ok(StreamMsg::Welcome(welcome)) => welcome,
            Ok(StreamMsg::Refused { code, message }) if give_up => {
                fail(lumacore_objects::errors::with_code(message, &code));
                return;
            }
            Ok(_) | Err(_) if first => {
                thread::sleep(RETRY_FIRST);
                continue;
            }
            Ok(_) | Err(_) => {
                if lost(&mut handle, &mut excluded, &mut chasing) {
                    continue;
                }
                return;
            }
        };
        // The show is checked here, off the state thread: a show this build
        // can't read leaves the station as it was (atomic join).
        let show = match welcome.show.as_deref().map(lumacore_objects::parse_show) {
            Some(Err(e)) => {
                fail(e);
                return;
            }
            Some(Ok(show)) => Some(show),
            None => None,
        };
        let _ = stream.set_read_timeout(None);
        excluded.clear();
        chasing = None;
        handle.stats.heard();
        handle.master = welcome.master.clone();
        handle.master_priority = welcome.master_priority;
        if let Ok(mut slot) = handle.socket.lock() {
            *slot = stream.try_clone().ok();
        }
        {
            let request = request.clone();
            let handle = handle.clone();
            let was_first = first;
            state.send(move |ctx| link::install(ctx, *welcome, show, handle, was_first, &request));
        }
        first = false;

        // Resend what no master has answered, then carry new messages.
        let broken = Arc::new(AtomicBool::new(false));
        let resend: Vec<StreamMsg> = unconfirmed
            .lock()
            .map(|q| q.iter().cloned().collect())
            .unwrap_or_default();
        if !resend.is_empty() {
            handle
                .stats
                .resends
                .fetch_add(resend.len() as u64, Ordering::Relaxed);
        }
        // A master that stops reading can't hold the writer forever (the
        // socket's timeout covers the resend and the writer's clone).
        let _ = stream.set_write_timeout(Some(lumacore_objects::session_link::lost_after()));
        if resend.iter().any(|m| write_frame(&mut stream, m).is_err()) {
            broken.store(true, Ordering::Relaxed);
        }
        let writer = {
            let Ok(mut out) = stream.try_clone() else {
                continue;
            };
            let _ = out.set_write_timeout(Some(lumacore_objects::session_link::lost_after()));
            let rx = rx.clone();
            let stop = handle.stop.clone();
            let broken = broken.clone();
            let unconfirmed = unconfirmed.clone();
            let stats = handle.stats.clone();
            spawn_named("lumasession-link-out", move || {
                beat_and_send(&mut out, &rx, &stats, Some((&stop, &broken)), |msg| {
                    if matches!(msg, StreamMsg::Command { .. })
                        && let Ok(mut q) = unconfirmed.lock()
                    {
                        q.push_back(msg.clone());
                    }
                });
                broken.store(true, Ordering::Relaxed);
            })
        };

        // A master that hands over or is fenced names the next one.
        let mut moved: Option<(String, String)> = None;
        while let Ok(msg) = read_frame_heard(&mut stream, &handle.stats) {
            match msg {
                StreamMsg::Ping { t } => {
                    let _ = handle.tx.send(LinkOut::Pong { t });
                }
                StreamMsg::Pong { t } => handle
                    .stats
                    .rtt_us
                    .store(link_rtt_us(t), Ordering::Relaxed),
                StreamMsg::Reply { id, ok, message } => {
                    if let Ok(mut q) = unconfirmed.lock() {
                        q.retain(|m| !matches!(m, StreamMsg::Command { id: sent, .. } if *sent == id));
                    }
                    state.send(move |ctx| link::deliver(ctx, id, ok, message));
                }
                StreamMsg::Op(op) => {
                    state.send(move |ctx| link::apply_op(ctx, *op));
                }
                // Written here, off the state thread.
                StreamMsg::File { sha256, ext, data } => {
                    if let Err(e) = keep_file(&sha256, &ext, &data) {
                        state.send(move |ctx| link::note_link(ctx, &e));
                    }
                }
                StreamMsg::Snapshot(welcome) => {
                    let show = match welcome.show.as_deref().map(lumacore_objects::parse_show) {
                        Some(Err(e)) => {
                            state.send(move |ctx| link::note_link(ctx, &e));
                            continue;
                        }
                        Some(Ok(show)) => Some(show),
                        None => None,
                    };
                    let request = request.clone();
                    let handle = handle.clone();
                    state.send(move |ctx| link::install(ctx, *welcome, show, handle, false, &request));
                }
                StreamMsg::NewMaster {
                    station,
                    host,
                    port,
                    epoch,
                    autosave,
                } => {
                    if station == hello.station {
                        let why = format!("{} handed the master role over", handle.master);
                        state.send(move |ctx| super::election::become_master(ctx, Some(epoch), &why));
                        handle.stop.store(true, Ordering::Relaxed);
                    } else {
                        let (from, to) = (handle.master.clone(), station.clone());
                        state.send(move |ctx| link::moved(ctx, &from, &to, autosave));
                        moved = Some((format!("{host}:{port}"), station));
                    }
                    break;
                }
                _ => {}
            }
        }
        broken.store(true, Ordering::Relaxed);
        let _ = stream.shutdown(std::net::Shutdown::Both);
        let _ = writer.join();
        if handle.stop.load(Ordering::Relaxed) {
            break;
        }
        if let Some((target, master)) = moved {
            handle.target = target;
            handle.master = master;
            handle.stats.heard();
            continue;
        }
        state.send(link::link_lost);
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
