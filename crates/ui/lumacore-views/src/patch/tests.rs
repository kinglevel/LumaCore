// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `patch`: the draft for its holder, the live devices for everyone else; the MVR tree's
//! counts.
//!
//! Owns:          the unit tests of patch.rs.
//! Doesn't own:   the draft rules (lumacore-objects' patch_draft_tests).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 "34g4 patch hardened" 9, 12.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use lumacore_base::node::NodeMeta;
use lumacore_tree::fixtures::{Fixture, MvrGroup, MvrLayer, Universe};

fn rig() -> ShowData {
    let mut show = ShowData::default();
    let universe = Universe::new(Some("Main".into()));
    let ug = universe.meta.identity.guid;
    show.devices.dmx.patch.universes.insert(1, universe);
    let layer = MvrLayer { meta: NodeMeta::new(Some("House".into())), matrix: Default::default(), order: 0 };
    let group = MvrGroup { meta: NodeMeta::new(Some("Truss 1".into())), parent: layer.meta.identity.guid, matrix: Default::default(), order: 0 };
    let g = group.meta.identity.guid;
    show.devices.metadata.mvr.layers.insert(1, layer);
    show.devices.metadata.mvr.groups.insert(1, group);
    for (no, addr) in [(101, 1), (102, 35)] {
        let nil = Uuid::nil();
        let mut f = Fixture::new(Some(format!("Spot {no}")), nil, nil, nil, ug, addr, 34);
        f.mvr.parent = Some(g);
        f.mvr.matrix.set_position_m(2, 7.2);
        show.devices.dmx.patch.insert_fid(no, f);
    }
    show
}

#[test]
fn the_holder_sees_the_draft_others_the_live_devices() {
    let show = rig();
    let mut draft = show.devices.clone();
    draft.dmx.patch.remove_fid(102);
    let live = view(&show, Some(("Bob", &draft, &Marks::new(), None)), "Alice");
    assert_eq!(live["mode"], "live");
    assert_eq!(live["holder"], "Bob");
    assert_eq!(live["fixtures"].as_array().unwrap().len(), 2);
    let mine = view(&show, Some(("Bob", &draft, &Marks::new(), None)), "bob");
    assert_eq!(mine["mode"], "draft");
    assert_eq!(mine["fixtures"].as_array().unwrap().len(), 1);
}

#[test]
fn rows_carry_the_columns_and_the_tree_counts_its_nodes() {
    let v = view(&rig(), None, "Alice");
    let f = &v["fixtures"][0];
    assert_eq!(f["address"], "1.001");
    assert_eq!(f["pos_z"], "7.2");
    assert_eq!(f["group"], "Truss 1");
    let tree = v["tree"].as_array().unwrap();
    let count = |kind: &str| tree.iter().find(|n| n["kind"] == kind).unwrap()["count"].clone();
    assert_eq!(count("layer"), 2, "a layer counts the fixtures of its groups");
    assert_eq!(count("group"), 2);
    assert_eq!(v["unsorted"], 0);
    // Each node's ≡ menu lines: a new group nests at once, only a group moves (§34 part 3c).
    let lines = |kind: &str| tree.iter().find(|n| n["kind"] == kind).unwrap()["lines"].clone();
    assert_eq!(lines("group")["child"], "Store MvrGroup 2 {name}; Move MvrGroup 2 At MvrGroup 1 /NoConfirmation");
    assert_eq!(lines("group")["move"], "Move MvrGroup 1 At {to}");
    assert_eq!(lines("layer")["move"], serde_json::Value::Null);
    assert_eq!(lines("layer")["copy"], "Copy MvrLayer 1 At 2");
}

#[test]
fn an_overlap_or_a_lost_universe_marks_the_address_and_the_draft_marks_its_changes() {
    let mut show = rig();
    // 102 starts inside 101's 34 slots: both are red.
    show.devices.dmx.patch.fixture_mut(102).unwrap().mvr.breaks[0].start_address = 20;
    let v = view(&show, None, "Alice");
    assert_eq!(v["fixtures"][0]["address_tone"], "alert");
    assert_eq!(v["fixtures"][1]["address_tone"], "alert");
    assert_eq!(v["path"], "Root.ShowData.Devices.Dmx.Patch");
    // Its universe gone: "no universe", a warning.
    show.devices.dmx.patch.universes.clear();
    let v = view(&show, None, "Alice");
    assert_eq!(v["fixtures"][0]["address"], "no universe");
    assert_eq!(v["fixtures"][0]["address_tone"], "warn");
    // The holder's draft names the cells that differ from the live rig.
    let show = rig();
    let mut draft = show.devices.clone();
    draft.dmx.patch.fixture_mut(101).unwrap().meta.identity.name = Some("Front".into());
    let mine = view(&show, Some(("Bob", &draft, &Marks::new(), None)), "Bob");
    assert_eq!(mine["fixtures"][0]["changed"], serde_json::json!(["name"]));
    assert_eq!(mine["fixtures"][1]["changed"], serde_json::json!([]));
    assert_eq!(view(&show, Some(("Bob", &draft, &Marks::new(), None)), "Alice")["fixtures"][0]["changed"], serde_json::Value::Null);
}

