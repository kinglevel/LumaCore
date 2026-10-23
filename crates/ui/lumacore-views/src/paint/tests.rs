// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The feeds carry each object's colour as its viewer sees it (decisions 0294), on the rig: the
//! `paint` feed's palette, slots, recent colours and mode; the pools, the Tree window and its
//! inspector, the view bar, the Device Sheet, the patch grid and the cue sheet; the feature
//! groups on their slots in the encoder and the attribute board.
//!
//! Owns:          the tests of paint.rs and of the colours the other views send.
//! Doesn't own:   the colour order itself (lumacore-objects `paint_tests`).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0294-object-paint-palette-free-theme-links.

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

fn ok(state: &mut AppState, line: &str) {
    let out = run(state, line);
    assert!(!out.contains("E-"), "{line}: {out}");
}

fn slot(name: &str) -> &'static str {
    lumacore_tree::media::PAINT_SLOTS.iter().find(|(n, _)| *n == name).map(|(_, c)| *c).unwrap()
}

fn named(word: &str) -> String {
    lumacore_base::Paint::parse(word).unwrap().unwrap().word()
}

/// The `paint` feed: the station's palette, the viewer's slots (changed and added marked, each
/// with its lines), recent colours and the mode, its line by the profile's number.
#[test]
fn the_paint_feed_carries_the_palette_the_slots_and_the_recent_colours() {
    let mut state = lumacore_objects::demo::fresh_state();
    let v = view(&state.root.show_data, "Admin", None);
    assert_eq!(v["palette"]["hues"].as_array().unwrap().len(), 12);
    assert_eq!(v["palette"]["hues"][9]["name"], "Blue");
    assert_eq!(v["palette"]["hues"][9]["tones"].as_array().unwrap().len(), 4);
    assert_eq!(v["palette"]["greys"].as_array().unwrap().len(), 8);
    assert_eq!(v["slots"].as_array().unwrap().len(), lumacore_tree::media::PAINT_SLOTS.len());
    assert_eq!(v["slots"][1]["name"], "Position");
    assert_eq!(v["slots"][1]["changed"], false);
    assert_eq!(v["mode"], "Palette");
    assert!(v["mode_line"].as_str().unwrap().starts_with("Set UserProfile "), "{v}");

    ok(&mut state, "Set Theme Slot \"Position\" \"#3a62b8\"");
    ok(&mut state, "Store Theme Slot \"Haze\" \"#8fa3b8\"");
    ok(&mut state, "Set Group 1 \"Paint\" Blue");
    let v = view(&state.root.show_data, "Admin", None);
    let position = &v["slots"][1];
    assert_eq!((position["colour"].as_str(), position["default"].as_str(), position["changed"].as_bool()), (Some("#3a62b8"), Some(slot("Position")), Some(true)));
    assert_eq!(position["line"], "Set Theme Slot \"Position\" {value}");
    assert_eq!(position["reset"], "Reset Theme Slot \"Position\"");
    let haze = v["slots"].as_array().unwrap().last().unwrap();
    assert_eq!((haze["name"].as_str(), haze["added"].as_bool()), (Some("Haze"), Some(true)));
    assert_eq!(v["recent"][0], named("Blue"));
    assert_eq!(v["reset_all"], "Reset Theme Slot \"Position\"; Reset Theme Slot \"Haze\"");
    // The Reset to LumaCore line puts them all back.
    ok(&mut state, v["reset_all"].as_str().unwrap());
    assert!(view(&state.root.show_data, "Admin", None)["slots"].as_array().unwrap().iter().all(|s| s["changed"] == false));
}

