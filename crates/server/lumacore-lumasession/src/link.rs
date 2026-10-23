// ██╗     ██╗███╗   ██╗██╗  ██╗
// ██║     ██║████╗  ██║██║ ██╔╝
// ██║     ██║██╔██╗ ██║█████╔╝
// ██║     ██║██║╚██╗██║██╔═██╗
// ███████╗██║██║ ╚████║██║  ██╗
// ╚══════╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝
// FILE: link

//! The LumaSession link on the state thread (decisions §10, §21 "6b
//! hardening"). The stream threads (`crate::stream`) only move frames; this
//! is what both ends do with them.
//!
//! **Member:** a line whose scope is Master (`registry::line_scope`) is not
//! run here: [`forward`] sends it to the master and keeps it in the pending
//! outbox until the master's reply arrives ([`deliver`]); the reply goes to
//! whoever sent the line (a Shell's output, a waiting UDP / HTTP request, the
//! log). Results arrive as ops and are applied strictly in `(epoch, seq)`
//! order ([`apply_op`]); a gap asks for a snapshot. Show changes a
//! station-local action makes here (live screens writing through to the
//! profile) go to the master as [`submit_changes`].
//!
//! **Master:** after every job [`flush`] commits the change path's outbox to
//! the journal and sends each op — with the undo steps and the session
//! runtime when they changed — to every member. [`admit`] answers a Hello;
//! [`run_forwarded`] runs a member's line as its user.
//!
//! Owns:          the master's and the member's link: admitting members, forwarding lines, applying ops in order, flush, resync, the merge on rejoin, link loss.
//! Doesn't own:   the stream's frames and threads (stream), the election (election), the change path (lumacore-state).
//! Entry points:  after_work, flush, admit, run_forwarded, apply_op, preview, merge_offer.
//! Decisions:     §10, §21 "6b", "6d", §34 19.3.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use lumacore_tree::{
    LocalSnapshot, MemberRuntime, OopsKind, PendingReplicationOp, ReplicationOp, ShowData, SubtreeSnapshot,
    UndoStack, VariableValue,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::host::SessionHost;
use super::stream::{Hello, StreamMsg, Welcome};
use lumacore_wire::SESSION_PROTOCOL;
use lumacore_objects::AppState;
use lumacore_objects::commands::registry::Undo;
use lumacore_objects::session_link::{LinkHandle, LinkRequest, LinkStats, ReplyTo};
use lumacore_objects::claims::Claims;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A forwarded command older than this tells its client it is pending.
pub const PENDING_NOTICE: Duration = Duration::from_secs(1);

/// How long an HTTP request waits for the master's reply.
pub const HTTP_WAIT: Duration = Duration::from_secs(10);

/// How long a UDP request waits for the master's reply (under the 2 s a
/// `lumacore-cli` request waits for its answer).
pub const UDP_WAIT: Duration = Duration::from_millis(1500);

/// Replies kept per member, to answer a command resent after a reconnect.
const KEPT_REPLIES: usize = 64;

/// The session-wide part of `Runtime`, replicated with every op that
/// changes it (decisions §21 "6b": modes, claims, variables, who is where).
/// Screens, clients, watches, interfaces and health stay on each station.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionRuntime {
    /// The master's station name.
    pub master: String,
    /// The master's epoch (§21 "6c").
    #[serde(default)]
    pub epoch: u64,
    /// Users logged in on the master.
    #[serde(default)]
    pub master_users: Vec<String>,
    /// Each user's modes (users without any are left out).
    #[serde(default)]
    pub modes: BTreeMap<String, lumacore_tree::Modes>,
    /// Each user's clipboard (decisions §35 10; users without one are left out).
    #[serde(default)]
    pub clipboards: BTreeMap<String, lumacore_tree::Clipboard>,
    #[serde(default)]
    pub claims: BTreeMap<Uuid, lumacore_tree::Claim>,
    #[serde(default)]
    pub claim_order: Vec<Uuid>,
    #[serde(default)]
    pub claim_asks: Vec<lumacore_tree::ClaimAsk>,
    #[serde(default)]
    pub variables: BTreeMap<String, VariableValue>,
    #[serde(default)]
    pub members: BTreeMap<String, MemberRuntime>,
    /// The master's protocol lines as it runs them (a member's DMX
    /// Protocols menu shows them, decisions §30 "12d part 4").
    #[serde(default)]
    pub lines: Vec<lumacore_tree::LineRuntime>,
    /// What ran of each member's forwarded commands, by station id (a
    /// command resent to this master or the next one never runs twice).
    #[serde(default)]
    pub commands_done: BTreeMap<String, lumacore_tree::CommandsDone>,
}

/// A member connected to this master.
#[derive(Debug)]
pub struct Peer {
    pub tx: Sender<StreamMsg>,
    /// Which connection (a reconnect replaces the peer).
    pub conn: u64,
    /// Its stable station id and this join's link (`Hello.link`).
    pub station_id: String,
    pub link: Uuid,
    /// Heartbeat and health.
    pub stats: Arc<LinkStats>,
}

