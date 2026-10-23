// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of presets linked into cues on the rig (decisions 0293): the demo's Sequence 1
//! "Presets" links Presets 1–3 (cue 3 holds a plain value beside its links). Store Cue keeps the
//! link only, playback reads the preset as it is now, tracking compares links as links; Update
//! (the ask, its lines, one Oops step), Recast, Extract and deleting a used preset.
//!
//! Owns:          the unit tests of links.rs, update.rs, recast.rs, extract.rs, delete.rs and the
//!                Sequence's Update / Extract.
//! Doesn't own:   the show-format upgrade (lumacore-luma), the views (lumacore-views).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0293-presets-link-into-cues-update-recast-extract.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_tree::{CueData, CueObject};
use uuid::Uuid;

use crate::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match crate::commands::dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        DispatchResult::Ask(a) => format!("ASK {}", a.title()),
        _ => String::new(),
    }
}

fn ask(state: &mut AppState, line: &str) -> crate::dialogs::Ask {
    match crate::commands::dispatch(state, line) {
        DispatchResult::Ask(a) => *a,
        DispatchResult::Output(s) => panic!("`{line}` didn't ask: {}", crate::text::strip_ansi(&s)),
        _ => panic!("`{line}` didn't ask"),
    }
}

fn guid(state: &AppState, fixture: u32) -> Uuid {
    state.root.show_data.fixture_guid(fixture).unwrap_or_else(|| panic!("Fixture {fixture}"))
}

fn preset_guid(state: &AppState, no: u32) -> Uuid {
    super::super::in_pool(state).and_then(|p| p.get(&no)).map(|p| p.meta.identity.guid).unwrap_or_else(|| panic!("Preset {no}"))
}

fn cue(state: &AppState, seq: u32, key: u32) -> CueObject {
    let dp = state.selected_datapool();
    state.root.show_data.datapools[&dp].sequences[&seq].cues.get(&key).cloned().unwrap_or_else(|| panic!("Sequence {seq} cue {key}"))
}

/// A cue's value of a fixture's Dimmer.
fn dimmer_in(state: &AppState, seq: u32, key: u32, fixture: u32) -> Option<CueData> {
    let g = guid(state, fixture);
    cue(state, seq, key).values().find(|v| v.fixture_id == g && v.attribute == "Dimmer").map(|v| v.value.clone())
}

/// What the stage plays for a fixture's Dimmer (percent).
fn played(state: &AppState, fixture: u32) -> Option<f64> {
    state.output_plan().resolved(guid(state, fixture), "Dimmer").and_then(|r| r.value.percent())
}

fn steps(state: &AppState) -> usize {
    state.undo_stack("Admin").map_or(0, |u| u.frames.len())
}

/// An attribute fixture 101 has besides Dimmer (the rig's PHOTON).
fn other_attr(state: &AppState) -> String {
    let show = &state.root.show_data;
    let f = show.devices.dmx.patch.fixtures.values().find(|f| f.meta.identity.guid == guid(state, 101)).expect("fixture 101");
    lumacore_render::merge::channels(show, f)
        .iter()
        .find(|cf| !cf.is_virtual && cf.attribute_name != "Dimmer")
        .map(|cf| cf.attribute_name.clone())
        .expect("a second attribute")
}

#[test]
fn the_rig_cues_hold_the_links_only() {
    let s = crate::demo::fresh_state();
    assert_eq!(dimmer_in(&s, 1, 1000, 101), Some(CueData::Link(preset_guid(&s, 1))));
    assert_eq!(dimmer_in(&s, 1, 2000, 110), Some(CueData::Link(preset_guid(&s, 2))));
    assert_eq!(dimmer_in(&s, 1, 3000, 102), Some(CueData::Link(preset_guid(&s, 3))));
    assert!(matches!(dimmer_in(&s, 1, 3000, 103), Some(CueData::Value(v)) if v.percent() == Some(30.0)), "cue 3's plain value");
}

#[test]
fn store_cue_keeps_the_link_and_a_plain_value_over_it_drops_it() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Fixture 104");
    run(&mut s, "At Preset 2");
    let out = run(&mut s, "Store Sequence 901 Cue 1");
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(dimmer_in(&s, 901, 1000, 104), Some(CueData::Link(preset_guid(&s, 2))), "the link, no copy");
    run(&mut s, "At 40");
    run(&mut s, "Store Sequence 901 Cue 2");
    assert!(matches!(dimmer_in(&s, 901, 2000, 104), Some(CueData::Value(v)) if v.preset.is_none() && v.percent() == Some(40.0)));
    // The programmer still knows where the value came from (Update's home).
    let pv = s.current_programmer().unwrap().get(guid(&s, 104), "Dimmer").cloned().unwrap();
    assert_eq!((pv.value.preset, pv.origin), (None, Some(preset_guid(&s, 2))));
}

