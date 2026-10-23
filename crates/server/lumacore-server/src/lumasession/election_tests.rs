//! Master loss, election, two masters, handover and known stations without
//! sockets (decisions §21 "6c hardening").

use std::sync::Arc;

use lumacore_lumasession::election::*;
use crate::context::ServerContext;

use lumacore_objects::session_link::{LinkRequest, ReplyTo, Role};
use lumacore_objects::AppState;

use crate::net::handler::execute_remote_command;
use lumacore_cmd_station::cmd_delete_station;
use lumacore_objects::session_link::{
    Forward, LinkHandle, status_word,
};
use lumacore_lumasession::link;
use lumacore_tree::StationPriority;
use lumacore_lumasession::stream::{Hello, StreamMsg};
use lumacore_wire::SESSION_PROTOCOL;
use lumacore_net::DiscoveryPeerState;

fn station(name: &str) -> ServerContext {
    let mut state = AppState::new();
    state.root.station_settings.station_name = name.to_string();
    state.local_settings.station_id = format!("id-{name}");
    state.switch_to_internal_root();
    ServerContext::new(state, None, false, 0, 0, 0)
}

fn hello(name: &str, priority: StationPriority) -> Hello {
    Hello {
        protocol: SESSION_PROTOCOL,
        format_version: lumacore_tree::FORMAT_VERSION,
        version: lumacore_objects::build_info().version,
        commit: lumacore_objects::build_info().git_commit,
        station: name.to_string(),
        station_id: format!("id-{name}"),
        priority,
        user: "Admin".to_string(),
        cmd_port: 47653,
        stream_port: 47658,
        link: uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, name.as_bytes()),
        resume: None,
        peek: false,
        merge: None,
    }
}

/// A beacon as discovery stores it.
fn seen_as(name: &str, status: &str, priority: &str, epoch: u64, applied: u64) -> DiscoveryPeerState {
    DiscoveryPeerState {
        key: format!("id-{name}:47653"),
        station_name: name.to_string(),
        authority: name.to_string(),
        endpoint: String::new(),
        cmd_port: 47653,
        stream_port: 47658,
        session_count: 0,
        reachable: true,
        last_seen_unix_ms: lumacore_tree::now_ms(),
        ip: "10.0.0.9".parse().unwrap(),
        lumasession: "LumaCore".to_string(),
        location: String::new(),
        priority: priority.to_string(),
        status: status.to_string(),
        version: String::new(),
        invite: true,
        station_id: format!("id-{name}"),
        epoch,
        applied,
        online_since_ms: 0,
        show_status: String::new(),
    }
}

fn discover(ctx: &ServerContext, peer: DiscoveryPeerState) {
    ctx.discovery_cache.lock().unwrap().insert(peer.key.clone(), peer);
}

fn member(ctx: &mut ServerContext, name: &str) {
    ctx.state.root.runtime.luma_sessions.members.insert(
        name.to_string(),
        lumacore_tree::MemberRuntime {
            station: name.to_string(),
            ip: "10.0.0.9".into(),
            cmd_port: 47653,
            stream_port: 47658,
            station_id: format!("id-{name}"),
            connected: true,
            ..Default::default()
        },
    );
}

/// A member of FOH-1 (link to it, members FOH-2 and FOH-3 replicated).
fn member_of_foh1(name: &str) -> ServerContext {
    let mut ctx = station(name);
    let (mut handle, _rx) = LinkHandle::test_handle("10.0.0.1:47658");
    handle.master = "FOH-1".into();
    ctx.state.link = Some(handle);
    for m in ["FOH-2", "FOH-3"] {
        member(&mut ctx, m);
    }
    ctx
}

#[test]
fn every_member_computes_the_same_winner_from_what_it_still_sees() {
    let mut b = member_of_foh1("FOH-2");
    b.state.last_applied = Some((uuid::Uuid::nil(), 10));
    // Nobody else is seen: alone.
    assert_eq!(elect(&b, &[]), Elected::Alone);
    // FOH-3 is seen with the same priority but more results applied: it wins.
    discover(&b, seen_as("FOH-3", "Member", "Normal", 1, 12));
    assert!(matches!(elect(&b, &[]), Elected::Other { name, .. } if name == "FOH-3"));
    // A higher priority beats more results.
    b.state.root.station_settings.station_priority = StationPriority::High;
    assert_eq!(elect(&b, &[]), Elected::Me);
    // FOH-3 can't be reached: excluded, FOH-2 is alone again.
    b.state.root.station_settings.station_priority = StationPriority::Normal;
    assert_eq!(elect(&b, &["FOH-3".to_string()]), Elected::Alone);
    // `Never` can't win.
    b.state.root.station_settings.station_priority = StationPriority::Never;
    discover(&b, seen_as("FOH-3", "Member", "Never", 1, 12));
    assert_eq!(elect(&b, &[]), Elected::Alone);
}