/// Recent answers to one member's forwarded commands (kept on this master
/// by station id, across reconnects; a Bye or a new link drops them): a
/// command resent after its answer was lost is answered again, never run
/// twice.
#[derive(Debug, Default)]
pub struct ReplyCache {
    pub link: Uuid,
    pub replies: BTreeMap<u64, StreamMsg>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

use lumacore_base::clock::now_ms;

fn note(state: &mut AppState, message: impl Into<String>) {
    state
        .sysmon_pending
        .push(("LumaSession".to_string(), message.into()));
}

/// Tell `to` how something ended.
pub fn reply<H: SessionHost>(ctx: &mut H, to: &ReplyTo, text: &str) {
    match to {
        ReplyTo::Log => note(ctx.state(), lumacore_objects::text::strip_ansi(text)),
        ReplyTo::Shell(user) => {
            if !ctx.push_output(user, text) {
                note(ctx.state(), lumacore_objects::text::strip_ansi(text));
            }
        }
        ReplyTo::Waiter(tx) => {
            let _ = tx.send(text.to_string());
        }
    }
}

/// Run `f` (a request's job) collecting the replies of lines it forwards;
/// the caller waits for them off the state thread with [`wait_replies`].
pub fn collecting<R, H: SessionHost>(ctx: &mut H, f: impl FnOnce(&mut H) -> R) -> (R, Vec<Receiver<String>>) {
    ctx.state().forward_waits = Some(Vec::new());
    let result = f(ctx);
    let waits = ctx.state().forward_waits.take().unwrap_or_default();
    (result, waits)
}

/// Wait (not on the state thread) for forwarded lines' replies, in order.
/// A reply that doesn't come in time reads as pending.
pub fn wait_replies(waits: Vec<Receiver<String>>, wait: Duration) -> Option<String> {
    if waits.is_empty() {
        return None;
    }
    let deadline = Instant::now() + wait;
    let replies: Vec<String> = waits
        .into_iter()
        .map(|rx| {
            rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .unwrap_or_else(|_| {
                    "Pending: the master hasn't confirmed yet; the command runs when it does.".into()
                })
        })
        .collect();
    Some(replies.join("\n"))
}

/// A request's reply with the master's answers to the lines it forwarded.
pub fn with_replies(
    mut reply: lumacore_wire::RemoteReply,
    waits: Vec<Receiver<String>>,
    wait: Duration,
) -> lumacore_wire::RemoteReply {
    if let Some(text) = wait_replies(waits, wait) {
        let plain = lumacore_objects::text::strip_ansi(&text);
        reply.ok = !plain.lines().any(|l| l.trim_start().starts_with("Error"));
        reply.message = text;
    }
    reply
}

// *************************************************************************************************
// Member: receiving
// *************************************************************************************************
/// The master answered a forwarded command.
pub fn deliver<H: SessionHost>(ctx: &mut H, id: u64, ok: bool, message: String) {
    let Some(forward) = ctx.state().forwards.remove(&id) else {
        return;
    };
    let text = if ok {
        message
    } else {
        lumacore_objects::errors::reply(&message, "E-CMD-011")
    };
    reply(ctx, &forward.reply_to, &text);
}

/// Take the master's show (first join, resync or the master loaded a show)
/// or catch up from its ops, then start or keep following it.
pub fn install<H: SessionHost>(
    ctx: &mut H,
    welcome: Welcome,
    show: Option<ShowData>,
    mut handle: LinkHandle,
    first: bool,
    request: &LinkRequest,
) {
    // Left meanwhile (or replaced by another join).
    if !first
        && ctx.state()
            .link
            .as_ref()
            .is_none_or(|l| !Arc::ptr_eq(&l.stop, &handle.stop))
    {
        return;
    }
    if first && handle.stop.load(Ordering::Relaxed) {
        return;
    }
    let state = &mut ctx.state();
    state.replaying_remote_op = true;
    if let Some(show) = show {
        lumacore_objects::objects::replace_show(state, show);
        state.clear_all_undo_stacks();
        for (user, stack) in welcome.undo {
            *state.undo_stack_mut(&user) = stack;
        }
        state.last_applied = Some((welcome.journal, welcome.seq));
    }
    state.replaying_remote_op = false;
    state.show_replaced = false;
    state.session_role = lumacore_objects::session_link::Role::Normal;
    state.decision = None;
    state.show_status = lumacore_objects::session_link::ShowStatus::Downloaded;
    state.session_epoch = welcome.epoch;
    handle.stats.heard();
    handle.master = welcome.master.clone();
    handle.master_priority = welcome.master_priority;
    handle.master_cmd_port = welcome.cmd_port;
    handle.resyncing.store(false, Ordering::Relaxed);
    let resuming = first.then_some(()).is_none() && !welcome.ops.is_empty();
    if first {
        state.link = Some(handle.clone());
    } else if let Some(link) = &mut state.link {
        link.master = welcome.master.clone();
        link.target = handle.target.clone();
        link.master_priority = welcome.master_priority;
        link.master_cmd_port = welcome.cmd_port;
    }
    for op in welcome.ops {
        apply_op(ctx, op);
    }
    let state = &mut ctx.state();
    apply_session_runtime(state, &welcome.runtime);
    // Logins are sent again on the new connection; files asked again too.
    ctx.session().sent_logins = None;
    ctx.state().files_wanted.clear();
    want_files(ctx.state());

    let state = &mut ctx.state();
    let target = handle.target.clone();
    let host = target.rsplit_once(':').map_or(target.as_str(), |(h, _)| h).to_string();
    state.root.runtime.joined_sessions.insert(
        target.clone(),
        lumacore_tree::JoinedSessionRuntime {
            target: target.clone(),
            user: request.user.clone(),
            authority: host,
            cmd_port: 0,
            shell_port: 0,
            connected: true,
            last_error: None,
        },
    );
    if first {
        // A member doesn't send: its master does (decisions §10, §11).
        state.is_authority = false;
        if !welcome.lumasession.is_empty() {
            state.root.station_settings.lumasession = welcome.lumasession.clone();
        }
        state.local_settings.store_from(&state.root.station_settings);
        state.protocols_dirty = true;
        let commit = &lumacore_objects::build_info().git_commit;
        let warning = if welcome.commit != *commit {
            format!(
                "\n  Note: {} runs build {} (this station: {commit}).",
                welcome.master, welcome.commit
            )
        } else {
            String::new()
        };
        let invited = request
            .invited_by
            .as_ref()
            .map(|by| format!(" (invited by {by})"))
            .unwrap_or_default();
        let mine = ctx.state().root.station_settings.station_priority;
        let outranks = welcome.master_priority.is_some_and(|p| mine > p);
        let hint = if outranks {
            format!(
                "\n  This station's priority ({mine}) is above the master's: Claim LumaSession makes it master."
            )
        } else {
            String::new()
        };
        let merged = if request.merge {
            let mut lines = format!(
                "\n  Merged into the session's show ({} change{}):",
                welcome.merged.len(),
                if welcome.merged.len() == 1 { "" } else { "s" }
            );
            for line in welcome.merged.iter().take(20) {
                lines.push_str(&format!("\n    {line}"));
            }
            if welcome.merged.len() > 20 {
                lines.push_str(&format!("\n    … {} more (sysmon on {})", welcome.merged.len() - 20, welcome.master));
            }
            lines
        } else {
            String::new()
        };
        let text = lumacore_objects::text::render_success(&format!(
            "Joined LumaSession \"{}\" at {} (master {}) as {}{invited}.{warning}{hint}{merged}",
            welcome.lumasession, target, welcome.master, request.user
        ));
        note(ctx.state(), lumacore_objects::text::strip_ansi(&text));
        if !matches!(request.reply_to, ReplyTo::Log) {
            reply(ctx, &request.reply_to, &text);
        }
    } else if resuming {
        note(state, format!("Back with {} — caught up.", welcome.master));
    } else {
        note(state, format!("Took the show from {}.", welcome.master));
    }
}

/// The first connection failed: nothing changed here.
pub fn join_failed<H: SessionHost>(ctx: &mut H, request: &LinkRequest, why: &str) {
    if ctx.state().link.is_none() {
        ctx.state().show_status = lumacore_objects::session_link::ShowStatus::Loaded;
    }
    let message = format!(
        "Join LumaSession at {}:{} failed: {why}. This station kept its show.",
        request.host, request.port
    );
    // A refusal from the master may carry its code already (E-LUMA-…).
    let text = lumacore_objects::errors::reply(message, "E-CMD-010");
    note(ctx.state(), lumacore_objects::text::strip_ansi(&text));
    reply(ctx, &request.reply_to, &text);
}

/// Something about a running link worth telling.
pub fn note_link<H: SessionHost>(ctx: &mut H, message: &str) {
    note(ctx.state(), message.to_string());
}

/// The master moved (handover, fenced master, Use Mine): follow it,
/// keeping a copy of this show first when asked.
pub fn moved<H: SessionHost>(ctx: &mut H, from: &str, to: &str, autosave: bool) {
    let kept = if autosave {
        match lumacore_objects::session_link::autosave_copy(ctx.state(), "use-mine") {
            Ok(saved) => saved.map(|p| format!(" (this show kept in {p})")).unwrap_or_default(),
            Err(e) => {
                // Following would replace this show with the new master's:
                // not without a copy (R1-16). Leave instead, keeping it.
                let left = lumacore_objects::commands::session::leave_lumasession(ctx.state());
                ctx.state().show_status = lumacore_objects::session_link::ShowStatus::Loaded;
                note(
                    ctx.state(),
                    format!(
                        "[ERROR] The master moved from {from} to {to}, but this show's unsaved changes couldn't be kept ({e}): \
                         not following, nothing was replaced. {} Save the show, then Join LumaSession again.",
                        lumacore_objects::text::strip_ansi(&left).replace('\n', " ")
                    ),
                );
                return;
            }
        }
    } else {
        String::new()
    };
    ctx.state().show_status = lumacore_objects::session_link::ShowStatus::WaitingForNewMaster;
    note(ctx.state(), format!("The master moved from {from} to {to}{kept}."));
}

/// The connection to the master dropped; the link retries.
pub fn link_lost<H: SessionHost>(ctx: &mut H) {
    let state = &mut ctx.state();
    for joined in state.root.runtime.joined_sessions.values_mut() {
        joined.connected = false;
    }
    let master = state.link.as_ref().map(|l| l.master.clone()).unwrap_or_default();
    note(
        state,
        format!("Lost the connection to {master}; reconnecting. Commands stay pending."),
    );
}

/// Ask the master for a snapshot (once until it arrives).
fn request_resync(state: &AppState) {
    if let Some(link) = &state.link
        && !link.resyncing.swap(true, Ordering::Relaxed)
    {
        let _ = link.tx.send(lumacore_objects::session_link::LinkOut::Resync {
            resume: state.last_applied,
        });
    }
}

/// Where an op stands against what a member applied last.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpOrder {
    /// The next one: apply it.
    Next,
    /// Already applied (a repeat): ignore it.
    Stale,
    /// Something is missing, or another journal: ask for a resync.
    Gap,
}

