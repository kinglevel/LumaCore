//! Step 11c (decisions §29 "11c hardening"): one Oops list per user with
//! category labels, the selection undone apart from the programmer (as
//! consoles do), `/NoOops`, the profile's create filters and Oops Confirmation,
//! station steps, UI steps, watches, and a show load clearing the lists.

use super::coverage_tests::{run, with_fixture};
use super::dispatch;
use crate::AppState;
use crate::DispatchResult;
use lumacore_tree::OopsCategory;

fn steps(state: &AppState) -> Vec<(OopsCategory, String, bool)> {
    state
        .undo_stack("Admin")
        .map(|u| {
            u.frames
                .iter()
                .map(|f| (f.category, f.description.clone(), f.main))
                .collect()
        })
        .unwrap_or_default()
}

fn newest(state: &AppState) -> (OopsCategory, String, bool) {
    steps(state).pop().expect("a step")
}

fn selected(state: &AppState) -> usize {
    state.current_programmer().unwrap().selected_fixtures.len()
}

fn values(state: &AppState) -> usize {
    state.current_programmer().unwrap().values.len()
}

#[test]
fn every_step_carries_its_category() {
    let mut state = with_fixture();
    run(&mut state, "Fixture 1");
    assert_eq!(newest(&state).0, OopsCategory::Selection);
    run(&mut state, "At 50");
    assert_eq!(newest(&state).0, OopsCategory::Programmer);
    run(&mut state, "Store Group 1");
    assert_eq!(newest(&state).0, OopsCategory::Show);
    assert!(newest(&state).2, "storing is a Main action");
    run(&mut state, "Set Fixture 1 \"Name\" \"Spot\"");
    assert_eq!(newest(&state).0, OopsCategory::Patch, "Set says what it changed");
    run(&mut state, "Set UserProfile \"MeowKey\" Ctrl");
    assert_eq!(newest(&state).0, OopsCategory::Settings);
    run(&mut state, "ResetAll");
    assert_eq!(newest(&state).0, OopsCategory::Programmer, "Reset* are steps now");

    // A label only: Oops takes the newest, whatever it is.
    run(&mut state, "Oops");
    assert!(values(&state) > 0, "ResetAll undone first");
    let list = run(&mut state, "List Undo /Patch");
    assert!(list.contains("Set Fixture 1 \"Name\"") && !list.contains("Store Group"), "{list}");
    assert!(run(&mut state, "List Undo /Nope").contains("E-CMD-018"), "an undeclared flag (decisions §31)");
}

#[test]
fn the_selection_is_undone_apart_from_the_programmer() {
    let mut state = with_fixture();
    run(&mut state, "Fixture 1");
    run(&mut state, "At 50");
    // Not filed: the selection changes after the value.
    run(&mut state, "ResetSelection /NoOops");
    assert_eq!(selected(&state), 0);
    // Oops undoes the value and leaves the selection as it is now.
    run(&mut state, "Oops");
    assert_eq!(values(&state), 0);
    assert_eq!(selected(&state), 0, "the unfiled selection change stays");
    // The next Oops is the selection step: back to before `Fixture 1`.
    run(&mut state, "Oops");
    assert_eq!(selected(&state), 0);
    assert!(steps(&state).is_empty());
}

#[test]
fn no_oops_and_the_create_filters_file_no_step() {
    let mut state = with_fixture();
    run(&mut state, "Fixture 1 At 50 /NoOops");
    assert!(values(&state) > 0, "the line ran");
    assert!(steps(&state).is_empty(), "/NoOops files nothing");
    run(&mut state, "Fixture 1 At 60 /NU");
    assert!(steps(&state).is_empty(), "/NU too");

    run(&mut state, "Set UserProfile \"OopsProgrammer\" No");
    let before = steps(&state).len();
    run(&mut state, "Fixture 1");
    assert_eq!(steps(&state).len(), before + 1, "selection steps still file");
    run(&mut state, "At 70");
    assert_eq!(steps(&state).len(), before + 1, "programmer steps are off");

    run(&mut state, "Set UserProfile \"CreateOops\" No");
    let before = steps(&state).len();
    run(&mut state, "Fixture 1");
    run(&mut state, "Store Group 2");
    assert_eq!(steps(&state).len(), before, "CreateOops off: nothing files");
}

