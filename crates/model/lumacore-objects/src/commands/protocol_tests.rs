//! Steps 4b/4c behaviour (decisions §11 "Step 4b/4c hardening"): networks,
//! output and input lines, routes, station mapping, Live Patch, DMX input
//! and merge settings — through real dispatch.

use super::dispatch;
use crate::AppState;
use crate::DispatchResult;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn with_universes(n: u32) -> AppState {
    let mut state = AppState::new();
    let mut ev = Vec::new();
    for _ in 0..n {
        lumacore_tree::UniverseOps::create_universe(&mut state.root.show_data, None, &mut ev);
    }
    state
}

#[test]
fn protocol_set_fixture_address_repatches_live() {
    let mut state = with_universes(2);
    let ft = lumacore_fixtures::fixture_type::FixtureType {
        meta: lumacore_tree::node::NodeMeta::new(Some("T".into())),
        manufacturer: "M".into(),
        gdtf_fixture_type_id: None,
        category: Default::default(),
        revisions: vec![lumacore_fixtures::fixture_type::Revision {
            meta: lumacore_tree::node::NodeMeta::new(None),
            geometries: Vec::new(),
            wheels: Vec::new(),
            emitters: Vec::new(),
            filters: Vec::new(),
            attributes: Vec::new(),
            gdtf_version: "1.2".into(),
            file_hash: String::new(),
            modes: vec![lumacore_fixtures::fixture_type::Mode {
                meta: lumacore_tree::node::NodeMeta::new(None),
                channel_functions: vec![lumacore_fixtures::fixture_type::ChannelFunction {
                    attribute_name: "Dimmer".into(),
                    attribute_type: lumacore_fixtures::fixture_type::AttributeType::Dimmer,
                    byte_count: 1,
                    resolved: true,
                    ..Default::default()
                }],
                dmx_footprint: 10,
                geometry: String::new(),
                cells: Vec::new(),
                relations: Vec::new(),
            }],
        }],
    };
    let ftg = ft.meta.identity.guid;
    let show = &mut state.root.show_data;
    show.devices.dmx.patch.fixture_types.insert(1, std::sync::Arc::new(ft));
    lumacore_patch::PatchEngine::patch_fixture(show, ftg, 0, 0, 1, 1, None).unwrap();
    lumacore_patch::PatchEngine::patch_fixture(show, ftg, 0, 0, 1, 20, None).unwrap();
    lumacore_tree::attribute_resolver::sync(show);
    assert!(run(&mut state, "Set Fixture 1 \"Address\" 1.015").contains("E-PATCH"));
    assert!(run(&mut state, "Set Fixture 1 \"Address\" 2.001").contains("now at 2.001"));
    // Its old slots are free now; occupancy also survives a save and load.
    let json = serde_json::to_string(&state.root.show_data).unwrap();
    let mut loaded = AppState::new();
    crate::objects::replace_show(&mut loaded, serde_json::from_str(&json).unwrap());
    assert!(run(&mut loaded, "Set Fixture 2 \"Address\" 2.005").contains("E-PATCH"));
    assert!(run(&mut loaded, "Set Fixture 2 \"Address\" 1.001").contains("now at 1.001"));
}

/// R1-52: a loaded show with overlapping fixtures, a break at address 0 and
/// one past 512 loads without a panic; every problem is listed (List
/// Reference /Broken, Info Show) and nothing is dropped; moving one fixture
/// keeps the other's slots.
#[test]
fn a_broken_patch_loads_is_listed_and_kept() {
    let mut state = super::playback_tests::rig(&[1, 2]);
    let mut show = state.root.show_data.clone();
    show.devices.dmx.patch.fixture_mut(2).unwrap().mvr.breaks[0].start_address = 1;
    let json = serde_json::to_string(&show).unwrap();
    crate::objects::replace_show(&mut state, serde_json::from_str(&json).unwrap());
    let out = crate::references::list(&mut state, &["/Broken"]);
    assert!(out.contains("Patch: Fixture 2 overlaps Fixture 1 in Universe 1"), "{out}");
    assert!(crate::references::summary(&state).contains("1 broken"));
    assert_eq!(state.root.show_data.devices.dmx.patch.fixtures.len(), 2);
    // Moving Fixture 2 away leaves Fixture 1 on its slot (and vice versa).
    let f1 = state.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").meta.identity.guid;
    assert!(run(&mut state, "Set Fixture 2 \"Address\" 1.100").contains("now at 1.100"));
    assert_eq!(state.root.show_data.devices.dmx.patch.universes[&1].occupied[0], Some(f1));
    assert!(run(&mut state, "Set Fixture 2 \"Address\" 1.001").contains("E-PATCH"));
    assert!(crate::references::patch_problems(&state).is_empty());

    // Address 0, and 510 + 24 channels on a 1-channel mode.
    let mut show = state.root.show_data.clone();
    show.devices.dmx.patch.fixture_mut(1).unwrap().mvr.breaks[0].start_address = 0;
    let brk = &mut show.devices.dmx.patch.fixture_mut(2).unwrap().mvr.breaks[0];
    brk.start_address = 510;
    brk.channel_count = 24;
    let json = serde_json::to_string(&show).unwrap();
    crate::objects::replace_show(&mut state, serde_json::from_str(&json).unwrap());
    let out = crate::references::list(&mut state, &["/Broken"]);
    assert!(out.contains("Patch: Fixture 1: start address 0 in Universe 1"), "{out}");
    // Fixture 2's 24 channels follow its mode's 1 (the slot is free, Q20):
    // nothing runs past 512 any more.
    assert_eq!(state.root.show_data.devices.dmx.patch.fixture(2).expect("resolved").mvr.breaks[0].channel_count, 1);
    assert!(!out.contains("Fixture 2"), "{out}");
    assert_eq!(state.root.show_data.devices.dmx.patch.fixtures.len(), 2);
    // The plan compiles from it (no panic on the bad addresses).
    let _ = state.published_frame(1);
    assert!(run(&mut state, "Set Fixture 1 \"Address\" 1.200").contains("now at 1.200"));
}

/// R1 review (fecb969): two fixtures that grow with their mode never take
/// the same slots — the first one's new slots count as taken.
#[test]
fn growing_fixtures_never_share_slots() {
    let mut state = super::playback_tests::rig(&[1, 2]);
    let mut show = state.root.show_data.clone();
    let patch = &mut show.devices.dmx.patch;
    patch.fixture_type_mut(1).unwrap().revisions[0].modes[0].dmx_footprint = 2;
    let b1 = &mut patch.fixture_mut(1).unwrap().mvr.breaks[0];
    (b1.start_address, b1.channel_count) = (10, 1);
    let b2 = &mut patch.fixture_mut(2).unwrap().mvr.breaks[0];
    (b2.start_address, b2.channel_count) = (11, 0);
    let json = serde_json::to_string(&show).unwrap();
    crate::objects::replace_show(&mut state, serde_json::from_str(&json).unwrap());
    let counts: Vec<u16> = state.root.show_data.devices.dmx.patch.fixtures.values().map(|f| f.mvr.breaks[0].channel_count).collect();
    assert_eq!(counts, vec![2, 0], "Fixture 1 took 10–11; Fixture 2 can't take 11 too");
}
