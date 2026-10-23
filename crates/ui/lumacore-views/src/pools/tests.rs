// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `pools`.
//!
//! Owns:          the unit tests of pools.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §22, §30 "12d", §34 20.7.

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

fn demo() -> AppState {
    lumacore_objects::demo::fresh_state()
}

#[test]
fn pools_every_pool_has_a_window_type() {
    let types = lumacore_registry::get().window_types();
    for (keyword, def) in pools() {
        let Some(window) = types.values().find(|t| t.name == def.window) else {
            panic!(
                "{keyword}'s pool window '{}' is not a window type",
                def.window
            );
        };
        assert!(!def.actions.is_empty(), "{keyword} has no tap action");
        // The window's Tap action setting offers exactly the pool's actions.
        let setting = window
            .setting("action")
            .expect("pool windows have an action");
        let names: Vec<&str> = def.actions.iter().map(|a| a.name).collect();
        let offered = serde_json::to_value(setting).unwrap();
        let offered: Vec<String> = offered["options"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| {
                o["label"]
                    .as_str()
                    .or(o[1].as_str())
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();
        assert_eq!(offered, names, "{keyword}");
    }
}

/// The demo with two new groups past the rig's 77: 111 "Left" (101–103) and 112 "Right" (104–106).
fn two_groups() -> AppState {
    let mut state = demo();
    run(&mut state, "Fixture 101 Thru 103");
    run(&mut state, "Store Group 111 \"Left\"");
    run(&mut state, "ResetAll");
    run(&mut state, "Fixture 104 Thru 106");
    run(&mut state, "Store Group 112 \"Right\"");
    run(&mut state, "ResetAll");
    state
}

#[test]
fn groups_pool_cells_show_selection_master_and_owner() {
    let mut state = two_groups();
    run(&mut state, "Fixture 101");
    run(&mut state, "Set Group 112 \"Mode\" Negative");
    run(&mut state, "Master Group 112 At 25");
    let user = state.effective_user_name().to_string();
    let guid = state.user_groups()[&112].meta.identity.guid;
    let claims = BTreeMap::from([(guid, "LD".to_string())]);
    let v = view(&state.root.show_data, &claims, &user, "Group", None, None, Default::default()).unwrap();
    assert_eq!(v["window"], "groups");
    assert_eq!(v["actions"][0]["name"], "Select");
    let cells = v["cells"].as_array().unwrap();
    let cell = |name: &str| cells.iter().find(|c| c["name"] == name).unwrap_or_else(|| panic!("{name}: {v}"));
    assert_eq!(cell("Left")["applicability"], "some");
    assert_eq!(cell("Right")["applicability"], "none");
    assert_eq!(cell("Right")["bar"]["text"], "25%");
    assert_eq!(cell("Right")["badges"][0], "Negative");
    assert_eq!(cell("Right")["owner"], "LD");
    // Your own claim is not a lock.
    let mine = BTreeMap::from([(guid, user.clone())]);
    let v = view(&state.root.show_data, &mine, &user, "Group", None, None, Default::default()).unwrap();
    let right = v["cells"].as_array().unwrap().iter().find(|c| c["name"] == "Right").unwrap();
    assert!(right["owner"].is_null());
    // The window type name finds the same pool.
    assert!(view(&state.root.show_data, &claims, &user, "groups", None, None, Default::default()).is_some());
}

#[test]
fn the_views_pool_draws_each_view_and_names_its_buttons() {
    let mut state = demo();
    let screen = state.root.runtime.live_screens.write().unwrap().join_or_create("Admin", None);
    state.current_live_screen = Some(screen);
    let out = run(&mut state, "Store ViewButton 1.7 \"Busk\"");
    assert!(out.contains("→ ViewButton 1.7"), "{out}");
    let user = state.effective_user_name().to_string();
    let v = view(&state.root.show_data, &BTreeMap::new(), &user, "views", None, None, Default::default()).unwrap();
    assert_eq!((v["pool"].as_str(), v["mine"].as_bool()), (Some("View"), Some(true)));
    let cells = v["cells"].as_array().unwrap();
    let cell = cells.iter().find(|c| c["name"] == "Busk").unwrap_or_else(|| panic!("{v}"));
    assert!(cell["badges"].as_array().unwrap().iter().any(|b| b == "VB 1.7"), "{cell}");
    let boxes = cell["layout"]["screens"][0]["windows"].as_array().unwrap();
    assert!(!boxes.is_empty(), "the stored screen's panes are drawn: {cell}");
    // Another profile's views: the pick, not the user's own.
    let mine = state.root.show_data.profile_id_for_user(&user);
    if let Some(other) = state.root.show_data.user_profiles.keys().copied().find(|n| Some(*n) != mine) {
        let v = view(&state.root.show_data, &BTreeMap::new(), &user, "views", None, Some(other), Default::default()).unwrap();
        assert_eq!(v["mine"], false);
    }
}

#[test]
fn the_macros_pool_counts_each_macros_lines() {
    let state = demo();
    let user = state.effective_user_name().to_string();
    let v = view(&state.root.show_data, &BTreeMap::new(), &user, "macros", None, None, Default::default()).unwrap();
    assert_eq!((v["pool"].as_str(), v["actions"][0]["name"].as_str()), (Some("Macro"), Some("Go")));
    let dp = lumacore_objects::pools::user_datapool(&state.root.show_data, &user);
    assert_eq!(v["cells"].as_array().unwrap().len(), state.root.show_data.datapools[&dp].macros.len());
}