#[test]
fn the_holder_sees_which_cells_a_live_edit_brought_in() {
    let show = rig();
    let draft = show.devices.clone();
    let g = draft.dmx.patch.fixture(102).expect("the fixture").meta.identity.guid;
    let mut marks = Marks::new();
    marks.entry(g).or_default().insert("address".into(), "Alice".into());
    let mine = view(&show, Some(("Bob", &draft, &marks, None)), "Bob");
    assert_eq!(mine["fixtures"][1]["live_changed"]["address"], "Alice");
    assert_eq!(mine["fixtures"][0]["live_changed"], serde_json::Value::Null);
}

/// An overlap parks both and lists one-tap fixes; every universe says its slots in use; the map
/// gets every run (decisions §34 round 3, 18).
#[test]
fn an_overlap_is_parked_listed_with_its_fixes_and_counted_once() {
    let mut show = rig();
    show.devices.dmx.patch.fixture_mut(102).unwrap().mvr.breaks[0].start_address = 30;
    let v = view(&show, None, "Alice");
    assert_eq!((v["fixtures"][0]["parked"].clone(), v["fixtures"][1]["parked"].clone()), (json!(true), json!(true)));
    assert_eq!(v["universes"][0]["used"], 63, "1–34 and 30–63, the shared slots once");
    assert_eq!(v["runs"].as_array().unwrap().len(), 2);
    assert_eq!(v["runs"][1]["start"], 30);
    let c = &v["conflicts"][0];
    assert_eq!((c["kind"].clone(), c["a"]["range"].clone(), c["b"]["range"].clone()), (json!("overlap"), json!("1.001–1.034"), json!("1.030–1.063")));
    let lines: Vec<&str> = c["fixes"].as_array().unwrap().iter().map(|f| f["line"].as_str().unwrap()).collect();
    assert_eq!(lines, ["Assign Fixture 102 At DMX", "Assign Fixture 101 At Fixture 102 DMX /Swap", "Unassign Fixture 102 At DMX"]);
}

/// A folder — a fixture without a number holding fixtures (decisions §40 "40d hardened") — is a
/// node of the sidebar's tree with its handle, never a row; the counts are of numbered fixtures;
/// a row names its node.
#[test]
fn a_folder_is_a_node_of_the_tree_not_a_row() {
    let mut show = rig();
    let g = show.devices.metadata.mvr.groups[&1].meta.identity.guid;
    let nil = Uuid::nil();
    let mut folder = Fixture::new(Some("Stage 1".into()), nil, nil, nil, Uuid::nil(), 0, 0);
    folder.mvr.breaks.clear();
    folder.mvr.fid = None;
    folder.mvr.id_type = lumacore_tree::fixtures::IdType::Media;
    folder.mvr.cid = Some(1);
    folder.mvr.parent = Some(g);
    let fg = folder.meta.identity.guid;
    show.devices.dmx.patch.insert_fixture(folder);
    show.devices.dmx.patch.fixture_mut(102).unwrap().mvr.parent = Some(fg);
    let v = view(&show, None, "Alice");
    let rows = v["fixtures"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "the two numbered fixtures");
    assert_eq!(rows[1]["group"], "Stage 1");
    let tree = v["tree"].as_array().unwrap();
    let node = tree.iter().find(|n| n["kind"] == "folder").expect("the folder node");
    assert_eq!(node["id"], "Media 1");
    assert_eq!(node["parent"], serde_json::json!(g));
    assert_eq!(node["count"], 1);
    assert_eq!(node["lines"]["at"], "Fixture Media 1");
    assert_eq!(node["lines"]["move"], "Move Fixture Media 1 At {to}");
    assert_eq!(node["lines"]["delete"], "Delete Fixture Media 1");
    let count = |kind: &str| tree.iter().find(|n| n["kind"] == kind).unwrap()["count"].clone();
    assert_eq!(count("group"), 2, "the group counts the folder's fixture, not the folder");
}
