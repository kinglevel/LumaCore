// ███████╗███████╗███████╗███████╗██╗ ██████╗ ███╗   ██╗
// ██╔════╝██╔════╝██╔════╝██╔════╝██║██╔═══██╗████╗  ██║
// ███████╗█████╗  ███████╗███████╗██║██║   ██║██╔██╗ ██║
// ╚════██║██╔══╝  ╚════██║╚════██║██║██║   ██║██║╚██╗██║
// ███████║███████╗███████║███████║██║╚██████╔╝██║ ╚████║
// ╚══════╝╚══════╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: session

//! LumaSession: the server-to-server side (decisions §11 "Menus,
//! LumaSession and staged patch"; the Network menu).
//!
//! - Sessions are named and found by discovery; a station is Standalone,
//!   Master (others joined it) or Member (it joined a master).
//! - A master invites a station (`Invite Station "X"`): with Invite on, the
//!   station joins without asking. A master dismisses a member
//!   (`Dismiss Station "X"`). Members join over the LumaSession stream
//!   (`crate::stream`, decisions §21 "6b"); the master's member list
//!   follows its connections.
//! - Requests that make a station join or leave answer at once and run the
//!   join/leave after the job (`Deferred`): the join talks to the sender,
//!   which is still waiting for the answer.
//! - A station that leaves or is dismissed holds its DMX output until its
//!   operator resumes it.
//! - Its state on the station is one [`LumaSession`]; the station reaches it through its
//!   [`SessionHost`] (decisions §34 19.3).
//!
//! Owns:          LumaSession (the session's fields on a station), Deferred, the invite and dismiss requests, the work they leave for after their job.
//! Doesn't own:   the link, the election, the stream (their modules), the command port (the server).
//! Entry points:  LumaSession, handle_invite, handle_dismiss, run_deferred, station_user.
//! Decisions:     §11, §21 "6b", "6c", §34 19.3.

use std::net::SocketAddr;

use lumacore_objects::AppState;
use lumacore_wire::RemoteReply;

