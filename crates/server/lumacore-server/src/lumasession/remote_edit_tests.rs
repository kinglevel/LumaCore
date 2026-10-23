//! Remote station edits (decisions §30 "Remote station edits"): FOH edits
//! Tower's own settings on Tower. Two in-process stations; the command
//! port's request and reply are handed across directly (no sockets), and the
//! retry runs against a UDP socket of the test's own (an ephemeral loopback
//! port, nothing else listening).

use lumacore_lumasession::election::*;
use crate::context::ServerContext;
use std::time::Duration;

use lumacore_objects::session_link::{ReplyTo, SessionRequest};
use lumacore_objects::AppState;

use lumacore_objects::commands::dispatch;
use lumacore_objects::DispatchResult;
use lumacore_objects::text::strip_ansi;

fn station(name: &str) -> ServerContext {
    let mut state = AppState::new();
    state.root.station_settings.station_name = name.to_string();
    state.root.station_settings.station_priority = lumacore_tree::StationPriority::Normal;
    state.local_settings.station_id = format!("id-{name}");
    state.user = "Admin".into();
    ServerContext::new(state, None, false, 0, 0, 0)
}

/// `ctx` sees `name` at `ip` (its beacon: priority, location, invite).
fn sees(ctx: &mut ServerContext, name: &str, ip: &str, priority: &str, location: &str) {
    ctx.state.root.runtime.luma_sessions.discovery.insert(
        format!("{ip}:47653"),
        lumacore_tree::DiscoveryPeerRuntime {
            name: name.into(),
            ip: ip.into(),
            cmd_port: 47653,
            station_id: format!("id-{name}"),
            priority: priority.into(),
            location: location.into(),
            ..Default::default()
        },
    );
}

/// FOH 10.0.0.1 and Tower 10.0.0.7, each seeing the other.
fn pair() -> (ServerContext, ServerContext) {
    let mut foh = station("FOH");
    let mut tower = station("Tower");
    sees(&mut foh, "Tower", "10.0.0.7", "Normal", "Truss");
    sees(&mut tower, "FOH", "10.0.0.1", "Normal", "");
    (foh, tower)
}

fn run(ctx: &mut ServerContext, line: &str) -> String {
    match dispatch(&mut ctx.state, line) {
        DispatchResult::Output(s) => strip_ansi(&s),
        DispatchResult::Ask(a) => panic!("'{line}' asked: {}", a.title()),
        _ => panic!("'{line}': no output"),
    }
}

/// The edit `line` queued on FOH: (id, station, the line sent).
fn queued(foh: &mut ServerContext) -> Vec<(u64, String, String, lumacore_wire::RemoteRequest)> {
    std::mem::take(&mut foh.state.session_requests)
        .into_iter()
        .filter_map(|r| match r {
            SessionRequest::EditStation { id, station, request, .. } => {
                let lumacore_wire::RemoteRequest::Command { cmd, .. } = &request else { panic!("not a command") };
                Some((id, station, cmd.clone(), request))
            }
            _ => None,
        })
        .collect()
}

/// Run `line` on FOH, carry the request to Tower's command port handler
/// (from FOH's address), and FOH's answer back: the reply's text.
fn edit(foh: &mut ServerContext, tower: &mut ServerContext, line: &str) -> String {
    let out = run(foh, line);
    let mut sent = queued(foh);
    assert_eq!(sent.len(), 1, "'{line}' → {out}");
    let (id, name, _, request) = sent.remove(0);
    tower.request_from = Some("10.0.0.1:50123".parse().unwrap());
    let (reply, _) = crate::net::handler::handle_request(tower, request);
    tower.request_from = None;
    answer(foh, id, &name, Some(reply))
}

fn answer(foh: &mut ServerContext, id: u64, name: &str, reply: Option<lumacore_wire::RemoteReply>) -> String {
    let (tx, rx) = crossbeam_channel::bounded(1);
    edit_answered(foh, id, name, reply, "timed out", &ReplyTo::Waiter(tx));
    strip_ansi(&rx.try_recv().expect("the answer goes to the line's client"))
}

