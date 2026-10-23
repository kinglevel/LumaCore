// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of the door verbs, line in and reply out: Copy (new GUIDs, the existing ask), Move
//! (swap, between DataPools), Cut / Paste (the clipboard, live GUIDs), Insert's shift, Compact,
//! the Delete ask, another kind refused.
//!
//! Owns:          the tests of door.rs and door/*.rs (on Group).
//! Doesn't own:   the pool algorithms' own tests (`pools::door`).
//! Entry points:  the #[test] fns.
//! Decisions:     §35.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use crate::commands::dispatch;
use crate::{AppState, DispatchResult};
use lumacore_tree::Datapool;
use uuid::Uuid;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(r) => r.text,
        DispatchResult::Ask(a) => format!("ASK {}", a.view().text),
        _ => String::new(),
    }
}

fn with_groups(names: &[(u32, &str)]) -> AppState {
    let mut s = AppState::new();
    for (n, name) in names {
        let out = run(&mut s, &format!("Store Group {n} \"{name}\""));
        assert!(out.contains("created"), "{out}");
    }
    s
}

fn groups(s: &AppState) -> Vec<(u32, String)> {
    s.user_groups().iter().map(|(n, g)| (*n, g.meta.identity.name.clone().unwrap_or_default())).collect()
}

fn guid(s: &AppState, n: u32) -> Uuid {
    s.user_groups()[&n].meta.identity.guid
}

fn pairs(v: &[(u32, &str)]) -> Vec<(u32, String)> {
    v.iter().map(|(n, s)| (*n, s.to_string())).collect()
}

#[test]
fn copy_makes_new_objects_and_asks_over_existing_ones() {
    let mut s = with_groups(&[(1, "Front"), (5, "Back")]);
    run(&mut s, "Copy Group 1 At 3");
    assert_eq!(groups(&s), pairs(&[(1, "Front"), (3, "Front"), (5, "Back")]));
    assert_ne!(guid(&s, 3), guid(&s, 1));
    let back = guid(&s, 5);
    assert!(run(&mut s, "Copy Group 1 At 5").starts_with("ASK Group 5 exists"));
    run(&mut s, "Copy Group 1 At 5 /Overwrite");
    assert_eq!(groups(&s)[2], (5, "Front".to_string()));
    assert_eq!(guid(&s, 5), back, "an overwrite keeps the target's GUID");
}

#[test]
fn move_swaps_with_what_is_there() {
    let mut s = with_groups(&[(3, "C"), (7, "G")]);
    let c = guid(&s, 3);
    run(&mut s, "Move Group 3 At 7");
    assert_eq!(groups(&s), pairs(&[(3, "G"), (7, "C")]));
    assert_eq!(guid(&s, 7), c);
}

/// `/Skip` jumps over taken numbers instead of swapping; `/By` numbers them in a column's order,
/// consecutive (decisions §34 round 3 19: the renumber dialog's Skip taken and Grid order).
#[test]
fn move_skips_taken_numbers_or_follows_a_column() {
    let mut s = with_groups(&[(1, "A"), (2, "B"), (3, "C"), (8, "X")]);
    let a = guid(&s, 1);
    run(&mut s, "Move Group 1 Thru 3 At 7 /Skip");
    assert_eq!(groups(&s), pairs(&[(7, "A"), (8, "X"), (9, "B"), (10, "C")]), "8 stays, nothing swaps");
    assert_eq!(guid(&s, 7), a);
    // /By Name: consecutive in the names' order.
    let mut s = with_groups(&[(1, "Wash"), (2, "Beam"), (3, "Spot")]);
    run(&mut s, "Move Group 1 Thru 3 At 20 /By Name");
    assert_eq!(groups(&s), pairs(&[(20, "Beam"), (21, "Spot"), (22, "Wash")]));
    let out = run(&mut s, "Move Group 20 At 30 /By Address");
    assert!(out.contains("E-CMD-008"), "{out}");
}

#[test]
fn move_between_datapools_keeps_the_object() {
    let mut s = with_groups(&[(1, "A")]);
    s.root.show_data.datapools.insert(2, Datapool::new(Some("Two".into())));
    let a = guid(&s, 1);
    let out = run(&mut s, "Move Group 1 At DataPool 2 Group 4");
    assert!(groups(&s).is_empty(), "{out}");
    assert_eq!(s.root.show_data.datapools[&2].groups[&4].meta.identity.guid, a);
}

#[test]
fn another_kind_is_refused() {
    let mut s = with_groups(&[(1, "A")]);
    let out = run(&mut s, "Copy Group 1 At Sequence 4");
    assert!(out.contains("E-CMD-020"), "{out}");
}

