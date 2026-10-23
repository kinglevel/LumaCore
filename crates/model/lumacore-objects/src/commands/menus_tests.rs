//! 12d part 3 (decisions §30 "12d hardening"): the menu registry → JSON,
//! `Menu "x" [/Tab]` (toggle, last tab on the profile, never an Oops step),
//! `List` / `Info Menu`, `GridColumns`, one `Set` line for many rows, and the
//! typed settings of Station, Interface, a network's NIC and the show.

use super::dispatch;
use crate::{AppState, DispatchResult};
use lumacore_wire::UiEvent;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn ui() -> AppState {
    let mut state = AppState::new();
    state.ui_client = true;
    state
}

fn last_menu(state: &AppState) -> (String, Option<String>, bool, String) {
    match state.ui_requests.last() {
        Some(UiEvent::OpenMenu { menu, tab, toggle, grids, .. }) => (menu.clone(), tab.clone(), *toggle, grids.clone()),
        other => panic!("no OpenMenu: {other:?}"),
    }
}

#[test]
fn a_locked_profile_keeps_no_tab() {
    let mut state = ui();
    state.current_user_profile_mut().unwrap().locked = true;
    let out = run(&mut state, "Menu Settings /Tab Show");
    assert!(!out.contains("Error"), "{out}");
    assert_eq!(last_menu(&state).1.as_deref(), Some("Show"));
    assert!(state.root.show_data.profile_for_user("Admin").unwrap().settings.ui.menu_tabs.is_empty());
}

