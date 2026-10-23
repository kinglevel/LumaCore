// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `identify`: a fixture's dimmer sine, defaults and rainbow over a running cue, a raw
//! probe's sine, Park winning, a conflict-parked fixture flashing on its own slots, `/As`.
//!
//! Owns:          the unit tests of identify.rs.
//! Doesn't own:   who identifies what (the state's Identify object).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 round 3, "Identify, Probe and Find hardened".

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use crate::bench_show;
use crate::render::render_now;
use std::sync::Arc;

/// Two bench fixtures (28 slots each from 1.001) whose Attr3–5 are ColorAdd R / G / B.
/// `identify` builds the targets from the fixtures' GUIDs (each show has its own).
fn input(identify: impl FnOnce([Uuid; 2]) -> Vec<Identify>) -> (crate::merge::EngineInput, [Uuid; 2]) {
    let mut show = bench_show::show(2);
    let ft = show.devices.dmx.patch.fixture_type_mut(1).unwrap();
    for (cf, name) in ft.revisions[0].modes[0].channel_functions[3..6].iter_mut().zip(["ColorAdd_R", "ColorAdd_G", "ColorAdd_B"]) {
        cf.attribute_name = name.into();
    }
    let g = |n: u32| show.devices.dmx.patch.fixtures.values().nth(n as usize).unwrap().meta.identity.guid;
    let guids = [g(0), g(1)];
    let mut input = bench_show::input(2);
    input.show = Arc::new(show);
    input.identify = identify(guids);
    (input, guids)
}

#[test]
fn a_fixture_shows_its_defaults_a_dimmer_sine_and_a_rainbow() {
    let (mut on, guids) = input(|_| Vec::new());
    let before = *render_now(&on, 0).get(1).unwrap();
    on.identify = vec![Identify::Fixture(guids[0])];
    let at = |ms| *render_now(&on, ms).get(1).unwrap();
    // Dimmer: 15 % at the sine's foot, full at its top (1 s).
    assert_eq!((at(0)[0], at(1000)[0]), (38, 255));
    // Pan holds its GDTF default (0.5 of 16 bits), not the cue's value.
    assert_eq!((at(0)[1], at(0)[2]), (0x80, 0x00));
    // Hue: red at 0 s, green at 2 s, blue at 4 s (6 s round).
    assert_eq!((at(0)[5], at(0)[6], at(0)[7]), (255, 0, 0));
    assert_eq!((at(2000)[5], at(2000)[6], at(2000)[7]), (0, 255, 0));
    assert_eq!((at(4000)[5], at(4000)[6], at(4000)[7]), (0, 0, 255));
    // The other fixture keeps what it had.
    assert_eq!(at(0)[28..56], before[28..56]);
}

#[test]
fn a_raw_probe_sines_and_park_wins() {
    let (on, _) = input(|_| vec![Identify::Dmx { universe: 1, start: 500, count: 13, as_fixture: None }]);
    let at = |ms| *render_now(&on, ms).get(1).unwrap();
    assert_eq!((at(0)[499], at(1000)[499]), (38, 255));
    assert_eq!(at(1000)[511], 9, "1.512 is parked at 9: Park wins");
}

#[test]
fn a_probe_as_a_fixture_takes_its_shape() {
    let (on, _) = input(|g| vec![Identify::Dmx { universe: 1, start: 101, count: 28, as_fixture: Some(g[0]) }]);
    let f = *render_now(&on, 1000).get(1).unwrap();
    assert_eq!((f[100], f[101], f[105]), (255, 0x80, 255), "dimmer, pan default, red");
}

#[test]
fn a_conflict_parked_fixture_flashes_on_its_own_slots() {
    let (mut on, guids) = input(|_| Vec::new());
    // Fixture 2 onto fixture 1's slots: both parked, both silent.
    let mut show = (*on.show).clone();
    show.devices.dmx.patch.fixtures.values_mut().nth(1).unwrap().mvr.breaks[0].start_address = 1;
    on.show = Arc::new(show);
    on.patch_generation += 1;
    assert_eq!(render_now(&on, 1000).get(1).unwrap()[0], 0, "parked: sends nothing");
    on.identify = vec![Identify::Fixture(guids[1])];
    assert_eq!(render_now(&on, 1000).get(1).unwrap()[0], 255, "identified: flashes there");
}