/// Strict order (decisions §10): only `seq + 1` of the same journal applies.
pub fn op_order(last_applied: Option<(uuid::Uuid, u64)>, op: &ReplicationOp) -> OpOrder {
    match last_applied {
        Some((journal, seq)) if journal == op.journal => {
            if op.seq <= seq {
                OpOrder::Stale
            } else if op.seq == seq + 1 {
                OpOrder::Next
            } else {
                OpOrder::Gap
            }
        }
        _ => OpOrder::Gap,
    }
}

/// Apply one result from the master, strictly in order.
pub fn apply_op<H: SessionHost>(ctx: &mut H, op: ReplicationOp) {
    let state = &mut ctx.state();
    if state
        .link
        .as_ref()
        .is_some_and(|l| l.resyncing.load(Ordering::Relaxed))
    {
        return;
    }
    match op_order(state.last_applied, &op) {
        OpOrder::Next => {}
        OpOrder::Stale => return,
        OpOrder::Gap => {
            request_resync(state);
            return;
        }
    }
    // What each live screen's saved layout was before (R1-58).
    let layouts_before = op
        .affected_subtrees
        .iter()
        .any(|s| matches!(s, lumacore_tree::ShowSubtree::Profile(_)))
        .then(|| lumacore_objects::commands::screens::saved_layouts(state));
    if let Err(e) = super::replication::apply_replicated_op(state, &op) {
        log::error!("applying op {} from the master failed: {e}", op.seq);
        request_resync(state);
        return;
    }
    if let Some(runtime) = op
        .runtime
        .as_ref()
        .and_then(|v| serde_json::from_value::<SessionRuntime>(v.clone()).ok())
    {
        apply_session_runtime(state, &runtime);
    }
    if let Some(before) = &layouts_before {
        lumacore_objects::commands::screens::follow_profiles(state, before);
    }
    if op.affected_subtrees.iter().any(|s| {
        matches!(
            s,
            lumacore_tree::ShowSubtree::Profile(_)
                | lumacore_tree::ShowSubtree::Images
                | lumacore_tree::ShowSubtree::Devices
        )
    }) {
        want_files(state);
    }
}

