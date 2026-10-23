// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `devices_meta`: the MVR tree's words through command lines — Store, Assign /
//! Unassign, the bare word's selection, the nesting question, Delete with what it takes along.
//!
//! Owns:          the unit tests of devices_meta.rs and its verbs.
//! Doesn't own:   the door's own tests (`commands::door`), the Tag nesting tests (cmd-tags).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 "34g4 patch hardened" 12, "34g4 part 3c hardened".

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

/// Six fixtures, the Devices claimed, a layer with two groups: 1 "Truss 1", 2 "Truss 2".
fn tree() -> AppState {
    let mut s = rig(&[1, 2, 3, 4, 5, 6]);
    run(&mut s, "Claim Root.ShowData.Devices");
    run(&mut s, "Store MvrLayer \"Rig\"");
    run(&mut s, "Store MvrGroup \"Truss 1\"");
    run(&mut s, "Store MvrGroup \"Truss 2\"");
    s
}

fn parent(s: &AppState, fixture: u32) -> Option<uuid::Uuid> {
    s.root.show_data.devices.dmx.patch.fixture(fixture).expect("resolved").mvr.parent
}

fn group_guid(s: &AppState, n: u32) -> uuid::Uuid {
    s.root.show_data.devices.metadata.mvr.groups[&n].meta.identity.guid
}

#[test]
fn store_needs_the_claim_and_a_layer_first() {
    let mut s = rig(&[1]);
    let out = run(&mut s, "Store MvrLayer \"Rig\"");
    assert!(out.contains("E-CMD-010") && out.contains("Claim Root.ShowData.Devices"), "{out}");
    run(&mut s, "Claim Root.ShowData.Devices");
    let out = run(&mut s, "Store MvrGroup \"Truss\"");
    assert!(out.contains("Store an MvrLayer first"), "{out}");
    let out = run(&mut s, "Store MvrLayer \"Rig\"");
    assert!(out.contains("Stored MvrLayer 1 \"Rig\""), "{out}");
    let out = run(&mut s, "Store MvrGroup \"Truss\"");
    assert!(out.contains("Stored MvrGroup 1 \"Truss\" in MvrLayer 1 \"Rig\""), "{out}");
    let out = run(&mut s, "Store MvrGroup 1");
    assert!(out.contains("E-CMD-010") && out.contains("Label MvrGroup 1"), "{out}");
}

#[test]
fn assign_gives_a_fixture_one_place_and_the_bare_word_selects() {
    let mut s = tree();
    let out = run(&mut s, "Assign Fixture 1 Thru 4 At MvrGroup 1");
    assert!(out.contains("4 fixtures (1 Thru 4) into MvrGroup 1 \"Truss 1\""), "{out}");
    // One place: they leave group 1 for group 2.
    run(&mut s, "Assign Fixture 3 Thru 4 At MvrGroup 2");
    assert_eq!(parent(&s, 1), Some(group_guid(&s, 1)));
    assert_eq!(parent(&s, 3), Some(group_guid(&s, 2)));
    let out = run(&mut s, "MvrGroup 1");
    assert!(out.starts_with("2 fixtures from MvrGroup 1"), "{out}");
    // The layer reaches everything under it.
    let out = run(&mut s, "MvrLayer 1");
    assert!(out.starts_with("4 fixtures"), "{out}");
    let out = run(&mut s, "Unassign Fixture 1 Thru 4 At MvrGroup 2");
    assert!(out.contains("2 fixtures (3 Thru 4) now in No MVR group"), "{out}");
    assert_eq!(parent(&s, 3), None);
    assert_eq!(parent(&s, 1), Some(group_guid(&s, 1)));
    let out = run(&mut s, "Assign Fixture 5 At Universe 1");
    assert!(out.contains("E-PATCH-011"), "{out}");
}

