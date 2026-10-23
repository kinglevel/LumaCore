// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `window state`.
//!
//! Owns:          the unit tests of window state.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12c", §34 20.7.

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

/// A station with Admin's screen; returns (state, live screen).
fn station() -> (AppState, u32) {
    let mut state = AppState::new();
    let screen = state.root.runtime.live_screens.write().unwrap().join_or_create("Admin", None);
    state.current_live_screen = Some(screen);
    (state, screen)
}

/// Add a window of `ty` and focus it; its number.
fn add(state: &mut AppState, ty: &str) -> u32 {
    let screen = state.current_live_screen.unwrap();
    let windows = |s: &AppState| -> Vec<u32> { s.root.runtime.live_screens.read().unwrap().get(screen).unwrap().windows.keys().copied().collect() };
    let before = windows(state);
    let out = run(state, &format!("Store Window \"{ty}\""));
    let w = windows(state).into_iter().find(|w| !before.contains(w)).unwrap_or_else(|| panic!("{out}"));
    state.current_window = Some(w);
    w
}

#[test]
fn the_windows_feed_resolves_titles_states_and_chips() {
    let (mut state, screen) = station();
    let dmx = add(&mut state, "DMX");
    run(&mut state, "Set Self \"Universe\" 2");
    run(&mut state, "Set Self \"View\" Input");
    let sm = add(&mut state, "System Monitor");
    run(&mut state, "Set Self \"Search\" \"act 2\"");
    run(&mut state, "Set Self \"AutoScroll\" Off");
    // A profile key whose LED reads Self.
    // (made on the profile itself: `Bind` is the keys' crate, above this one's tests).
    state.current_user_profile_mut().expect("a profile").bind_command("F5", "Set Self \"AutoScroll\" Toggle", "");
    if let Some(k) = state.root.show_data.profile_mut(2).and_then(|p| p.content.ui_keys.values_mut().next()) {
        k.state = "Self.AutoScroll".into();
    }
    let store = state.root.runtime.live_screens.read().unwrap().get(screen).cloned().unwrap();
    let v = screen_view(&state.root.show_data, "Admin", &store);
    let d = &v["windows"][dmx.to_string()];
    assert_eq!(d["type"], "dmx");
    assert!(d["title"].as_str().unwrap().starts_with("U 2"), "{d}");
    assert_eq!(d["states"]["input"], true);
    assert_eq!(d["items"], json!(["universe", "prev", "next", "readout"]));
    let s = &v["windows"][sm.to_string()];
    assert!(s["title"].as_str().unwrap().ends_with("errors"));
    assert_eq!(s["states"]["pause"], true);
    assert_eq!(s["chips"][0]["id"], "search");
    assert_eq!(s["chips"][0]["label"], "× Search “act 2”");
    assert_eq!(s["values"]["auto_scroll"], "No");
    let keys = s["keys"].as_object().unwrap();
    assert_eq!(keys.values().next(), Some(&json!(false)), "the key's LED: AutoScroll is off");
}

#[test]
fn the_windows_feed_carries_the_pick_lists_of_pinned_links() {
    let (mut state, screen) = station();
    let groups = add(&mut state, "Groups");
    let dmx = add(&mut state, "DMX");
    let store = state.root.runtime.live_screens.read().unwrap().get(screen).cloned().unwrap();
    let v = screen_view(&state.root.show_data, "Admin", &store);
    // The DataPool link is in the Groups bar: its pick list (follow row + the show's datapools).
    let pick = &v["windows"][groups.to_string()]["picks"]["datapool"];
    assert_eq!(pick["follow"], "Selected DataPool", "{pick}");
    assert!(!pick["items"].as_array().unwrap().is_empty(), "{pick}");
    // Universe is pinned in the DMX bar too; settings not in the bar carry none.
    assert!(v["windows"][dmx.to_string()]["picks"].get("universe").is_some());
    assert!(v["windows"][dmx.to_string()]["picks"].get("view").is_none());
}

/// A held pane shows maximised on the shared screen (decisions §30 "12c2").
#[test]
fn the_windows_feed_names_the_held_pane() {
    let (mut state, screen) = station();
    run(&mut state, &format!("Set Window {screen}.2 \"Maximized\" On"));
    let store = state.root.runtime.live_screens.read().unwrap().get(screen).cloned().unwrap();
    assert_eq!(screen_view(&state.root.show_data, "Admin", &store)["maximized"], 2);
}