/// Ask the master for the show's files this station's stores lack (pictures
/// and GDTF files, by SHA-256; decisions §30 "12a2"). Each file is asked once.
pub fn want_files(state: &mut AppState) {
    let show = &state.root.show_data;
    let mut files: Vec<lumacore_objects::session_link::WantedFile> = lumacore_luma::media::missing(show)
        .into_iter()
        .map(|(sha256, ext)| lumacore_objects::session_link::WantedFile { sha256, ext })
        .collect();
    files.extend(
        show.devices
            .dmx
            .patch
            .fixture_types
            .values()
            .flat_map(|ft| &ft.revisions)
            .filter(|r| !r.file_hash.is_empty() && lumacore_luma::show_file::stored_gdtf(&r.file_hash).is_none())
            .map(|r| lumacore_objects::session_link::WantedFile { sha256: r.file_hash.clone(), ext: "gdtf".into() }),
    );
    files.retain(|f| state.files_wanted.insert(format!("{}.{}", f.sha256, f.ext)));
    if files.is_empty() {
        return;
    }
    if let Some(link) = &state.link {
        let _ = link.tx.send(lumacore_objects::session_link::LinkOut::FileWant { files });
    }
}

/// Take the session runtime the master sent.
fn apply_session_runtime(state: &mut AppState, runtime: &SessionRuntime) {
    for (name, user) in state.root.runtime.users.iter_mut() {
        user.modes = runtime.modes.get(name).copied().unwrap_or_default();
    }
    for (name, modes) in &runtime.modes {
        state.root.runtime.user(name).modes = *modes;
    }
    for (name, user) in state.root.runtime.users.iter_mut() {
        user.clipboard = runtime.clipboards.get(name).cloned();
    }
    for (name, clipboard) in &runtime.clipboards {
        state.root.runtime.user(name).clipboard = Some(clipboard.clone());
    }
    state.root.runtime.claims = runtime.claims.clone();
    state.root.runtime.claim_order = runtime.claim_order.clone();
    state.root.runtime.claim_asks = runtime.claim_asks.clone();
    state.root.runtime.global_variables = runtime.variables.clone();
    state.root.runtime.luma_sessions.master_lines = runtime.lines.clone();
    state.root.runtime.luma_sessions.members = runtime.members.clone();
    state.root.runtime.luma_sessions.master = runtime.master.clone();
    state.root.runtime.luma_sessions.master_users = runtime.master_users.clone();
    state.root.runtime.luma_sessions.commands_done = runtime.commands_done.clone();
    if runtime.epoch > 0 {
        state.session_epoch = runtime.epoch;
    }
    // Lines of ours the master ran are never run here (R1-21).
    lumacore_objects::session_link::mark_ran(state);
}

// *************************************************************************************************
// Master
// *************************************************************************************************
/// The session runtime as members should see it.
fn session_runtime<H: SessionHost>(ctx: &H) -> SessionRuntime {
    let state = &ctx.state_ref();
    let runtime = &state.root.runtime;
    SessionRuntime {
        master: state.root.station_settings.station_name.clone(),
        epoch: ctx.session_ref().journal.epoch(),
        master_users: ctx.shell_users(),
        modes: runtime
            .users
            .iter()
            .filter(|(_, u)| u.modes != lumacore_tree::Modes::default())
            .map(|(name, u)| (name.clone(), u.modes))
            .collect(),
        clipboards: runtime.users.iter().filter_map(|(name, u)| u.clipboard.clone().map(|c| (name.clone(), c))).collect(),
        claims: runtime.claims.clone(),
        claim_order: runtime.claim_order.clone(),
        claim_asks: runtime.claim_asks.clone(),
        variables: runtime.global_variables.clone(),
        members: runtime.luma_sessions.members.clone(),
        lines: runtime.network.lines.clone(),
        commands_done: runtime.luma_sessions.commands_done.clone(),
    }
}

/// Every user's undo history (a joining member mirrors it).
fn undo_stacks(state: &AppState) -> BTreeMap<String, UndoStack> {
    state
        .root
        .runtime
        .users
        .iter()
        .filter(|(_, u)| !u.undo.frames.is_empty() || !u.undo.redo_frames.is_empty())
        .map(|(name, u)| (name.clone(), u.undo.clone()))
        .collect()
}

/// A peeking station's view of this show: the show and this station's
/// split base (a merge prompt lists what differs, §21 "6d").
pub fn preview<H: SessionHost>(ctx: &mut H) -> super::stream::Preview {
    let base = ctx.state_ref().split_base.as_ref();
    super::stream::Preview {
        master: ctx.state_ref().root.station_settings.station_name.clone(),
        show: serde_json::to_string(&ctx.state_ref().root.show_data).unwrap_or_default(),
        journal: ctx.session_ref().journal.id(),
        base: base.and_then(|b| serde_json::to_string(&*b.show).ok()),
        base_journal: base.and_then(|b| b.journal),
    }
}

