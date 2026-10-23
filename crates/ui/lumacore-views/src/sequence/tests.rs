// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `sequence`.
//!
//! Owns:          the unit tests of sequence.rs.
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
use lumacore_objects::AppState;

fn run(state: &mut AppState, line: &str) -> String {
    match lumacore_objects::commands::dispatch(state, line) {
        lumacore_objects::DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        _ => String::new(),
    }
}

fn demo() -> AppState {
    lumacore_objects::demo::fresh_state()
}

/// The sheet follows the selected sequence: CueZero, the cues, OffCue; a feature group's own
/// timing and Path ride per cue, unset ones are null; the lines it gives run.
#[test]
fn the_sheet_shows_the_selected_sequence_with_its_feature_group_timing() {
    let mut s = demo();
    assert_eq!(view(&s.root.show_data, "Admin", &json!("selected"))["missing"], true);
    run(&mut s, "Fixture 101 At 50");
    run(&mut s, "Store Sequence 3 Cue 1");
    run(&mut s, "Store Sequence 3 Cue 2");
    run(&mut s, "Select Sequence 3");
    run(&mut s, "Set Sequence 3 Cue 2 FeatureGroup \"Dimmer\" \"Fade\" 4");
    run(&mut s, "Set Sequence 3 Cue 2 FeatureGroup \"Dimmer\" \"Path\" SCurve");
    run(&mut s, "Go Sequence 3");
    let v = view(&s.root.show_data, "Admin", &json!("selected"));
    let cues = v["cues"].as_array().unwrap();
    let labels: Vec<&str> = cues.iter().map(|c| c["cue"].as_str().unwrap()).collect();
    assert_eq!(labels, ["0", "1", "2", "OffCue"]);
    assert_eq!(v["running"], true);
    assert_eq!(v["current"], "1");
    let dim = v["feature_groups"].as_array().unwrap().iter().position(|g| g["name"] == "Dimmer").unwrap();
    assert_eq!(cues[2]["groups"][dim]["fade"], 4.0);
    assert_eq!(cues[2]["groups"][dim]["path"], "SCurve");
    assert!(cues[1]["groups"][dim]["fade"].is_null());
    // The row's line + a word is a line that runs.
    let line = format!("{} {} \"Delay\" 1.5", cues[2]["line"].as_str().unwrap(), v["feature_groups"][dim]["word"].as_str().unwrap());
    let r = run(&mut s, &line);
    assert!(!r.contains("E-"), "{line}: {r}");
    let again = view(&s.root.show_data, "Admin", &v["guid"]);
    assert_eq!(again["cues"][2]["groups"][dim]["delay"], 1.5);
}

/// The timeline's cue-head lines run: Goto from the row's line, and a feature group put back
/// to the cue's timing (several statements in one line).
#[test]
fn the_timeline_cue_options_run() {
    let mut s = demo();
    run(&mut s, "Fixture 101 At 50");
    run(&mut s, "Store Sequence 3 Cue 1");
    run(&mut s, "Store Sequence 3 Cue 2");
    run(&mut s, "Select Sequence 3");
    run(&mut s, "Set Sequence 3 Cue 2 FeatureGroup \"Dimmer\" \"Fade\" 4");
    run(&mut s, "Set Sequence 3 Cue 2 FeatureGroup \"Dimmer\" \"Path\" SCurve");
    let v = view(&s.root.show_data, "Admin", &json!("selected"));
    let cue2 = &v["cues"][2];
    let goto = cue2["line"].as_str().unwrap().replacen("Set ", "Goto ", 1);
    let r = run(&mut s, &goto);
    assert!(!r.contains("E-"), "{goto}: {r}");
    let v = view(&s.root.show_data, "Admin", &json!("selected"));
    assert_eq!(v["current"], "2");
    let dim = v["feature_groups"].as_array().unwrap().iter().position(|g| g["name"] == "Dimmer").unwrap();
    let word = v["feature_groups"][dim]["word"].as_str().unwrap();
    let line = cue2["line"].as_str().unwrap();
    // The cue's own timing again: Reset (decisions §33 61, 87).
    let reset = line.replacen("Set ", "Reset ", 1);
    let back = format!("{reset} {word} \"Fade\"; {reset} {word} \"Path\"");
    let r = run(&mut s, &back);
    assert!(!r.contains("E-"), "{back}: {r}");
    let v = view(&s.root.show_data, "Admin", &json!("selected"));
    assert!(v["cues"][2]["groups"][dim]["fade"].is_null());
    assert!(v["cues"][2]["groups"][dim]["path"].is_null());
}

/// A sequence of another DataPool is named owner first, and that line runs there.
#[test]
fn a_sheet_of_another_datapool_sends_owner_first_lines() {
    let mut s = demo();
    s.root.show_data.datapools.insert(2, lumacore_tree::Datapool::new(Some("Second".into())));
    s.root.show_data.after_load();
    run(&mut s, "Fixture 101 At 50");
    let r = run(&mut s, "Store DataPool 2 Sequence 4 Cue 1");
    assert!(!r.contains("E-"), "{r}");
    assert!(!s.root.show_data.datapools[&1].sequences.contains_key(&4), "stored in DataPool 2 only");
    let guid = s.root.show_data.datapools[&2].sequences[&4].meta.identity.guid.to_string();
    let v = view(&s.root.show_data, "Admin", &json!(guid));
    let line = format!("{} \"Fade\" 3", v["cues"][1]["line"].as_str().unwrap());
    assert_eq!(line, "Set DataPool 2 Sequence 4 Cue 1 \"Fade\" 3");
    let r = run(&mut s, &line);
    assert!(!r.contains("E-"), "{r}");
    assert_eq!(s.root.show_data.datapools[&2].sequences[&4].cues[&1000].parts[&0].fade_in, 3.0);
}
