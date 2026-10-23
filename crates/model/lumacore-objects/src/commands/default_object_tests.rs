//! The default object (decisions §33 21, 25, 27, 39; issue 02): a counted keyword alone sets it,
//! a bare number or name names it, written out before the line runs — so it takes the change
//! path like any full line.

use super::dispatch;
use super::dispatch::write_out_bare;
use crate::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn demo() -> AppState {
    crate::demo::fresh_state()
}

fn selection(state: &AppState) -> Vec<u32> {
    let guids = state.current_programmer().unwrap().selected_fixtures.clone();
    state.root.show_data.fixture_ids_for_guids(&guids)
}

fn undo_depth(state: &AppState) -> usize {
    state.undo_stack("Admin").map_or(0, |u| u.len())
}

#[test]
fn a_keyword_alone_sets_the_default_object_and_files_no_oops_step() {
    let mut state = demo();
    assert_eq!(crate::objects::default_object(&state).keyword, "Fixture");
    let depth = undo_depth(&state);
    let out = run(&mut state, "Group");
    assert!(out.contains("Group>"), "{out}");
    assert_eq!(crate::objects::default_object(&state).keyword, "Group");
    assert_eq!(state.root.show_data.user("Admin").unwrap().default_object.as_deref(), Some("Group"));
    assert_eq!(undo_depth(&state), depth, "never an Oops step");
    run(&mut state, "Fixture");
    assert_eq!(state.root.show_data.user("Admin").unwrap().default_object, None, "Fixture is the default");
}

#[test]
fn a_bare_number_or_name_names_the_default_object() {
    let mut state = demo();
    run(&mut state, "Group");
    run(&mut state, "2");
    let odd = selection(&state);
    assert!(!odd.is_empty(), "Group 2 selected its fixtures");
    run(&mut state, "Fixture");
    assert_eq!(write_out_bare(&state, "\"A3\"").as_deref(), Some("Fixture \"A3\""));
    run(&mut state, "\"A3\"");
    // A3 is one of the rig's fixtures (a Deco LED, decisions §40 14).
    let a3 = state.root.show_data.devices.dmx.patch.fixtures.values().find(|f| f.meta.name() == Some("A3")).and_then(|f| f.mvr.fid).expect("A3");
    assert!(selection(&state).contains(&a3), "a bare quoted name names one of it (decision 39)");
}

/// Issue 02: `101 At 50` took no change path (no Oops step). Written out first, it does.
#[test]
fn a_bare_number_takes_the_change_path() {
    let mut state = demo();
    let depth = undo_depth(&state);
    run(&mut state, "101 At 50");
    assert!(undo_depth(&state) > depth, "an Oops step, like Fixture 101 At 50");
}

#[test]
fn the_line_is_written_out_by_the_verbs_object_then_yours() {
    let mut state = demo();
    let w = |s: &AppState, l: &str| write_out_bare(s, l);
    assert_eq!(w(&state, "3 At 50").as_deref(), Some("Fixture 3 At 50"));
    assert_eq!(w(&state, "Store 2").as_deref(), Some("Store Cue 2"), "the verb's own default object first (25)");
    assert_eq!(w(&state, "Go 3").as_deref(), Some("Go Sequence 3"));
    assert_eq!(w(&state, "Goto 3").as_deref(), Some("Goto Cue 3"));
    assert_eq!(w(&state, "Delete 3").as_deref(), Some("Delete Fixture 3"));
    run(&mut state, "Group");
    assert_eq!(w(&state, "3 At 50").as_deref(), Some("Group 3 At 50"));
    assert_eq!(w(&state, "Delete 3").as_deref(), Some("Delete Group 3"));
    assert_eq!(w(&state, "Store 2").as_deref(), Some("Store Cue 2"));
    // Not ids: left alone.
    for line in ["Oops 3", "Redo 2", "At 50", "Join \"Festival\"", "Group 3", "Store Group 3", "SelectFixtures"] {
        assert_eq!(w(&state, line), None, "{line}");
    }
}

