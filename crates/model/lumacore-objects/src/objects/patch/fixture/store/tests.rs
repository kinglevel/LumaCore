// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `store`: `Store Fixture` reading `/FixtureType` (slot, name, shared name), `/Mode`,
//! `/Revision`, the numbers typed, the claim.
//!
//! Owns:          the unit tests of store.rs.
//! Doesn't own:   where they land (address/tests.rs), importing a file (commands::import).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 round 3 1, 5, 19.

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
fn store_makes_fixtures_at_the_numbers_typed() {
    let mut s = rig(&[1]);
    let out = run(&mut s, "Store Fixture 201 Thru 203 /FixtureType 1");
    assert!(out.contains("E-CMD-010") && out.contains("Claim Root.ShowData.Devices"), "{out}");
    run(&mut s, "Claim Root.ShowData.Devices");
    let out = run(&mut s, "Store Fixture 201 Thru 203 + 210 /FixtureType 1");
    assert!(out.contains("Stored Fixture 201 Thru 203 + 210") && out.contains("no address"), "{out}");
    let patch = &s.root.show_data.devices.dmx.patch;
    assert!([201, 202, 203, 210].iter().all(|n| patch.fixture(n).expect("stored").mvr.breaks.is_empty()));
    let out = run(&mut s, "Store Fixture 203 Thru 205 /FixtureType 1");
    assert!(out.contains("E-CMD-010") && out.contains("Fixture 203 exists"), "{out}");
    let out = run(&mut s, "Store Fixture 300");
    assert!(out.contains("E-CMD-006") && out.contains("/FixtureType"), "{out}");
}

#[test]
fn the_type_by_slot_or_name_the_mode_and_the_revision() {
    let mut s = rig(&[1]);
    wide(&mut s);
    run(&mut s, "Claim Root.ShowData.Devices");
    let out = run(&mut s, "Store Fixture 5 /FixtureType \"dim\" /Mode \"Wide\"");
    assert!(out.contains("mode \"Wide\""), "names in any case: {out}");
    assert!(run(&mut s, "Store Fixture 6 /FixtureType 1 /Mode 2").contains("mode \"Wide\""));
    assert!(run(&mut s, "Store Fixture 7 /FixtureType 1 /Mode \"Nope\"").contains("E-CMD-008"));
    assert!(run(&mut s, "Store Fixture 7 /FixtureType 1 /Revision 2").contains("E-CMD-008"));
    assert!(run(&mut s, "Store Fixture 7 /FixtureType 9").contains("E-PATCH-005"));
    assert!(run(&mut s, "Store Fixture 7 /FixtureType \"Nope\"").contains("E-PATCH-005"));
}

#[test]
fn a_shared_type_name_needs_its_number() {
    let mut s = rig(&[1]);
    let mut twin = s.root.show_data.devices.dmx.patch.fixture_types[&1].as_ref().clone();
    twin.meta = lumacore_tree::node::NodeMeta::new(Some("Dim".into()));
    s.root.show_data.devices.dmx.patch.fixture_types.insert(2, std::sync::Arc::new(twin));
    s.root.show_data.after_load();
    run(&mut s, "Claim Root.ShowData.Devices");
    let out = run(&mut s, "Store Fixture 5 /FixtureType \"Dim\"");
    assert!(out.contains("E-PATCH-010") && out.contains("1 Thru 2"), "{out}");
    assert!(run(&mut s, "Store Fixture 5 /FixtureType 2").contains("Stored"));
}

/// `Import FixtureType "file.gdtf"` brings a type in (the claim); the same file again changes
/// nothing; the bare `Import "x"` went (decisions §34 round 3 2).
#[test]
fn import_fixture_type_names_its_kind() {
    // A file of this test's library/gdtf/ (the station reads its own library/).
    std::fs::create_dir_all("library/gdtf").unwrap();
    std::fs::copy(std::path::Path::new(env!("LUMACORE_ROOT")).join("library/gdtf/AuraXB.gdtf"), "library/gdtf/AuraXB.gdtf").unwrap();
    let mut s = AppState::new();
    let out = run(&mut s, "Import \"AuraXB.gdtf\"");
    assert!(out.contains("E-CMD-006") && out.contains("Import FixtureType \"AuraXB.gdtf\""), "{out}");
    run(&mut s, "Claim Root.ShowData.Devices");
    let out = run(&mut s, "Import FixtureType \"AuraXB.gdtf\"");
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(s.root.show_data.devices.dmx.patch.fixture_types.len(), 1);
    let out = run(&mut s, "Import FixtureType \"AuraXB.gdtf\"");
    assert!(out.contains("already has this file"), "{out}");
    assert!(run(&mut s, "Import FixtureType \"../x.gdtf\"").contains("E-"));
}
