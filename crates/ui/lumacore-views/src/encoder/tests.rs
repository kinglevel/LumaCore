// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `encoder`.
//!
//! Owns:          the unit tests of encoder.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12f", §34 20.7.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use lumacore_objects::gesture::GestureLine;
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
fn the_demo_bar_has_a_strip_and_the_masters() {
    let state = lumacore_objects::demo::fresh_state();
    let no = state.root.runtime.live_screens.write().unwrap().join_or_create("Admin", None);
    let screen = state.root.runtime.live_screens.read().unwrap().get(no).unwrap().clone();
    let v = view(&state.root.show_data, "Admin", &screen, None, &json!({}));
    assert_eq!(v["mode"], "Attributes");
    assert_eq!(v["on"], true, "Screen 1 shows its bar by default");
    assert!(!v["strip"].as_array().unwrap().is_empty(), "{v}");
    assert_eq!(v["encoders"].as_array().unwrap().len(), PER_PAGE);
    state.root.runtime.live_screens.write().unwrap().set_encoder(no, |e| e.mode = EncoderMode::Masters).unwrap();
    let screen = state.root.runtime.live_screens.read().unwrap().get(no).unwrap().clone();
    let v = view(&state.root.show_data, "Admin", &screen, None, &json!({ "highlight": { "level": 70 } }));
    let labels: Vec<&str> = v["masters"].as_array().unwrap().iter().map(|m| m["label"].as_str().unwrap()).collect();
    assert!(labels.starts_with(&["Grand", "Highlight", "Solo", "My HL"]), "{labels:?}");
}

/// Cue timing opens on the current cue; a group's unset time shows the cue's in brackets; a
/// timing drag is one Oops step (decisions §30 "12f").
#[test]
fn cue_timing_opens_on_the_current_cue() {
    let mut state = demo();
    run(&mut state, "Fixture 101");
    run(&mut state, "At 100");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "At 50");
    run(&mut state, "Store Sequence 1 Cue 2");
    run(&mut state, "Select Sequence 1");
    run(&mut state, "Set Sequence 1 Cue 2 \"Fade\" 3");
    run(&mut state, "Goto Sequence 1 Cue 2");
    let no = state.root.runtime.live_screens.write().unwrap().join_or_create("Admin", None);
    state.current_live_screen = Some(no);
    run(&mut state, "Set Screen \"EncoderMode\" CueTimingVertical");
    let screen = state.root.runtime.live_screens.read().unwrap().get(no).unwrap().clone();
    assert_eq!(screen.encoder.mode, EncoderMode::CueTimingVertical);
    let v = view(&state.root.show_data, "Admin", &screen, None, &serde_json::json!({}));
    assert_eq!(v["cue"], "2", "{v}");
    assert_eq!(v["encoders"][0]["in"]["fade"]["text"], "3s");
    if !state.root.show_data.feature_groups.is_empty() {
        assert_eq!(v["encoders"][1]["in"]["fade"]["text"], "[3s]", "a group inherits the cue's");
    }
    // A drag on the cue's Fade: one Oops step back to where it began.
    let steps = |s: &AppState| s.undo_stack("Admin").map_or(0, |u| u.frames.len());
    let before = steps(&state);
    for f in ["4", "5"] {
        state.gesture = Some(GestureLine::Move { id: "t".into() });
        run(&mut state, &format!("Set Sequence 1 Cue 2 \"Fade\" {f}"));
        state.gesture = None;
    }
    state.gesture = Some(GestureLine::End { id: "t".into(), summary: "Fader Cue 2 Fade: 3 → 6".into() });
    run(&mut state, "Set Sequence 1 Cue 2 \"Fade\" 6");
    state.gesture = None;
    assert_eq!(steps(&state), before + 1);
    run(&mut state, "Oops");
    let fade = state.root.show_data.datapools[&lumacore_objects::pools::user_datapool(&state.root.show_data, "Admin")].sequences[&1].cues[&2000].parts[&0].fade_in;
    assert_eq!(fade, 3.0, "Oops goes back to before the drag");
}