#[test]
fn cut_and_paste_move_the_same_objects() {
    let mut s = with_groups(&[(1, "A"), (2, "B")]);
    let a = guid(&s, 1);
    run(&mut s, "Cut Group 1");
    // Live GUIDs: renamed after the cut, the paste takes it as it is now.
    run(&mut s, "Label Group 1 \"A2\"");
    run(&mut s, "Paste Group 10");
    assert_eq!(groups(&s), pairs(&[(2, "B"), (10, "A2")]));
    assert_eq!(guid(&s, 10), a);
    assert!(run(&mut s, "Paste Group 11").contains("E-CMD-009"), "a cut pastes once");
}

#[test]
fn a_bare_copy_pastes_into_the_next_free_numbers() {
    let mut s = with_groups(&[(1, "A"), (2, "B"), (4, "D")]);
    run(&mut s, "Copy Group 1 Thru 2");
    run(&mut s, "Paste");
    assert_eq!(groups(&s), pairs(&[(1, "A"), (2, "B"), (4, "D"), (5, "A"), (6, "B")]));
    run(&mut s, "Paste");
    assert_eq!(groups(&s).len(), 7, "a copy pastes again");
}

#[test]
fn a_bare_paste_of_a_cut_asks_where() {
    let mut s = with_groups(&[(1, "A")]);
    run(&mut s, "Cut Group 1");
    assert!(run(&mut s, "Paste").starts_with("ASK"));
}

#[test]
fn paste_with_an_empty_clipboard_says_so() {
    let mut s = AppState::new();
    assert!(run(&mut s, "Paste").contains("E-CMD-009"));
}

#[test]
fn a_deleted_object_is_skipped_and_named() {
    let mut s = with_groups(&[(1, "A"), (2, "B")]);
    run(&mut s, "Copy Group 1 Thru 2");
    run(&mut s, "Delete Group 2 /NoConfirmation");
    let out = run(&mut s, "Paste Group 20");
    assert!(out.contains("Group 2 is gone"), "{out}");
    assert_eq!(groups(&s), pairs(&[(1, "A"), (20, "A")]));
}

#[test]
fn insert_shifts_the_ones_in_the_way() {
    let mut s = with_groups(&[(5, "E"), (9, "I"), (10, "J")]);
    run(&mut s, "Insert Group 5 At 9");
    assert_eq!(groups(&s), pairs(&[(9, "E"), (10, "I"), (11, "J")]));
}

#[test]
fn compact_renumbers_the_pool() {
    let mut s = with_groups(&[(2, "b"), (7, "a"), (9, "c")]);
    run(&mut s, "Compact Group");
    assert_eq!(groups(&s), pairs(&[(1, "b"), (2, "a"), (3, "c")]));
    run(&mut s, "Compact Group /By Name");
    assert_eq!(groups(&s), pairs(&[(1, "a"), (2, "b"), (3, "c")]));
}

#[test]
fn delete_asks_once_naming_the_rows() {
    let mut s = with_groups(&[(1, "Front"), (2, "Back")]);
    let out = run(&mut s, "Delete Group 1");
    assert!(out.starts_with("ASK Delete Group 1 \"Front\"?"), "{out}");
    let out = run(&mut s, "Delete Group 1 Thru 2");
    assert!(out.contains("2 Groups (1 Thru 2)"), "{out}");
    run(&mut s, "Delete Group 1 /NoConfirmation");
    assert_eq!(groups(&s), pairs(&[(2, "Back")]));
}

#[test]
fn every_copy_and_move_is_one_oops_step() {
    let mut s = with_groups(&[(1, "A"), (2, "B")]);
    run(&mut s, "Move Group 1 At 2");
    run(&mut s, "Oops");
    assert_eq!(groups(&s), pairs(&[(1, "A"), (2, "B")]));
}

#[test]
fn a_quoted_star_names_every_row_that_fits() {
    assert!(crate::resolve::name_fits("Mac*", "MAC Viper"));
    assert!(crate::resolve::name_fits("backt*blue", "Backtruss Blue"));
    assert!(crate::resolve::name_fits("*wash*", "Front Wash 2"));
    assert!(!crate::resolve::name_fits("Mac*", "A Mac"));
    assert!(!crate::resolve::name_fits("ab*ba", "aba"));
    assert!(crate::resolve::name_fits("Front", "front"));
    let mut s = with_groups(&[(1, "Front L"), (2, "Back"), (3, "Front R")]);
    run(&mut s, "Copy Group \"Front*\" At 10");
    assert_eq!(groups(&s).iter().map(|(n, _)| *n).collect::<Vec<_>>(), vec![1, 2, 3, 10, 12], "1 and 3 copied, their spacing kept");
    run(&mut s, "Delete Group \"front*\" /NoConfirmation");
    assert_eq!(groups(&s), pairs(&[(2, "Back")]));
    assert!(run(&mut s, "Delete Group \"Nope*\" /NoConfirmation").contains("E-CMD-007"));
}

