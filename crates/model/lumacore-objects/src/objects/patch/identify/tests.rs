// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `identify`: a fixture flashes above the programmer and stops with Off, a bare
//! Identify takes the selection, a probe and /As, a part ends, no Oops step, Setup and up, the
//! station's ends (logout, 5 minutes, Unclaim) with their System Monitor line.
//!
//! Owns:          the unit tests of identify.rs.
//! Doesn't own:   the wave shapes (lumacore-render's identify tests).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 round 3, "Identify, Probe and Find hardened".

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use crate::commands::dispatch;
use crate::commands::playback_tests::rig;
use crate::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        DispatchResult::Ask(a) => format!("ASK {}", a.view().text),
        _ => String::new(),
    }
}

fn undo_len(state: &AppState) -> usize {
    state.root.runtime.user_ref("Admin").map_or(0, |u| u.undo.len())
}

#[test]
fn a_fixture_flashes_above_the_programmer_and_off_ends_it() {
    let mut s = rig(&[1, 2]);
    run(&mut s, "Fixture 1 Thru 2 At 0");
    let steps = undo_len(&s);
    let out = run(&mut s, "Identify Fixture 1");
    assert!(out.contains("Identify Fixture 1: flashing"), "{out}");
    let f = s.published_frame(1).unwrap();
    assert!(f[0] >= 38 && f[1] == 0, "1 flashes (15–100 %), 2 stays at 0: {:?}", &f[..2]);
    assert_eq!(undo_len(&s), steps, "never an Oops step");
    run(&mut s, "Off Identify");
    assert_eq!(s.published_frame(1).unwrap()[0], 0);
    assert!(s.identify.is_empty());
}

#[test]
fn a_bare_identify_takes_the_selection_and_a_new_line_replaces_it() {
    let mut s = rig(&[1, 2]);
    let out = run(&mut s, "Identify");
    assert!(out.contains("E-CMD-009"), "nothing selected: {out}");
    run(&mut s, "Fixture 2");
    assert!(run(&mut s, "Identify").contains("Identify Fixture 2"));
    run(&mut s, "Identify DMX 1.100 Thru 1.103 /As Fixture 1");
    let mine = &s.identify["Admin"];
    assert_eq!(mine.what, "DMX 1.100–1.103", "replaced, one per user");
    assert!(s.published_frame(1).unwrap()[99] >= 38, "1.100 in fixture 1's shape (its dimmer)");
}

#[test]
fn off_ends_a_part_and_dmx_thru_every_run() {
    let mut s = rig(&[1, 2]);
    run(&mut s, "Identify Fixture 1 Thru 2 + DMX 1.150");
    assert_eq!(s.identify["Admin"].targets.len(), 3);
    assert!(run(&mut s, "Off Identify Fixture 1").contains("the rest goes on"));
    assert_eq!(s.identify["Admin"].targets.len(), 2);
    run(&mut s, "Off Identify Fixture Thru; Off Identify DMX Thru");
    assert!(s.identify.is_empty(), "the leave's lines end everything of yours");
    assert!(run(&mut s, "Off Identify").contains("Nothing of yours"));
}

#[test]
fn identify_needs_setup() {
    let mut s = rig(&[1]);
    run(&mut s, "Store User \"Ann\"");
    run(&mut s, "Set User \"Ann\" \"Rights\" Program");
    s.acting_user = Some("Ann".into());
    s.root.runtime.user("Ann");
    let out = run(&mut s, "Identify Fixture 1");
    assert!(out.contains("E-RIGHTS-001") && out.contains("Setup"), "{out}");
}

#[test]
fn the_station_ends_what_is_left_on_and_logs_it() {
    let mut s = rig(&[1, 2]);
    let here: std::collections::HashSet<String> = ["Admin".to_string()].into();
    // A lost holder.
    run(&mut s, "Identify Fixture 1");
    super::tick(&mut s, &Default::default());
    assert!(s.identify.is_empty());
    assert!(s.sysmon_pending.iter().any(|(_, m)| m == "Identify left on (Admin: Fixture 1) — ended (logged out)"), "{:?}", s.sysmon_pending);
    // 5 minutes.
    run(&mut s, "Identify Fixture 2");
    super::tick(&mut s, &here);
    assert!(!s.identify.is_empty(), "still here, still running");
    s.identify.get_mut("Admin").unwrap().started_ms = 0;
    super::tick(&mut s, &here);
    assert!(s.identify.is_empty() && s.sysmon_pending.iter().any(|(_, m)| m.ends_with("ended (5 minutes)")));
    // Leaving the patch.
    run(&mut s, "Claim Root.ShowData.Devices");
    run(&mut s, "Identify DMX 1.150");
    run(&mut s, "Unclaim Root.ShowData.Devices /Discard");
    assert!(s.identify.is_empty() && s.sysmon_pending.iter().any(|(_, m)| m == "Identify left on (Admin: DMX 1.150) — ended (Unclaim)"));
}