#[test]
fn move_at_a_node_nests_after_one_question() {
    let mut s = tree();
    run(&mut s, "Assign Fixture 1 Thru 2 At MvrGroup 2");
    let out = run(&mut s, "Move MvrGroup 2 At MvrGroup 1");
    assert!(
        out.starts_with("ASK") && out.contains("MvrGroup 2 \"Truss 2\" becomes a child of MvrGroup 1 \"Truss 1\" (1 group, 2 fixtures)?"),
        "{out}"
    );
    assert_ne!(s.root.show_data.devices.metadata.mvr.groups[&2].parent, group_guid(&s, 1), "nothing moved before the answer");
    let out = run(&mut s, "Move MvrGroup 2 At MvrGroup 1 /NoConfirmation");
    assert!(out.contains("Moved MvrGroup 2 into MvrGroup 1"), "{out}");
    assert_eq!(s.root.show_data.devices.metadata.mvr.groups[&2].parent, group_guid(&s, 1));
    // Group 1 now reaches the fixtures of its child.
    let out = run(&mut s, "MvrGroup 1");
    assert!(out.starts_with("2 fixtures"), "{out}");
    // Never under itself or its own groups; a layer nests nowhere.
    let out = run(&mut s, "Move MvrGroup 1 At MvrGroup 2");
    assert!(out.contains("E-PATCH-011"), "{out}");
    let out = run(&mut s, "Move MvrLayer 1 At MvrGroup 1");
    assert!(out.contains("E-PATCH-011"), "{out}");
    // At a number stays the door's renumber.
    let out = run(&mut s, "Move MvrGroup 2 At 7");
    assert!(!out.contains("ASK"), "{out}");
    assert!(s.root.show_data.devices.metadata.mvr.groups.contains_key(&7));
}

/// A node that holds something asks: Keep the fixtures (the default, they move up one level) or
/// Delete them too (decisions §34 round 3 18, 19).
#[test]
fn delete_keeps_the_fixtures_unless_told() {
    let mut s = tree();
    run(&mut s, "Move MvrGroup 2 At MvrGroup 1 /NoConfirmation");
    run(&mut s, "Assign Fixture 1 Thru 3 At MvrGroup 2");
    run(&mut s, "Assign Fixture 4 At MvrGroup 1");
    let out = run(&mut s, "Delete MvrGroup 1");
    assert!(out.starts_with("ASK") && out.contains("It holds 1 group, 4 fixtures"), "{out}");
    assert!(run(&mut s, "Delete MvrGroup 1 /KeepChildren /WithChildren /NoConfirmation").contains("E-CMD-006"));
    // The default: group 2 and fixture 4 move up to the layer; nothing is deleted but group 1.
    run(&mut s, "Delete MvrGroup 1 /NoConfirmation");
    let layer = s.root.show_data.devices.metadata.mvr.layers[&1].meta.identity.guid;
    let devices = &s.root.show_data.devices;
    assert_eq!(devices.metadata.mvr.groups.keys().copied().collect::<Vec<_>>(), vec![2]);
    assert_eq!(devices.metadata.mvr.groups[&2].parent, layer);
    assert_eq!(devices.dmx.patch.fixtures.len(), 6);
    assert_eq!(parent(&s, 4), Some(layer));
    assert_eq!(parent(&s, 1), Some(group_guid(&s, 2)), "still in its own group");
    // /WithChildren: the group and its fixtures go.
    run(&mut s, "Delete MvrGroup 2 /WithChildren /NoConfirmation");
    assert_eq!(s.root.show_data.devices.dmx.patch.fids().into_iter().map(|(n, _)| n).collect::<Vec<_>>(), vec![4, 5, 6]);
    // A layer kept: its fixtures go to No MVR group, its groups with it.
    run(&mut s, "Delete MvrLayer 1 /KeepChildren /NoConfirmation");
    assert!(s.root.show_data.devices.metadata.mvr.layers.is_empty());
    assert_eq!(parent(&s, 4), None);
}

#[test]
fn list_reads_the_tree() {
    let mut s = tree();
    run(&mut s, "Move MvrGroup 2 At MvrGroup 1 /NoConfirmation");
    run(&mut s, "Assign Fixture 1 At MvrGroup 2");
    let out = run(&mut s, "List MvrGroup");
    assert!(out.contains("MvrGroup 2 \"Truss 2\" · in MvrGroup 1 \"Truss 1\" · 0 groups, 1 fixture"), "{out}");
    assert!(out.contains("MvrGroup 1 \"Truss 1\" · in MvrLayer 1 \"Rig\" · 1 group, 1 fixture"), "{out}");
    let out = run(&mut s, "List MvrClass");
    assert!(out.contains("No such MvrClass") && out.contains("E-CMD-007"), "{out}");
}


