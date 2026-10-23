// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of the Rules window's view on the rig (decisions 0294).
//!
//! Owns:          the unit tests of rules.rs.
//! Doesn't own:   the rule words (lumacore-objects `attributes`).
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

/// The show's rules in order with their groups in the viewer's colours; a learned rule says so;
/// Which rule? names the rule that places a name and marks its row; a cell and a drag are lines.
#[test]
fn the_rules_window_lists_the_rules_and_finds_which_one_places_a_name() {
    let mut state = lumacore_objects::demo::fresh_state();
    let user = state.effective_user_name().to_string();
    let rows = |s: &AppState, which: &str| view(&s.root.show_data, &user, BoardTarget::Show, which);
    let before = rows(&state, "");
    let n = before["rows"].as_array().unwrap().len();
    assert!(n > 10, "the rig's show carries its template's rules: {before}");
    assert_eq!(before["rows"][0]["no"], 1);
    assert_eq!(before["count"]["learned"], 0);
    assert!(before["rows"].as_array().unwrap().iter().all(|r| r["kind"] == "standard"));

    let out = run(&mut state, "Store AttributeRule \"Pixel*\" FeatureGroup \"FX\"");
    assert!(out.contains("learned"), "{out}");
    let after = rows(&state, "PixelMSpeed");
    let which = &after["which"];
    let no = which["rule"].as_u64().expect("a rule places it");
    let row = after["rows"].as_array().unwrap().iter().find(|r| r["no"] == no).unwrap().clone();
    assert_eq!((row["pattern"].as_str(), row["group"].as_str(), row["kind"].as_str()), (Some("Pixel*"), Some("FX"), Some("learned")));
    assert_eq!(row["hit"], true);
    assert_eq!(which["group"], "FX");
    // The FX group links its theme slot: the row wears the viewer's colour of it.
    assert_eq!(row["colour"], lumacore_tree::media::PAINT_SLOTS.iter().find(|(s, _)| *s == "FX").unwrap().1);
    assert_eq!(after["rows"].as_array().unwrap().iter().filter(|r| r["hit"] == true).count(), 1);

    // The cells' lines: the pattern, the group, the order.
    let out = run(&mut state, &format!("Set AttributeRule {no} \"Pattern\" \"Pix*\""));
    assert!(out.contains("Pattern = Pix*"), "{out}");
    let out = run(&mut state, &format!("Set AttributeRule {no} \"FeatureGroup\" \"Control\""));
    assert!(out.contains("FeatureGroup = Control"), "{out}");
    let out = run(&mut state, &format!("Move AttributeRule {no} At 1"));
    assert!(out.contains("is now 1"), "{out}");
    let moved = rows(&state, "PixMode");
    assert_eq!((moved["rows"][0]["pattern"].as_str(), moved["rows"][0]["group"].as_str()), (Some("Pix*"), Some("Control")));
    assert_eq!(moved["which"]["rule"], 1);
    assert_eq!(moved["lines"]["move"], "Move AttributeRule {from} At {to}");
    assert!(run(&mut state, "Set AttributeRule 1 \"FeatureGroup\" \"Nope\"").contains("E-"), "a group the layer lacks is refused");
}
