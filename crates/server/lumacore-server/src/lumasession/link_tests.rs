//! The LumaSession link without sockets: a master and a member
//! `ServerContext` exchange the stream's messages directly (decisions §21
//! "6b").

use lumacore_lumasession::link::*;
use crate::context::ServerContext;
use std::sync::Arc;

use lumacore_tree::ReplicationOp;

use lumacore_lumasession::stream::{Hello, StreamMsg, Welcome};
use crossbeam_channel::Receiver;
use lumacore_wire::SESSION_PROTOCOL;
use lumacore_objects::AppState;
use lumacore_objects::session_link::{LinkHandle, LinkRequest, ReplyTo};

use lumacore_objects::commands::registry::{Scope, Undo, line_scope};
use lumacore_objects::session_link::*;
use lumacore_objects::DispatchResult;
use crate::net::handler::execute_remote_command;

fn station(name: &str) -> ServerContext {
    let mut state = AppState::new();
    state.root.station_settings.station_name = name.to_string();
    state.local_settings.station_id = format!("id-{name}");
    state.switch_to_internal_root();
    ServerContext::new(state, None, false, 0, 0, 0)
}

/// The link a station's join uses in these tests (one per station name).
fn link_of(name: &str) -> uuid::Uuid {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, name.as_bytes())
}

fn hello(name: &str) -> Hello {
    Hello {
        protocol: SESSION_PROTOCOL,
        format_version: lumacore_tree::FORMAT_VERSION,
        version: lumacore_objects::build_info().version,
        commit: lumacore_objects::build_info().git_commit,
        station: name.to_string(),
        station_id: format!("id-{name}"),
        priority: lumacore_tree::StationPriority::Normal,
        user: "Admin".to_string(),
        cmd_port: 47653,
        stream_port: 47658,
        link: link_of(name),
        resume: None,
        peek: false,
        merge: None,
    }
}

fn request() -> LinkRequest {
    LinkRequest {
        host: "127.0.0.1".into(),
        port: 47658,
        user: "Admin".into(),
        reply_to: ReplyTo::Log,
        invited_by: None,
        patient: false,
        merge: false,
    }
}

/// A master with `FOH-2` admitted, and FOH-2 holding the master's show.
/// Returns (master, member, what the master sends FOH-2, what FOH-2 sends).
fn session() -> (
    ServerContext,
    ServerContext,
    Receiver<StreamMsg>,
    Receiver<LinkOut>,
) {
    let mut master = station("FOH-1");
    let (to_member, from_master) = crossbeam_channel::unbounded();
    let (answer, conn) = admit(
        &mut master,
        hello("FOH-2"),
        "127.0.0.1:50000".parse().unwrap(),
        to_member,
        Arc::default(),
    );
    assert!(conn > 0);
    let StreamMsg::Welcome(welcome) = answer else {
        panic!("refused: {answer:?}");
    };
    let (member, from_member) = join(*welcome, "FOH-2");
    (master, member, from_master, from_member)
}

/// Station `name` takes the master's Welcome over its link.
fn join(welcome: Welcome, name: &str) -> (ServerContext, Receiver<LinkOut>) {
    let mut member = station(name);
    let show = lumacore_objects::parse_show(welcome.show.as_deref().unwrap()).unwrap();
    let (mut handle, from_member) = LinkHandle::new("127.0.0.1:47658".into());
    handle.link = link_of(name);
    install(&mut member, welcome, Some(show), handle, true, &request());
    (member, from_member)
}

/// Apply everything the master queued for a member (ops, snapshots), and
/// return the replies it sent.
fn pump_all(member: &mut ServerContext, from_master: &Receiver<StreamMsg>) -> Vec<(u64, String)> {
    let mut replies = Vec::new();
    while let Ok(msg) = from_master.try_recv().map(StreamMsg::decoded) {
        match msg {
            StreamMsg::Op(op) => apply_op(member, *op),
            StreamMsg::Reply { id, message, .. } => replies.push((id, message)),
            _ => {}
        }
    }
    replies
}