#[test]
fn oops_confirmation_asks_for_main_steps_or_always() {
    let mut state = with_fixture();
    run(&mut state, "Set UserProfile \"OopsConfirmation\" Main");
    run(&mut state, "Fixture 1 At 50");
    assert!(run(&mut state, "Oops").contains("Undone"), "a programmer step doesn't ask");
    run(&mut state, "Fixture 1");
    run(&mut state, "Store Group 3");
    let asked = dispatch(&mut state, "Oops");
    let DispatchResult::Ask(ask) = asked else {
        panic!("a Main step asks first");
    };
    assert!(state.root.show_data.datapools.values().any(|d| d.groups.contains_key(&3)));
    let done = dispatch(&mut state, &ask.primary_line().unwrap());
    assert!(matches!(done, DispatchResult::Output(ref s) if s.contains("Undone")));
    assert!(!state.root.show_data.datapools.values().any(|d| d.groups.contains_key(&3)));

    run(&mut state, "Set UserProfile \"OopsConfirmation\" Always");
    run(&mut state, "At 40");
    assert!(matches!(dispatch(&mut state, "Oops"), DispatchResult::Ask(_)));
    assert!(run(&mut state, "Oops /NoConfirmation").contains("Undone"));
}

#[test]
fn a_layout_change_is_one_ui_step_and_a_tab_switch_none() {
    let mut state = AppState::new();
    let no = state
        .root
        .runtime
        .live_screens
        .write()
        .unwrap()
        .open_screen("Admin", 1, 1, None);
    // A new window: one UI step, a Main action (layout).
    state
        .root
        .runtime
        .live_screens
        .write()
        .unwrap()
        .add_window(no, "dmx", None, None)
        .unwrap();
    crate::commands::screens::sync(&mut state);
    let (category, _, main) = newest(&state);
    assert_eq!((category, main), (OopsCategory::UI, true));
    let count = steps(&state).len();

    // Showing another tab of a pane is not a step.
    {
        let mut store = state.root.runtime.live_screens.write().unwrap();
        let screen = store.get(no).unwrap().clone();
        let mut root = screen.root.clone();
        fn next_tab(node: &mut lumacore_tree::runtime::LayoutNode) -> bool {
            match node {
                lumacore_tree::runtime::LayoutNode::Leaf { windows, active } if windows.len() > 1 => {
                    *active = (*active + 1) % windows.len();
                    true
                }
                lumacore_tree::runtime::LayoutNode::Split { children, .. } => children.iter_mut().any(next_tab),
                _ => false,
            }
        }
        assert!(next_tab(&mut root), "a pane with two tabs");
        store.replace_layout(no, root, screen.windows).unwrap();
    }
    crate::commands::screens::sync(&mut state);
    assert_eq!(steps(&state).len(), count, "a tab switch files nothing");

    // Oops takes the window away again, live too.
    run(&mut state, "Oops /NoConfirmation");
    let windows = state.root.runtime.live_screens.read().unwrap().get(no).unwrap().windows.len();
    assert!(windows < 3, "the live screen follows the undone layout ({windows})");
    assert_eq!(steps(&state).len(), count - 1);
}

#[test]
fn loading_a_show_clears_every_oops_list() {
    let mut state = with_fixture();
    run(&mut state, "Fixture 1 At 50");
    assert!(!steps(&state).is_empty());
    let show = state.root.show_data.clone();
    crate::objects::replace_show(&mut state, show);
    assert!(steps(&state).is_empty());
}

fn fixture_name(state: &AppState) -> Option<String> {
    state.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").meta.identity.name.clone()
}

