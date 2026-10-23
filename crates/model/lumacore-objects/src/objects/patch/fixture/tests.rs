// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `fixture`: the MVR ids and the place, through `Set` lines.
//!
//! Owns:          the unit tests of fixture.rs.
//! Doesn't own:   the matrix maths' own tests (lumacore-fixtures).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 "34g4 patch hardened" 3, 4, 5.

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
        _ => panic!("'{line}' did not produce output"),
    }
}

/// What the property reads for Fixture 1 (its declared getter).
fn get(state: &AppState, prop: &str) -> String {
    let p = super::PLACE_PROPS.iter().find(|p| p.name == prop).unwrap();
    let crate::props::Access::Keyed { get, .. } = p.access else { unreachable!() };
    get(state, "1")
}

#[test]
fn place_reads_and_writes_the_one_world_matrix() {
    let mut s = rig(&[1, 2]);
    run(&mut s, "Set Fixture 1 \"PosZ\" 6.5");
    run(&mut s, "Set Fixture 1 \"PosX\" - 1.25");
    run(&mut s, "Set Fixture 1 \"RotY\" -90");
    run(&mut s, "Set Fixture 1 \"RotZ\" + 45");
    let m = s.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").mvr.matrix;
    assert_eq!(m.o, [-1250.0, 0.0, 6500.0], "millimetres in the matrix");
    assert_eq!(m.rotation_deg().map(|d| d.round()), [0.0, -90.0, 45.0]);
    assert_eq!(get(&s, "PosZ"), "6.5");
    assert_eq!(get(&s, "PosX"), "-1.25");
    assert_eq!(get(&s, "RotY"), "-90");
    let out = run(&mut s, "Set Fixture 1 \"PosX\" \"far\"");
    assert!(out.contains("E-CMD-008"), "{out}");
    // Fixture 2 never moved.
    assert_eq!(s.root.show_data.devices.dmx.patch.fixture(2).expect("resolved").mvr.matrix, lumacore_fixtures::Matrix::default());
}

#[test]
fn the_second_id_is_mvrs_type_and_number() {
    let mut s = rig(&[1]);
    // Not live columns: they change in the Devices draft.
    run(&mut s, "Claim Root.ShowData.Devices");
    run(&mut s, "Set Fixture 1 \"IDType\" Houselights");
    run(&mut s, "Set Fixture 1 \"CID\" 4");
    run(&mut s, "Set Fixture 1 \"FixtureID\" \"FOH-12\"");
    let f = &s.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").mvr;
    assert_eq!(f.id_type, lumacore_fixtures::IdType::Houselights);
    assert_eq!(f.cid, Some(4));
    assert_eq!(f.fixture_id, "FOH-12");
    let out = run(&mut s, "Set Fixture 1 \"IDType\" Lamp");
    assert!(out.contains("E-"), "a type outside MVR's list is refused: {out}");
}

/// `Fixture Houselights 4` selects by second id — MVR's IDType + CID, one per fixture (decisions
/// §40 7: the triple; `MoreIds` is gone).
#[test]
fn a_second_id_selects_the_fixture() {
    let mut s = rig(&[1, 2, 3, 4]);
    run(&mut s, "Claim Root.ShowData.Devices");
    for (n, cid) in [(1, 4), (2, 5), (3, 6)] {
        run(&mut s, &format!("Set Fixture {n} \"IDType\" Houselights"));
        run(&mut s, &format!("Set Fixture {n} \"CID\" {cid}"));
    }
    assert!(run(&mut s, "Set Fixture 4 \"MoreIds\" \"Channel 12\"").contains("E-"), "MoreIds is gone (decisions §40 7)");
    run(&mut s, "Unclaim Root.ShowData.Devices /Keep");
    let selected = |s: &AppState| s.current_programmer().map(|p| p.selected_fixtures.len()).unwrap_or(0);
    let out = run(&mut s, "Fixture Houselights 5");
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(selected(&s), 1);
    run(&mut s, "ResetAll");
    run(&mut s, "Fixture houselights 4 Thru 9 - Fixture Houselights 5");
    assert_eq!(selected(&s), 2, "4 and 6");
    run(&mut s, "ResetAll");
    run(&mut s, "Fixture Houselights 6 + Fixture 1");
    assert_eq!(selected(&s), 2);
}