/// Apply every op the master queued for the member.
fn pump(member: &mut ServerContext, from_master: &Receiver<StreamMsg>) -> usize {
    let mut n = 0;
    while let Ok(msg) = from_master.try_recv().map(StreamMsg::decoded) {
        if let StreamMsg::Op(op) = msg {
            apply_op(member, *op);
            n += 1;
        }
    }
    n
}

fn run(master: &mut ServerContext, line: &str) {
    let (reply, _) = execute_remote_command(master, "Admin", line);
    assert!(reply.ok, "{line}: {}", reply.message);
    flush(master);
}

#[test]
fn a_member_takes_the_masters_show_and_follows_its_results_in_order() {
    let (mut master, mut member, from_master, _) = session();
    assert_eq!(lumacore_objects::session_link::status_word(&member.state), "Member");
    assert_eq!(lumacore_objects::session_link::status_word(&master.state), "Master");
    assert!(!member.state.is_authority, "a member doesn't send DMX");

    run(&mut master, "Store Network \"Rig\"");
    assert_eq!(pump(&mut member, &from_master), 1);
    assert!(member.state.root.show_data.protocols.networks.values().any(|n| n.meta.identity.name.as_deref() == Some("Rig")));
    // The member mirrors the undo step the master filed for Admin.
    assert_eq!(member.state.undo_stack("Admin").map_or(0, |s| s.len()), 1);

    // Oops travels as a result too: the network goes, the step moves to redo.
    run(&mut master, "Oops");
    pump(&mut member, &from_master);
    assert!(member.state.root.show_data.protocols.networks.is_empty());
    let stack = member.state.undo_stack("Admin").unwrap();
    assert_eq!((stack.len(), stack.redo_frames.len()), (0, 1));
    let master_stack = master.state.undo_stack("Admin").unwrap();
    assert_eq!(master_stack.redo_frames.len(), 1);

    // Modes are session runtime: they replicate without a show change.
    run(&mut master, "On Blind");
    pump(&mut member, &from_master);
    assert!(member.state.root.runtime.user_ref("Admin").unwrap().modes.blind);

    // The member's journal shows who did what where.
    let journal = member.state.root.runtime.journal.entries();
    assert!(journal.into_iter().any(|e| e.command.contains("Store Network") && e.station == "FOH-1"));
}

#[test]
fn a_gap_asks_for_a_snapshot_and_ops_wait_for_it() {
    let (mut master, mut member, from_master, from_member) = session();
    run(&mut master, "Store Network \"A\"");
    run(&mut master, "Store Network \"B\"");
    let ops: Vec<ReplicationOp> = from_master
        .try_iter()
        .map(StreamMsg::decoded)
        .filter_map(|m| match m {
            StreamMsg::Op(op) => Some(*op),
            _ => None,
        })
        .collect();
    assert_eq!(ops.len(), 2);
    // The first one went missing: the second is a gap.
    apply_op(&mut member, ops[1].clone());
    assert!(member.state.root.show_data.protocols.networks.is_empty());
    assert!(matches!(from_member.try_recv().map(StreamMsg::from), Ok(StreamMsg::Resync { .. })));
    // Until the snapshot comes, ops wait (no second Resync either).
    apply_op(&mut member, ops[0].clone());
    assert!(member.state.root.show_data.protocols.networks.is_empty());
    assert!(from_member.try_recv().map(StreamMsg::from).is_err());

    // The master answers with what was missed; the member catches up.
    let resume = member.state.last_applied;
    let conn = master.lumasession.peers["FOH-2"].conn;
    resync(&mut master, "FOH-2", conn, resume);
    let StreamMsg::Snapshot(snapshot) = from_master.try_recv().unwrap() else {
        panic!("expected a snapshot");
    };
    assert!(snapshot.show.is_none(), "only the missed ops");
    let handle = member.state.link.clone().unwrap();
    install(&mut member, *snapshot, None, handle, false, &request());
    assert_eq!(member.state.root.show_data.protocols.networks.len(), 2);
}

