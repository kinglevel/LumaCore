//! Step 6d acceptance tests, after a console's data-negotiation system test:
//! two stations diverge from one show (commands run on each side), then the
//! 3-way merge by GUID (decisions §21 "6d hardening").

use super::dispatch;
use crate::AppState;
use crate::DispatchResult;
use lumacore_fixtures::fixture_type::{AttributeType, ChannelFunction, FixtureType, Mode, Revision};
use lumacore_tree::merge::{MergeReport, three_way};
use lumacore_tree::node::NodeMeta;
use lumacore_tree::{ShowData, cue_key};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// Ten one-channel dimmers on universe 1.
fn rig() -> AppState {
    let mut state = AppState::new();
    let patch = &mut state.root.show_data.devices.dmx.patch;
    let universe = lumacore_fixtures::Universe::new(None);
    let ug = universe.meta.identity.guid;
    patch.universes.insert(1, universe);
    let ft = FixtureType {
        meta: NodeMeta::new(Some("Dim".into())),
        manufacturer: "Test".into(),
        gdtf_fixture_type_id: None,
        category: Default::default(),
        revisions: vec![Revision {
            meta: NodeMeta::new(None),
            geometries: Vec::new(),
            wheels: Vec::new(),
            emitters: Vec::new(),
            filters: Vec::new(),
            attributes: Vec::new(),
            gdtf_version: "1.2".into(),
            file_hash: String::new(),
            modes: vec![Mode {
                meta: NodeMeta::new(None),
                channel_functions: vec![ChannelFunction {
                    attribute_name: "Dimmer".into(),
                    attribute_type: AttributeType::Dimmer,
                    dmx_offset: 0,
                    byte_count: 1,
                    default_value: 0,
                    default_fraction: None,
                    physical_range: None,
                    is_virtual: false,
                    resolved: true,
                    ..Default::default()
                }],
                dmx_footprint: 1,
                geometry: String::new(),
                cells: Vec::new(),
                relations: Vec::new(),
            }],
        }],
    };
    let ftg = ft.meta.identity.guid;
    let (rev, mode) = ft.mode_guids(0, 0).unwrap();
    patch.fixture_types.insert(1, std::sync::Arc::new(ft));
    for id in 1..=10u32 {
        patch.insert_fid(id, lumacore_fixtures::Fixture::new(None, ftg, rev, mode, ug, id as u16, 1),
        );
    }
    state.root.show_data.after_load();
    state
}

/// A station holding `show`.
fn station(show: &ShowData) -> AppState {
    let mut state = AppState::new();
    crate::objects::replace_show(&mut state, show.clone());
    state
}

fn merge(base: &ShowData, master: &AppState, mine: &AppState) -> (ShowData, MergeReport) {
    let (out, report) = three_way(base, &master.root.show_data, &mine.root.show_data).unwrap();
    // The session stays valid: unique GUIDs, every link resolves, and the
    // merged show loads like a saved one.
    let id = out.identity();
    assert!(id.is_clean(), "{id:?}");
    let json = serde_json::to_string(&out).unwrap();
    crate::parse_show(&json).unwrap();
    (out, report)
}

fn cue_keys(show: &ShowData) -> Vec<u32> {
    show.datapools[&1].sequences[&1].cues.keys().copied().collect()
}

/// The percent fixture `f` has in cue `key` of sequence 1.
fn cue_value(show: &ShowData, key: u32, f: u32) -> Option<f64> {
    let g = show.fixture_guid(f)?;
    show.datapools[&1].sequences[&1].cues[&key]
        .values()
        .find(|v| v.fixture_id == g)
        .and_then(|v| v.value.plain().and_then(lumacore_tree::Value::percent))
}

/// `PrepareSeq1to10`: cue n holds fixture n at n × 10.
fn seq_1_to_10() -> AppState {
    let mut state = rig();
    for n in 1..=10 {
        run(&mut state, &format!("Fixture {n} At {}", n * 10));
        run(&mut state, &format!("Store Sequence 1 Cue {n}"));
        run(&mut state, "Select Sequence 1");
    }
    run(&mut state, "Clear");
    run(&mut state, "Clear");
    run(&mut state, "Clear");
    state
}

#[test]
fn merge_a_cue_inserted_while_not_in_session_is_kept() {
    let mut base = rig();
    run(&mut base, "Fixture 1 At 10");
    run(&mut base, "Store Sequence 1 Cue 1");
    run(&mut base, "Select Sequence 1");
    run(&mut base, "At 20");
    run(&mut base, "Store Cue 2");
    let show = base.root.show_data.clone();
    let master = station(&show);
    let mut mine = station(&show);
    run(&mut mine, "Fixture 1 At 30");
    run(&mut mine, "Store Cue 1.5");
    let (out, _) = merge(&show, &master, &mine);
    let k = cue_key("1.5").unwrap();
    assert_eq!(cue_keys(&out), vec![1000, k, 2000]);
    assert_eq!(cue_value(&out, k, 1), Some(30.0));
}

