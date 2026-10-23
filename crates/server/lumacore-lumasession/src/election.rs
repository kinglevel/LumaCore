// ███████╗██╗     ███████╗ ██████╗████████╗██╗ ██████╗ ███╗   ██╗
// ██╔════╝██║     ██╔════╝██╔════╝╚══██╔══╝██║██╔═══██╗████╗  ██║
// █████╗  ██║     █████╗  ██║        ██║   ██║██║   ██║██╔██╗ ██║
// ██╔══╝  ██║     ██╔══╝  ██║        ██║   ██║██║   ██║██║╚██╗██║
// ███████╗███████╗███████╗╚██████╗   ██║   ██║╚██████╔╝██║ ╚████║
// ╚══════╝╚══════╝╚══════╝ ╚═════╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: election

//! LumaSession health, master loss and election (decisions §10, §21 "6c
//! hardening").
//!
//! - **Heartbeat** (`crate::stream`): every 500 ms both ways; 1.5 s without
//!   one warns, 5 s means the other side is lost.
//! - **Master lost → election first:** every member computes the same
//!   winner from the replicated member list and what discovery still sees
//!   ([`elect`]): priority → most results applied → online time (whole 5 s buckets) →
//!   lowest station id; `Never` can't win. The winner becomes master with a
//!   new epoch ([`become_master`]), the others move to it and resend their
//!   pending commands. Only a station with nobody to reach becomes
//!   **IdleMaster** ([`become_idle`]): it runs its pending commands itself.
//! - **Two masters** (old master back, split healed): the lower epoch is
//!   fenced at once (no DMX) and its operator decides — Use Master / Use
//!   Mine / Cancel ([`tick`]).
//! - **Known stations**: every station that ever joined is in the show and
//!   is re-invited when it comes back.
//!
//! Owns:          SessionWatch, the election (elect, best, ranks_above), becoming master or idle, fencing, the rejoin decision, re-invites, member health.
//! Doesn't own:   the link and the stream (link, stream), discovery (the server's threads).
//! Entry points:  elect, tick, become_master, become_idle, hand_over, invited_back, SessionWatch.
//! Decisions:     §21 "6c", "6d", §34 19.3.

use std::collections::{BTreeMap, VecDeque};
use std::time::{Duration, Instant};

use lumacore_tree::{KnownStation, StationPriority};

use crate::host::SessionHost;
use lumacore_objects::session_link::{
    LinkRequest, ReplyTo, Role, SessionRequest, ShowStatus, SplitBase, WARN, lost_after,
};
use super::stream::{Hello, StreamMsg};
use lumacore_objects::AppState;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Heartbeat interval on the stream.
pub const BEAT: Duration = Duration::from_millis(500);
/// Online times compare in buckets of this size in the election.
const ONLINE_TIE_MS: u64 = 5000;
/// Re-invites: at most one per station this often, and no more than
/// [`REINVITE_MAX`] a minute (then the master stops trying).
const REINVITE_EVERY: Duration = Duration::from_secs(10);
const REINVITE_MAX: usize = 5;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Keep the show as it is now as the split base (memory and an autosave).
fn keep_split_base(state: &mut AppState) {
    let (journal, seq) = state.last_applied.map_or((None, 0), |(j, s)| (Some(j), s));
    let show = std::sync::Arc::new(state.root.show_data.clone());
    // The base is kept in memory either way (nothing is replaced here),
    // and noted on disk so a restart can still Merge (R1-22).
    let base = SplitBase { show: show.clone(), journal, seq };
    match lumacore_objects::session_link::autosave_show(state, &show, "split-base") {
        Ok(path) => lumacore_objects::session_link::remember_split_base(&path, &base),
        Err(e) => note(state, format!("[ERROR] The split base couldn't be saved ({e}); it is kept in memory only.")),
    }
    state.split_base = Some(base);
}

/// The other master of a two-masters meeting (or the session an
/// IdleMaster found again).
#[derive(Debug, Clone)]
pub struct Rival {
    pub name: String,
    pub ip: String,
    pub cmd_port: u16,
    pub stream_port: u16,
    pub epoch: u64,
}

/// Election state kept by the server (not replicated).
#[derive(Debug, Default)]
pub struct SessionWatch {
    pub last_tick: Option<Instant>,
    /// The other master / the session found again.
    pub rival: Option<Rival>,
    /// Stations that left on purpose: not re-invited until they join again.
    pub left: std::collections::BTreeSet<String>,
    /// Re-invites sent, per station id.
    pub reinvites: BTreeMap<String, VecDeque<Instant>>,
    /// Warnings already given (so each is said once).
    pub warned_late: bool,
    pub warned_link_down: Option<String>,
    pub noted_rival: Option<String>,
    pub next_decision: u64,
    /// When the decision's countdown runs out (decision id, monotonic): the
    /// default runs then. `Decision::deadline_ms` (unix) is only what the
    /// clients count down on (R1-02: a wall-clock step moves neither).
    pub decision_due: Option<(u64, Instant)>,
    /// The session's show read for the decision (decision id, preview, this
    /// station's change count when the list was made): the list follows
    /// changes made here while the prompt waits.
    pub preview: Option<(u64, super::stream::Preview, u64)>,
}

fn note(state: &mut AppState, message: impl Into<String>) {
    state
        .sysmon_pending
        .push(("LumaSession".to_string(), message.into()));
}

/// Tell everyone logged in on this station (every Shell) and the log.
fn announce<H: SessionHost>(ctx: &mut H, text: &str) {
    note(ctx.state(), text.to_string());
    for user in ctx.shell_users() {
        ctx
            .push_output(&user, &lumacore_objects::text::render_warning(text));
    }
}

