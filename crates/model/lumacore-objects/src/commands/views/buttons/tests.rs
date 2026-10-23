// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of the view bar's buttons on the rig: any object on a button, per screen, the tap as
//! the object's own line, an empty tap that only says so, Store on an empty button.
//!
//! Owns:          the unit tests of buttons.rs.
//! Doesn't own:   the view store / recall tests (view_ask_tests), the feed (lumacore-views).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0287-view-bar-any-object-pool-windows.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use crate::commands::dispatch;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        DispatchResult::Ask(a) => format!("ASK {}", a.title()),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// The rig with two of Admin's screens, the line on the first; returns the live screens.
fn rig() -> (AppState, u32, u32) {
    let mut state = crate::demo::fresh_state();
    let (a, b) = {
        let mut store = state.root.runtime.live_screens.write().unwrap();
        (store.join_or_create("Admin", None), store.join_or_create("Admin", None))
    };
    state.current_live_screen = Some(a);
    (state, a, b)
}

/// The rig's first group, and a sequence stored past the rig, in the user's DataPool.
fn first_of(state: &mut AppState) -> (u32, u32) {
    let dp = crate::pools::user_datapool(&state.root.show_data, "Admin");
    let out = run(state, "Store Sequence 901 \"Busk\"");
    let pool = &state.root.show_data.datapools[&dp];
    assert!(pool.sequences.contains_key(&901), "{out}");
    (*pool.groups.keys().next().expect("the rig has groups"), 901)
}

fn bar_of(state: &AppState, screen: u32) -> Vec<BarButton> {
    let profile = state.root.show_data.profile_for_user("Admin").unwrap();
    bar(&state.root.show_data, profile, screen)
}

#[test]
fn a_sequence_on_a_button_is_selected_by_a_tap() {
    let (mut state, _, _) = rig();
    let (_, seq) = first_of(&mut state);
    let out = run(&mut state, &format!("Assign Sequence {seq} At ViewButton 1.6"));
    assert!(out.contains("ViewButton 1.6 → Sequence"), "{out}");
    assert_eq!(tap_line(&state, "ViewButton 1.6"), Some(Ok(format!("Sequence {seq}"))));
    assert_eq!(tap_line(&state, "vb 6"), Some(Ok(format!("Sequence {seq}"))), "a bare slot is the line's screen");
    run(&mut state, "Deselect Sequence");
    run(&mut state, "ViewButton 1.6");
    let guid = state.root.show_data.user("Admin").unwrap().selected.sequence;
    let want = state.root.show_data.datapools.values().find_map(|p| p.sequences.get(&seq)).map(|s| s.meta.identity.guid);
    assert_eq!(guid, want, "the tap ran Select Sequence {seq}");
}

#[test]
fn buttons_are_per_screen() {
    let (mut state, _, b) = rig();
    let (group, seq) = first_of(&mut state);
    run(&mut state, &format!("Assign Group {group} At ViewButton 1.2"));
    run(&mut state, &format!("Assign Sequence {seq} At ViewButton 2.2"));
    assert_eq!(bar_of(&state, 1)[0].kind, ButtonKind::Group);
    assert_eq!(bar_of(&state, 2)[0].kind, ButtonKind::Sequence);
    // The line from the second screen: a bare 2 is screen 2's button.
    let screen_no = state.root.runtime.live_screens.read().unwrap().get(b).unwrap().screen;
    state.current_live_screen = Some(b);
    assert_eq!(address(&state, "2"), Ok((screen_no, 2)));
}

#[test]
fn an_empty_button_only_says_so() {
    let (mut state, a, _) = rig();
    let before = state.root.runtime.live_screens.read().unwrap().get(a).unwrap().windows.len();
    let out = run(&mut state, "ViewButton 1.9");
    assert!(out.contains("ViewButton 1.9 is empty"), "{out}");
    assert!(out.contains("Store ViewButton 1.9"), "it names what to do: {out}");
    assert_eq!(state.root.runtime.live_screens.read().unwrap().get(a).unwrap().windows.len(), before, "the screen is kept");
}

#[test]
fn store_on_an_empty_button_stores_a_view_and_assigns_it() {
    let (mut state, _, _) = rig();
    let (_, seq) = first_of(&mut state);
    let out = run(&mut state, "Store ViewButton 1.3 \"Busk\"");
    assert!(out.contains("→ ViewButton 1.3"), "{out}");
    let bar = bar_of(&state, 1);
    let held = bar[0].held.as_ref().expect("the view");
    assert_eq!((held.kind, held.name.as_str()), (ButtonKind::View, "Busk"));
    assert_eq!(tap_line(&state, "ViewButton 1.3 /Screen 2"), Some(Ok(format!("View {} /Screen 2", held.no))));
    // A button holding another kind is not stored over.
    run(&mut state, &format!("Assign Sequence {seq} At ViewButton 1.4"));
    let out = run(&mut state, "Store ViewButton 1.4");
    assert!(out.contains("holds Sequence"), "{out}");
}

#[test]
fn view_zero_clears_and_deleting_a_view_clears_its_buttons() {
    let (mut state, _, _) = rig();
    run(&mut state, "Store ViewButton 1.1");
    let view = bar_of(&state, 1)[0].held.as_ref().unwrap().no;
    run(&mut state, &format!("Assign View {view} At ViewButton 2.5"));
    assert_eq!(bar_of(&state, 2).len(), 1);
    run(&mut state, "Assign View 0 At ViewButton 2.5");
    assert!(bar_of(&state, 2).is_empty());
    let out = run(&mut state, &format!("Delete View {view} /NoConfirmation"));
    assert!(out.contains("Deleted View"), "{out}");
    assert!(bar_of(&state, 1).is_empty(), "the view's buttons go with it");
    let profile = state.root.show_data.profile_for_user("Admin").unwrap();
    assert!(!profile.content.view_buttons.contains_key(&2), "no empty bars are kept");
}

#[test]
fn an_object_in_another_datapool_is_named_owner_first() {
    let (state, _, _) = rig();
    let mine = crate::pools::user_datapool(&state.root.show_data, "Admin");
    let held = Held { kind: ButtonKind::Group, no: 3, datapool: Some(mine + 1), name: "x".into(), guid: Uuid::nil() };
    assert_eq!(object_line(&state, &held), format!("DataPool {} Group 3", mine + 1));
    let held = Held { datapool: Some(mine), ..held };
    assert_eq!(object_line(&state, &held), "Group 3");
}

#[test]
fn assign_refuses_what_it_does_not_take() {
    let (mut state, _, _) = rig();
    let out = run(&mut state, "Assign Fixture 1 At ViewButton 1.1");
    assert!(out.contains("Usage: Assign <Object>"), "{out}");
    let out = run(&mut state, "Assign Sequence 9999 At ViewButton 1.1");
    assert!(out.contains("Sequence 9999 is empty"), "{out}");
    assert!(bar_of(&state, 1).is_empty());
}