#[test]
fn joining_needs_the_same_protocol_and_show_format() {
    let mut master = station("FOH-1");
    let addr = "127.0.0.1:50000".parse().unwrap();
    let refused = |answer: StreamMsg| match answer {
        StreamMsg::Refused { code, .. } => code,
        other => panic!("{other:?}"),
    };
    let (tx, _rx) = crossbeam_channel::unbounded();
    let old = Hello {
        protocol: SESSION_PROTOCOL + 1,
        ..hello("FOH-2")
    };
    assert_eq!(refused(admit(&mut master, old, addr, tx.clone(), Arc::default()).0), "E-LUMA-001");
    let newer = Hello {
        format_version: lumacore_tree::FORMAT_VERSION + 1,
        ..hello("FOH-2")
    };
    assert_eq!(refused(admit(&mut master, newer, addr, tx.clone(), Arc::default()).0), "E-LUMA-002");
    assert_eq!(refused(admit(&mut master, hello("FOH-1"), addr, tx.clone(), Arc::default()).0), "E-LUMA-003");
    assert!(master.state.root.runtime.luma_sessions.members.is_empty());
    // Another build is refused: a LumaSession runs one build (decisions §34 "34f7b hardening" 3).
    let other_commit = Hello {
        commit: "0000000-dirty".into(),
        ..hello("FOH-2")
    };
    assert_eq!(refused(admit(&mut master, other_commit, addr, tx.clone(), Arc::default()).0), "E-LUMA-006");
    let other_version = Hello {
        version: "0.0.0".into(),
        ..hello("FOH-2")
    };
    assert_eq!(refused(admit(&mut master, other_version, addr, tx.clone(), Arc::default()).0), "E-LUMA-006");
    assert!(matches!(admit(&mut master, hello("FOH-2"), addr, tx, Arc::default()).0, StreamMsg::Welcome(_)));
}

#[test]
fn a_member_forwards_show_changes_and_runs_station_lines_itself() {
    let (_master, mut member, _from_master, from_member) = session();
    let state = &mut member.state;
    let _ = state.begin_acting_as_existing_user("Admin");
    state.reply_to = ReplyTo::Shell("Admin".into());

    let out = lumacore_objects::commands::dispatch(state, "Store Network \"Rig\"");
    assert!(matches!(out, DispatchResult::Output(ref s) if s.is_empty()));
    assert!(state.root.show_data.protocols.networks.is_empty(), "nothing ran here");
    let Ok(StreamMsg::Command { id, user, line, .. }) = from_member.try_recv().map(StreamMsg::from) else {
        panic!("not forwarded");
    };
    assert_eq!((user.as_str(), line.as_str()), ("Admin", "Store Network \"Rig\""));
    assert_eq!(state.forwards.len(), 1);

    // Station lines run here; their Oops step (a Station step: the
    // station settings before and after) is filed by the master.
    lumacore_objects::commands::dispatch(state, "Set Station \"Location\" \"Stage left\"");
    assert_eq!(state.root.station_settings.location, "Stage left");
    let Ok(StreamMsg::Changes { snapshots, local, undo, .. }) = from_member.try_recv().map(StreamMsg::from) else {
        panic!("no Station step sent");
    };
    assert!(snapshots.is_empty(), "the show didn't change");
    assert_eq!(undo.map(|k| k.category), Some(lumacore_tree::OopsCategory::Settings));
    assert!(matches!(local.as_slice(), [l] if l.what == lumacore_tree::LocalSubtree::Station));

    // The reply ends the pending command.
    state.clear_acting_user();
    deliver(&mut member, id, true, "ok".into());
    assert!(member.state.forwards.is_empty());

    // A UDP / HTTP request gets the master's reply through a waiter.
    let ((), waits) = collecting(&mut member, |ctx| {
        let _ = ctx.state.begin_acting_as_existing_user("Admin");
        lumacore_objects::commands::dispatch(&mut ctx.state, "On Blind");
        ctx.state.clear_acting_user();
    });
    assert_eq!(waits.len(), 1);
    let Ok(StreamMsg::Command { id, .. }) = from_member.try_recv().map(StreamMsg::from) else {
        panic!("not forwarded");
    };
    deliver(&mut member, id, true, "Blind on".into());
    assert_eq!(waits[0].try_recv().unwrap(), "Blind on");
}