/// A fixture may sit straight in a layer (MVR's ChildList of a Layer); the layer then selects it
/// and Unassign takes it back out (the round 3 audit).
#[test]
fn a_fixture_can_sit_in_a_layer_itself() {
    let mut s = tree();
    let layer = s.root.show_data.devices.metadata.mvr.layers[&1].meta.identity.guid;
    let out = run(&mut s, "Assign Fixture 5 At MvrLayer 1");
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(parent(&s, 5), Some(layer));
    let out = run(&mut s, "MvrLayer 1");
    assert!(out.starts_with("1 fixture from MvrLayer 1"), "{out}");
    run(&mut s, "Unassign Fixture 5 At MvrLayer 1");
    assert_eq!(parent(&s, 5), None);
}

/// MvrClass and MvrPosition come with an import; typed alone they select their fixtures, and
/// they can't be stored by hand (the round 3 audit).
#[test]
fn a_class_and_a_position_alone_select_their_fixtures() {
    let mut s = tree();
    let md = &mut s.root.show_data.devices.metadata.mvr;
    let class = lumacore_fixtures::MvrClass { meta: lumacore_base::NodeMeta::new(Some("Spots".into())) };
    let position = lumacore_fixtures::MvrPosition { meta: lumacore_base::NodeMeta::new(Some("FOH".into())) };
    let (cg, pg) = (class.meta.identity.guid, position.meta.identity.guid);
    md.classes.insert(1, class);
    md.positions.insert(1, position);
    for n in [1, 2, 3] {
        s.root.show_data.devices.dmx.patch.fixture_mut(n).unwrap().mvr.class = Some(cg);
    }
    s.root.show_data.devices.dmx.patch.fixture_mut(6).unwrap().mvr.position = Some(pg);
    s.root.show_data.after_load();
    let out = run(&mut s, "MvrClass 1");
    assert!(out.starts_with("3 fixtures from MvrClass 1"), "{out}");
    let out = run(&mut s, "MvrPosition 1");
    assert!(out.starts_with("1 fixture from MvrPosition 1"), "{out}");
    let out = run(&mut s, "List MvrClass");
    assert!(out.contains("3 fixtures"), "{out}");
    let out = run(&mut s, "Store MvrClass \"Washes\"");
    assert!(out.contains("E-"), "{out}");
}

/// Copy and Label of a group go through the one door: a copy is a new group (new GUID) in the
/// same parent without the fixtures (a fixture has one place); Label renames (the round 3 audit).
#[test]
fn copy_and_label_of_a_group() {
    let mut s = tree();
    run(&mut s, "Assign Fixture 1 Thru 2 At MvrGroup 1");
    let out = run(&mut s, "Copy MvrGroup 1 At 5");
    assert!(!out.contains("E-"), "{out}");
    let groups = &s.root.show_data.devices.metadata.mvr.groups;
    assert_ne!(groups[&5].meta.identity.guid, groups[&1].meta.identity.guid);
    assert_eq!(groups[&5].parent, groups[&1].parent);
    assert_eq!(parent(&s, 1), Some(group_guid(&s, 1)), "the fixtures stay in their one place");
    let out = run(&mut s, "Label MvrGroup 1 \"Front truss\"");
    assert!(!out.contains("E-"), "{out}");
    let name = s.root.show_data.devices.metadata.mvr.groups[&1].meta.identity.name.clone();
    assert_eq!(name.as_deref(), Some("Front truss"));
}