use crate::host::SessionHost;
use crate::{election, link};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a station keeps of its LumaSession (decisions §34 19.3: the session's fields, one
/// struct on the server's context).
#[derive(Default)]
pub struct LumaSession {
    /// The replication journal (`(epoch, seq)` ops a master sent; §21 "6b").
    pub journal: lumacore_tree::ReplicationJournal,
    /// Work requests left for after their job (`Deferred`).
    pub deferred: Vec<Deferred>,
    /// The master's members, by station (their stream connection).
    pub peers: std::collections::BTreeMap<String, link::Peer>,
    /// Replies a master keeps for members that ask again (by station).
    pub replies: std::collections::BTreeMap<String, link::ReplyCache>,
    /// The next stream connection number.
    pub next_conn: u64,
    /// The session runtime last sent to the members.
    pub sent_runtime: Option<serde_json::Value>,
    /// The logged-in users last sent to the master.
    pub sent_logins: Option<Vec<String>>,
    /// Elections, fencing, rejoin decisions (`election`).
    pub watch: election::SessionWatch,
}

/// Work a request leaves for after its job.
#[derive(Debug, Clone)]
pub enum Deferred {
    /// Join the master that invited us.
    Join {
        host: String,
        stream_port: u16,
        lumasession: String,
        from: String,
        /// Where this show's unsaved changes were kept (`invite` autosave).
        kept: Option<String>,
    },
    /// Leave: dismissed by the master.
    Leave { from: String },
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn reply<H: SessionHost>(ctx: &H, ok: bool, message: String) -> RemoteReply {
    ctx.make_reply(ok, message)
}

fn note(state: &mut AppState, message: String) {
    state
        .sysmon_pending
        .push(("LumaSession".to_string(), message));
}

/// `RemoteRequest::Invite` on the invited station.
pub fn handle_invite<H: SessionHost>(
    ctx: &mut H,
    lumasession: String,
    from: String,
    stream_port: u16,
) -> RemoteReply {
    let station = &ctx.state_ref().root.station_settings;
    let refuse = if !station.online {
        Some("LumaSession is off on this station")
    } else if !station.invite {
        Some("this station doesn't accept invites (Invite is off)")
    } else if lumacore_objects::session_link::status_word(ctx.state_ref()) == "Member" {
        Some("this station is already in a LumaSession")
    } else {
        None
    };
    if let Some(why) = refuse {
        return reply(ctx, false, format!("{}: {why}", station.station_name));
    }
    let Some(SocketAddr::V4(src)) = ctx.request_from() else {
        return reply(ctx, false, "invite without a sender address".to_string());
    };
    // This station leads the LumaSession it is invited to — a master with
    // members, or alone after changing the show — so the inviter was away
    // and its show may be older: never join silently, its operator decides
    // (R1-22, Q26). An IdleMaster that changed nothing rejoins.
    use lumacore_objects::session_link::Role;
    let leads = station.lumasession.eq_ignore_ascii_case(&lumasession)
        && (lumacore_objects::session_link::status_word(ctx.state_ref()) == "Master"
            || matches!(ctx.state_ref().session_role, Role::Fenced { .. } | Role::IdleMaster { diverged: true, .. }));
    let changed_alone = matches!(ctx.state_ref().session_role, Role::Fenced { .. } | Role::IdleMaster { diverged: true, .. });
    if leads || changed_alone {
        let name = station.station_name.clone();
        election::invited_back(
            ctx,
            election::Rival {
                name: from.clone(),
                ip: src.ip().to_string(),
                cmd_port: src.port(),
                stream_port,
                epoch: 0,
            },
        );
        return reply(
            ctx,
            false,
            format!("{name} kept its own show: its operator decides (Use Mine / Use Master / Cancel)"),
        );
    }
    // The session's show replaces this one: keep unsaved changes first, and
    // decline when they can't be kept (R1-16).
    let kept = match lumacore_objects::session_link::autosave_copy(ctx.state_ref(), "invite") {
        Ok(kept) => kept,
        Err(e) => {
            let name = ctx.state_ref().root.station_settings.station_name.clone();
            note(
                ctx.state(),
                format!("[ERROR] Invite from {from} declined: this show's unsaved changes couldn't be kept ({e}); nothing was replaced."),
            );
            return reply(
                ctx,
                false,
                lumacore_objects::errors::with_code(
                    format!("{name} cannot keep its unsaved changes ({e}); nothing was replaced"),
                    "E-CMD-012",
                ),
            );
        }
    };
    ctx.session().deferred.push(Deferred::Join {
        host: src.ip().to_string(),
        stream_port,
        lumasession: lumasession.clone(),
        from: from.clone(),
        kept,
    });
    let name = ctx.state_ref().root.station_settings.station_name.clone();
    reply(
        ctx,
        true,
        format!("{name} joins LumaSession \"{lumasession}\" (invited by {from})"),
    )
}

/// `RemoteRequest::Dismiss` on a member.
pub fn handle_dismiss<H: SessionHost>(ctx: &mut H, from: String) -> RemoteReply {
    let src = ctx.request_from().map(|a| a.ip().to_string());
    let from_master = ctx.state()
        .root
        .runtime
        .joined_sessions
        .values()
        .any(|j| Some(&j.authority) == src.as_ref() || src.as_deref() == Some("127.0.0.1"));
    if !from_master {
        return reply(ctx, false, "only this station's master can dismiss it".to_string());
    }
    ctx.session().deferred.push(Deferred::Leave { from: from.clone() });
    let name = ctx.state().root.station_settings.station_name.clone();
    reply(ctx, true, format!("{name} leaves (dismissed by {from})"))
}

/// The user an invited join logs in as at the master: the first user
/// logged in on this station, else Admin.
pub fn station_user(state: &AppState) -> String {
    let mut users: Vec<&String> = state
        .root
        .runtime
        .users
        .keys()
        .filter(|u| !u.eq_ignore_ascii_case(lumacore_objects::INTERNAL_ROOT_USER))
        .collect();
    users.sort();
    users
        .first()
        .map(|u| u.to_string())
        .unwrap_or_else(|| "Admin".to_string())
}

/// Run what requests left for after their job.
pub fn run_deferred<H: SessionHost>(ctx: &mut H) {
    let saved_user = ctx.state().user.clone();
    ctx.state().user = station_user(ctx.state());
    for work in std::mem::take(&mut ctx.session().deferred) {
        let out = match work {
            Deferred::Join {
                host,
                stream_port,
                lumasession,
                from,
                kept: saved,
            } => {
                let out = lumacore_objects::commands::session::join_lumasession_at(
                    ctx.state(),
                    &host,
                    stream_port,
                    Some(from.clone()),
                );
                format!(
                    "Invited by {from} into \"{lumasession}\"{}: {}",
                    saved.map_or(String::new(), |p| format!(" (changes kept in {p})")),
                    lumacore_objects::text::strip_ansi(&out)
                )
            }
            Deferred::Leave { from } => {
                let out = lumacore_objects::commands::session::leave_lumasession(ctx.state());
                format!(
                    "Dismissed by {from}: {}",
                    lumacore_objects::text::strip_ansi(&out)
                )
            }
        };
        note(ctx.state(), out);
        ctx.drain_sysmon_pending();
        ctx.refresh_runtime();
        ctx.state().protocols_dirty = true;
    }
    ctx.state().user = saved_user;
}