#[test]
fn a_members_own_show_changes_are_filed_by_the_master() {
    let (mut master, mut member, from_master, from_member) = session();
    let state = &mut member.state;
    let _ = state.begin_acting_as_existing_user("Admin");
    lumacore_objects::commands::record_change(state, "Screen 1", Undo::Untracked, |state| {
        state.root.show_data.show_settings.show_name = "From FOH-2".into();
    });
    state.clear_acting_user();
    let Ok(StreamMsg::Changes {
        user,
        line,
        undo,
        snapshots,
        local,
    }) = from_member.try_recv().map(StreamMsg::from)
    else {
        panic!("not submitted");
    };
    assert_eq!((user.as_str(), undo), ("Admin", None));

    apply_changes(&mut master, "FOH-2", &user, &line, undo, snapshots, local);
    assert_eq!(master.state.root.show_data.show_settings.show_name, "From FOH-2");
    // …and sends the result to every member, the sender included.
    assert_eq!(pump(&mut member, &from_master), 1);
    assert_eq!(member.state.last_applied.unwrap().1, master.lumasession.journal.tip_seq());
}

#[test]
fn a_command_resent_after_a_reconnect_runs_once() {
    let (mut master, _member, from_master, _) = session();
    let conn = master.lumasession.peers["FOH-2"].conn;
    let tip = master.lumasession.journal.tip_seq();
    run_forwarded(&mut master, "FOH-2", conn, 1, "Admin", "Store Network \"Rig\"", None);
    run_forwarded(&mut master, "FOH-2", conn, 1, "Admin", "Store Network \"Rig\"", None);
    let replies = from_master
        .try_iter()
        .map(StreamMsg::decoded)
        .filter(|m| matches!(m, StreamMsg::Reply { id: 1, .. }))
        .count();
    assert_eq!(replies, 2, "answered again");
    assert_eq!(master.lumasession.journal.tip_seq(), tip + 1, "ran once");

    // A dropped connection keeps the member (it comes back); Bye removes it.
    peer_gone(&mut master, "FOH-2", conn, false);
    let member = &master.state.root.runtime.luma_sessions.members["FOH-2"];
    assert!(!member.connected);
    assert_eq!(lumacore_objects::session_link::status_word(&master.state), "Master");
}

#[test]
fn lines_run_where_their_words_say() {
    let master = |l: &str| line_scope(l) == Scope::Master;
    assert!(master("Fixture 1 At 50"));
    assert!(master("Store Group 1"));
    assert!(master("Oops"));
    assert!(master("Clear"));
    assert!(master("On Blind"));
    assert!(master("List Fixture; Store Group 2"));
    assert!(!master("List Fixture"));
    assert!(!master("cd Root.ShowData"));
    assert!(!master("Set Station \"Location\" \"FOH\""));
    assert!(!master("Set Interface \"en1\" \"Shell\" No"));
    assert!(!master("Set Network \"Rig\" \"Interface\" \"en1\""));
    assert!(master("Set Network \"Rig\" \"Note\" \"main\""));
    assert!(!master("Join LumaSession \"Festival\""));
    assert!(!master("Leave LumaSession"));
    assert!(!master("Login \"LD\""));
}