/// R1-63: Redo is refused like Oops when another user changed one of its
/// subtrees since; their change stays.
#[test]
fn redo_is_blocked_when_another_user_changed_the_subtree() {
    let mut state = with_fixture();
    run(&mut state, "Set Fixture 1 \"Name\" \"Mine\"");
    run(&mut state, "Oops");
    assert_eq!(fixture_name(&state), None);
    state.user = "Debug".into();
    state.root.runtime.user("Debug");
    let out = run(&mut state, "Set Fixture 1 \"Name\" \"Theirs\"");
    assert_eq!(fixture_name(&state).as_deref(), Some("Theirs"), "{out}");
    state.user = "Admin".into();
    let out = run(&mut state, "Redo");
    assert!(out.contains("Redo blocked") && out.contains("E-CMD-010"), "{out}");
    assert_eq!(fixture_name(&state).as_deref(), Some("Theirs"), "their change stays");
    assert_eq!(state.undo_stack("Admin").unwrap().redo_frames.len(), 1, "the redo step stays");
}

/// R1-64: Oops is all or nothing — a subtree that can't go back (its slot
/// taken by another object) leaves the show and the Oops list as they were.
#[test]
fn an_oops_that_cant_finish_changes_nothing() {
    use lumacore_tree::ShowSubtree;
    let mut state = with_fixture();
    for n in [2, 3] {
        let pool = lumacore_tree::Datapool::new(Some(format!("Pool {n}")));
        state.root.show_data.datapools.insert(n, pool);
    }
    state.root.show_data.after_load();
    let g3 = state.root.show_data.datapools[&3].meta.identity.guid;
    // A's step: select DataPool 3, then (as one step) DataPool 3 deleted.
    run(&mut state, "Select DataPool 3");
    let pool3 = super::oops::snapshot_subtree(&state, &ShowSubtree::DataPool(g3)).unwrap();
    {
        let frame = state.undo_stack_mut("Admin").frames.back_mut().unwrap();
        assert!(matches!(frame.snapshots[0].subtree, ShowSubtree::Selection(_)));
        frame.snapshots.push(lumacore_tree::SubtreeSnapshot {
            subtree: ShowSubtree::DataPool(g3),
            data: pool3,
        });
    }
    state.root.show_data.datapools.remove(&3);
    // B stores a new DataPool 3 (another GUID) in the freed slot.
    let theirs = lumacore_tree::Datapool::new(Some("Theirs".into()));
    state.root.show_data.datapools.insert(3, theirs);
    state.root.show_data.after_load();
    // The selection of the gone DataPool 3 is cleared as after any delete.
    state.clear_dangling_selections();

    let show_before = serde_json::to_string(&state.root.show_data).unwrap();
    let frames_before = state.undo_stack("Admin").unwrap().frames.len();
    let out = run(&mut state, "Oops");
    assert!(out.contains("E-CMD-010"), "{out}");
    assert_eq!(serde_json::to_string(&state.root.show_data).unwrap(), show_before, "nothing changed");
    let stack = state.undo_stack("Admin").unwrap();
    assert_eq!(stack.frames.len(), frames_before, "the step stays");
    assert!(stack.redo_frames.is_empty());
}

/// R1-62: Oops Confirmation checks every statement of a line.
#[test]
fn oops_confirmation_checks_every_statement() {
    let mut state = with_fixture();
    run(&mut state, "Set UserProfile \"OopsConfirmation\" Main");
    run(&mut state, "Fixture 1");
    run(&mut state, "Store Group 3");
    let asked = dispatch(&mut state, "List Undo; Oops");
    let DispatchResult::Ask(ask) = asked else {
        panic!("an Oops later in the line asks too");
    };
    let line = ask.primary_line().unwrap();
    // Every statement is its own step: List Undo ran, the Oops asks alone.
    assert_eq!(line, "Oops 1 /NoConfirmation");
    assert!(state.root.show_data.datapools.values().any(|d| d.groups.contains_key(&3)));
    dispatch(&mut state, &line);
    assert!(!state.root.show_data.datapools.values().any(|d| d.groups.contains_key(&3)));
}