#[test]
fn the_answer_shows_old_to_new_the_put_back_line_and_the_next_election() {
    let (mut foh, mut tower) = pair();
    let out = run(&mut foh, "Set Station \"Tower\" \"Priority\" High");
    assert!(out.contains("sent, waiting for its answer"), "{out}");
    let sent = queued(&mut foh);
    // Absolute, and asks already happened here.
    assert_eq!(sent[0].2, "Set Station \"Priority\" High /NoConfirmation");
    // Pending until Tower reports High.
    let (pending, _) = lumacore_objects::stations::remote_marks(&foh.state, "Tower");
    assert_eq!(pending.get("priority").and_then(|v| v.as_str()), Some("High"));
    tower.request_from = Some("10.0.0.1:50123".parse().unwrap());
    let (reply, _) = crate::net::handler::handle_request(&mut tower, sent[0].3.clone());
    assert_eq!(tower.state.root.station_settings.station_priority, lumacore_tree::StationPriority::High);
    assert!(tower.state.remote_from.is_none(), "only while the line runs");
    let text = answer(&mut foh, sent[0].0, "Tower", Some(reply));
    assert!(text.contains("Station \"Tower\": Priority Normal → High"), "{text}");
    assert!(text.contains("next election"), "{text}");
    assert!(text.contains("Put back: Set Station \"Tower\" \"Priority\" Normal"), "{text}");
    // Answered, still pending: discovery says Normal.
    lumacore_objects::stations::tidy_remote_edits(&mut foh.state);
    assert!(lumacore_objects::stations::remote_marks(&foh.state, "Tower").0.contains_key("priority"));
    // The beacon reports High: settled.
    sees(&mut foh, "Tower", "10.0.0.7", "High", "Truss");
    lumacore_objects::stations::tidy_remote_edits(&mut foh.state);
    assert!(foh.state.remote_edits.is_empty());
    // FOH's Oops sends nothing to Tower (identity is outside the change path).
    let _ = dispatch(&mut foh.state, "Oops");
    assert!(queued(&mut foh).is_empty());
    // A setting Tower doesn't report settles on the answer.
    let text = edit(&mut foh, &mut tower, "Set Station \"Tower\" \"OutputWhenAlone\" On");
    assert!(text.contains("OutputWhenAlone Off → On"), "{text}");
    assert!(foh.state.remote_edits.is_empty());
    assert!(tower.state.root.station_settings.output_when_alone);
}

#[test]
fn rights_are_checked_only_on_the_target_at_each_settings_level() {
    let (mut foh, mut tower) = pair();
    for ctx in [&mut foh, &mut tower] {
        run(ctx, "Store User \"Sam\" /Rights Setup");
    }
    run(&mut foh, "Store User \"Gus\" /Rights Guest");
    // Declared: Name / LumaSession / Online / MulticastBase Admin, the rest Setup.
    let def = &lumacore_cmd_station::STATION_SETTINGS;
    for (name, level) in [("Name", "Admin"), ("LumaSession", "Admin"), ("Online", "Admin"), ("MulticastBase", "Admin"), ("Priority", "Setup"), ("Location", "Setup")] {
        assert_eq!(lumacore_objects::props::rights_of(&lumacore_cmd_station::STATION, def.get(name).unwrap()).label(), level, "{name}");
    }
    // Sam (Setup) on FOH: FOH sends it, Tower refuses a Name change.
    foh.state.user = "Sam".into();
    let text = edit(&mut foh, &mut tower, "Set Station \"Tower\" \"Name\" \"Truss\"");
    assert!(text.contains("refused") && text.contains("E-RIGHTS-001") && text.contains("Admin"), "{text}");
    assert_eq!(tower.state.root.station_settings.station_name, "Tower");
    let (_, refused) = lumacore_objects::stations::remote_marks(&foh.state, "Tower");
    assert!(refused.get("name").and_then(|v| v.as_str()).is_some_and(|w| w.contains("E-RIGHTS-001")), "{refused:?}");
    // … and takes a Location (Setup).
    let text = edit(&mut foh, &mut tower, "Set Station \"Tower\" \"Location\" \"Stage left\"");
    assert!(text.contains("Location \"\" → \"Stage left\"") || text.contains("→ \"Stage left\""), "{text}");
    // But Sam's own Name here is checked here.
    assert!(run(&mut foh, "Set Station \"FOH\" \"Name\" \"Booth\"").contains("E-RIGHTS-001"));
    assert!(run(&mut foh, "Set Station \"Name\" \"Booth\"").contains("E-RIGHTS-001"));
    // Gus (Guest) passes FOH's gate; Tower doesn't know him.
    foh.state.user = "Gus".into();
    let text = edit(&mut foh, &mut tower, "Set Station \"Tower\" \"Priority\" Low");
    assert!(text.contains("refused") && text.contains("unknown user"), "{text}");
    assert_eq!(tower.state.root.station_settings.station_priority, lumacore_tree::StationPriority::Normal);
}