#[test]
fn playback_plays_the_preset_as_it_is_now_and_tracks_links() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Goto Sequence 1 Cue 3");
    assert_eq!(played(&s, 101), Some(20.0), "Preset 3's own value for 101");
    assert_eq!(played(&s, 105), Some(50.0), "tracked from cue 2: Preset 2");
    assert_eq!(played(&s, 103), Some(30.0), "the plain value");
    // Change Preset 2: the cue that links it follows, nothing else to do.
    run(&mut s, "Fixture 101 Thru 110");
    run(&mut s, "At Preset 2");
    run(&mut s, "At 65");
    let out = run(&mut s, "Update Preset 2 /Merge");
    assert!(out.contains("Updated Preset 2") && out.contains("1 cue follows"), "{out}");
    run(&mut s, "ResetAll");
    assert_eq!(played(&s, 105), Some(65.0), "the cue plays the updated preset");
    // A preset without the fixture's value gives none: 104 has no Preset 3 entry, cue 3 links
    // nothing for it, so it still plays Preset 2 from cue 2.
    assert_eq!(played(&s, 104), Some(65.0));
}

#[test]
fn a_link_equal_to_what_tracks_in_is_stored_as_blocked() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Fixture 101 Thru 110 At Preset 2");
    let out = run(&mut s, "Store Sequence 1 Cue 2.5");
    assert!(out.contains("(10 blocked"), "the same link tracks in from cue 2: {out}");
}

#[test]
fn update_preset_merge_add_and_overwrite() {
    let mut s = crate::demo::fresh_state();
    let other = other_attr(&s);
    let p1 = preset_guid(&s, 1);
    run(&mut s, "Fixture 101 Thru 110");
    run(&mut s, "At Preset 1");
    run(&mut s, "At 70");
    run(&mut s, &format!("Attribute \"{other}\" At 40"));
    run(&mut s, "Update Preset 1 /Merge");
    let p = super::super::in_pool(&s).unwrap()[&1].clone();
    assert_eq!(p.attributes(), vec!["Dimmer"], "Merge: only what it held");
    assert_eq!(p.value_for(guid(&s, 105), "Dimmer").and_then(|v| v.percent()), Some(70.0));
    let pv = s.current_programmer().unwrap().get(guid(&s, 105), "Dimmer").cloned().unwrap();
    assert_eq!(pv.value.preset, Some(p1), "the programmer links the preset again");
    run(&mut s, "Update Preset 1 /Add");
    let p = super::super::in_pool(&s).unwrap()[&1].clone();
    assert!(p.attributes().contains(&other.as_str()), "Add: the new attribute too: {:?}", p.attributes());
    // Overwrite: only these fixtures' values, the preset's other entries go.
    run(&mut s, "ResetAll");
    run(&mut s, "Fixture 101");
    run(&mut s, "At Preset 1");
    run(&mut s, "At 5");
    run(&mut s, "Update Preset 1 /Overwrite");
    let p = super::super::in_pool(&s).unwrap()[&1].clone();
    assert!(p.values.iter().all(|e| e.fixture_id == Some(guid(&s, 101)) || e.fixture_id.is_none()), "{:?}", p.values);
    assert!(p.value_for(guid(&s, 105), "Dimmer").is_none(), "Overwrite dropped the other fixtures");
    // Nothing from it in the programmer: refused.
    run(&mut s, "ResetAll");
    assert!(run(&mut s, "Update Preset 2").contains("E-CMD-009"));
}

#[test]
fn the_update_ask_lists_presets_with_their_cues_and_the_plain_cues() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Goto Sequence 1 Cue 3");
    run(&mut s, "Fixture 101 + 102");
    run(&mut s, "At Preset 3");
    run(&mut s, "At 90");
    run(&mut s, "Fixture 103 At 45");
    let a = ask(&mut s, "Update");
    let tree = a.view().tree.clone().expect("a tree");
    let ids: Vec<&str> = tree.nodes.iter().map(|n| n.id.as_str()).collect();
    let dp = s.selected_datapool();
    let p3 = tree.nodes.iter().find(|n| n.id == format!("p{dp}_3")).unwrap_or_else(|| panic!("{ids:?}"));
    assert!(p3.on && p3.tickable && p3.no == "P3" && p3.badge == "S", "{p3:?}");
    let cue3 = tree.nodes.iter().find(|n| n.kind == "cue" && n.no == "Cue 3").unwrap_or_else(|| panic!("{ids:?}"));
    assert!(cue3.on && cue3.depth == 1, "a cue that follows starts ticked");
    let plain = tree.nodes.iter().find(|n| n.kind == "plaincue").unwrap_or_else(|| panic!("{ids:?}"));
    assert!(plain.on && plain.note.contains("Dimmer"), "{plain:?}");
    assert_eq!(a.view().selectors[0].value, "OnlyExisting");
    assert_eq!(a.view().selectors[0].choices, ["OnlyExisting", "AddNew", "Overwrite"]);
    let line = a.line_of("Update").unwrap();
    assert_eq!(line, "Update `Update Preset 3 /Merge; Update Sequence 1 Cue 3 /Original /Shield`");
    let details: Vec<&str> = a.view().details.iter().map(|d| d.k.as_str()).collect();
    assert_eq!(details, ["Update Preset 3 /Merge", "Update Sequence 1 Cue 3 /Original /Shield"]);
    // Preview in Blind is planned: greyed with the reason.
    assert!(a.view().buttons.iter().any(|b| b.label == "Preview in Blind" && b.disabled.is_some()));
}