fn unix_ms() -> u64 {
    lumacore_tree::now_ms()
}

// *************************************************************************************************
// Ranking
// *************************************************************************************************
/// A station that could be master.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub name: String,
    pub station_id: String,
    pub priority: StationPriority,
    pub applied: u64,
    pub online_since_ms: u64,
    pub ip: String,
    pub stream_port: u16,
    pub me: bool,
}

/// The candidate every member picks (decisions §21 "6c"): whatever order
/// they are listed in, the same one.
pub fn best(all: &[Candidate]) -> Option<&Candidate> {
    all.iter().fold(None::<&Candidate>, |best, c| match best {
        Some(b) if ranks_above(b, c) => Some(b),
        _ => Some(c),
    })
}

/// Whether `a` should be master rather than `b`: priority → most results
/// applied → longer online, compared in whole 5 s buckets → lowest station
/// id. Buckets (not "within ±5 s") keep the order transitive, so every
/// member picks the same winner whatever order it lists them in (decisions
/// §29).
pub fn ranks_above(a: &Candidate, b: &Candidate) -> bool {
    if a.priority != b.priority {
        return a.priority > b.priority;
    }
    if a.applied != b.applied {
        return a.applied > b.applied;
    }
    let (bucket_a, bucket_b) = (a.online_since_ms / ONLINE_TIE_MS, b.online_since_ms / ONLINE_TIE_MS);
    if bucket_a != bucket_b {
        return bucket_a < bucket_b;
    }
    a.station_id < b.station_id
}

fn me(state: &AppState) -> Candidate {
    Candidate {
        name: state.root.station_settings.station_name.clone(),
        station_id: state.local_settings.station_id.clone(),
        priority: state.root.station_settings.station_priority,
        applied: state.last_applied.map_or(0, |(_, seq)| seq),
        online_since_ms: state.online_since_ms,
        ip: String::new(),
        stream_port: state.lumasession_ports.2,
        me: true,
    }
}

/// Stations discovery sees now, by station name.
fn seen<H: SessionHost>(ctx: &H) -> Vec<lumacore_net::DiscoveryPeerState> {
    ctx.discovery_cache()
        .lock()
        .map(|cache| cache.values().cloned().collect())
        .unwrap_or_default()
}

fn candidate(peer: &lumacore_net::DiscoveryPeerState) -> Candidate {
    Candidate {
        name: peer.station_name.clone(),
        station_id: peer.station_id.clone(),
        priority: peer.priority.parse().unwrap_or_default(),
        applied: peer.applied,
        online_since_ms: peer.online_since_ms,
        ip: peer.ip.to_string(),
        stream_port: peer.stream_port,
        me: false,
    }
}

/// The outcome of an election as this station computes it.
#[derive(Debug, Clone, PartialEq)]
pub enum Elected {
    /// This station wins.
    Me,
    /// Another one: connect to it.
    Other { name: String, target: String },
    /// Nobody else to reach.
    Alone,
}

/// Compute the winner among this station and the session's other members
/// that discovery still sees (`excluded`: winners that couldn't be reached).
pub fn elect<H: SessionHost>(ctx: &H, excluded: &[String]) -> Elected {
    let state = &ctx.state_ref();
    log::info!(
        "election on {}: members {:?}, seen {:?}, excluded {excluded:?}",
        state.root.station_settings.station_name,
        state.root.runtime.luma_sessions.members.keys().collect::<Vec<_>>(),
        seen(ctx)
            .iter()
            .map(|p| format!("{}({}, port {})", p.station_name, p.status, p.stream_port))
            .collect::<Vec<_>>()
    );
    let mine = me(state);
    let members = &state.root.runtime.luma_sessions.members;
    let lost_master = state.link.as_ref().map(|l| l.master.clone()).unwrap_or_default();
    let others: Vec<Candidate> = seen(ctx)
        .iter()
        .filter(|p| members.contains_key(&p.station_name))
        .filter(|p| p.station_name != mine.name && p.station_name != lost_master)
        .filter(|p| !excluded.contains(&p.station_name))
        .filter(|p| p.stream_port != 0)
        .map(candidate)
        .collect();
    if others.is_empty() {
        return Elected::Alone;
    }
    let mut all = others;
    if mine.priority != StationPriority::Never {
        all.push(mine);
    }
    all.retain(|c| c.priority != StationPriority::Never);
    let Some(best) = best(&all) else {
        return Elected::Alone;
    };
    if best.me {
        Elected::Me
    } else {
        Elected::Other {
            name: best.name.clone(),
            target: format!("{}:{}", best.ip, best.stream_port),
        }
    }
}

// *************************************************************************************************
// Becoming master / IdleMaster
// *************************************************************************************************
/// Run the commands still pending here, as their users (this station is
/// its own master now).
fn run_pending_locally<H: SessionHost>(ctx: &mut H) {
    let pending = std::mem::take(&mut ctx.state().forwards);
    for (_, forward) in pending {
        let text = if forward.ran {
            // Its result already arrived from the old master: only the
            // answer was lost (R1-21). Never run twice.
            lumacore_objects::text::render_success(lumacore_objects::session_link::ANSWER_LOST)
        } else {
            let (reply, _) = ctx.execute_remote_command(&forward.user, &forward.raw);
            if reply.ok {
                reply.message
            } else {
                lumacore_objects::errors::reply(&reply.message, "E-CMD-011")
            }
        };
        match &forward.reply_to {
            ReplyTo::Shell(user) => {
                ctx.push_output(user, &text);
            }
            ReplyTo::Waiter(tx) => {
                let _ = tx.send(text);
            }
            ReplyTo::Log => note(ctx.state(), lumacore_objects::text::strip_ansi(&text)),
        }
    }
}