#[test]
fn asks_happen_on_the_sender_and_the_line_goes_without_one() {
    let (mut foh, mut tower) = pair();
    let DispatchResult::Ask(ask) = dispatch(&mut foh.state, "Set Station \"Tower\" \"Online\" Off") else {
        panic!("Online Off on Tower asks here");
    };
    let line = ask.line_of("Go offline").unwrap();
    assert_eq!(line, "Set Station \"Tower\" \"Online\" Off /NoConfirmation");
    assert!(queued(&mut foh).is_empty(), "the ask sends nothing");
    let text = edit(&mut foh, &mut tower, &line);
    assert!(text.contains("Online On → Off"), "Tower ran it without asking: {text}");
    assert!(!tower.state.root.station_settings.online);
    // HoldOutput On asks; Off doesn't.
    assert!(matches!(dispatch(&mut foh.state, "Set Station \"Tower\" \"HoldOutput\" On"), DispatchResult::Ask(_)));
    run(&mut foh, "Set Station \"Tower\" \"HoldOutput\" Off");
    queued(&mut foh);
    // Priority Never on the master asks (FOH is the master: it has a member).
    foh.state.root.runtime.luma_sessions.members.insert(
        "Tower".into(),
        lumacore_tree::MemberRuntime { station: "Tower".into(), ip: "10.0.0.7".into(), connected: true, ..Default::default() },
    );
    let DispatchResult::Ask(ask) = dispatch(&mut foh.state, "Set Station \"Priority\" never") else {
        panic!("Priority Never on the master asks");
    };
    assert_eq!(ask.line_of("Set Never").unwrap(), "Set Station \"Priority\" Never /NoConfirmation");
    assert!(run(&mut foh, "Set Station \"Tower\" \"Priority\" Never").contains("sent"), "Tower isn't the master");
}

#[test]
fn values_go_absolute_and_unreachable_or_silent_stations_are_refused() {
    let (mut foh, _tower) = pair();
    // Toggle / relative need the station's value: refused unless reported.
    assert!(run(&mut foh, "Set Station \"Tower\" \"HoldOutput\" Toggle").contains("E-CMD-008"));
    assert!(run(&mut foh, "Set Station \"Tower\" \"RecoveryKeepDays\" + 5").contains("E-CMD-008"));
    assert!(run(&mut foh, "Set Station \"Tower\" \"Priority\" \"Loud\"").contains("E-SET-003"));
    assert!(queued(&mut foh).is_empty());
    // Invite is in its beacon (Off here): Toggle sends On.
    run(&mut foh, "Set Station \"Tower\" \"Invite\" Toggle");
    assert_eq!(queued(&mut foh)[0].2, "Set Station \"Invite\" On /NoConfirmation");
    // Offline: refused, named.
    run(&mut foh, "Store Station \"Ghost\"");
    let out = run(&mut foh, "Set Station \"Ghost\" \"Priority\" High");
    assert!(out.contains("Ghost") && out.contains("E-CMD-010"), "{out}");
    // No answer after the tries: E-CMD-010, the cell marked.
    run(&mut foh, "Set Station \"Tower\" \"Location\" \"Wing\"");
    let (id, ..) = queued(&mut foh).remove(0);
    let text = answer(&mut foh, id, "Tower", None);
    assert!(text.contains("didn't answer") && text.contains("E-CMD-010"), "{text}");
    assert!(lumacore_objects::stations::remote_marks(&foh.state, "Tower").1.contains_key("location"));
}

#[test]
fn the_send_retries_until_the_station_answers() {
    use std::net::UdpSocket;
    let ctx = station("Tower");
    let reply = crate::net::handler::make_reply(&ctx, true, "Priority Normal → High".into());
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = socket.local_addr().unwrap().port();
    let answering = std::thread::spawn(move || {
        let mut buf = [0u8; 65535];
        // The first request is lost; the second is answered.
        let _ = socket.recv_from(&mut buf).unwrap();
        let (_, from) = socket.recv_from(&mut buf).unwrap();
        socket.send_to(&serde_json::to_vec(&reply).unwrap(), from).unwrap();
    });
    let request = lumacore_wire::RemoteRequest::Command { user: "Admin".into(), cmd: "Set Station \"Priority\" High /NoConfirmation".into() };
    let r = send_with_retries("127.0.0.1", port, &request, Duration::from_millis(300), EDIT_TRIES).expect("the second try is answered");
    assert!(r.ok && r.message.contains("→ High"));
    answering.join().unwrap();
    // Nobody answers: an error after every try.
    let silent = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = silent.local_addr().unwrap().port();
    let started = std::time::Instant::now();
    assert!(send_with_retries("127.0.0.1", port, &request, Duration::from_millis(100), 3).is_err());
    assert!(started.elapsed() >= Duration::from_millis(300));
}

/// Merge of bugfixes (decisions §30 busy coverage): a busy station answers
/// E-CMD-017 within its wait (the edit never runs late); the sender says so.
#[test]
fn a_busy_station_answers_busy_and_the_sender_says_so() {
    let (mut foh, tower) = pair();
    run(&mut foh, "Set Station \"Tower\" \"Location\" \"Side Wing\"");
    let (id, _, cmd, _) = queued(&mut foh).remove(0);
    assert_eq!(cmd, "Set Station \"Location\" \"Side Wing\" /NoConfirmation", "the quoted value keeps its quotes");
    let busy = crate::net::handler::make_reply(
        &tower,
        false,
        crate::state::CallError::Busy { running: false }.to_string(),
    );
    let text = answer(&mut foh, id, "Tower", Some(busy));
    assert!(text.contains("E-CMD-017") && !text.contains("refused"), "{text}");
}