#[test]
fn members_that_joined_earlier_learn_who_joined_after_them() {
    let (mut master, _member, from_master, _) = session();
    let (tx, _rx) = crossbeam_channel::unbounded();
    admit(&mut master, hello("FOH-3"), "127.0.0.1:50001".parse().unwrap(), tx, Arc::default());
    flush(&mut master);
    let runtime = from_master
        .try_iter()
        .map(StreamMsg::decoded)
        .filter_map(|m| match m {
            StreamMsg::Op(op) => op.runtime.clone(),
            _ => None,
        })
        .last()
        .and_then(|v| serde_json::from_value::<SessionRuntime>(v).ok())
        .expect("a runtime op for FOH-2");
    assert!(runtime.members.contains_key("FOH-2") && runtime.members.contains_key("FOH-3"));
}

// ── Merge on rejoin (§21 "6d") ───────────────────────────────────────────

fn networks(ctx: &ServerContext) -> Vec<String> {
    let mut names: Vec<String> = ctx
        .state
        .root
        .show_data
        .protocols
        .networks
        .values()
        .filter_map(|n| n.meta.identity.name.clone())
        .collect();
    names.sort();
    names
}

/// FOH-2 lost FOH-1 and worked alone; it rejoins with Merge.
fn split_and_diverge() -> (ServerContext, ServerContext) {
    let (mut master, mut member, from_master, _) = session();
    run(&mut master, "Store Network \"Rig\"");
    pump(&mut member, &from_master);
    lumacore_lumasession::election::become_idle(&mut member, "FOH-1");
    assert!(member.state.split_base.is_some(), "the split keeps a base");
    let (reply, _) = execute_remote_command(&mut member, "Admin", "Store Network \"Mine\"");
    assert!(reply.ok, "{}", reply.message);
    flush(&mut member);
    run(&mut master, "Store Network \"Theirs\"");
    // FOH-2 comes back: the master took it off its member list meanwhile.
    peer_gone(&mut master, "FOH-2", 1, true);
    (master, member)
}

fn merge_hello(member: &ServerContext) -> Hello {
    Hello {
        merge: Some(merge_offer(&member.state, member.lumasession.journal.id())),
        ..hello("FOH-2")
    }
}

#[test]
fn merge_brings_both_sides_changes_into_the_session_by_guid() {
    let (mut master, member) = split_and_diverge();
    let (to_member, _rx) = crossbeam_channel::unbounded();
    let (answer, _) = admit(
        &mut master,
        merge_hello(&member),
        "127.0.0.1:50000".parse().unwrap(),
        to_member,
        Arc::default(),
    );
    let StreamMsg::Welcome(welcome) = answer else {
        panic!("refused: {answer:?}");
    };
    assert_eq!(networks(&master), vec!["Mine", "Rig", "Theirs"]);
    assert!(welcome.merged.iter().any(|l| l.contains("added (mine)")), "{:?}", welcome.merged);
    // Both added Network 2: the session's keeps the slot, mine moves on.
    assert!(welcome.merged.iter().any(|l| l.contains("mine moved to a free slot")));
    assert!(master.state.root.show_data.identity().is_clean());
    // The joiner gets the merged show.
    let show = lumacore_objects::parse_show(welcome.show.as_deref().unwrap()).unwrap();
    assert_eq!(show.protocols.networks.len(), 3);
}

#[test]
fn merge_is_refused_when_both_sides_changed_the_patch() {
    let (mut master, mut member) = split_and_diverge();
    master.state.root.show_data.session.patch_version += 1;
    member.state.root.show_data.session.patch_version += 1;
    let (to_member, _rx) = crossbeam_channel::unbounded();
    let (answer, _) = admit(
        &mut master,
        merge_hello(&member),
        "127.0.0.1:50000".parse().unwrap(),
        to_member,
        Arc::default(),
    );
    let StreamMsg::Refused { code, message } = answer else {
        panic!("merged: {answer:?}");
    };
    assert_eq!(code, "E-LUMA-005");
    assert!(message.contains("patch changed on both sides"), "{message}");
    assert_eq!(networks(&master), vec!["Rig", "Theirs"], "the session's show is untouched");
}