#[test]
fn the_winner_becomes_master_with_a_new_epoch_and_runs_what_was_pending() {
    let mut b = member_of_foh1("FOH-2");
    b.state.session_epoch = 3;
    let (tx, rx) = crossbeam_channel::bounded(1);
    b.state.forwards.insert(
        1,
        Forward {
            user: "Admin".into(),
            line: "Store Network \"Late\"".into(),
            raw: "Store Network \"Late\"".into(),
            reply_to: ReplyTo::Waiter(tx),
            sent: std::time::Instant::now(),
            noted: false,
            ran: false,
        },
    );
    become_master(&mut b, None, "test");
    assert!(b.state.link.is_none());
    assert_eq!(b.lumasession.journal.epoch(), 4);
    assert_eq!(status_word(&b.state), "Master");
    assert!(b.state.root.show_data.protocols.networks.values().any(|n| n.meta.identity.name.as_deref() == Some("Late")));
    assert!(rx.try_recv().unwrap().contains("Late"));
    // The other members are waited for (they reconnect).
    assert!(!b.state.root.runtime.luma_sessions.members["FOH-3"].connected);
    assert!(lumacore_objects::protocol_plan::may_send(&b.state));
}

#[test]
fn a_station_alone_is_idlemaster_and_silent_unless_output_when_alone() {
    let mut b = member_of_foh1("FOH-2");
    become_idle(&mut b, "FOH-1");
    assert_eq!(status_word(&b.state), "IdleMaster");
    assert!(!lumacore_objects::protocol_plan::may_send(&b.state));
    b.state.root.station_settings.output_when_alone = true;
    assert!(lumacore_objects::protocol_plan::may_send(&b.state));
    // Changing the show while alone means it diverged.
    let (reply, _) = execute_remote_command(&mut b, "Admin", "Store Network \"Alone\"");
    assert!(reply.ok, "{}", reply.message);
    link::flush(&mut b);
    assert!(matches!(b.state.session_role, Role::IdleMaster { diverged: true, .. }));
}

#[test]
fn the_lower_epoch_master_is_fenced_and_asked() {
    let mut a = station("FOH-1");
    let (to_member, from_a) = crossbeam_channel::unbounded();
    let (answer, _) = link::admit(
        &mut a,
        hello("FOH-2", StationPriority::Normal),
        "127.0.0.1:50000".parse().unwrap(),
        to_member,
        Arc::default(),
    );
    assert!(matches!(answer, StreamMsg::Welcome(_)));
    assert_eq!(status_word(&a.state), "Master");
    // Meanwhile FOH-3 was elected with epoch 2.
    discover(&a, seen_as("FOH-3", "Master", "Normal", 2, 0));
    tick(&mut a);
    assert!(matches!(&a.state.session_role, Role::Fenced { by } if by == "FOH-3"));
    assert!(!lumacore_objects::protocol_plan::may_send(&a.state), "fenced: no DMX");
    let d = a.state.decision.clone().expect("a decision");
    let labels: Vec<&str> = d.options.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(labels, ["Merge", "Use Master", "Use Mine", "Cancel"]);
    assert_eq!(d.default, "Merge");
    // Its member was sent to the new master.
    assert!(from_a.try_iter().any(|m| matches!(m, StreamMsg::NewMaster { station, .. } if station == "FOH-3")));
    // Cancel: stay alone, no more prompts.
    lumacore_objects::commands::dispatch(&mut a.state, "Leave LumaSession");
    assert!(a.state.decision.is_none());
    assert!(matches!(a.state.session_role, Role::IdleMaster { quiet: true, .. }));
}