/// The show both sides had at the split: the joiner's split base when it
/// came from this session's history, else this station's own (taken when it
/// became master) when the joiner is the master it took over from.
pub fn common_base(
    master_journals: &[Uuid],
    master_base: Option<(&lumacore_tree::ShowData, Option<Uuid>)>,
    joiner_base: Option<(lumacore_tree::ShowData, Option<Uuid>)>,
    joiner_journal: Option<Uuid>,
) -> Option<lumacore_tree::ShowData> {
    if let Some((base, Some(j))) = joiner_base
        && (master_journals.contains(&j) || master_base.is_some_and(|(_, mj)| mj == Some(j)))
    {
        return Some(base);
    }
    match master_base {
        Some((base, Some(j))) if joiner_journal == Some(j) => Some(base.clone()),
        _ => None,
    }
}

/// Merge a rejoining station's show into this one (Merge, §21 "6d"): this
/// show is autosaved first, the merged one replaces it for everyone.
fn merge_in<H: SessionHost>(ctx: &mut H, from: &str, offer: &super::stream::MergeOffer) -> Result<Vec<String>, String> {
    let parse = |s: &str| lumacore_objects::parse_show(s).map(|mut s| {
        s.after_load();
        s
    });
    let mine = parse(&offer.show)?;
    let joiner_base = match &offer.base {
        Some(b) => Some((parse(b)?, offer.base_journal)),
        None => None,
    };
    let master_base = ctx.state_ref()
        .split_base
        .as_ref()
        .map(|b| (b.show.as_ref(), b.journal));
    let base = common_base(&[ctx.session_ref().journal.id()], master_base, joiner_base, offer.journal);
    let master = &ctx.state_ref().root.show_data;
    if let Err(blocked) = lumacore_tree::merge::can_merge(base.as_ref(), master, &mine) {
        return Err(format!("Merge with {from} isn't possible: {}.", blocked.reason()));
    }
    let base = base.expect("checked by can_merge");
    let (merged, report) = lumacore_tree::merge::three_way(&base, master, &mine)?;
    let lines = report.lines();
    // The session's show is replaced by the merged one: only with a copy.
    if let Err(e) = lumacore_objects::session_link::autosave_show(ctx.state_ref(), &ctx.state_ref().root.show_data, "before-merge") {
        note(ctx.state(), format!("[ERROR] Merge with {from} refused: the session's show couldn't be kept first ({e})."));
        return Err(format!(
            "{} cannot keep the session's show before the merge ({e}); nothing was merged.",
            ctx.state().root.station_settings.station_name
        ));
    }
    lumacore_objects::objects::replace_show(ctx.state(), merged);
    ctx.state().show_dirty = true;
    ctx.state().split_base = None;
    lumacore_objects::session_link::forget_split_base();
    lumacore_objects::references::check(ctx.state(), "merge");
    announce_merge(ctx, from, &lines);
    // Members and the joiner start from the merged show.
    flush(ctx);
    Ok(lines)
}

fn announce_merge<H: SessionHost>(ctx: &mut H, from: &str, lines: &[String]) {
    for line in lines {
        lumacore_sysmon::sysmon_note("Merge", format!("{from}: {line}"));
    }
    note(
        ctx.state(),
        format!(
            "Merged {from}'s show into this LumaSession ({} change{}; List Reference /Broken checks links).",
            lines.len(),
            if lines.len() == 1 { "" } else { "s" }
        ),
    );
}

/// A Welcome: the whole show, or only what a resuming member missed.
fn welcome<H: SessionHost>(ctx: &H, resume: Option<(Uuid, u64)>) -> Welcome {
    let journal = &ctx.session_ref().journal;
    let missed = resume
        .filter(|(id, _)| *id == journal.id())
        .and_then(|(_, seq)| journal.ops_from(seq + 1))
        .map(|ops| ops.into_iter().cloned().collect::<Vec<_>>());
    let (show, undo, ops) = match missed {
        Some(ops) => (None, BTreeMap::new(), ops),
        None => (
            serde_json::to_string(&ctx.state_ref().root.show_data).ok(),
            undo_stacks(ctx.state_ref()),
            Vec::new(),
        ),
    };
    Welcome {
        master: ctx.state_ref().root.station_settings.station_name.clone(),
        master_priority: Some(ctx.state_ref().root.station_settings.station_priority),
        lumasession: ctx.state_ref().root.station_settings.lumasession.clone(),
        commit: lumacore_objects::build_info().git_commit,
        journal: journal.id(),
        epoch: journal.epoch(),
        seq: journal.tip_seq(),
        show,
        undo,
        ops,
        runtime: session_runtime(ctx),
        merged: Vec::new(),
        cmd_port: ctx.state_ref().lumasession_ports.0,
    }
}

fn broadcast<H: SessionHost>(ctx: &H, msg: &StreamMsg) {
    // Several members: encode once, every member's writer sends the same bytes (decisions §31).
    let shared = (ctx.session_ref().peers.len() > 1).then(|| super::stream::encode_frame(msg).ok()).flatten();
    for peer in ctx.session_ref().peers.values() {
        let out = match &shared {
            Some(frame) => StreamMsg::Encoded(frame.clone()),
            None => msg.clone(),
        };
        let _ = peer.tx.send(out);
    }
}