#[test]
fn an_old_master_merges_from_the_base_its_successor_kept() {
    // The winner keeps the old master's show as its split base; the old
    // master (its journal) finds it there.
    let master_base = lumacore_tree::ShowData::new();
    let old_journal = uuid::Uuid::new_v4();
    let found = common_base(&[uuid::Uuid::new_v4()], Some((&master_base, Some(old_journal))), None, Some(old_journal));
    assert!(found.is_some());
    // Another lineage: no base, so no Merge.
    assert!(common_base(&[uuid::Uuid::new_v4()], Some((&master_base, Some(old_journal))), None, Some(uuid::Uuid::new_v4())).is_none());
}

#[test]
fn a_members_station_step_is_undone_there_by_an_oops_on_the_master() {
    let (mut master, mut member, from_master, from_member) = session();
    master.state.local_settings.station_id = "id-FOH-1".into();
    member.state.local_settings.station_id = "id-FOH-2".into();
    let state = &mut member.state;
    let _ = state.begin_acting_as_existing_user("Admin");
    lumacore_objects::commands::dispatch(state, "Set Station \"Location\" \"Stage left\"");
    state.clear_acting_user();
    let Ok(StreamMsg::Changes { user, line, undo, snapshots, local }) =
        from_member.try_recv().map(StreamMsg::from)
    else {
        panic!("no Station step sent");
    };
    apply_changes(&mut master, "FOH-2", &user, &line, undo, snapshots, local);
    pump(&mut member, &from_master);
    let frame = master.state.undo_stack("Admin").unwrap().frames.back().unwrap().clone();
    assert_eq!(frame.category, lumacore_tree::OopsCategory::Station);
    assert_eq!(member.state.undo_stack("Admin").unwrap().len(), master.state.undo_stack("Admin").unwrap().len());

    // Oops on the master: the master's own settings stay, FOH-2 puts its back.
    run(&mut master, "Oops /NoConfirmation");
    assert_eq!(master.state.root.station_settings.location, "");
    pump(&mut member, &from_master);
    assert_eq!(member.state.root.station_settings.location, "", "undone on FOH-2");
    run(&mut master, "Redo");
    pump(&mut member, &from_master);
    assert_eq!(member.state.root.station_settings.location, "Stage left");
}

#[test]
fn a_member_asks_its_master_once_for_files_it_lacks() {
    let (_master, mut member, _, from_member) = session();
    while from_member.try_recv().is_ok() {}
    let sha = "ab".repeat(32);
    member.state.root.show_data.images.insert(
        1,
        lumacore_tree::MediaFile {
            meta: lumacore_base::NodeMeta::new(Some("Stage".into())),
            sha256: sha.clone(),
            ext: "png".into(),
            size: 10,
            width: 1,
            height: 1,
            source: String::new(),
        },
    );
    want_files(&mut member.state);
    let asked: Vec<WantedFile> = from_member
        .try_iter()
        .filter_map(|m| match m {
            LinkOut::FileWant { files } => Some(files),
            _ => None,
        })
        .flatten()
        .collect();
    assert_eq!(asked, vec![WantedFile { sha256: sha, ext: "png".into() }]);
    want_files(&mut member.state);
    assert!(from_member.try_recv().is_err(), "asked once");
}

// ── R1 hardening: commands never run twice, members keyed by id ─────────

/// FOH-2 reconnects (a new connection, the same link).
fn readmit(master: &mut ServerContext, hello: Hello) -> (u64, Receiver<StreamMsg>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    let (answer, conn) = admit(master, hello, "127.0.0.1:50000".parse().unwrap(), tx, Arc::default());
    assert!(matches!(answer, StreamMsg::Welcome(_)), "{answer:?}");
    (conn, rx)
}