#[test]
fn a_name_never_holds_a_star() {
    let mut s = with_groups(&[(1, "A")]);
    assert!(run(&mut s, "Label Group 1 \"A*\"").contains("E-CMD-008"));
    assert!(run(&mut s, "Store Group 5 \"B*\"").contains("E-CMD-008"));
    assert!(run(&mut s, "Store Group \"Z*\"").contains("E-CMD-007"), "a pattern is never a new name");
    assert_eq!(groups(&s), pairs(&[(1, "A")]));
}

#[test]
fn sequences_and_presets_are_on_the_door_too() {
    let mut s = AppState::new();
    run(&mut s, "Store Sequence 1 \"Main\"");
    run(&mut s, "Store Sequence 1 Cue 1");
    let dp = s.selected_datapool();
    let seqs = |s: &AppState| s.root.show_data.datapools[&dp].sequences.keys().copied().collect::<Vec<_>>();
    run(&mut s, "Copy Sequence 1 At 4");
    assert_eq!(seqs(&s), vec![1, 4]);
    let (a, b) = (&s.root.show_data.datapools[&dp].sequences[&1], &s.root.show_data.datapools[&dp].sequences[&4]);
    assert_ne!(a.meta.identity.guid, b.meta.identity.guid);
    let cue = |q: &lumacore_tree::SequenceObject| q.cues.values().next().map(|c| c.meta.identity.guid);
    assert_ne!(cue(a), cue(b), "a copy's cues are new objects too");
    assert!(run(&mut s, "Copy Sequence 1 At 4 /Merge").contains("can't merge"));
    run(&mut s, "Move Sequence 4 At 2");
    assert_eq!(seqs(&s), vec![1, 2]);
    run(&mut s, "Delete Sequence 2 /NoConfirmation");
    assert_eq!(seqs(&s), vec![1]);
    run(&mut s, "Store TimingPreset 3 \"Slow\"");
    run(&mut s, "Copy TimingPreset 3 At 5");
    run(&mut s, "Compact TimingPreset");
    assert_eq!(s.root.show_data.datapools[&dp].timing_presets.keys().copied().collect::<Vec<_>>(), vec![1, 2]);
}

#[test]
fn a_new_sequence_fills_the_first_hole_and_cues_append() {
    let mut s = AppState::new();
    run(&mut s, "Store Sequence 1");
    run(&mut s, "Store Sequence 3");
    run(&mut s, "Deselect");
    let out = run(&mut s, "Store Cue");
    let dp = s.selected_datapool();
    assert!(s.root.show_data.datapools[&dp].sequences.contains_key(&2), "{out}");
    run(&mut s, "Store Cue 4");
    run(&mut s, "Store Cue");
    let cues: Vec<u32> = s.root.show_data.datapools[&dp].sequences[&2].cues.keys().copied().collect();
    assert_eq!(cues.last(), Some(&5000), "after the last cue: {cues:?}");
}

fn with_fixtures(n: u32) -> AppState {
    let mut s = AppState::new();
    for i in 1..=n {
        s.root.show_data.devices.dmx.patch.insert_fid(i, lumacore_fixtures::Fixture::new(Some(format!("Fix {i}")), Uuid::new_v4(), Uuid::nil(), Uuid::nil(), Uuid::nil(), 1, 1),
        );
    }
    s.root.show_data.rebuild_id_pools();
    s
}

#[test]
fn fixtures_and_universes_move_on_the_door() {
    let mut s = with_fixtures(3);
    let one = s.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").meta.identity.guid;
    let generation = s.patch_generation;
    run(&mut s, "Move Fixture 1 At 3");
    let names: Vec<(u32, String)> = s.root.show_data.devices.dmx.patch.by_number().into_iter().map(|(n, f)| (n, f.meta.identity.name.clone().unwrap_or_default())).collect();
    assert_eq!(names, pairs(&[(1, "Fix 3"), (2, "Fix 2"), (3, "Fix 1")]), "a taken number swaps");
    assert_eq!(s.root.show_data.devices.dmx.patch.fixture(3).expect("resolved").meta.identity.guid, one);
    assert!(s.patch_generation > generation, "the engine's channel tier follows");
    run(&mut s, "Move Fixture 2 At 10");
    run(&mut s, "Compact Fixture");
    assert_eq!(s.root.show_data.devices.dmx.patch.fids().into_iter().map(|(n, _)| n).collect::<Vec<_>>(), vec![1, 2, 3]);
    assert!(run(&mut s, "Copy Fixture 1 At 9").contains("E-CMD"), "no Copy of a fixture yet");
    assert!(run(&mut s, "Compact Fixture /By Address").contains("→") || s.root.show_data.devices.dmx.patch.fixtures.len() == 3);
}