/// A CID without a type is Generic (decisions §40 10); a CID is one fixture's within its type and
/// a number is one fixture's in the show (§40 7); `Set "FID"` renumbers, the slot stays.
#[test]
fn ids_are_unique_and_a_cid_defaults_to_generic() {
    let mut s = rig(&[1, 2, 3]);
    run(&mut s, "Claim Root.ShowData.Devices");
    run(&mut s, "Set Fixture 1 \"CID\" 12");
    let f = s.root.show_data.devices.dmx.patch.fixture(1).expect("resolved");
    assert_eq!((f.mvr.id_type, f.mvr.cid), (lumacore_fixtures::IdType::Generic, Some(12)));
    let out = run(&mut s, "Set Fixture 2 \"CID\" 12");
    assert!(out.contains("E-CMD-010"), "Generic 12 is Fixture 1's: {out}");
    run(&mut s, "Set Fixture 2 \"IDType\" Media");
    assert!(!run(&mut s, "Set Fixture 2 \"CID\" 12").contains("E-"), "Media 12 is another type's namespace");
    let out = run(&mut s, "Set Fixture 3 \"FID\" 2");
    assert!(out.contains("E-CMD-010"), "Fixture 2 exists: {out}");
    let slot = s.root.show_data.devices.dmx.patch.fixture_by_fid(3).expect("resolved").0;
    assert!(!run(&mut s, "Set Fixture 3 \"FID\" 300").contains("E-"));
    let patch = &s.root.show_data.devices.dmx.patch;
    assert!(patch.fixture(3).is_none() && patch.fixture(300).is_some(), "renumbered");
    assert_eq!(patch.fixture_by_fid(300).expect("resolved").0, slot, "the slot stayed (decisions §40 5)");
    run(&mut s, "Unclaim Root.ShowData.Devices /Keep");
    let selected = |s: &AppState| s.current_programmer().map(|p| p.selected_fixtures.len()).unwrap_or(0);
    run(&mut s, "Fixture Generic 12");
    assert_eq!(selected(&s), 1);
    run(&mut s, "ResetAll");
    run(&mut s, "Fixture Media 12 + Fixture 300");
    assert_eq!(selected(&s), 2);
}

/// A fixture without a number (a folder, decisions §40 3, 7): in the pool by its slot, named by
/// its second id, never a `Fixture n`.
#[test]
fn a_fixture_without_a_number_lives_by_its_second_id() {
    let mut s = rig(&[1, 2]);
    let patch = &mut s.root.show_data.devices.dmx.patch;
    let mut folder = patch.fixture(1).expect("resolved").clone();
    folder.meta = lumacore_tree::NodeMeta::new(Some("Stage 1".into()));
    folder.mvr.fid = None;
    folder.mvr.id_type = lumacore_fixtures::IdType::Media;
    folder.mvr.cid = Some(7003);
    folder.mvr.breaks.clear();
    folder.mvr.order = 0; // a fresh one takes the next order; a copy keeps its own
    let slot = patch.insert_fixture(folder);
    assert_eq!(patch.fixtures.len(), 3);
    assert!(patch.fids().iter().all(|(n, s)| *n != 0 && *s != slot), "no number lists it");
    assert_eq!(patch.fixture_by_id(lumacore_fixtures::IdType::Media, 7003).map(|(s, _)| s), Some(slot));
    assert_eq!(patch.fixtures[&slot].handle(), "Fixture Media 7003");
    assert_eq!(patch.next_order(None), 40, "three fixtures under the patch, ten apart");
    let selected = |s: &AppState| s.current_programmer().map(|p| p.selected_fixtures.len()).unwrap_or(0);
    run(&mut s, "Fixture Media 7003");
    assert_eq!(selected(&s), 1);
    run(&mut s, "ResetAll");
    run(&mut s, "Fixture 1 Thru");
    assert_eq!(selected(&s), 2, "a range over numbers never reaches a folder");
    let out = run(&mut s, "List Fixture");
    assert!(out.contains("Stage 1") || !out.contains("E-"), "{out}");
}
