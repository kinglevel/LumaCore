//! R1-61, R1-59: the window and view Rename and the Store View pop-up are
//! station asks (one ask path, decisions §30 "12b"); the view bar hears about
//! every change to a profile.

use std::sync::{Arc, Mutex};

use super::dispatch;
use crate::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        DispatchResult::Ask(a) => format!("ASK {}", a.title()),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn ask(state: &mut AppState, line: &str) -> crate::dialogs::Ask {
    match dispatch(state, line) {
        DispatchResult::Ask(a) => *a,
        DispatchResult::Output(s) => panic!("'{line}' didn't ask: {s}"),
        _ => panic!("'{line}' didn't ask"),
    }
}

/// A station with two of Admin's screens; returns (state, first, second).
fn station() -> (AppState, u32, u32) {
    let mut state = AppState::new();
    let (a, b) = {
        let mut store = state.root.runtime.live_screens.write().unwrap();
        (store.join_or_create("Admin", None), store.join_or_create("Admin", None))
    };
    state.current_live_screen = Some(a);
    (state, a, b)
}

#[test]
fn label_self_without_a_name_asks() {
    let (mut state, screen, _) = station();
    state.current_window = Some(2);
    run(&mut state, "Label Self \"Busk\"");
    let a = ask(&mut state, "Label Self");
    assert_eq!(a.view().inputs[0].value, "Busk", "the current name is offered");
    assert_eq!(a.primary_line().as_deref(), Some(format!("Label Window {screen}.2 \"Busk\"").as_str()));
    // Asking changed nothing; an empty answer clears.
    let label = |s: &AppState| s.root.runtime.live_screens.read().unwrap().get(screen).unwrap().windows[&2].label.clone();
    assert_eq!(label(&state).as_deref(), Some("Busk"));
    run(&mut state, &format!("Label Window {screen}.2 /Clear"));
    assert_eq!(label(&state), None);
    // A one-shot path is refused with the flag.
    assert!(a.refusal().contains("/Clear"), "{}", a.refusal());
}

#[test]
fn label_viewbutton_without_a_name_asks() {
    let (mut state, _, _) = station();
    run(&mut state, "Store ViewButton 4 \"Busk\"");
    let a = ask(&mut state, "Label ViewButton 4");
    assert_eq!(a.view().inputs[0].value, "Busk");
    assert_eq!(a.primary_line().as_deref(), Some("Label ViewButton 1.4 \"Busk\""));
    let a = ask(&mut state, "Label View 1");
    assert_eq!(a.primary_line().as_deref(), Some("Label View 1 \"Busk\""));
    // An empty button asks nothing: it has no view to rename.
    assert!(run(&mut state, "Label ViewButton 9").contains("ViewButton 1.9 is empty"));
}

#[test]
fn the_store_view_ask_has_a_toggle_per_screen_and_runs_store_view() {
    let (mut state, a, b) = station();
    let ask = crate::commands::views::store_ask(&state, true, 4).unwrap();
    let toggles = &ask.view().toggles;
    assert_eq!(toggles.len(), 2, "one per screen of the user");
    assert!(toggles[0].on && !toggles[1].on, "only this screen is on");
    assert_eq!(ask.view().inputs[0].value, "View 1", "the next free view's name");
    let line = ask.primary_line().unwrap();
    assert_eq!(line, format!("Store ViewButton 4 \"View 1\" /Overwrite /Screen \"{a},\""));
    // The button's line is a real Store View.
    let out = run(&mut state, &line);
    assert!(out.contains("Stored View 1"), "{out}");
    let profile = state.root.show_data.profile_for_user("Admin").unwrap();
    assert_eq!(profile.content.views[&1].screens.keys().copied().collect::<Vec<_>>(), vec![a]);
    // Both screens, and none.
    let out = run(&mut state, &format!("Store ViewButton 4 \"Two\" /Overwrite /Screen \"{a},{b},\""));
    assert!(out.contains("from screens"), "{out}");
    let out = run(&mut state, "Store ViewButton 4 \"None\" /Overwrite /Screen \"\"");
    assert!(out.contains("at least one screen"), "{out}");
    // Over a stored view: its name, and the title says it overwrites.
    let ask = crate::commands::views::store_ask(&state, true, 4).unwrap();
    assert_eq!(ask.view().inputs[0].value, "Two");
    assert!(ask.title().contains("overwrite"), "{}", ask.title());
}

#[test]
fn the_view_bar_hears_every_profile_change() {
    let (mut state, _, _) = station();
    let heard = Arc::new(Mutex::new(0));
    let seen = heard.clone();
    state
        .root
        .runtime
        .live_screens
        .write()
        .unwrap()
        .subscribe_views(move |_| *seen.lock().unwrap() += 1);
    let count = |h: &Arc<Mutex<i32>>| *h.lock().unwrap();

    // A view command (through the change path).
    run(&mut state, "Store View 3 \"Busk\"");
    let after_store = count(&heard);
    assert!(after_store >= 1);
    // Oops puts the profile back: heard.
    run(&mut state, "Oops /NoConfirmation");
    assert!(count(&heard) > after_store, "Oops");
    let after_oops = count(&heard);
    // A replicated / restored profile (the path members apply ops with).
    let profile = state.root.show_data.profile_for_user("Admin").unwrap().meta.identity.guid;
    let snap = crate::commands::oops::snapshot_subtrees(&state, &[lumacore_tree::ShowSubtree::Profile(profile)]).unwrap();
    crate::commands::oops::restore_subtrees(&mut state, &snap).unwrap();
    // The state only notes the profile; the layer above refreshes (replication does this).
    crate::commands::views::follow_touched_profiles(&mut state);
    assert!(count(&heard) > after_oops, "restore");
    let after_restore = count(&heard);
    // Another show.
    let show = state.root.show_data.clone();
    crate::objects::replace_show(&mut state, show);
    assert!(count(&heard) > after_restore, "show load");
}
