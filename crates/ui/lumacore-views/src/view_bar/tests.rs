// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `view_bar` on the rig.
//!
//! Owns:          the unit tests of view_bar.rs.
//! Doesn't own:   the buttons' commands (lumacore-objects `views::buttons`).
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
use lumacore_objects::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match lumacore_objects::commands::dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        DispatchResult::Ask(a) => format!("ASK {}", a.title()),
        _ => String::new(),
    }
}

#[test]
fn the_bar_names_what_each_button_holds_on_its_screen() {
    let mut state = lumacore_objects::demo::fresh_state();
    let screen = state.root.runtime.live_screens.write().unwrap().join_or_create("Admin", None);
    state.current_live_screen = Some(screen);
    let user = state.effective_user_name().to_string();
    let dp = lumacore_objects::pools::user_datapool(&state.root.show_data, &user);
    let group = *state.root.show_data.datapools[&dp].groups.keys().next().expect("the rig has groups");
    run(&mut state, &format!("Assign Group {group} At ViewButton 1.3"));
    run(&mut state, "Store ViewButton 1.1 \"Busk\"");
    run(&mut state, "Assign Group 9999 At ViewButton 2.1");
    let bar = view(&state.root.show_data, &user, 1);
    let buttons = bar["buttons"].as_array().unwrap();
    assert_eq!(buttons.len(), 2, "{bar}");
    assert_eq!((buttons[0]["slot"].as_u64(), buttons[0]["kind"].as_str(), buttons[0]["name"].as_str()), (Some(1), Some("View"), Some("Busk")));
    assert_eq!((buttons[1]["slot"].as_u64(), buttons[1]["kind"].as_str(), buttons[1]["no"].as_u64()), (Some(3), Some("Group"), Some(group as u64)));
    assert_eq!(buttons[1]["datapool"].as_u64(), Some(dp as u64));
    assert!(view(&state.root.show_data, &user, 2)["buttons"].as_array().unwrap().is_empty(), "nothing was assigned: Group 9999 is empty");
}