/// After every job on the master: number what changed and send it to the
/// members. Standalone (no members) the outbox is simply emptied.
pub fn flush<H: SessionHost>(ctx: &mut H) {
    let state = &mut ctx.state();
    if state.link.is_some() {
        // A member never numbers changes; its own go out as Changes.
        state.replication_outbox.clear();
        state.undo_log.clear();
        state.show_replaced = false;
        return;
    }
    // An IdleMaster that changes the show has diverged from its session.
    if let lumacore_objects::session_link::Role::IdleMaster { diverged, .. } = &mut state.session_role
        && !state.replication_outbox.is_empty()
    {
        *diverged = true;
    }
    let in_session = !state.root.runtime.luma_sessions.members.is_empty();
    state.has_members = in_session;
    if !in_session {
        state.replication_outbox.clear();
        state.undo_log.clear();
        state.show_replaced = false;
        ctx.session().sent_runtime = None;
        return;
    }
    if std::mem::take(&mut state.show_replaced) {
        // A new show: members start over from a snapshot.
        state.replication_outbox.clear();
        state.undo_log.clear();
        ctx.session().journal.restart();
        let snapshot = welcome(ctx, None);
        ctx.session().sent_runtime = serde_json::to_value(&snapshot.runtime).ok();
        broadcast(ctx, &StreamMsg::Snapshot(Box::new(snapshot)));
        return;
    }
    let runtime = serde_json::to_value(session_runtime(ctx)).ok();
    let (state, session) = ctx.split();
    let changed_runtime = runtime != session.sent_runtime;
    if state.replication_outbox.is_empty() && state.undo_log.is_empty() && !changed_runtime {
        return;
    }
    let mut pending = std::mem::take(&mut state.replication_outbox);
    if pending.is_empty() {
        pending.push(PendingReplicationOp {
            user: state.effective_user_name().to_string(),
            ..Default::default()
        });
    }
    if let Some(last) = pending.last_mut() {
        last.undo = std::mem::take(&mut state.undo_log);
        if changed_runtime {
            last.runtime = runtime.clone();
        }
    }
    session.sent_runtime = runtime;
    let station = state
        .command_station
        .clone()
        .unwrap_or_else(|| state.root.station_settings.station_name.clone());
    for p in pending {
        let op = ctx.session().journal.commit(p, &station, now_ms());
        broadcast(ctx, &StreamMsg::Op(Box::new(op)));
    }
}

/// Answer a member's Hello: check versions and this station's role, then
/// register it and send the show (or what it missed). Returns the answer
/// and the connection number.
pub fn admit<H: SessionHost>(
    ctx: &mut H,
    hello: Hello,
    addr: SocketAddr,
    tx: Sender<StreamMsg>,
    stats: Arc<LinkStats>,
) -> (StreamMsg, u64) {
    let refuse = |code: &str, message: String| {
        (
            StreamMsg::Refused {
                code: code.to_string(),
                message,
            },
            0,
        )
    };
    let me = ctx.state().root.station_settings.station_name.clone();
    if hello.protocol != SESSION_PROTOCOL {
        return refuse(
            "E-LUMA-001",
            format!(
                "{me} speaks LumaSession protocol {SESSION_PROTOCOL}, {} speaks {}: run the same LumaCore version.",
                hello.station, hello.protocol
            ),
        );
    }
    if hello.format_version != lumacore_tree::FORMAT_VERSION {
        return refuse(
            "E-LUMA-002",
            format!(
                "{me} uses show format {}, {} uses {}: run the same LumaCore version.",
                lumacore_tree::FORMAT_VERSION,
                hello.station,
                hello.format_version
            ),
        );
    }
    let build = lumacore_objects::build_info();
    if hello.version != build.version || hello.commit != build.git_commit {
        return refuse(
            "E-LUMA-006",
            format!(
                "{me} runs LumaCore {} ({}), {} runs {} ({}): every station of a LumaSession runs the same build.",
                build.version, build.git_commit, hello.station, hello.version, hello.commit
            ),
        );
    }
    if !ctx.state().root.station_settings.online {
        return refuse("E-LUMA-003", format!("LumaSession is off on {me}."));
    }
    if ctx.state().link.is_some() {
        return refuse(
            "E-LUMA-003",
            format!("{me} is a member of a LumaSession: join its master instead."),
        );
    }
    if let lumacore_objects::session_link::Role::Fenced { by } = &ctx.state().session_role {
        return refuse(
            "E-LUMA-003",
            format!("{me} is waiting for its operator (two masters with {by}): join {by} instead."),
        );
    }
    if hello.station.eq_ignore_ascii_case(&me) {
        return refuse(
            "E-LUMA-003",
            format!("A station can't join itself ({me}); give the stations different names."),
        );
    }
    // Members are one per station: the name is the key, the id says which
    // station it is (R1-24). A second station with a member's name (in the
    // session or reconnecting) would take its place and its record.
    if hello.station_id.is_empty() {
        return refuse(
            "E-LUMA-003",
            format!("{} sent no station id: run the same LumaCore version.", hello.station),
        );
    }
    if let Some(other) = ctx.state()
        .root
        .runtime
        .luma_sessions
        .members
        .iter()
        .find(|(name, m)| name.eq_ignore_ascii_case(&hello.station) && m.station_id != hello.station_id)
        .map(|(name, m)| (name.clone(), m.connected))
    {
        let (name, connected) = other;
        let state = if connected { "is in" } else { "is reconnecting to" };
        return refuse(
            "E-LUMA-003",
            format!(
                "Another station called {name} {state} this LumaSession: give the stations different names (Set Station Name)."
            ),
        );
    }
    // Join with Merge: the joiner's show is merged into this one first
    // (members get the merged show as a snapshot, §21 "6d").
    let merged = match &hello.merge {
        Some(offer) => match merge_in(ctx, &hello.station, offer) {
            Ok(lines) => lines,
            Err(message) => return refuse("E-LUMA-005", message),
        },
        None => Vec::new(),
    };
    // Every station that joined is remembered in the show (re-invites);
    // the joiner gets it with the snapshot, the others as an op.
    super::election::remember_station(
        ctx,
        lumacore_tree::KnownStation {
            meta: lumacore_base::NodeMeta::with_guid(lumacore_tree::KnownStation::guid_for(&hello.station_id), Some(hello.station.clone())),
            station_id: hello.station_id.clone(),
            ip: addr.ip().to_string(),
            stream_port: hello.stream_port,
            priority: hello.priority,
            joined_unix_ms: now_ms(),
        },
    );
    // Number what happened so far before the snapshot is taken.
    flush(ctx);
    ctx.session().next_conn += 1;
    let conn = ctx.session().next_conn;
    let rejoin = ctx.state()
        .root
        .runtime
        .luma_sessions
        .members
        .contains_key(&hello.station);
    // What already ran is kept by station id, outside the connection (in
    // the replicated session runtime, answers here): a reconnect resends
    // what wasn't answered and nothing runs twice. A new link (a new join)
    // starts its command ids over.
    if ctx.session().replies.get(&hello.station_id).is_some_and(|c| c.link != hello.link) {
        ctx.session().replies.remove(&hello.station_id);
    }
    let sessions = &mut ctx.state().root.runtime.luma_sessions;
    if sessions.commands_done.get(&hello.station_id).is_some_and(|d| d.link != hello.link) {
        sessions.commands_done.remove(&hello.station_id);
    }
    ctx.session().peers.insert(
        hello.station.clone(),
        Peer {
            tx,
            conn,
            station_id: hello.station_id.clone(),
            link: hello.link,
            stats,
        },
    );
    ctx.state().root.runtime.luma_sessions.members.insert(
        hello.station.clone(),
        MemberRuntime {
            station: hello.station.clone(),
            ip: addr.ip().to_string(),
            cmd_port: hello.cmd_port,
            joined_unix_ms: now_ms(),
            connected: true,
            users: Vec::new(),
            station_id: hello.station_id.clone(),
            stream_port: hello.stream_port,
        },
    );
    ctx.state().has_members = true;
    // An IdleMaster that someone joins is a master again.
    ctx.state().session_role = lumacore_objects::session_link::Role::Normal;

    // The joiner gets the runtime in its Welcome; the others learn the new
    // member list from the next flush (don't mark it sent here).
    let mut welcome = welcome(ctx, if merged.is_empty() { hello.resume } else { None });
    welcome.merged = merged;
    note(
        ctx.state(),
        format!(
            "{} {} this LumaSession as {}",
            hello.station,
            if rejoin { "rejoined" } else { "joined" },
            hello.user
        ),
    );
    ctx.refresh_runtime();
    (StreamMsg::Welcome(Box::new(welcome)), conn)
}

