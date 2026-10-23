// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `address`: `At DMX …` through Store, Assign and Unassign lines — exact or a run, taken
//! slots, `/Every`, `/KeepSpacing`, one universe or nothing, the bare kind on Unassign.
//!
//! Owns:          the unit tests of address.rs and the addressing lines.
//! Doesn't own:   Store's type / mode / revision (store/tests.rs), the DMX readout (cmd-protocols).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 round 3 19.

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

/// The rig's type 1 "Dim" gets a 16-channel mode "Wide" (mode 2) and a 0-channel "Virtual" (3).
fn wide(state: &mut AppState) {
    let ft = state.root.show_data.devices.dmx.patch.fixture_type_mut(1).unwrap();
    let mut wide = ft.revisions[0].modes[0].clone();
    wide.meta = lumacore_tree::node::NodeMeta::new(Some("Wide".into()));
    wide.dmx_footprint = 16;
    let mut virt = wide.clone();
    virt.meta = lumacore_tree::node::NodeMeta::new(Some("Virtual".into()));
    virt.channel_functions.clear();
    virt.dmx_footprint = 0;
    ft.revisions[0].modes.push(wide);
    ft.revisions[0].modes.push(virt);
}

/// Where fixture `n` starts: (universe, address).
fn at(state: &AppState, n: u32) -> Option<(u32, u16)> {
    let show = &state.root.show_data;
    let b = show.devices.dmx.patch.fixture(n)?.mvr.breaks.first()?;
    Some((show.universe_id_for_guid(b.universe_id)?, b.start_address))
}

#[test]
fn store_at_dmx_packs_a_run_and_skips_what_is_taken() {
    let mut s = rig(&[1]);
    wide(&mut s);
    run(&mut s, "Claim Root.ShowData.Devices");
    // Fixture 1 (the rig's) holds 1.001; a 16-slot run from 1.001 starts after it.
    let out = run(&mut s, "Store Fixture 201 Thru 203 At DMX 1.001 /FixtureType 1 /Mode \"Wide\"");
    assert!(out.contains("Stored Fixture 201 Thru 203") && out.contains("Skipped taken slots: 1.001–1.001 (Fixture 1)"), "{out}");
    assert_eq!((at(&s, 201), at(&s, 202), at(&s, 203)), (Some((1, 2)), Some((1, 18)), Some((1, 34))));
    // /Every n: start to start; under the footprint refused.
    let out = run(&mut s, "Store Fixture 301 Thru 302 At DMX 2 /FixtureType 1 /Mode 2 /Every 20");
    assert!(out.contains("Universe 2 created"), "{out}");
    assert_eq!((at(&s, 301), at(&s, 302)), (Some((2, 1)), Some((2, 21))));
    let out = run(&mut s, "Store Fixture 401 Thru 402 At DMX 2 /FixtureType 1 /Mode 2 /Every 10");
    assert!(out.contains("E-CMD-008") && out.contains("16"), "{out}");
    assert!(!s.root.show_data.devices.dmx.patch.has_fid(401), "nothing stored");
}

#[test]
fn an_exact_place_overlaps_and_says_whose() {
    let mut s = rig(&[1]);
    run(&mut s, "Claim Root.ShowData.Devices");
    let out = run(&mut s, "Store Fixture 5 At DMX 1.001 /FixtureType 1");
    assert!(out.contains("E-PATCH-001") && out.contains("1.001 (Fixture 1)"), "{out}");
    assert_eq!(at(&s, 5), Some((1, 1)), "one fixture at an exact place goes there");
}

#[test]
fn a_run_never_splits_and_a_refusal_leaves_nothing() {
    let mut s = rig(&[1]);
    wide(&mut s);
    run(&mut s, "Claim Root.ShowData.Devices");
    let before = serde_json::to_string(&s.root.show_data.devices).unwrap();
    let out = run(&mut s, "Store Fixture 201 Thru 240 At DMX 9.001 /FixtureType 1 /Mode 2");
    assert!(out.contains("E-CMD-010"), "40 × 16 slots don't fit one universe: {out}");
    assert_eq!(serde_json::to_string(&s.root.show_data.devices).unwrap(), before, "no fixture, no universe 9");
    // A bare DMX: the first universe with room — a new one when none has.
    let out = run(&mut s, "Store Fixture 201 Thru 231 At DMX /FixtureType 1 /Mode 2");
    assert!(out.contains("Stored"), "{out}");
    assert_eq!(at(&s, 201), Some((1, 2)), "31 × 16 fit in universe 1 after Fixture 1");
    let out = run(&mut s, "Store Fixture 301 At DMX /FixtureType 1 /Mode 2");
    assert!(out.contains("Universe 2 created"), "{out}");
}

#[test]
fn assign_at_dmx_moves_them_and_keeps_spacing_on_request() {
    let mut s = rig(&[1]);
    wide(&mut s);
    run(&mut s, "Claim Root.ShowData.Devices");
    run(&mut s, "Store Fixture 201 Thru 203 At DMX 2.001 /FixtureType 1 /Mode 2 /Every 20");
    // Packed by default, wherever they were.
    run(&mut s, "Assign Fixture 201 Thru 203 At DMX 3.001");
    assert_eq!((at(&s, 201), at(&s, 202), at(&s, 203)), (Some((3, 1)), Some((3, 17)), Some((3, 33))));
    // Their own slots are free to them; /KeepSpacing keeps the gaps.
    run(&mut s, "Assign Fixture 202 At DMX 3.101");
    run(&mut s, "Assign Fixture 201 Thru 202 At DMX 4.001 /KeepSpacing");
    assert_eq!((at(&s, 201), at(&s, 202)), (Some((4, 1)), Some((4, 101))));
    let out = run(&mut s, "Assign Fixture 201 At DMX 4.001 /KeepSpacing /Every 20");
    assert!(out.contains("E-CMD-006"), "{out}");
    let out = run(&mut s, "Assign Fixture 201 At MvrGroup 1 /Every 20");
    assert!(out.contains("E-CMD-006"), "{out}");
}