#[test]
fn the_whole_update_is_one_oops_step_and_an_unticked_cue_keeps_its_look() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Fixture 101 Thru 110");
    run(&mut s, "At Preset 2");
    run(&mut s, "At 75");
    let a = ask(&mut s, "Update");
    let tree = a.view().tree.clone().unwrap();
    let cue2 = tree.nodes.iter().find(|n| n.kind == "cue" && n.no == "Cue 2").expect("cue 2 follows Preset 2").id.clone();
    // Untick cue 2: it keeps its old look (an Extract first).
    let mut answer = lumacore_wire::DialogAnswer::default();
    answer.toggles.insert(cue2, false);
    answer.selectors.insert("content".into(), "OnlyExisting".into());
    let i = a.view().buttons.iter().position(|b| b.label == "Update").unwrap();
    let line = a.line_for(i, &answer).unwrap().unwrap();
    assert_eq!(line, "Update `Extract Preset 2 Sequence 1 Cue 2; Update Preset 2 /Merge`");
    let before = steps(&s);
    let out = run(&mut s, &line);
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(steps(&s), before + 1, "one Oops step for the whole update");
    assert!(matches!(dimmer_in(&s, 1, 2000, 105), Some(CueData::Value(v)) if v.percent() == Some(50.0)), "the old look, plain");
    assert_eq!(super::super::in_pool(&s).unwrap()[&2].value_for(guid(&s, 105), "Dimmer").and_then(|v| v.percent()), Some(75.0));
    run(&mut s, "Oops");
    assert_eq!(dimmer_in(&s, 1, 2000, 105), Some(CueData::Link(preset_guid(&s, 2))), "Oops brings the link back");
    assert_eq!(super::super::in_pool(&s).unwrap()[&2].value_for(guid(&s, 105), "Dimmer").and_then(|v| v.percent()), Some(50.0));
    // The ask's choices are kept on the profile: Add next time.
    run(&mut s, "Update `Update Preset 2 /Add`");
    assert_eq!(s.root.show_data.profile_for_user("Admin").unwrap().settings.preferences.update.content, "AddNew");
    // Only the update words run inside.
    assert!(run(&mut s, "Update `Delete Preset 2`").contains("E-CMD-008"));
}

#[test]
fn update_a_cue_where_it_was_stored_or_here_and_shield_the_next() {
    let mut s = crate::demo::fresh_state();
    // Sequence 901, past the rig: 105 stored in cue 1, tracking through cues 2 and 3.
    run(&mut s, "Fixture 105 At 10");
    run(&mut s, "Store Sequence 901 Cue 1");
    run(&mut s, "ResetAll");
    run(&mut s, "Fixture 106 At 10");
    run(&mut s, "Store Sequence 901 Cue 2");
    run(&mut s, "Store Sequence 901 Cue 3");
    run(&mut s, "ResetAll");
    run(&mut s, "Goto Sequence 901 Cue 2");
    run(&mut s, "Fixture 105 At 60");
    let out = run(&mut s, "Update Sequence 901 Cue 2 /Original");
    assert!(out.contains("in Cue 1"), "{out}");
    assert!(matches!(dimmer_in(&s, 901, 1000, 105), Some(CueData::Value(v)) if v.percent() == Some(60.0)));
    assert!(dimmer_in(&s, 901, 2000, 105).is_none());
    run(&mut s, "Fixture 105 At 70");
    run(&mut s, "Update Sequence 901 Cue 2 /Active /Shield");
    assert!(matches!(dimmer_in(&s, 901, 2000, 105), Some(CueData::Value(v)) if v.percent() == Some(70.0)), "into this cue");
    assert!(matches!(dimmer_in(&s, 901, 3000, 105), Some(CueData::Value(v)) if v.percent() == Some(60.0)), "the next cue keeps its look");
    assert!(run(&mut s, "ResetAll; Update Sequence 901 Cue 2").contains("E-CMD-009"), "nothing changed");
}