/// A member asked for a snapshot (it found a gap).
pub fn resync<H: SessionHost>(ctx: &mut H, station: &str, conn: u64, resume: Option<(Uuid, u64)>) {
    flush(ctx);
    let snapshot = welcome(ctx, resume);
    if let Some(peer) = ctx.session().peers.get(station).filter(|p| p.conn == conn) {
        let _ = peer.tx.send(StreamMsg::Snapshot(Box::new(snapshot)));
    }
}

/// Run a member's line as its user, send the results, then the reply.
pub fn run_forwarded<H: SessionHost>(
    ctx: &mut H,
    station: &str,
    conn: u64,
    id: u64,
    user: &str,
    line: &str,
    gesture: Option<lumacore_objects::gesture::GestureLine>,
) {
    let Some(peer) = ctx.session_ref().peers.get(station).filter(|p| p.conn == conn) else {
        return;
    };
    let (station_id, link) = (peer.station_id.clone(), peer.link);
    let done = ctx.state_ref()
        .root
        .runtime
        .luma_sessions
        .commands_done
        .get(&station_id)
        .is_some_and(|d| d.ran(link, id));
    if done {
        // Resent after a reconnect or to a new master: it already ran here
        // or on the master before (decisions §10: never run twice).
        let reply = ctx.session_ref().replies
            .get(&station_id)
            .filter(|c| c.link == link)
            .and_then(|c| c.replies.get(&id).cloned())
            .unwrap_or_else(|| StreamMsg::Reply {
                id,
                ok: true,
                message: lumacore_objects::text::render_success(
                    "Already ran on the master (its answer was lost).",
                ),
            });
        let _ = peer.tx.send(reply);
        return;
    }
    ctx.state().command_station = Some(station.to_string());
    ctx.state().gesture = gesture;
    let (result, exit) = ctx.execute_remote_command(user, line);
    ctx.state().gesture = None;
    if exit {
        ctx.request_shutdown();
    }
    // Recorded before the results go out, so it rides with them: every
    // member (and so the next master) knows it ran.
    ctx.state()
        .root
        .runtime
        .luma_sessions
        .commands_done
        .insert(station_id.clone(), lumacore_tree::CommandsDone { link, last: id });
    flush(ctx);
    ctx.state().command_station = None;
    let reply = StreamMsg::Reply {
        id,
        ok: result.ok,
        message: result.message,
    };
    let cache = ctx.session().replies.entry(station_id).or_default();
    if cache.link != link {
        *cache = ReplyCache { link, replies: BTreeMap::new() };
    }
    cache.replies.insert(id, reply.clone());
    while cache.replies.len() > KEPT_REPLIES {
        cache.replies.pop_first();
    }
    if let Some(peer) = ctx.session().peers.get(station).filter(|p| p.conn == conn) {
        let _ = peer.tx.send(reply);
    }
}

/// A member's own show changes: file them here as its user.
pub fn apply_changes<H: SessionHost>(
    ctx: &mut H,
    station: &str,
    user: &str,
    line: &str,
    undo: Option<OopsKind>,
    snapshots: Vec<SubtreeSnapshot>,
    local: Vec<LocalSnapshot>,
) {
    if !ctx.session().peers.contains_key(station) {
        return;
    }
    let state = &mut ctx.state();
    if state.begin_acting_as_existing_user(user).is_none() {
        return;
    }
    state.command_station = Some(station.to_string());
    let undo = undo.map_or(Undo::Untracked, Undo::Tracked);
    // The member's station settings / watches ride in the step (they are
    // put back only there).
    state.pending_local = local;
    lumacore_objects::commands::record_change(state, line, undo, |state| {
        if let Err(e) = lumacore_objects::commands::oops::restore_subtrees(state, &snapshots) {
            log::error!("changes from {station} didn't apply: {e}");
        }
    });
    state.clear_acting_user();
    flush(ctx);
    ctx.state().command_station = None;
}