#[test]
fn merge_new_cues_from_the_rejoining_station_are_all_kept() {
    let base = seq_1_to_10();
    let show = base.root.show_data.clone();
    let master = station(&show);
    let mut mine = station(&show);
    run(&mut mine, "Fixture 9 At 100");
    for c in ["1.1", "2.1", "3.1", "4.1"] {
        run(&mut mine, &format!("Store Cue {c}"));
    }
    let (out, report) = merge(&show, &master, &mine);
    assert_eq!(cue_keys(&out).len(), 14);
    assert_eq!(cue_value(&out, cue_key("1.1").unwrap(), 9), Some(100.0));
    assert_eq!(cue_value(&out, 3000, 3), Some(30.0));
    assert!(report.added_mine.len() >= 4, "{report:?}");
}

#[test]
fn merge_cues_removed_on_the_rejoining_station_are_gone() {
    let base = seq_1_to_10();
    let show = base.root.show_data.clone();
    let master = station(&show);
    let mut mine = station(&show);
    for k in [3000, 5000, 7000] {
        mine.root
            .show_data
            .sequence_mut(1, 1)
            .unwrap()
            .cues
            .remove(&k);
    }
    let (out, _) = merge(&show, &master, &mine);
    assert_eq!(
        cue_keys(&out),
        vec![1000, 2000, 4000, 6000, 8000, 9000, 10000]
    );
}

fn groups(show: &ShowData) -> Vec<(u32, String)> {
    show.datapools[&1]
        .groups
        .iter()
        .map(|(&n, g)| (n, g.meta.identity.name.clone().unwrap_or_default()))
        .collect()
}

/// Groups 1–3 stored on one show.
fn three_groups() -> AppState {
    let mut state = rig();
    for n in 1..=3 {
        run(&mut state, &format!("Fixture {n}"));
        run(&mut state, &format!("Store Group {n} \"G{n}\""));
        run(&mut state, "Clear");
    }
    state
}

#[test]
fn merge_new_objects_on_both_sides_are_all_kept() {
    let base = three_groups();
    let show = base.root.show_data.clone();
    let mut master = station(&show);
    run(&mut master, "Fixture 5");
    run(&mut master, "Store Group 5 \"remote\"");
    let mut mine = station(&show);
    run(&mut mine, "Fixture 4");
    run(&mut mine, "Store Group 4 \"local\"");
    let (out, _) = merge(&show, &master, &mine);
    let names: Vec<String> = groups(&out).into_iter().map(|(_, n)| n).collect();
    assert_eq!(names, vec!["G1", "G2", "G3", "local", "remote"]);
}

#[test]
fn merge_deletes_on_both_sides_apply() {
    let base = three_groups();
    let show = base.root.show_data.clone();
    let mut master = station(&show);
    run(&mut master, "Delete Group 1 /nc");
    let mut mine = station(&show);
    run(&mut mine, "Delete Group 3 /nc");
    let (out, _) = merge(&show, &master, &mine);
    assert_eq!(groups(&out), vec![(2, "G2".to_string())]);
}

#[test]
fn merge_applies_a_move_and_use_master_drops_it() {
    let base = three_groups();
    let show = base.root.show_data.clone();
    let master = station(&show);
    let mut mine = station(&show);
    run(&mut mine, "Move Group 1 At 50");
    let (out, report) = merge(&show, &master, &mine);
    // Merge: the same object moved — no copy left at 1 (a plain merge would duplicate it).
    assert_eq!(
        groups(&out),
        vec![(2, "G2".into()), (3, "G3".into()), (50, "G1".into())]
    );
    assert_eq!(report.moves.len(), 1, "{report:?}");
    // Use Master keeps the session's show: the move is gone ("keep master").
    assert!(groups(&master.root.show_data).iter().all(|(n, _)| *n != 50));
}

#[test]
fn merge_same_slot_on_both_sides_keeps_both() {
    let base = three_groups();
    let show = base.root.show_data.clone();
    let mut master = station(&show);
    run(&mut master, "Fixture 6");
    run(&mut master, "Store Group 4 \"remote\"");
    let mut mine = station(&show);
    run(&mut mine, "Fixture 7");
    run(&mut mine, "Store Group 4 \"local\"");
    let (out, report) = merge(&show, &master, &mine);
    let g = groups(&out);
    assert!(g.contains(&(4, "remote".into())) && g.contains(&(5, "local".into())), "{g:?}");
    assert_eq!(out.datapools[&1].groups[&5].meta.name(), Some("local"));
    assert_eq!(report.moved_slots.len(), 1);
}