/// A fixture's class by hand (round 3, the left-out items): `Assign … At MvrClass n` gives it one,
/// `Unassign … At MvrClass` takes it away; classes still come only from an import.
#[test]
fn assign_gives_a_fixture_its_class() {
    let mut s = tree();
    let class = lumacore_fixtures::MvrClass { meta: lumacore_base::NodeMeta::new(Some("Spots".into())) };
    let cg = class.meta.identity.guid;
    s.root.show_data.devices.metadata.mvr.classes.insert(1, class);
    s.root.show_data.after_load();
    let class_of = |s: &AppState, n: u32| s.root.show_data.devices.dmx.patch.fixture(n).expect("resolved").mvr.class;
    let out = run(&mut s, "Assign Fixture 4 Thru 5 At MvrClass 1");
    assert!(out.contains("in class Spots"), "{out}");
    assert_eq!((class_of(&s, 4), class_of(&s, 5)), (Some(cg), Some(cg)));
    let out = run(&mut s, "Unassign Fixture 4 At MvrClass");
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(class_of(&s, 4), None);
    let out = run(&mut s, "Assign Fixture 4 At MvrClass 9");
    assert!(out.contains("E-CMD-007"), "{out}");
}

// *************************************************************************************************
// Folders (decisions §40 "40d hardened" 3–4), on the rig
// *************************************************************************************************

fn ask(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Ask(a) => format!("ASK {} [{}]", a.view().text, a.view().buttons.iter().map(|b| b.label.clone()).collect::<Vec<_>>().join(" | ")),
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => String::new(),
    }
}

fn folder(s: &AppState, cid: u32) -> uuid::Uuid {
    s.root.show_data.devices.dmx.patch.fixture_by_id(lumacore_fixtures::IdType::Media, cid).expect("the folder").1.meta.identity.guid
}

#[test]
fn assign_puts_fixtures_into_a_folder_and_never_under_themselves() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Claim Root.ShowData.Devices");
    // Smoke (Media 9) holds 6: two Photons join it, last in its order.
    let out = run(&mut s, "Assign Fixture 101 Thru 102 At Fixture Media 9");
    assert!(out.contains("2 fixtures (101 Thru 102) into Fixture Media 9 \"Smoke\""), "{out}");
    assert_eq!(parent(&s, 101), Some(folder(&s, 9)));
    assert_eq!(super::nodes::fixtures_under(&s.root.show_data.devices, folder(&s, 9)).len(), 8);
    let orders: Vec<u32> = [101, 102].iter().map(|n| s.root.show_data.devices.dmx.patch.fixture(n).unwrap().mvr.order).collect();
    assert!(orders[1] > orders[0], "{orders:?}");
    // A folder into one of its own: refused. Any fixture may hold fixtures (§40 3).
    let out = run(&mut s, "Assign Fixture Media 1 At Fixture Media 7003");
    assert!(out.contains("E-PATCH-011") && out.contains("can't go under itself"), "{out}");
    let out = run(&mut s, "Assign Fixture 103 At Fixture 104");
    assert!(out.contains("Fixture 103 into Fixture 104"), "{out}");
    assert_eq!(parent(&s, 103), Some(s.root.show_data.fixture_guid(104).unwrap()));
    // Something else than a node: the usage names the three kinds.
    let out = run(&mut s, "Assign Fixture 103 At Universe 1");
    assert!(out.contains("E-PATCH-011") && out.contains("At Fixture <id>"), "{out}");
}

#[test]
fn move_at_a_node_nests_after_one_question_and_at_a_number_renumbers() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Claim Root.ShowData.Devices");
    // Deco LED (Media 7006, 19 fixtures) under Stage 2 Keylights (Media 2): asked first.
    let out = ask(&mut s, "Move Fixture Media 7006 At Fixture Media 2");
    assert!(out.starts_with("ASK Fixture Media 7006 \"Deco LED\" becomes a child of Fixture Media 2 \"Stage 2 Keylights\" (1 folder, 19 fixtures)?"), "{out}");
    assert!(out.contains("[Nest | Cancel]"), "{out}");
    assert_eq!(parent_of_folder(&s, 7006), Some(folder(&s, 1)), "nothing moved before the answer");
    let out = run(&mut s, "Move Fixture Media 7006 At Fixture Media 2 /NoConfirmation");
    assert!(out.contains("Moved 1 into Fixture Media 2 \"Stage 2 Keylights\""), "{out}");
    assert_eq!(parent_of_folder(&s, 7006), Some(folder(&s, 2)));
    assert_eq!(super::nodes::fixtures_under(&s.root.show_data.devices, folder(&s, 2)).len(), 56 + 19);
    // A numbered fixture At a node nests too; At a bare number renumbers; a folder has no number.
    let out = run(&mut s, "Move Fixture 101 At Fixture Media 1 /NoConfirmation");
    assert!(out.contains("Moved 1 into Fixture Media 1"), "{out}");
    assert_eq!(parent(&s, 101), Some(folder(&s, 1)));
    let out = run(&mut s, "Move Fixture 101 At 30001");
    assert!(out.contains("Fixture 101 moved to Fixture 30001"), "{out}");
    let out = run(&mut s, "Move Fixture Media 1 At 5");
    assert!(out.contains("E-PATCH-011") && out.contains("A folder has no number"), "{out}");
    // Never under itself.
    let out = run(&mut s, "Move Fixture Media 1 At Fixture Media 7003");
    assert!(out.contains("E-PATCH-011"), "{out}");
}

