// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `dmx_slots`: free runs park nothing, an overlap parks both, a run past 512 parks
//! itself, a multipatch copy counts, a shared slot is used once, a gone universe takes no slot.
//!
//! Owns:          the unit tests of dmx_slots.rs.
//! Doesn't own:   the plan's skip (lumacore-render's tests).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 round 3, 18.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use lumacore_fixtures::{Fixture, FixtureBreak, MultipatchCopy, Universe};

fn patch(runs: &[(u32, u16, u16)]) -> (PatchData, Uuid) {
    let mut p = PatchData::default();
    let u = Universe::new(None);
    let ug = u.meta.identity.guid;
    p.universes.insert(1, u);
    for (no, start, count) in runs {
        let f = Fixture::new(None, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), ug, *start, *count);
        p.insert_fid(no, f);
    }
    (p, ug)
}

#[test]
fn runs_side_by_side_park_nothing() {
    let (p, ug) = patch(&[(101, 1, 14), (102, 15, 14)]);
    let m = SlotMap::of(&p);
    assert_eq!(m.runs.len(), 2);
    assert!(m.conflicts.is_empty() && m.parked.is_empty());
    assert_eq!(m.used[&ug], 28);
}

#[test]
fn an_overlap_parks_both_and_counts_a_shared_slot_once() {
    let (p, ug) = patch(&[(301, 1, 48), (302, 49, 48), (303, 81, 48), (304, 200, 1)]);
    let m = SlotMap::of(&p);
    assert_eq!(m.conflicts, vec![Conflict { run: 1, other: Some(2) }]);
    let parked: Vec<u32> = m.runs.iter().filter(|r| m.is_parked(r.guid)).map(|r| r.fixture).collect();
    assert_eq!(parked, vec![302, 303]);
    assert_eq!(m.used[&ug], 128 + 1);
}

#[test]
fn a_run_past_512_parks_itself() {
    let (p, _) = patch(&[(1, 500, 20)]);
    let m = SlotMap::of(&p);
    assert_eq!(m.conflicts, vec![Conflict { run: 0, other: None }]);
    assert_eq!(m.runs[0].end(), 519);
    assert_eq!(m.parked.len(), 1);
}

#[test]
fn a_multipatch_copy_takes_slots_and_parks_its_fixture() {
    let (mut p, ug) = patch(&[(201, 1, 10), (202, 100, 10)]);
    let f = p.fixture_mut(201).unwrap();
    f.luma.multipatch.push(MultipatchCopy { meta: lumacore_base::NodeMeta::new(None), breaks: vec![FixtureBreak::new(ug, 105, 10)], matrix: f.mvr.matrix });
    let m = SlotMap::of(&p);
    assert_eq!(m.runs[1].copy, Some(1));
    assert_eq!(m.conflicts, vec![Conflict { run: 1, other: Some(2) }]);
    assert!(m.is_parked(p.fixture(201).expect("the fixture").meta.identity.guid) && m.is_parked(p.fixture(202).expect("the fixture").meta.identity.guid));
}

#[test]
fn a_gone_universe_takes_no_slot() {
    let (mut p, _) = patch(&[(1, 1, 10)]);
    p.universes.clear();
    let m = SlotMap::of(&p);
    assert!(m.runs.is_empty() && m.parked.is_empty() && m.used.is_empty());
}