#[test]
fn unassign_names_what_it_takes_away() {
    let mut s = rig(&[1, 2, 3]);
    run(&mut s, "Claim Root.ShowData.Devices");
    let out = run(&mut s, "Unassign Fixture 1");
    assert!(out.contains("E-CMD-006") && out.contains("At DMX"), "{out}");
    run(&mut s, "Assign Fixture 3 At DMX 2.001");
    let out = run(&mut s, "Unassign Fixture 1 Thru 3 At DMX 2");
    assert!(out.contains("Fixture 3 lost its address"), "only the ones in universe 2: {out}");
    assert_eq!((at(&s, 1), at(&s, 3)), (Some((1, 1)), None));
    let out = run(&mut s, "Unassign Fixture 2 At DMX 1.005");
    assert!(out.contains("E-CMD-009"), "Fixture 2 isn't at 1.005: {out}");
    run(&mut s, "Unassign Fixture 1 Thru 2 At DMX");
    assert_eq!((at(&s, 1), at(&s, 2)), (None, None));
    assert_eq!(s.root.show_data.devices.dmx.patch.fixtures.len(), 3, "the fixtures stay");
}

#[test]
fn a_virtual_mode_is_placed_and_occupies_nothing() {
    let mut s = rig(&[1]);
    wide(&mut s);
    run(&mut s, "Claim Root.ShowData.Devices");
    let out = run(&mut s, "Store Fixture 9 At DMX 1.001 /FixtureType 1 /Mode \"Virtual\"");
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(at(&s, 9), Some((1, 1)));
}

/// `Assign Fixture A At Fixture B DMX /Swap`: pairs trade addresses, up to the shorter list;
/// without /Swap each goes exactly to its partner's address (an overlap, named).
#[test]
fn swap_trades_addresses_and_fixture_dmx_names_them() {
    let mut s = rig(&[1, 2, 3, 4, 5]);
    run(&mut s, "Claim Root.ShowData.Devices");
    run(&mut s, "Assign Fixture 3 Thru 5 At DMX 2.001");
    let out = run(&mut s, "Assign Fixture 1 Thru 3 At Fixture 4 Thru 5 DMX /Swap");
    assert!(out.contains("1 ↔ 4") && out.contains("2 ↔ 5"), "{out}");
    assert_eq!((at(&s, 1), at(&s, 4), at(&s, 3)), (Some((2, 2)), Some((1, 1)), Some((2, 1))), "3 stays");
    let out = run(&mut s, "Info Fixture 1 Thru 2 DMX");
    assert!(out.contains("Fixture 1: DMX 1 2.002") && out.contains("Fixture 2: DMX 1 2.003"), "{out}");
    let out = run(&mut s, "Assign Fixture 3 At Fixture 4 DMX");
    assert!(out.contains("E-PATCH-001") && out.contains("Fixture 4"), "{out}");
    assert_eq!(at(&s, 3), Some((1, 1)));
    assert!(run(&mut s, "Assign Fixture 3 At DMX 1.001 /Swap").contains("E-CMD-006"));
}

/// `Unassign Fixture … At Fixture … DMX n` takes one break away; without n every break.
#[test]
fn one_break_goes_by_its_number() {
    let mut s = rig(&[1]);
    run(&mut s, "Claim Root.ShowData.Devices");
    let ug = s.root.show_data.universe_guid(1).unwrap();
    s.root.show_data.devices.dmx.patch.fixture_mut(1).unwrap().mvr.breaks.push(lumacore_fixtures::FixtureBreak::new(ug, 100, 1));
    let out = run(&mut s, "Unassign Fixture 1 At Fixture 1 DMX 2");
    assert!(out.contains("Fixture 1 lost its address"), "{out}");
    let breaks = &s.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").mvr.breaks;
    assert_eq!(breaks.iter().map(|b| b.start_address).collect::<Vec<_>>(), vec![1], "break 1 stays");
}

/// An overlap parks both fixtures (decisions §34 round 3, 18): they keep their address and values
/// but send nothing until it is resolved; nothing asks at patch time.
#[test]
fn an_overlap_parks_both_until_resolved() {
    let mut s = rig(&[1, 2]);
    run(&mut s, "Fixture 1 Thru 2 At 100");
    let out = run(&mut s, "Assign Fixture 2 At DMX 1.001");
    assert!(out.contains("1.001 (Fixture 1)"), "it says whose: {out}");
    assert_eq!(at(&s, 2), Some((1, 1)));
    let frame = s.published_frame(1).unwrap();
    assert_eq!((frame[0], frame[1]), (0, 0), "both parked");
    run(&mut s, "Assign Fixture 2 At DMX 1.010");
    let frame = s.published_frame(1).unwrap();
    assert!(frame[0] > 0 && frame[9] > 0, "resolved, both send: {:?}", &frame[..10]);
}
