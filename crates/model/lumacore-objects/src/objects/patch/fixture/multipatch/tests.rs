// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `multipatch`: copies stored, addressed, listed, deleted, selected through their
//! fixture, and the mirrored output.
//!
//! Owns:          the unit tests of multipatch.rs.
//! Doesn't own:   the render plan's own tests (lumacore-render).
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

#[test]
fn a_copy_mirrors_its_fixture_at_its_own_address() {
    let mut s = rig(&[1, 2]);
    run(&mut s, "Claim Root.ShowData.Devices");
    let out = run(&mut s, "Store Fixture 1 Multipatch 1 Thru 2 At DMX 1.100");
    assert!(out.contains("Stored Fixture 1 Multipatch 1 Thru 2 at 1.100–1.101"), "{out}");
    let out = run(&mut s, "Store Fixture 1 Multipatch 5");
    assert!(out.contains("E-CMD-010") && out.contains("from 3"), "the next number only: {out}");
    let out = run(&mut s, "Assign Fixture 1 Multipatch 2 At DMX 1.200");
    assert!(!out.contains("E-"), "{out}");
    let out = run(&mut s, "Info Fixture 1 Multipatch");
    assert!(out.contains("Fixture 1 Multipatch 1 · 1.100") && out.contains("Fixture 1 Multipatch 2 · 1.200"), "{out}");
    run(&mut s, "Unclaim Root.ShowData.Devices /Keep");
    // A copy selects its fixture; the value goes to the fixture and every copy outputs it.
    run(&mut s, "Fixture 1 Multipatch 1 At 50");
    let frame = s.published_frame(1).unwrap();
    assert_eq!((frame[0], frame[99], frame[199], frame[1]), (frame[0], frame[0], frame[0], 0));
    assert!(frame[0] > 0, "Fixture 1 at 50");
    // A copy's slots are taken: a run skips them.
    run(&mut s, "Claim Root.ShowData.Devices");
    run(&mut s, "Assign Fixture 2 At DMX 1.100");
    let out = run(&mut s, "Delete Fixture 1 Multipatch 1 /NoConfirmation");
    assert!(out.contains("Deleted 1 multipatch"), "{out}");
    let out = run(&mut s, "Info Fixture 1 Multipatch");
    assert!(out.contains("Fixture 1 Multipatch 1 · 1.200"), "the second moved down: {out}");
}

/// A bare `Multipatch` in Store is each fixture's next copy (round 3, the left-out items); in
/// Delete or Assign it names no copy and asks which.
#[test]
fn a_bare_multipatch_is_each_fixtures_next_copy() {
    let mut s = rig(&[1, 2]);
    run(&mut s, "Claim Root.ShowData.Devices");
    run(&mut s, "Store Fixture 1 Multipatch 1 At DMX 1.100");
    let out = run(&mut s, "Store Fixture 1 Thru 2 Multipatch At DMX 1.200");
    assert!(!out.contains("E-"), "{out}");
    let copies = |s: &AppState, n: u32| s.root.show_data.devices.dmx.patch.fixture(n).expect("resolved").luma.multipatch.len();
    assert_eq!((copies(&s, 1), copies(&s, 2)), (2, 1), "1 gets its 2, 2 its 1");
    let out = run(&mut s, "Delete Fixture 1 Multipatch /NoConfirmation");
    assert!(out.contains("Which copy") && out.contains("E-CMD-006"), "{out}");
    let out = run(&mut s, "Assign Fixture 2 Multipatch At DMX 1.300");
    assert!(out.contains("Which copy"), "{out}");
}

/// The conflict list's fixes for a copy (decisions §34 round 3, 18): to the next free address,
/// or deleted.
#[test]
fn a_copy_moves_to_the_next_free_address_or_is_deleted() {
    let mut s = rig(&[1, 2]);
    run(&mut s, "Claim Root.ShowData.Devices");
    run(&mut s, "Store Fixture 1 Multipatch 1 At DMX 1.002");
    run(&mut s, "Unclaim Root.ShowData.Devices /Keep");
    let copy = |s: &AppState| s.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").luma.multipatch[0].breaks.first().map(|b| b.start_address);
    assert_eq!(copy(&s), Some(2), "on Fixture 2's slot");
    let out = run(&mut s, "Assign Fixture 1 Multipatch 1 At DMX");
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(copy(&s), Some(3));
    run(&mut s, "Claim Root.ShowData.Devices");
    let out = run(&mut s, "Delete Fixture 1 Multipatch 1 /NoConfirmation");
    assert!(out.contains("Deleted 1 multipatch"), "{out}");
}