#[test]
fn recast_adds_and_drops_links_and_asks_first() {
    let mut s = crate::demo::fresh_state();
    let other = other_attr(&s);
    let p3 = preset_guid(&s, 3);
    run(&mut s, "Fixture 101 + 102");
    run(&mut s, "At Preset 3");
    run(&mut s, &format!("Attribute \"{other}\" At 40"));
    run(&mut s, "Update Preset 3 /Add");
    run(&mut s, "ResetAll");
    let a = ask(&mut s, "Recast Preset 3");
    assert!(a.view().meta.as_deref().is_some_and(|m| m.contains(&format!("{other} added"))), "{:?}", a.view().meta);
    assert!(a.view().rows.iter().any(|r| r.k.starts_with("Sequence 1 Cue 3") && r.v.contains(&format!("+ {other}"))), "{:?}", a.view().rows);
    let out = run(&mut s, &a.line_of("Recast 1 cue").expect("the button"));
    assert!(out.contains("1 cue changed"), "{out}");
    let g = guid(&s, 101);
    assert!(cue(&s, 1, 3000).values().any(|v| v.fixture_id == g && v.attribute == other && v.value == CueData::Link(p3)));
    // Take Dimmer out of Preset 3: a Recast drops those links.
    run(&mut s, "Fixture 101 At 1");
    run(&mut s, "Store Preset 3 /Remove");
    run(&mut s, "ResetAll");
    run(&mut s, "Recast Preset 3 Sequence 1 /NoConfirmation");
    assert!(dimmer_in(&s, 1, 3000, 101).is_none(), "the Dimmer link went");
    assert!(run(&mut s, "Recast Preset 3").contains("already match"));
}

#[test]
fn extract_turns_links_into_the_presets_values() {
    let mut s = crate::demo::fresh_state();
    let out = run(&mut s, "Extract Preset 3 Sequence 1 Cue 3");
    assert!(out.contains("2 values"), "{out}");
    assert!(matches!(dimmer_in(&s, 1, 3000, 102), Some(CueData::Value(v)) if v.percent() == Some(80.0)));
    run(&mut s, "Extract Sequence 1");
    assert!(cue(&s, 1, 1000).values().all(|v| v.value.link().is_none()), "every link in the sequence");
    // In the programmer.
    run(&mut s, "Fixture 101 At Preset 1");
    assert!(run(&mut s, "Extract Preset 1").contains("1 value"));
    let pv = s.current_programmer().unwrap().get(guid(&s, 101), "Dimmer").cloned().unwrap();
    assert_eq!((pv.value.preset, pv.value.percent()), (None, Some(100.0)));
    assert!(run(&mut s, "Extract Preset 1").contains("E-CMD-009"));
}

#[test]
fn deleting_a_used_preset_keeps_the_values_and_oops_brings_the_links_back() {
    let mut s = crate::demo::fresh_state();
    let p2 = preset_guid(&s, 2);
    let a = ask(&mut s, "Delete Preset 2");
    assert_eq!(a.title(), "Delete Preset 2 \"Half\"");
    assert_eq!(a.view().meta.as_deref(), Some("1 cue uses it"));
    assert!(a.view().details.iter().any(|d| d.k == "Sequence 1 Cue 2 \"Half\""), "{:?}", a.view().details);
    let before = steps(&s);
    let out = run(&mut s, &a.line_of("Delete, keep the values").unwrap());
    assert!(out.contains("deleted"), "{out}");
    assert!(super::super::in_pool(&s).unwrap().get(&2).is_none());
    assert!(matches!(dimmer_in(&s, 1, 2000, 105), Some(CueData::Value(v)) if v.percent() == Some(50.0)), "the values stay");
    assert_eq!(steps(&s), before + 1, "one step");
    run(&mut s, "Oops");
    assert_eq!(preset_guid(&s, 2), p2, "the preset is back");
    assert_eq!(dimmer_in(&s, 1, 2000, 105), Some(CueData::Link(p2)), "and every link");
    // A preset no cue uses asks the door's plain question.
    run(&mut s, "Fixture 101 At 33");
    run(&mut s, "Store Preset 901");
    assert_eq!(ask(&mut s, "Delete Preset 901").title(), "Delete");
}

#[test]
fn the_pool_counts_the_cues_that_follow_a_preset() {
    let s = crate::demo::fresh_state();
    let uses = lumacore_tree::PresetUses::build(&s.root.show_data);
    assert_eq!(uses.count(preset_guid(&s, 1)), 1);
    assert_eq!(uses.count(preset_guid(&s, 3)), 1);
    assert_eq!(uses.cues(preset_guid(&s, 3))[0].attributes, vec!["Dimmer".to_string()]);
}