#[test]
fn claim_lumasession_hands_the_master_role_over() {
    let mut a = station("FOH-1");
    let (to_member, from_a) = crossbeam_channel::unbounded();
    link::admit(
        &mut a,
        hello("FOH-2", StationPriority::Normal),
        "127.0.0.1:50000".parse().unwrap(),
        to_member,
        Arc::default(),
    );
    let epoch = a.lumasession.journal.epoch();
    hand_over(&mut a, "FOH-2");
    let moved = from_a.try_iter().find_map(|m| match m {
        StreamMsg::NewMaster { station, epoch, .. } => Some((station, epoch)),
        _ => None,
    });
    assert_eq!(moved, Some(("FOH-2".to_string(), epoch + 1)));
    // The old master follows the new one.
    assert!(a.state.root.runtime.luma_sessions.members.is_empty());
    assert!(matches!(&a.state.link_requests[..], [LinkRequest { patient: true, .. }]));
}

#[test]
fn joined_stations_are_known_in_the_show_until_deleted() {
    let mut a = station("FOH-1");
    let (tx, _rx) = crossbeam_channel::unbounded();
    link::admit(
        &mut a,
        hello("FOH-2", StationPriority::High),
        "127.0.0.1:50000".parse().unwrap(),
        tx,
        Arc::default(),
    );
    let known = &a.state.root.show_data.session.stations["id-FOH-2"];
    assert_eq!((known.meta.name().unwrap_or_default(), known.priority), ("FOH-2", StationPriority::High));
    // Connected: Dismiss first.
    a.state.user = "Admin".into();
    let out = cmd_delete_station(&mut a.state, &["\"FOH-2\""]);
    assert!(out.contains("Dismiss"), "{out}");
    a.state.root.runtime.luma_sessions.members.clear();
    let out = cmd_delete_station(&mut a.state, &["\"FOH-2\""]);
    assert!(a.state.root.show_data.session.stations.is_empty(), "{out}");
}

#[test]
fn a_live_patch_change_raises_the_patch_version_and_undo_frames_know_their_station() {
    let mut a = station("FOH-1");
    let before = a.state.root.show_data.session.patch_version;
    execute_remote_command(&mut a, "Admin", "Claim Root.ShowData.Devices");
    execute_remote_command(&mut a, "Admin", "Store Universe 7");
    let (reply, _) = execute_remote_command(&mut a, "Admin", "Unclaim Root.ShowData.Devices /Keep");
    assert_eq!(a.state.root.show_data.session.patch_version, before + 1, "{}", reply.message);
    // Undo restores the patch, the version still rises (never undone).
    execute_remote_command(&mut a, "Admin", "Oops");
    assert_eq!(a.state.root.show_data.session.patch_version, before + 2);
    let frame = a.state.undo_stack("Admin").unwrap().redo_frames.back().unwrap().clone();
    assert_eq!(frame.station, "FOH-1");
    assert!(frame.time_ms > 0);
}

#[test]
fn the_merge_countdown_runs_on_the_monotonic_clock() {
    let mut a = station("FOH-1");
    let (to_member, _from_a) = crossbeam_channel::unbounded();
    link::admit(
        &mut a,
        hello("FOH-2", StationPriority::Normal),
        "127.0.0.1:50000".parse().unwrap(),
        to_member,
        Arc::default(),
    );
    discover(&a, seen_as("FOH-3", "Master", "Normal", 2, 0));
    tick(&mut a);
    let id = a.state.decision.as_ref().expect("a decision").id;
    // The wall clock jumps past the countdown: nothing is decided.
    a.state.decision.as_mut().unwrap().deadline_ms = Some(0);
    a.lumasession.watch.last_tick = None;
    tick(&mut a);
    assert!(a.state.decision.is_some(), "a wall-clock step decides nothing");
    // The monotonic countdown ends: the default runs.
    a.lumasession.watch.decision_due = Some((id, std::time::Instant::now()));
    a.lumasession.watch.last_tick = None;
    tick(&mut a);
    assert!(a.state.decision.is_none(), "decided by default");
}