/// Every feed sends the colour its viewer sees: a theme link in each viewer's own theme, a
/// fixture's, a cue's, a view button's, a tree row's; the inspector says where it comes from.
#[test]
fn the_feeds_send_each_viewer_their_own_colours() {
    let mut state = lumacore_objects::demo::fresh_state();
    ok(&mut state, "Store User \"Bob\"");
    ok(&mut state, "Set Group 1 \"Paint\" Theme \"Position\"");
    ok(&mut state, "Set Theme Slot \"Position\" \"#3a62b8\"");
    let show = state.root.show_data.clone();
    let claims = Default::default();
    let cell = |user: &str| {
        let v = crate::pools::view(&show, &claims, user, "Group", None, None, Default::default()).unwrap();
        v["cells"].as_array().unwrap().iter().find(|c| c["no"] == 1).unwrap().clone()
    };
    assert_eq!((cell("Admin")["color"].as_str(), cell("Admin")["paint_from"].as_str()), (Some("#3a62b8"), Some("Theme Position")));
    assert_eq!(cell("Bob")["color"].as_str(), Some(slot("Position")), "Bob's own theme");
    assert!(cell("Admin").get("color").is_some());

    // The Tree window's row and its inspector.
    let t = crate::tree_window::tree(&mut state, &crate::tree_window::TreeParams { path: "Root.ShowData.DataPools.1.Groups.1".into(), ..Default::default() });
    let row = t["rows"].as_array().unwrap().iter().find(|r| r["path"] == "Root.ShowData.DataPools.1.Groups.1").unwrap();
    assert_eq!(row["paint"], "#3a62b8");
    let i = crate::tree_window::inspector(&mut state, "Root.ShowData.DataPools.1.Groups.1");
    let from = i["rows"].as_array().unwrap().iter().find(|r| r["name"] == "Colour from").expect("the inspector says where");
    assert_eq!(from["value"], "#3a62b8 · Theme Position");

    // A fixture: the Device Sheet's name and the patch grid's row.
    ok(&mut state, "Set Fixture 101 \"Paint\" Red");
    let plan = state.output_plan();
    let frame = state.output_frame();
    let sheet = crate::devices::view(&state.root.show_data, "Admin", Some(&plan), Some(&frame), lumacore_base::clock::session_ms(), &crate::devices::Params { folders: false, ..Default::default() });
    let fixture = sheet["rows"].as_array().unwrap().iter().find(|r| r["fid"] == 101).expect("Fixture 101 in the first rows");
    assert_eq!(fixture["paint"], named("Red"));
    let patch = crate::patch::view(&state.root.show_data, None, "Admin");
    let row = patch["fixtures"].as_array().unwrap().iter().find(|r| r["no"] == 101).unwrap();
    assert_eq!((row["paint"].as_str(), row["paint_from"].as_str()), (Some(named("Red").as_str()), Some("own paint")));

    // A cue's row in the cue sheet.
    ok(&mut state, "Select Sequence 1");
    ok(&mut state, "Set Cue 2 \"Paint\" Theme \"Dimmer\"");
    let sheet = crate::sequence::view(&state.root.show_data, "Admin", &serde_json::json!("selected"));
    let cue = sheet["cues"].as_array().unwrap().iter().find(|c| c["cue"] == "2").unwrap();
    assert_eq!(cue["paint"], slot("Dimmer"));
    // The cue sheet's feature groups wear their slots.
    let position = sheet["feature_groups"].as_array().unwrap().iter().find(|g| g["name"] == "Position").unwrap();
    assert_eq!(position["color"], "#3a62b8", "Admin's Position slot");

    // A view button holding the group.
    let screen = state.root.runtime.live_screens.write().unwrap().join_or_create("Admin", None);
    state.current_live_screen = Some(screen);
    ok(&mut state, "Assign Group 1 At ViewButton 1.3");
    let bar = crate::view_bar::view(&state.root.show_data, "Admin", 1);
    let button = bar["buttons"].as_array().unwrap().iter().find(|b| b["slot"] == 3).unwrap();
    assert_eq!(button["paint"], "#3a62b8");
}

/// The attribute board's feature groups wear their slots, as the viewer's theme has them.
#[test]
fn feature_groups_wear_their_slots_on_the_board() {
    let mut state = lumacore_objects::demo::fresh_state();
    ok(&mut state, "Set Theme Slot \"Gobo\" \"#123456\"");
    let board = crate::attribute_board::view(&state.root.show_data, "Admin", lumacore_objects::attribute_board::BoardTarget::Show).unwrap();
    let groups = board["groups"].as_array().unwrap();
    let colour = |name: &str| groups.iter().find(|g| g["name"] == name).map(|g| g["color"].clone());
    assert_eq!(colour("Dimmer"), Some(serde_json::json!(slot("Dimmer"))));
    if let Some(gobo) = colour("Gobo") {
        assert_eq!(gobo, "#123456", "the viewer's own slot");
    }
}