fn parent_of_folder(s: &AppState, cid: u32) -> Option<uuid::Uuid> {
    s.root.show_data.devices.dmx.patch.fixture_by_id(lumacore_fixtures::IdType::Media, cid).map(|(_, f)| f.mvr.parent).unwrap_or(None)
}

#[test]
fn delete_of_a_node_asks_with_both_ways_out() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Claim Root.ShowData.Devices");
    let before = s.root.show_data.devices.dmx.patch.fixtures.len();
    // Stage 1 Mainstage: 9 folders and 1 246 fixtures under it.
    let out = ask(&mut s, "Delete Fixture Media 1");
    assert!(out.starts_with("ASK Delete Fixture Media 1 \"Stage 1 Mainstage\" — 9 folders and 1246 fixtures under it?"), "{out}");
    assert!(out.contains("[Delete all | Keep them: move up to MvrLayer 1 \"Fixture Layer\" | Cancel]"), "{out}");
    assert_eq!(s.root.show_data.devices.dmx.patch.fixtures.len(), before, "nothing went before the answer");
    // Keep them: the nine folders move up to the layer, their fixtures with them.
    let layer = s.root.show_data.devices.metadata.mvr.layers[&1].meta.identity.guid;
    let out = run(&mut s, "Delete Fixture Media 1 /KeepChildren /NoConfirmation");
    assert!(out.contains("Deleted Fixture Media 1 \"Stage 1 Mainstage\"."), "{out}");
    assert_eq!(s.root.show_data.devices.dmx.patch.fixtures.len(), before - 1);
    assert_eq!(parent_of_folder(&s, 7003), Some(layer));
    assert_eq!(super::nodes::fixtures_under(&s.root.show_data.devices, layer).len(), 1348);
    // Delete all: Deco LED with its 19.
    let out = run(&mut s, "Delete Fixture Media 7006 /NoConfirmation");
    assert!(out.contains("Deleted Fixture Media 7006 \"Deco LED\" with 19 under it."), "{out}");
    assert_eq!(s.root.show_data.devices.dmx.patch.fixtures.len(), before - 1 - 20);
    assert!(s.root.show_data.devices.dmx.patch.fixture(1198).is_none(), "A1 went with its folder");
    // A leaf asks the plain question (dispatch's), and goes.
    let out = ask(&mut s, "Delete Fixture 101");
    assert!(out.starts_with("ASK Delete Fixture 101?"), "{out}");
    let out = run(&mut s, "Delete Fixture 101 /NoConfirmation");
    assert!(out.contains("Deleted Fixture 101"), "{out}");
}

#[test]
fn list_and_info_take_a_folder_by_its_second_id() {
    let mut s = crate::demo::fresh_state();
    let out = run(&mut s, "List Fixture Media 9");
    assert_eq!(out.matches("Generic Grouping").count(), 0, "no folder rows: {out}");
    assert_eq!(out.matches("│ 300").count(), 6, "Smoke's six: {out}");
    assert!(!out.contains("│ 101 "), "{out}");
    let out = run(&mut s, "Info Fixture Media 1");
    assert!(out.contains("Fixture Media 1") && out.contains("Stage 1 Mainstage") && out.contains("9 folders, 1246 fixtures") && out.contains("MvrLayer 1 \"Fixture Layer\""), "{out}");
    // A number still lists the one.
    let out = run(&mut s, "List Fixture 101");
    assert!(out.contains("101"), "{out}");
}