#[test]
fn a_command_resent_after_the_connection_dropped_runs_once() {
    let (mut master, _member, _from_master, _) = session();
    let conn = master.lumasession.peers["FOH-2"].conn;
    let tip = master.lumasession.journal.tip_seq();
    run_forwarded(&mut master, "FOH-2", conn, 1, "Admin", "Store Network \"Rig\"", None);
    assert!(master.lumasession.journal.tip_seq() > tip);
    // The reply was lost with the connection; FOH-2 comes back and resends.
    peer_gone(&mut master, "FOH-2", conn, false);
    let (conn, rx) = readmit(&mut master, hello("FOH-2"));
    let tip = master.lumasession.journal.tip_seq();
    run_forwarded(&mut master, "FOH-2", conn, 1, "Admin", "Store Network \"Rig\"", None);
    assert_eq!(master.lumasession.journal.tip_seq(), tip, "not run again");
    assert_eq!(networks(&master), vec!["Rig"]);
    assert!(rx.try_iter().any(|m| matches!(m, StreamMsg::Reply { id: 1, ok: true, .. })), "answered again");
}

#[test]
fn a_new_join_starts_its_command_ids_over() {
    let (mut master, _member, _from_master, _) = session();
    let conn = master.lumasession.peers["FOH-2"].conn;
    run_forwarded(&mut master, "FOH-2", conn, 1, "Admin", "Store Network \"Rig\"", None);
    // FOH-2 left and joined again: a new link, command 1 is a new command.
    peer_gone(&mut master, "FOH-2", conn, true);
    assert!(master.lumasession.replies.is_empty(), "a Bye drops the answers");
    let (conn, _rx) = readmit(&mut master, Hello { link: uuid::Uuid::new_v4(), ..hello("FOH-2") });
    run_forwarded(&mut master, "FOH-2", conn, 1, "Admin", "Store Network \"Two\"", None);
    assert_eq!(networks(&master), vec!["Rig", "Two"]);
    // A reconnect with yet another link (no Bye seen) runs its id 1 too.
    peer_gone(&mut master, "FOH-2", conn, false);
    let (conn, _rx) = readmit(&mut master, Hello { link: uuid::Uuid::new_v4(), ..hello("FOH-2") });
    run_forwarded(&mut master, "FOH-2", conn, 1, "Admin", "Store Network \"Three\"", None);
    assert_eq!(networks(&master), vec!["Rig", "Three", "Two"]);
}

#[test]
fn a_result_applied_with_its_answer_lost_never_runs_again_on_a_new_master() {
    let (mut master, mut member, from_master, from_member) = session();
    let _ = member.state.begin_acting_as_existing_user("Admin");
    lumacore_objects::commands::dispatch(&mut member.state, "Store Network \"Once\"");
    member.state.clear_acting_user();
    let Ok(StreamMsg::Command { id, user, line, .. }) = from_member.try_recv().map(StreamMsg::from) else {
        panic!("not forwarded");
    };
    let conn = master.lumasession.peers["FOH-2"].conn;
    run_forwarded(&mut master, "FOH-2", conn, id, &user, &line, None);
    // The result arrives, the answer is lost with the master.
    while let Ok(msg) = from_master.try_recv().map(StreamMsg::decoded) {
        if let StreamMsg::Op(op) = msg {
            apply_op(&mut member, *op);
        }
    }
    assert_eq!(networks(&member), vec!["Once"]);
    assert!(member.state.forwards[&id].ran, "the result says it ran");
    // FOH-2 wins the election: what ran isn't run again.
    lumacore_lumasession::election::become_master(&mut member, None, "test");
    assert_eq!(networks(&member), vec!["Once"], "ran once");
    assert!(member.state.forwards.is_empty());
}