/// R1-16: a show is only replaced when its unsaved changes were kept.
#[test]
fn unsaved_changes_that_cant_be_kept_are_never_replaced() {
    struct DoorOpen;
    impl Drop for DoorOpen {
        fn drop(&mut self) {
            lumacore_luma::drive::set_off(false);
        }
    }
    let _open = DoorOpen;
    let mut a = station("FOH-1");
    a.state.show_dirty = true;
    a.request_from = Some("10.0.0.9:47653".parse().unwrap());
    lumacore_luma::drive::set_off(true);

    // Invite: declined, the inviter is told why.
    let reply = lumacore_lumasession::handle_invite(&mut a, "LumaCore".into(), "FOH-9".into(), 47658);
    assert!(!reply.ok && reply.message.contains("E-CMD-012"), "{}", reply.message);
    assert!(a.lumasession.deferred.is_empty(), "no join");

    // Use Master (an IdleMaster joins the session): refused.
    a.state.session_role = Role::IdleMaster { diverged: true, lost: "FOH-9".into(), quiet: false };
    let (reply, _) = execute_remote_command(&mut a, "Admin", "Join LumaSession 10.0.0.9");
    assert!(reply.message.contains("E-CMD-012"), "{}", reply.message);
    assert!(a.state.link_requests.is_empty(), "no join");

    // Yield (the other side chose Use Mine): this master stays.
    a.state.session_role = Role::Normal;
    let reply = handle_yield(&mut a, "FOH-9".into(), 7, 47658);
    assert!(!reply.ok && reply.message.contains("E-CMD-012"), "{}", reply.message);
    assert!(a.state.link_requests.is_empty(), "not following");

    // A member told to follow a new master with an autosave: it leaves,
    // keeping its show.
    let mut b = member_of_foh1("FOH-2");
    b.state.show_dirty = true;
    link::moved(&mut b, "FOH-1", "FOH-3", true);
    assert!(b.state.link.is_none(), "not following");
    assert!(b.state.root.station_settings.output_held);

    // With the door open the invite goes ahead.
    lumacore_luma::drive::set_off(false);
    let reply = lumacore_lumasession::handle_invite(&mut a, "LumaCore".into(), "FOH-9".into(), 47658);
    assert!(reply.ok, "{}", reply.message);
    assert_eq!(a.lumasession.deferred.len(), 1);
}

/// R1-22 (Q26): a station that leads the LumaSession — a master with
/// members, or alone after changing the show — never joins an invite
/// silently (the inviter was away; its show may be older): its operator
/// gets Use Mine (the default) / Use Master / Cancel, Merge greyed.
#[test]
fn an_invite_to_the_session_this_station_leads_asks() {
    let mut a = station("FOH-1");
    a.request_from = Some("10.0.0.9:47653".parse().unwrap());
    a.state.session_role = Role::IdleMaster { diverged: true, lost: "FOH-9".into(), quiet: false };
    let session = a.state.root.station_settings.lumasession.clone();
    let reply = lumacore_lumasession::handle_invite(&mut a, session, "FOH-9".into(), 47658);
    assert!(!reply.ok && reply.message.contains("decides"), "{}", reply.message);
    assert!(a.lumasession.deferred.is_empty(), "no join");
    let d = a.state.decision.as_ref().expect("the operator is asked");
    assert_eq!(d.default, "Use Mine");
    assert!(d.merge_blocked.is_some());
    assert!(d.options.iter().any(|(l, c)| l == "Use Mine" && c == "Claim LumaSession"));

    // R1 review 19: a master with members — Use Mine invites the station
    // that came back, Cancel only closes; every answer closes the decision.
    let mut m = station("FOH-3");
    m.request_from = Some("10.0.0.9:47653".parse().unwrap());
    member(&mut m, "FOH-4");
    let session = m.state.root.station_settings.lumasession.clone();
    let reply = lumacore_lumasession::handle_invite(&mut m, session, "FOH-9".into(), 47658);
    assert!(!reply.ok, "{}", reply.message);
    let d = m.state.decision.clone().expect("the operator is asked");
    assert!(d.options.iter().any(|(l, c)| l == "Use Mine" && c == "Invite Station \"FOH-9\""), "{:?}", d.options);
    let cancel = d.options.iter().position(|(l, _)| l == "Cancel").unwrap();
    let who = lumacore_objects::dialogs::Who { user: "Admin".into(), device: None, client: None };
    let answer = lumacore_wire::DialogAnswer { id: "merge".into(), button: cancel, ..Default::default() };
    assert_eq!(lumacore_objects::dialogs::answer(&mut m.state, &who, &answer), Ok(None), "Cancel runs nothing");
    assert!(m.state.decision.is_none(), "and closes the decision");

    // An IdleMaster that changed nothing rejoins as before.
    let mut b = station("FOH-2");
    b.request_from = Some("10.0.0.9:47653".parse().unwrap());
    b.state.session_role = Role::IdleMaster { diverged: false, lost: "FOH-9".into(), quiet: false };
    let session = b.state.root.station_settings.lumasession.clone();
    let reply = lumacore_lumasession::handle_invite(&mut b, session, "FOH-9".into(), 47658);
    assert!(reply.ok, "{}", reply.message);
}