/// The users logged in on a member.
pub fn logins<H: SessionHost>(ctx: &mut H, station: &str, users: Vec<String>) {
    if let Some(member) = ctx.state().root.runtime.luma_sessions.members.get_mut(station) {
        member.users = users;
    }
}

/// A member's connection ended: `bye` = it left; else it may come back.
pub fn peer_gone<H: SessionHost>(ctx: &mut H, station: &str, conn: u64, bye: bool) {
    if ctx.session().peers.get(station).is_none_or(|p| p.conn != conn) {
        return;
    }
    let Some(peer) = ctx.session().peers.remove(station) else {
        return;
    };
    let members = &mut ctx.state().root.runtime.luma_sessions.members;
    if bye {
        if let Some(id) = members.get(station).map(|m| m.station_id.clone()) {
            ctx.session().watch.left.insert(id);
        }
        // It left: its pending commands were dropped there, so what ran of
        // them is forgotten here (a lost connection keeps both).
        ctx.session().replies.remove(&peer.station_id);
        let sessions = &mut ctx.state().root.runtime.luma_sessions;
        sessions.commands_done.remove(&peer.station_id);
        let members = &mut sessions.members;
        members.remove(station);
        note(ctx.state(), format!("{station} left this LumaSession"));
    } else if let Some(member) = members.get_mut(station) {
        member.connected = false;
        note(
            ctx.state(),
            format!("Lost {station}; it rejoins when it comes back"),
        );
    }
    ctx.refresh_runtime();
}

// *************************************************************************************************
// Both
// *************************************************************************************************
/// After every job: the master sends what changed; a member starts joins
/// commands asked for, reports its logins and tells clients about commands
/// still pending.
pub fn after_work<H: SessionHost>(ctx: &mut H) {
    for request in std::mem::take(&mut ctx.state().link_requests) {
        start_link(ctx, request);
    }
    // Takeovers nobody answered are handed over (the master decides).
    if ctx.state().link.is_none() {
        ctx.state().expire_claim_asks();
    }
    for (user, text) in std::mem::take(&mut ctx.state().claim_notices) {
        let notice = lumacore_objects::text::render_success(&text);
        ctx.push_output(&user, &notice);
    }
    flush(ctx);
    super::election::run_requests(ctx);
    super::election::tick(ctx);
    if ctx.state().link.is_none() {
        return;
    }
    let users = ctx.shell_users();
    if ctx.session().sent_logins.as_ref() != Some(&users)
        && let Some(link) = &ctx.state().link
        && link.tx.send(lumacore_objects::session_link::LinkOut::Logins { users: users.clone() }).is_ok()
    {
        ctx.session().sent_logins = Some(users);
    }
    let master = ctx.state()
        .link
        .as_ref()
        .map(|l| l.master.clone())
        .unwrap_or_default();
    let late: Vec<ReplyTo> = ctx.state()
        .forwards
        .values_mut()
        .filter(|f| !f.noted && f.sent.elapsed() >= PENDING_NOTICE)
        .map(|f| {
            f.noted = true;
            f.reply_to.clone()
        })
        .collect();
    for to in late {
        if matches!(to, ReplyTo::Shell(_)) {
            let text = format!(
                "Pending: {master} hasn't confirmed yet — the command stays in the outbox and runs when it does."
            );
            reply(ctx, &to, &lumacore_objects::text::render_warning(&text));
        }
    }
}

/// What this station brings to a join with Merge: its show, its split base
/// and its own journal (§21 "6d").
pub fn merge_offer(state: &AppState, own_journal: Uuid) -> super::stream::MergeOffer {
    let base = state.split_base.as_ref();
    super::stream::MergeOffer {
        show: serde_json::to_string(&state.root.show_data).unwrap_or_default(),
        base: base.and_then(|b| serde_json::to_string(&*b.show).ok()),
        base_journal: base.and_then(|b| b.journal),
        journal: Some(own_journal),
    }
}

/// Start the link thread for a join.
fn start_link<H: SessionHost>(ctx: &mut H, request: LinkRequest) {
    let Some(state_handle) = ctx.state_handle().clone() else {
        join_failed(ctx, &request, "the server isn't running its threads yet");
        return;
    };
    if ctx.state().link.is_some() {
        join_failed(ctx, &request, "this station is already in a LumaSession");
        return;
    }
    let target = format!("{}:{}", request.host, request.port);
    let (handle, rx) = LinkHandle::new(target);
    let hello = Hello {
        protocol: SESSION_PROTOCOL,
        format_version: lumacore_tree::FORMAT_VERSION,
        version: lumacore_objects::build_info().version,
        commit: lumacore_objects::build_info().git_commit,
        station: ctx.state().root.station_settings.station_name.clone(),
        station_id: ctx.state().local_settings.station_id.clone(),
        priority: ctx.state().root.station_settings.station_priority,
        user: request.user.clone(),
        cmd_port: ctx.cmd_port(),
        stream_port: ctx.stream_port(),
        link: handle.link,
        resume: None,
        peek: false,
        merge: request.merge.then(|| merge_offer(ctx.state_ref(), ctx.session_ref().journal.id())),
    };
    ctx.state().show_status = lumacore_objects::session_link::ShowStatus::Downloading;
    super::stream::spawn_link(state_handle, request, hello, handle, rx);
}