#[test]
fn a_new_master_never_runs_a_command_the_old_one_ran() {
    // FOH-1 is master of FOH-2 and FOH-3.
    let (mut master, mut b, from_master_b, _) = session();
    let (tx, from_master_c) = crossbeam_channel::unbounded();
    let (answer, conn_c) = admit(&mut master, hello("FOH-3"), "127.0.0.1:50001".parse().unwrap(), tx, Arc::default());
    let StreamMsg::Welcome(welcome) = answer else {
        panic!("{answer:?}");
    };
    let (mut c, _from_c) = join(*welcome, "FOH-3");
    pump_all(&mut b, &from_master_b);
    // FOH-3's command runs on FOH-1; FOH-2 gets the result; FOH-3's answer
    // is lost when FOH-1 dies.
    run_forwarded(&mut master, "FOH-3", conn_c, 1, "Admin", "Store Network \"Once\"", None);
    pump_all(&mut b, &from_master_b);
    let _ = pump_all(&mut c, &from_master_c);
    assert_eq!(networks(&b), vec!["Once"]);
    // FOH-2 wins; FOH-3 follows it and resends command 1.
    lumacore_lumasession::election::become_master(&mut b, None, "test");
    let (to_c, from_b) = crossbeam_channel::unbounded();
    let (answer, conn) = admit(&mut b, hello("FOH-3"), "127.0.0.1:50001".parse().unwrap(), to_c, Arc::default());
    assert!(matches!(answer, StreamMsg::Welcome(_)), "{answer:?}");
    run_forwarded(&mut b, "FOH-3", conn, 1, "Admin", "Store Network \"Once\"", None);
    assert_eq!(networks(&b), vec!["Once"], "not run again");
    let replies: Vec<String> = from_b
        .try_iter()
        .filter_map(|m| match m {
            StreamMsg::Reply { id: 1, ok: true, message } => Some(message),
            _ => None,
        })
        .collect();
    assert!(replies.iter().any(|m| m.contains("Already ran on the master")), "{replies:?}");
    // FOH-3's next command runs.
    run_forwarded(&mut b, "FOH-3", conn, 2, "Admin", "Store Network \"Next\"", None);
    assert_eq!(networks(&b), vec!["Next", "Once"]);
}

#[test]
fn a_second_station_with_a_members_name_is_refused() {
    let (mut master, _member, _from_master, _) = session();
    let (tx, _rx) = crossbeam_channel::unbounded();
    let twin = Hello {
        station_id: "id-other".into(),
        ..hello("FOH-2")
    };
    let (answer, _) = admit(&mut master, twin.clone(), "127.0.0.1:50002".parse().unwrap(), tx.clone(), Arc::default());
    assert!(matches!(&answer, StreamMsg::Refused { code, .. } if code == "E-LUMA-003"), "{answer:?}");
    // Lost (reconnecting) counts too.
    let conn = master.lumasession.peers["FOH-2"].conn;
    peer_gone(&mut master, "FOH-2", conn, false);
    let (answer, _) = admit(&mut master, twin, "127.0.0.1:50002".parse().unwrap(), tx.clone(), Arc::default());
    assert!(matches!(&answer, StreamMsg::Refused { code, .. } if code == "E-LUMA-003"), "{answer:?}");
    assert_eq!(master.state.root.runtime.luma_sessions.members["FOH-2"].station_id, "id-FOH-2");
    // A station without an id can't join.
    let nameless = Hello {
        station_id: String::new(),
        ..hello("FOH-9")
    };
    let (answer, _) = admit(&mut master, nameless, "127.0.0.1:50003".parse().unwrap(), tx, Arc::default());
    assert!(matches!(&answer, StreamMsg::Refused { code, .. } if code == "E-LUMA-003"), "{answer:?}");
    // The same station (name and id) reconnects.
    readmit(&mut master, hello("FOH-2"));
}
