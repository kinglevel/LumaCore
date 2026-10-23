// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The LumaSession words line in, reply out: what Join / Leave / Invite / Dismiss say when
//! something is missing, the master handing its role on, Info, no session.
//!
//! Owns:          the tests of the LumaSession words.
//! Doesn't own:   the stream and the election (lumacore-lumasession).
//! Entry points:  the #[test] fns.
//! Decisions:     §21, §30 "12d".

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::commands::dispatch;
use lumacore_objects::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// The line the ask's `button` runs (the ask itself changes nothing).
fn ask(state: &mut AppState, line: &str, button: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Ask(a) => a
            .line_of(button)
            .unwrap_or_else(|| panic!("'{line}': no button {button}")),
        _ => panic!("'{line}' didn't ask"),
    }
}

fn station() -> AppState {
    let mut s = AppState::new();
    s.local_settings.station_id = "0b7c8a2e-5d4f-4c3b-9a1e-2f3d4c5b6a7e".into();
    s.root.station_settings.station_name = "FOH".into();
    run(&mut s, "Store Network \"Main\"");
    s
}

#[test]
fn lumasession_join_invite_dismiss_explain_what_is_missing() {
    let mut state = AppState::new();
    let out = run(&mut state, "Join LumaSession \"Nowhere\"");
    assert!(out.contains("No LumaSession \"Nowhere\""), "{out}");
    assert!(run(&mut state, "Leave LumaSession").contains("isn't in a LumaSession"));
    assert!(run(&mut state, "Invite Station \"Ghost\"").contains("No station \"Ghost\""));
    assert!(run(&mut state, "Dismiss Station \"Ghost\"").contains("isn't a member"));
    run(&mut state, "Set Station \"Online\" Off /NoConfirmation");
    assert!(run(&mut state, "Join LumaSession 10.0.0.9").contains("LumaSession is off"));
    // The old words are gone (Shell, the terminal sessions, is cmd-station's).
    assert!(run(&mut state, "JoinSession 10.0.0.9").contains("E-CMD-001"));
}

#[test]
fn the_master_hands_its_role_to_a_member_after_an_ask() {
    let mut s = station();
    s.root.runtime.luma_sessions.members.insert(
        "Tower".into(),
        lumacore_tree::MemberRuntime {
            station: "Tower".into(),
            ip: "10.0.0.7".into(),
            connected: true,
            ..Default::default()
        },
    );
    assert!(run(&mut s, "Claim LumaSession Station \"Ghost\"").contains("E-CMD-007"));
    let line = ask(&mut s, "Claim LumaSession Station \"tower\"", "Hand over");
    assert_eq!(line, "Claim LumaSession Station \"Tower\" /NoConfirmation");
    assert!(s.session_requests.is_empty(), "the ask sends nothing");
    assert!(run(&mut s, &line).contains("Handing the master role to Tower"));
    assert!(
        matches!(&s.session_requests[..], [lumacore_objects::session_link::SessionRequest::HandTo(n)] if n == "Tower")
    );
    // Already the master: plain Claim says so.
    assert!(run(&mut s, "Claim LumaSession").contains("already the master"));
}

/// `Info LumaSession` names this station's session and its role.
#[test]
fn info_lumasession_names_the_session_and_the_role() {
    let mut state = AppState::new();
    run(&mut state, "Set Station \"LumaSession\" \"Festival\"");
    let info = run(&mut state, "Info LumaSession");
    assert!(
        info.contains("Festival") && info.contains("Standalone"),
        "{info}"
    );
}

/// Not in a LumaSession: Leave / Claim say so (no ask).
#[test]
fn leave_and_claim_say_when_there_is_no_session() {
    let mut s = station();
    assert!(run(&mut s, "Leave LumaSession").contains("isn't in a LumaSession"));
    assert!(run(&mut s, "Claim LumaSession Station \"Tower\"").contains("E-CMD-010"));
    assert!(run(&mut s, "Claim LumaSession Nope").contains("E-CMD-006"));
}
