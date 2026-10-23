// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `attributes`.
//!
//! Owns:          the unit tests of attributes.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12f-a", §34 20.7.

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
fn the_demo_board_has_every_group_and_its_attributes_in_order() {
    let state = lumacore_objects::demo::fresh_state();
    let show = &state.root.show_data;
    let v = view(show, "Admin", BoardTarget::Show).unwrap();
    let groups = v["groups"].as_array().unwrap();
    assert_eq!(groups.len(), show.feature_groups.len());
    let placed: usize = groups.iter().map(|g| g["members"].as_array().unwrap().len()).sum();
    assert_eq!(placed, show.attribute_defs.len(), "every defined attribute is on the board once");
    assert_eq!(v["owner"], "");
    let no = show.profile_id_for_user("Admin").unwrap();
    let mine = view(show, "Admin", BoardTarget::Profile(no)).unwrap();
    assert_eq!(mine["owner"], "UserProfile");
    let dimmer = mine["attrs"].as_array().unwrap().iter().find(|a| a["name"] == "Dimmer").unwrap();
    assert_eq!(dimmer["show_group"], "Dimmer");
    assert!(dimmer["used"].as_bool().unwrap());
    assert!(!dimmer["types"].as_array().unwrap().is_empty());
}

/// The Encoder bar tab sees your layouts (12f-a).
#[test]
fn the_tab_sees_your_encoder_bar_layouts() {
    let mut s = demo();
    run(&mut s, "Store UserProfile EncoderBar \"Busking\"");
    let show = &s.root.show_data;
    let v = view(show, "Admin", BoardTarget::parse(show, "Admin", &json!("mine"))).unwrap();
    let bars = v["encoder_bars"].as_array().unwrap();
    assert_eq!(bars.len(), 2);
    assert_eq!(bars[1]["name"], "Busking");
    assert!(!bars[0]["buttons"].as_array().unwrap().is_empty());
}