/// This station is the session's master now (`epoch`: the new one; None =
/// one past the epoch it followed).
pub fn become_master<H: SessionHost>(ctx: &mut H, epoch: Option<u64>, why: &str) {
    let state = &mut ctx.state();
    // Taking over from a master: its show up to here is what a returning
    // old master merges from.
    if state.link.is_some() {
        keep_split_base(state);
    }
    if let Some(link) = state.link.take() {
        link.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    let epoch = epoch.unwrap_or(state.session_epoch + 1);
    ctx.session().journal.restart();
    ctx.session().journal.set_epoch(epoch);
    let state = &mut ctx.state();
    state.session_epoch = epoch;
    state.root.runtime.joined_sessions.clear();
    let me = state.root.station_settings.station_name.clone();
    let members = &mut state.root.runtime.luma_sessions.members;
    members.remove(&me);
    for member in members.values_mut() {
        member.connected = false;
    }
    state.is_authority = true;
    state.session_role = Role::Normal;
    state.decision = None;
    state.show_status = lumacore_objects::session_link::ShowStatus::Loaded;
    state.protocols_dirty = true;
    ctx.session().sent_runtime = None;
    ctx.session().watch.rival = None;
    let name = ctx.state().root.station_settings.lumasession.clone();
    announce(
        ctx,
        &format!("{me} is now master of LumaSession \"{name}\" (epoch {epoch}): {why}."),
    );
    run_pending_locally(ctx);
    ctx.refresh_runtime();
}

/// Nobody to reach: this station is its own master (IdleMaster).
pub fn become_idle<H: SessionHost>(ctx: &mut H, lost: &str) {
    let state = &mut ctx.state();
    // Before the pending commands run here: the show both sides had.
    keep_split_base(state);
    if let Some(link) = state.link.take() {
        link.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    state.root.runtime.joined_sessions.clear();
    state.root.runtime.luma_sessions.members.clear();
    state.is_authority = true;
    state.session_role = Role::IdleMaster {
        diverged: false,
        lost: lost.to_string(),
        quiet: false,
    };
    state.show_status = ShowStatus::Loaded;
    state.protocols_dirty = true;
    state.root.runtime.journal.record(lumacore_tree::runtime::JournalEntry {
        time_ms: unix_ms(),
        epoch: state.session_epoch,
        seq: 0,
        user: String::new(),
        station: state.root.station_settings.station_name.clone(),
        command: format!("— split from {lost} —"),
        changed: Vec::new(),
        result: lumacore_tree::runtime::JournalResult::Ok,
    });
    let output = if state.root.station_settings.output_when_alone {
        "output continues (output when alone)"
    } else {
        "no DMX output (Set Station \"OutputWhenAlone\" On to send)"
    };
    announce(
        ctx,
        &format!("Disconnected from {lost}: this station is IdleMaster, {output}."),
    );
    run_pending_locally(ctx);
    ctx.refresh_runtime();
}

// *************************************************************************************************
// Commands
// *************************************************************************************************
// *************************************************************************************************
// Master side
// *************************************************************************************************
/// Remember a station that joined (show data: replicated, not undoable).
pub fn remember_station<H: SessionHost>(ctx: &mut H, known: KnownStation) {
    ctx.session().watch.left.remove(&known.station_id);
    ctx.session().watch.reinvites.remove(&known.station_id);
    let state = &mut ctx.state();
    let stations = &state.root.show_data.session.stations;
    let unchanged = stations.get(&known.station_id).is_some_and(|k| {
        k.meta.name() == known.meta.name() && k.ip == known.ip && k.stream_port == known.stream_port
    });
    if unchanged {
        return;
    }
    let label = format!("Station \"{}\" joined", known.meta.name().unwrap_or_default());
    lumacore_objects::commands::record_change(
        state,
        &label,
        lumacore_objects::commands::registry::Undo::Untracked,
        |state| {
            let joined = state
                .root
                .show_data
                .session
                .stations
                .get(&known.station_id)
                .map_or(known.joined_unix_ms, |k| k.joined_unix_ms);
            state.root.show_data.session.stations.insert(
                known.station_id.clone(),
                KnownStation {
                    joined_unix_ms: joined,
                    ..known
                },
            );
        },
    );
}

/// A member asked for the master role: hand over (new epoch) and follow it.
pub fn hand_over<H: SessionHost>(ctx: &mut H, station: &str) {
    let Some(member) = ctx.state()
        .root
        .runtime
        .luma_sessions
        .members
        .get(station)
        .cloned()
    else {
        return;
    };
    super::link::flush(ctx);
    let epoch = ctx.session().journal.epoch() + 1;
    let msg = StreamMsg::NewMaster {
        station: station.to_string(),
        host: member.ip.clone(),
        port: member.stream_port,
        epoch,
        autosave: false,
    };
    for peer in ctx.session().peers.values() {
        let _ = peer.tx.send(msg.clone());
    }
    follow(ctx, &member.ip, member.stream_port, epoch, &format!("handed over to {station}"));
}

/// Stop being master and join `host:port` (the new master).
fn follow<H: SessionHost>(ctx: &mut H, host: &str, port: u16, epoch: u64, why: &str) {
    ctx.session().peers.clear();
    let state = &mut ctx.state();
    state.root.runtime.luma_sessions.members.clear();
    state.session_epoch = epoch;
    state.session_role = Role::Normal;
    state.decision = None;
    state.show_status = ShowStatus::WaitingForNewMaster;
    state.protocols_dirty = true;
    let user = super::station_user(state);
    state.link_requests.push(LinkRequest {
        host: host.to_string(),
        port,
        user,
        reply_to: ReplyTo::Log,
        invited_by: None,
        patient: true,
        merge: false,
    });
    note(state, format!("Following the new master at {host}:{port}: {why}."));
}

/// `RemoteRequest::Yield` on a master: the other side chose Use Mine. Save
/// this show, send the members along and join the claimant.
pub fn handle_yield<H: SessionHost>(
    ctx: &mut H,
    from: String,
    epoch: u64,
    stream_port: u16,
) -> lumacore_wire::RemoteReply {
    let Some(src) = ctx.request_from().map(|a| a.ip().to_string()) else {
        return ctx.make_reply(false, "yield without a sender".into());
    };
    if ctx.state().link.is_some() {
        return ctx.make_reply(false, "not a master".into());
    }
    // Taking the claimant's show replaces this one: only with a copy kept
    // (R1-16). Refused, this master stays; the claimant's higher epoch
    // fences it and its operator decides.
    let saved = match lumacore_objects::session_link::autosave_copy(ctx.state(), "use-mine") {
        Ok(saved) => saved,
        Err(e) => {
            let me = ctx.state().root.station_settings.station_name.clone();
            note(
                ctx.state(),
                format!("[ERROR] {from} chose Use Mine, but this show's unsaved changes couldn't be kept ({e}): not following, nothing was replaced."),
            );
            return ctx.make_reply(false,
                lumacore_objects::errors::with_code(
                    format!("{me} cannot keep its unsaved changes ({e}); nothing was replaced"),
                    "E-CMD-012",
                ),
            );
        }
    };
    let msg = StreamMsg::NewMaster {
        station: from.clone(),
        host: src.clone(),
        port: stream_port,
        epoch,
        autosave: true,
    };
    for peer in ctx.session().peers.values() {
        let _ = peer.tx.send(msg.clone());
    }
    let kept = saved.map(|p| format!(" (this show kept in {p})")).unwrap_or_default();
    announce(
        ctx,
        &format!("{from} chose Use Mine: taking its show{kept}."),
    );
    follow(ctx, &src, stream_port, epoch, &format!("{from} kept its show"));
    ctx.make_reply(true, "following".into())
}

/// Ask the operator (the session data merge pop-up): Merge / Use Master
/// / Use Mine / Cancel, with the show's default and timeout. The session's
/// show is read in the background to list what differs (§21 "6d").
fn decision<H: SessionHost>(ctx: &mut H, rival: &Rival, title: &str, message: String) {
    decision_with(ctx, rival, title, message, None);
}

/// [`decision`]; `no_merge`: Merge greyed out with this reason, Use Mine the
/// default and no preview read.
fn decision_with<H: SessionHost>(ctx: &mut H, rival: &Rival, title: &str, message: String, no_merge: Option<String>) {
    ctx.session().watch.next_decision += 1;
    let id = ctx.session().watch.next_decision;
    let (state, session) = ctx.split();
    let settings = &state.root.show_data.show_settings;
    let target = format!("{}:{}", rival.ip, rival.stream_port);
    let deadline_ms = settings
        .merge_timeout
        .map(|s| unix_ms() + u64::from(s) * 1000);
    session.watch.decision_due = settings
        .merge_timeout
        .map(|s| (id, Instant::now() + Duration::from_secs(u64::from(s))));
    state.decision = Some(lumacore_wire::Decision {
        id,
        title: title.to_string(),
        message,
        options: vec![
            (
                "Merge".into(),
                if no_merge.is_some() { String::new() } else { format!("Join LumaSession {target} /Merge") },
            ),
            ("Use Master".into(), format!("Join LumaSession {target}")),
            ("Use Mine".into(), "Claim LumaSession".into()),
            ("Cancel".into(), "Leave LumaSession".into()),
        ],
        details: no_merge.iter().map(|r| format!("Merge isn't possible: {r}")).collect(),
        merge_blocked: no_merge.clone(),
        default: if no_merge.is_some() { "Use Mine".to_string() } else { settings.merge_default.word().to_string() },
        deadline_ms,
        paused: false,
    });
    ctx.session().watch.rival = Some(rival.clone());
    if no_merge.is_some() {
        return;
    }
    let Some(handle) = ctx.state_handle().clone() else {
        return;
    };
    let me = Hello {
        protocol: lumacore_wire::SESSION_PROTOCOL,
        format_version: lumacore_tree::FORMAT_VERSION,
        version: lumacore_objects::build_info().version,
        commit: lumacore_objects::build_info().git_commit,
        station: ctx.state().root.station_settings.station_name.clone(),
        station_id: ctx.state().local_settings.station_id.clone(),
        priority: ctx.state().root.station_settings.station_priority,
        user: String::new(),
        cmd_port: ctx.cmd_port(),
        stream_port: ctx.stream_port(),
        link: uuid::Uuid::nil(),
        resume: None,
        peek: true,
        merge: None,
    };
    super::stream::spawn_named("lumasession-peek", move || {
        let preview = super::stream::peek(&target, me);
        handle.send(move |ctx| previewed(ctx, id, preview));
    });
}

/// The session's show arrived: list what Merge would do, or why it can't.
fn previewed<H: SessionHost>(ctx: &mut H, id: u64, preview: Result<super::stream::Preview, String>) {
    if ctx.state().decision.as_ref().is_none_or(|d| d.id != id) {
        return;
    }
    if let Ok(p) = &preview {
        ctx.session().watch.preview = Some((id, p.clone(), show_changes(ctx.state())));
    }
    list_differences(ctx, preview);
    // Timeout 0: no prompt, the default applies at once.
    if ctx.state().root.show_data.show_settings.merge_timeout == Some(0) {
        decide_by_default(ctx);
    }
}

/// Counts every change of this station's show (undo frames, loads).
fn show_changes(state: &AppState) -> u64 {
    state.undo_sequence + state.show_generation
}

/// This station changed its show while the prompt waits: list again.
fn refresh_differences<H: SessionHost>(ctx: &mut H) {
    let tip = show_changes(ctx.state());
    let Some((id, preview, seq)) = ctx.session().watch.preview.clone() else {
        return;
    };
    if ctx.state().decision.as_ref().is_none_or(|d| d.id != id) {
        ctx.session().watch.preview = None;
        return;
    }
    if seq != tip {
        ctx.session().watch.preview = Some((id, preview.clone(), tip));
        list_differences(ctx, Ok(preview));
    }
}

/// Fill the decision's list (what Merge would do) or block Merge.
fn list_differences<H: SessionHost>(ctx: &mut H, preview: Result<super::stream::Preview, String>) {
    let own_journal = ctx.session().journal.id();
    let result = preview.and_then(|p| {
        let parse = |s: &str| lumacore_objects::parse_show(s);
        let master = parse(&p.show)?;
        let master_base = match &p.base {
            Some(b) => Some((parse(b)?, p.base_journal)),
            None => None,
        };
        let joiner_base = ctx.state()
            .split_base
            .as_ref()
            .map(|b| ((*b.show).clone(), b.journal));
        let base = super::link::common_base(
            &[p.journal],
            master_base.as_ref().map(|(s, j)| (s, *j)),
            joiner_base,
            Some(own_journal),
        );
        let mine = &ctx.state().root.show_data;
        match lumacore_tree::merge::can_merge(base.as_ref(), &master, mine) {
            Err(blocked) => Ok(Err(blocked.reason().to_string())),
            Ok(()) => lumacore_tree::merge::three_way(base.as_ref().expect("checked"), &master, mine)
                .map(|(_, report)| Ok(report.lines())),
        }
    });
    let target = ctx.session().watch
        .rival
        .as_ref()
        .map(|r| format!("Join LumaSession {}:{} /Merge", r.ip, r.stream_port));
    let Some(d) = ctx.state().decision.as_mut() else {
        return;
    };
    match result {
        Ok(Ok(lines)) => {
            d.merge_blocked = None;
            if let (Some(merge), Some(cmd)) = (d.options.iter_mut().find(|(l, _)| l == "Merge"), target) {
                merge.1 = cmd;
            }
            d.details = if lines.is_empty() {
                vec!["Merge: nothing differs".to_string()]
            } else {
                lines
            };
        }
        Ok(Err(reason)) | Err(reason) => {
            d.merge_blocked = Some(reason.clone());
            if let Some(merge) = d.options.iter_mut().find(|(l, _)| l == "Merge") {
                merge.1.clear();
            }
            d.details = vec![format!("Merge isn't possible: {reason}")];
        }
    }
}

/// Nobody answered in time: run the default answer.
fn decide_by_default<H: SessionHost>(ctx: &mut H) {
    let Some(d) = ctx.state().decision.take() else {
        return;
    };
    let Some(command) = d.default_command().map(str::to_string) else {
        return;
    };
    let label = d
        .options
        .iter()
        .find(|(_, c)| *c == command)
        .map_or("?", |(l, _)| l.as_str())
        .to_string();
    note(
        ctx.state(),
        format!("{}: nobody answered — {label} (the show's merge default).", d.title),
    );
    let user = super::station_user(ctx.state());
    let (reply, _) = ctx.execute_remote_command(&user, &command);
    note(ctx.state(), lumacore_objects::text::strip_ansi(&reply.message));
}

/// Another master outranks this one: stop output, send the members to it
/// and ask the operator.
fn fence<H: SessionHost>(ctx: &mut H, rival: Rival) {
    let msg = StreamMsg::NewMaster {
        station: rival.name.clone(),
        host: rival.ip.clone(),
        port: rival.stream_port,
        epoch: rival.epoch,
        autosave: false,
    };
    for peer in ctx.session().peers.values() {
        let _ = peer.tx.send(msg.clone());
    }
    ctx.session().peers.clear();
    let state = &mut ctx.state();
    state.root.runtime.luma_sessions.members.clear();
    state.session_role = Role::Fenced {
        by: rival.name.clone(),
    };
    state.show_status = ShowStatus::NegotiatingMember;
    state.protocols_dirty = true;
    let name = state.root.station_settings.lumasession.clone();
    let text = format!(
        "Two masters of LumaSession \"{name}\": {} (epoch {}) took over while this station was away. \
         DMX output is stopped. Use Master = take its show (this show is autosaved first), \
         Use Mine = this show wins (the others autosave theirs), Cancel = stay alone.",
        rival.name, rival.epoch
    );
    decision(ctx, &rival, "Two masters", text.clone());
    announce(ctx, &text);
}

// *************************************************************************************************
// Periodic
// *************************************************************************************************
/// About once a second: health warnings, link down, two masters, a lost
/// session found again, re-invites.
pub fn tick<H: SessionHost>(ctx: &mut H) {
    let now = Instant::now();
    if ctx.session().watch
        .last_tick
        .is_some_and(|t| now.duration_since(t) < Duration::from_secs(1))
    {
        return;
    }
    ctx.session().watch.last_tick = Some(now);
    if decision_due(ctx, now) {
        decide_by_default(ctx);
    }
    refresh_differences(ctx);
    for request in std::mem::take(&mut ctx.state().session_requests) {
        match request {
            SessionRequest::HandOver => {
                if let Some(link) = &ctx.state().link {
                    let _ = link.tx.send(lumacore_objects::session_link::LinkOut::Claim);
                }
            }
            SessionRequest::UseMine => use_mine(ctx),
            SessionRequest::HandTo(station) => hand_over(ctx, &station),
            SessionRequest::Send {
                station,
                ip,
                port,
                request,
                reply_to,
            } => send_to_station(ctx, station, ip, port, request, reply_to),
            SessionRequest::EditStation {
                id,
                station,
                ip,
                port,
                request,
                reply_to,
            } => edit_station(ctx, id, station, ip, port, request, reply_to),
        }
    }
    lumacore_objects::stations::tidy_remote_edits(ctx.state());
    member_health(ctx);
    match ctx.state().session_role.clone() {
        Role::Normal if ctx.state().link.is_none() => {
            if !ctx.state().root.runtime.luma_sessions.members.is_empty() {
                two_masters(ctx);
            }
            // A master, or a station that restarted and knows its stations
            // (listed stations are invited when they come back).
            if ctx.state().session_role == Role::Normal && ctx.state().root.station_settings.online {
                reinvite(ctx);
            }
        }
        Role::IdleMaster { diverged, quiet, .. } if !quiet => session_found(ctx, diverged),
        _ => {}
    }
}

/// The decision's countdown ran out (on the monotonic clock; a paused one —
/// `Pause LumaSession` clears its `deadline_ms` — never does).
fn decision_due<H: SessionHost>(ctx: &H, now: Instant) -> bool {
    let Some(d) = ctx.state_ref().decision.as_ref().filter(|d| d.deadline_ms.is_some()) else {
        return false;
    };
    ctx.session_ref().watch
        .decision_due
        .is_some_and(|(id, at)| id == d.id && now >= at)
}

/// Requests run right after the job that asked for them (not once a second).
pub fn run_requests<H: SessionHost>(ctx: &mut H) {
    if !ctx.state().session_requests.is_empty() {
        ctx.session().watch.last_tick = None;
        tick(ctx);
    }
}

/// Send a command-port request to another station on its own thread (no
/// network I/O on the state thread); the answer goes to `reply_to`.
fn send_to_station<H: SessionHost>(
    ctx: &mut H,
    station: String,
    ip: String,
    port: u16,
    request: lumacore_wire::RemoteRequest,
    reply_to: ReplyTo,
) {
    let Some(handle) = ctx.state_handle().clone() else {
        return;
    };
    super::stream::spawn_named("station-request", move || {
        let text = match lumacore_wire::send_request_to(&ip, port, &request, Duration::from_secs(2)) {
            Ok(r) if r.ok => lumacore_objects::text::render_success(&r.message),
            Ok(r) => lumacore_objects::errors::reply(&r.message, "E-CMD-010"),
            Err(e) => lumacore_objects::errors::render_code(format!("{station} didn't answer: {e}"), "E-CMD-010"),
        };
        handle.send(move |ctx| super::link::reply(ctx, &reply_to, &text));
    });
}

/// One try's wait for a station's answer to an edit, and how many tries
/// (decisions §30 "Remote station edits"; `Set` is absolute, so a retry
/// after a lost answer sets the same again).
pub const EDIT_WAIT: Duration = Duration::from_millis(1000);
pub const EDIT_TRIES: u32 = 3;

/// Send an edit of another station's own setting on a thread of its own
/// (no network I/O on the state thread), waiting [`EDIT_WAIT`] per try; the
/// answer — `old → new` and the line that puts it back, the station's
/// refusal as it came, or that it didn't answer — goes to `reply_to`, into
/// this station's log, and settles the grid's pending cell.
fn edit_station<H: SessionHost>(
    ctx: &mut H,
    id: u64,
    station: String,
    ip: String,
    port: u16,
    request: lumacore_wire::RemoteRequest,
    reply_to: ReplyTo,
) {
    let Some(handle) = ctx.state_handle().clone() else {
        lumacore_objects::stations::remote_answered(ctx.state(), id, Err("not sent (no network here)".into()));
        return;
    };
    super::stream::spawn_named("station-edit", move || {
        let (answer, last) = match send_with_retries(&ip, port, &request, EDIT_WAIT, EDIT_TRIES) {
            Ok(r) => (Some(r), String::new()),
            Err(e) => (None, e),
        };
        handle.send(move |ctx| edit_answered(ctx, id, &station, answer, &last, &reply_to));
    });
}

/// One command-port request, tried up to `tries` times, `wait` each (off
/// the state thread). Err = the last try's error.
pub fn send_with_retries(
    ip: &str,
    port: u16,
    request: &lumacore_wire::RemoteRequest,
    wait: Duration,
    tries: u32,
) -> Result<lumacore_wire::RemoteReply, String> {
    let mut last = String::from("not sent");
    for _ in 0..tries.max(1) {
        match lumacore_wire::send_request_to(ip, port, request, wait) {
            Ok(r) => return Ok(r),
            Err(e) => last = e,
        }
    }
    Err(last)
}

/// The station's answer to edit `id` (None = no answer after every try).
pub fn edit_answered<H: SessionHost>(
    ctx: &mut H,
    id: u64,
    station: &str,
    answer: Option<lumacore_wire::RemoteReply>,
    last_error: &str,
    reply_to: &ReplyTo,
) {
    use lumacore_objects::text::strip_ansi;
    let (text, outcome) = match answer {
        Some(r) => {
            let plain = strip_ansi(&r.message).trim().to_string();
            let refused = !r.ok || plain.lines().any(|l| l.trim_start().starts_with("Error"));
            if refused && plain.contains("E-CMD-017") {
                // Busy there (merge of bugfixes, §30): the edit didn't run
                // (or still runs; discovery shows the value when it does).
                let why = plain.trim_start_matches("Error:").trim().to_string();
                (lumacore_objects::errors::reply(format!("Station \"{station}\": {why}"), "E-CMD-017"), Err(why))
            } else if refused {
                let why = plain.trim_start_matches("Error:").trim().to_string();
                (
                    lumacore_objects::errors::reply(format!("Station \"{station}\" refused: {why}"), "E-CMD-010"),
                    Err(why),
                )
            } else {
                (lumacore_objects::text::render_success(&format!("Station \"{station}\": {plain}")), Ok(()))
            }
        }
        None => {
            let why = format!("didn't answer ({EDIT_TRIES} tries, {last_error})");
            (
                lumacore_objects::errors::render_code(format!("Station \"{station}\" {why}: nothing changed there that this station knows of."), "E-CMD-010"),
                Err(why),
            )
        }
    };
    let setting = ctx.state()
        .remote_edits
        .iter()
        .find(|e| e.id == id)
        .map_or_else(String::new, |e| format!(" ({})", e.setting));
    lumacore_objects::stations::remote_answered(ctx.state(), id, outcome);
    ctx.state()
        .sysmon_pending
        .push(("Station".into(), format!("Answer from \"{station}\"{setting}: {}", strip_ansi(&text).trim())));
    ctx.drain_sysmon_pending();
    super::link::reply(ctx, reply_to, &text);
}

fn member_health<H: SessionHost>(ctx: &mut H) {
    let Some(link) = ctx.state().link.clone() else {
        ctx.session().watch.warned_late = false;
        ctx.session().watch.warned_link_down = None;
        return;
    };
    let silent = link.stats.silent_for();
    if silent >= WARN && !ctx.session().watch.warned_late {
        ctx.session().watch.warned_late = true;
        note(
            ctx.state(),
            format!(
                "No answer from {} for {:.1} s; commands wait (lost after {} s).",
                link.master,
                silent.as_secs_f32(),
                lost_after().as_secs()
            ),
        );
    } else if silent < WARN && ctx.session().watch.warned_late {
        ctx.session().watch.warned_late = false;
        note(ctx.state(), format!("{} answers again.", link.master));
    }
    // Link down on the stream's NIC: cut off at once (the timeout still
    // decides IdleMaster), "No Cable".
    let local = link
        .socket
        .lock()
        .ok()
        .and_then(|s| s.as_ref().and_then(|s| s.local_addr().ok()))
        .map(|a| a.ip());
    if let Some(std::net::IpAddr::V4(ip)) = local
        && !ip.is_loopback()
    {
        let up = lumacore_net::nics::list().iter().any(|n| n.ip == ip);
        if !up && ctx.session().watch.warned_link_down.is_none() {
            ctx.session().watch.warned_link_down = Some(ip.to_string());
            announce(
                ctx,
                &format!("No cable / address on the interface of {ip}: cut off from {}; commands wait.", link.master),
            );
            if let Ok(mut socket) = link.socket.lock()
                && let Some(socket) = socket.take()
            {
                let _ = socket.shutdown(std::net::Shutdown::Both);
            }
        } else if up {
            ctx.session().watch.warned_link_down = None;
        }
    }
}

fn rival_of(peer: &lumacore_net::DiscoveryPeerState) -> Rival {
    Rival {
        name: peer.station_name.clone(),
        ip: peer.ip.to_string(),
        cmd_port: peer.cmd_port,
        stream_port: peer.stream_port,
        epoch: peer.epoch,
    }
}

/// This master sees another master of the same LumaSession.
fn two_masters<H: SessionHost>(ctx: &mut H) {
    let name = ctx.state().root.station_settings.lumasession.clone();
    let me_name = ctx.state().root.station_settings.station_name.clone();
    let mut mine = me(ctx.state());
    mine.applied = ctx.session().journal.tip_seq();
    let my_epoch = ctx.session().journal.epoch();
    for peer in seen(ctx) {
        if peer.status != "Master"
            || peer.station_name == me_name
            || !peer.lumasession.eq_ignore_ascii_case(&name)
        {
            continue;
        }
        let other = candidate(&peer);
        let other_wins = peer.epoch > my_epoch
            || (peer.epoch == my_epoch && ranks_above(&other, &mine));
        if other_wins {
            fence(ctx, rival_of(&peer));
            return;
        }
        if ctx.session().watch.noted_rival.as_deref() != Some(&peer.station_name) {
            ctx.session().watch.noted_rival = Some(peer.station_name.clone());
            ctx.state().show_status = ShowStatus::NegotiatingMaster;
            announce(
                ctx,
                &format!(
                    "{} is a master of LumaSession \"{name}\" too (epoch {}): it is fenced and its operator decides.",
                    peer.station_name, peer.epoch
                ),
            );
        }
    }
}

/// An IdleMaster sees its session again: rejoin silently, or ask.
fn session_found<H: SessionHost>(ctx: &mut H, diverged: bool) {
    let name = ctx.state().root.station_settings.lumasession.clone();
    let Some(peer) = seen(ctx).into_iter().find(|p| {
        p.status == "Master" && p.lumasession.eq_ignore_ascii_case(&name) && p.stream_port != 0
    }) else {
        return;
    };
    let rival = rival_of(&peer);
    if !diverged {
        let user = super::station_user(ctx.state());
        ctx.state().link_requests.push(LinkRequest {
            host: rival.ip.clone(),
            port: rival.stream_port,
            user,
            reply_to: ReplyTo::Log,
            invited_by: None,
            patient: true,
            merge: false,
        });
        ctx.state().session_role = Role::Normal;
        ctx.state().show_status = ShowStatus::Downloading;
        note(
            ctx.state(),
            format!("{} is back as master of \"{name}\": rejoining (nothing changed here).", rival.name),
        );
        return;
    }
    if ctx.state().decision.is_some() {
        return;
    }
    let text = format!(
        "LumaSession \"{name}\" is back at {} (epoch {}), and this station changed the show while it was alone. \
         Use Master = take the session's show (this show is autosaved first), Use Mine = this show wins \
         (the others autosave theirs), Cancel = keep working alone.",
        rival.name, rival.epoch
    );
    ctx.state().show_status = ShowStatus::NegotiatingMember;
    decision(ctx, &rival, "Session found again", text.clone());
    announce(ctx, &text);
}

/// An invite from a station of the LumaSession this station leads (it kept
/// the session going while that one was away, R1-22, Q26): never join
/// silently — that would roll the session back to the show on the other
/// station's disk. Ask: Use Mine (the default: this show stays, the other
/// station takes it) / Use Master (take its show, this one autosaved
/// first) / Cancel; Merge greyed.
pub fn invited_back<H: SessionHost>(ctx: &mut H, rival: Rival) {
    if ctx.state().decision.is_some() {
        return;
    }
    let name = ctx.state().root.station_settings.lumasession.clone();
    let text = format!(
        "{} invites this station to LumaSession \"{name}\", which this station has kept going. Use Mine = this \
         show stays the session's and {} takes it; Use Master = take {}'s show (this one is autosaved first); \
         Cancel = keep leading the session.",
        rival.name, rival.name, rival.name
    );
    decision_with(
        ctx,
        &rival,
        "Old master is back",
        text.clone(),
        Some("the invite came from a station that was away; its show may be older".to_string()),
    );
    // A master with members (R1 review 19): Use Mine invites the station
    // that came back (it joins this session); Cancel only closes the
    // question — this master keeps leading. (Claim / Leave LumaSession are
    // for a station apart from its session.)
    if ctx.state().link.is_none() && ctx.state().session_role == Role::Normal
        && let Some(d) = ctx.state().decision.as_mut()
    {
        for (label, line) in &mut d.options {
            match label.as_str() {
                "Use Mine" => *line = format!("Invite Station \"{}\"", rival.name),
                "Cancel" => line.clear(),
                _ => {}
            }
        }
    }
    announce(ctx, &text);
}

/// Use Mine: become master with a new epoch and make the other master
/// yield (it takes this show and brings its members).
fn use_mine<H: SessionHost>(ctx: &mut H) {
    let Some(rival) = ctx.session().watch.rival.clone() else {
        note(ctx.state(), "Use Mine: no other master to take over from.");
        return;
    };
    let epoch = rival.epoch.max(ctx.state().session_epoch).max(ctx.session().journal.epoch()) + 1;
    become_master(ctx, Some(epoch), &format!("Use Mine over {}", rival.name));
    let request = lumacore_wire::RemoteRequest::Yield {
        lumasession: ctx.state().root.station_settings.lumasession.clone(),
        from: ctx.state().root.station_settings.station_name.clone(),
        epoch,
        stream_port: ctx.stream_port(),
    };
    let (ip, port) = (rival.ip.clone(), rival.cmd_port);
    super::stream::spawn_named("lumasession-yield", move || {
        let _ = lumacore_wire::send_request_to(&ip, port, &request, Duration::from_secs(2));
    });
}

/// Known stations that came back (and didn't leave on purpose) are invited
/// again, with back-off.
fn reinvite<H: SessionHost>(ctx: &mut H) {
    let now = Instant::now();
    let me_name = ctx.state().root.station_settings.station_name.clone();
    let known: Vec<KnownStation> = ctx.state_ref()
        .root
        .show_data
        .session
        .stations
        .values()
        .filter(|k| k.meta.name() != Some(me_name.as_str()) && !ctx.session_ref().peers.contains_key(k.meta.name().unwrap_or_default()))
        .filter(|k| !ctx.session_ref().watch.left.contains(&k.station_id))
        .cloned()
        .collect();
    let peers = seen(ctx);
    // Two standalone stations that know each other: only the one that would
    // win an election invites (else each would join the other).
    let standalone = ctx.state().root.runtime.luma_sessions.members.is_empty();
    let mine = me(ctx.state());
    for k in known {
        let Some(peer) = peers
            .iter()
            .find(|p| {
                p.station_id == k.station_id
                    && (p.status == "Standalone" || p.status == "IdleMaster")
                    && p.invite
            })
        else {
            continue;
        };
        if standalone && !ranks_above(&mine, &candidate(peer)) {
            continue;
        }
        let sent = ctx.session().watch.reinvites.entry(k.station_id.clone()).or_default();
        sent.retain(|t| now.duration_since(*t) < Duration::from_secs(60));
        if sent.len() >= REINVITE_MAX || sent.back().is_some_and(|t| now.duration_since(*t) < REINVITE_EVERY) {
            continue;
        }
        sent.push_back(now);
        let request = lumacore_wire::RemoteRequest::Invite {
            lumasession: ctx.state().root.station_settings.lumasession.clone(),
            from: me_name.clone(),
            stream_port: ctx.stream_port(),
        };
        let (ip, port) = (peer.ip.to_string(), peer.cmd_port);
        note(ctx.state(), format!("Re-inviting {} (known station back online).", k.meta.name().unwrap_or_default()));
        super::stream::spawn_named("lumasession-invite", move || {
            let _ = lumacore_wire::send_request_to(&ip, port, &request, Duration::from_secs(2));
        });
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
