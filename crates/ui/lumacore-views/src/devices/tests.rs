// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `devices`, on the rig (`demo::fresh_state`).
//!
//! Owns:          the unit tests of devices.rs.
//! Doesn't own:   the feed (lumacore-http), the kit element (tests/e2e).
//! Entry points:  the #[test] fns.
//! Decisions:     device-grid-and-value-marks, device-sheet-window.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use lumacore_objects::{AppState, DispatchResult};
use serde_json::Value as Json;

fn run(state: &mut AppState, line: &str) -> String {
    match lumacore_objects::commands::dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        DispatchResult::Ask(a) => format!("ASK {}", a.title()),
        _ => String::new(),
    }
}

/// The sheet as Admin's window would get it: the plan and frame the commands' output reads.
fn sheet(state: &AppState, p: &Params) -> Json {
    let plan = state.output_plan();
    let frame = state.output_frame();
    view(&state.root.show_data, "Admin", Some(&plan), Some(&frame), lumacore_base::clock::session_ms(), p)
}

/// The rig's numbered, patched devices that have a Dimmer, in sheet order: (fid, guid).
fn dimmers(state: &AppState) -> Vec<(u64, String)> {
    let plan = state.output_plan();
    let patched = |key: &str| key.parse::<uuid::Uuid>().is_ok_and(|g| plan.channel.attr_index.contains_key(&(g, "Dimmer".to_string())));
    let all = sheet(state, &Params { folders: false, count: MAX_ROWS, ..Params::default() });
    let mut out = Vec::new();
    let mut from = 0;
    let total = all["total"].as_u64().unwrap() as usize;
    while from < total && out.len() < 8 {
        let v = sheet(state, &Params { folders: false, from, count: MAX_ROWS, ..Params::default() });
        for r in v["rows"].as_array().unwrap().iter().skip(from).take(MAX_ROWS) {
            let key = r["key"].as_str().unwrap();
            if r["fid"].is_u64() && v["cells"][key]["Dimmer"].is_object() && patched(key) {
                out.push((r["fid"].as_u64().unwrap(), key.to_string()));
            }
        }
        from += MAX_ROWS;
    }
    out
}

fn cell<'a>(v: &'a Json, key: &str, attr: &str) -> &'a Json {
    &v["cells"][key][attr]
}

#[test]
fn the_rig_opens_with_its_folders_in_tree_order_and_cells_only_for_the_window() {
    let state = lumacore_objects::demo::fresh_state();
    let v = sheet(&state, &Params::default());
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(v["total"].as_u64().unwrap() as usize, rows.len());
    let folders: Vec<&Json> = rows.iter().filter(|r| r["kind"] == "folder").collect();
    assert!(!folders.is_empty(), "the rig's folders are tree rows");
    // A folder comes before the devices below it, which sit one deeper.
    let at = rows.iter().position(|r| r["kind"] == "folder").unwrap();
    assert!(rows[at + 1]["depth"].as_u64() > rows[at]["depth"].as_u64(), "{} / {}", rows[at], rows[at + 1]);
    assert!(folders.iter().all(|f| f["count"].as_u64().unwrap() > 0));
    let cols = v["columns"].as_array().unwrap();
    assert!(cols.iter().any(|c| c["id"] == "Dimmer"), "{cols:?}");
    assert!(cols.iter().all(|c| c["group"].is_string() && c["priority"].is_u64()));
    // Cells only for the rows the window shows.
    assert_eq!(v["cells"].as_object().unwrap().len(), Params::default().count.min(rows.len()));
    let first = rows[0]["key"].as_str().unwrap();
    assert!(v["cells"][first].is_object());
    let far = rows.last().unwrap()["key"].as_str().unwrap();
    assert!(v["cells"][far].is_null());
}

#[test]
fn a_programmer_value_is_active_then_inactive_and_marks_its_row() {
    let mut state = lumacore_objects::demo::fresh_state();
    let (fid, key) = dimmers(&state)[0].clone();
    run(&mut state, &format!("Fixture {fid}"));
    run(&mut state, "At 50");
    let p = Params { folders: false, programmer_only: true, ..Params::default() };
    let v = sheet(&state, &p);
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "programmer only: the one device: {rows:?}");
    assert_eq!(rows[0]["bar"], "abs");
    assert_eq!(rows[0]["sel"], 0);
    let c = cell(&v, &key, "Dimmer");
    assert_eq!(c["s"], "prgA", "{c}");
    assert_eq!(c["t"], "50");
    assert_eq!(c["l"], serde_json::json!(["abs"]));
    // With folders the device keeps the folders above it, each with the bar.
    let tree = sheet(&state, &Params { programmer_only: true, ..Params::default() });
    let tree_rows = tree["rows"].as_array().unwrap();
    assert!(tree_rows.iter().filter(|r| r["kind"] == "folder").all(|f| f["bar"] == "abs"), "{tree_rows:?}");
    // Off: kept in the programmer, not stored.
    run(&mut state, &format!("Off Fixture {fid} Attribute \"Dimmer\""));
    let v = sheet(&state, &p);
    let c = cell(&v, &key, "Dimmer");
    if !v["rows"].as_array().unwrap().is_empty() && c.is_object() {
        assert_eq!(c["s"], "prgI", "{c}");
        assert_eq!(c["l"], serde_json::json!(["deact"]));
    }
}

#[test]
fn a_parked_value_is_park() {
    let mut state = lumacore_objects::demo::fresh_state();
    let (fid, key) = dimmers(&state)[1].clone();
    let reply = run(&mut state, &format!("Park Fixture {fid} At 30"));
    let v = sheet(&state, &Params { folders: false, from: 0, count: MAX_ROWS, ..Params::default() });
    let at = v["rows"].as_array().unwrap().iter().position(|r| r["key"] == key.as_str()).unwrap();
    let v = sheet(&state, &Params { folders: false, from: at, count: 1, ..Params::default() });
    let c = cell(&v, &key, "Dimmer");
    assert_eq!(c["s"], "park", "{c} after: {reply}");
    assert_eq!(c["l"], serde_json::json!(["park"]));
}

/// Cue 2 of the selected sequence: a higher dimmer is new, a lower one is lower, an unchanged one
/// tracked; once the sequence is no longer selected the same values are another sequence's.
#[test]
fn cue_values_are_new_lower_or_tracked() {
    let mut state = lumacore_objects::demo::fresh_state();
    let d = dimmers(&state);
    let (a, ka) = d[0].clone();
    let (b, kb) = d[1].clone();
    let (c, kc) = d[2].clone();
    let me = state.root.show_data.user_no("Admin").unwrap();
    let clear = format!("Clear Programmer User {me}");
    for line in [
        format!("Fixture {a} + {b} + {c}"),
        "At 50".into(),
        "Store Sequence 901 Cue 1".into(),
        clear.clone(),
        format!("Fixture {a}"),
        "At 80".into(),
        format!("Fixture {b}"),
        "At 20".into(),
        "Store Sequence 901 Cue 2".into(),
        clear.clone(),
        "Select Sequence 901".into(),
        "Goto Sequence 901 Cue 2".into(),
    ] {
        run(&mut state, &line);
    }
    let p = Params { folders: false, selection_first: false, count: MAX_ROWS, ..Params::default() };
    let find = |state: &AppState, key: &str| {
        let all = sheet(state, &p);
        let at = all["rows"].as_array().unwrap().iter().position(|r| r["key"] == key).unwrap();
        sheet(state, &Params { from: at, count: 1, ..p.clone() })["cells"][key]["Dimmer"].clone()
    };
    assert_eq!(find(&state, &ka)["s"], "new", "{}", find(&state, &ka));
    assert_eq!(find(&state, &kb)["s"], "lower", "{}", find(&state, &kb));
    assert_eq!(find(&state, &kc)["s"], "track", "{}", find(&state, &kc));
    assert!(find(&state, &ka)["src"].as_str().unwrap().contains("Sequence 901"));
    run(&mut state, "Deselect Sequence");
    assert_eq!(find(&state, &ka)["s"], "other", "{}", find(&state, &ka));
    assert_eq!(find(&state, &kc)["s"], "otherT", "{}", find(&state, &kc));
}

#[test]
fn selected_only_and_selection_first_follow_the_pick_order() {
    let mut state = lumacore_objects::demo::fresh_state();
    let d = dimmers(&state);
    let (a, ka) = d[3].clone();
    let (b, kb) = d[0].clone();
    run(&mut state, &format!("Fixture {a}"));
    run(&mut state, &format!("+ Fixture {b}"));
    let v = sheet(&state, &Params { folders: false, ..Params::default() });
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows[0]["key"], ka.as_str(), "picked first, shown first");
    assert_eq!(rows[1]["key"], kb.as_str());
    assert_eq!(v["selected"], 2);
    let only = sheet(&state, &Params { folders: false, selected_only: true, ..Params::default() });
    assert_eq!(only["rows"].as_array().unwrap().len(), 2);
}

#[test]
fn the_fade_and_dmx_layers_read_their_own_values() {
    let mut state = lumacore_objects::demo::fresh_state();
    let (fid, key) = dimmers(&state)[0].clone();
    run(&mut state, &format!("Fixture {fid}"));
    run(&mut state, "At 100");
    let reply = run(&mut state, "Attribute \"Dimmer\" Fade 2");
    let p = Params { folders: false, programmer_only: true, ..Params::default() };
    let abs = sheet(&state, &p);
    assert_eq!(cell(&abs, &key, "Dimmer")["l"], serde_json::json!(["abs", "fade"]), "{} after: {reply}", cell(&abs, &key, "Dimmer"));
    let fade = sheet(&state, &Params { layer: Layer::Fade, ..p.clone() });
    assert_eq!(cell(&fade, &key, "Dimmer")["t"], "2s", "{}", cell(&fade, &key, "Dimmer"));
    assert_eq!(cell(&fade, &key, "Dimmer")["s"], "prgA");
    let dmx = sheet(&state, &Params { layer: Layer::Dmx, ..p.clone() });
    let c = cell(&dmx, &key, "Dimmer");
    assert_eq!(c["s"], "out");
    assert!(c["t"].as_str().unwrap().parse::<u32>().is_ok(), "{c}");
    assert_eq!(Layer::parse("dmx"), Layer::Dmx);
    assert_eq!(Layer::parse("nonsense"), Layer::Absolute);
}

/// One build of the opening window (the ALL folder at the top reads the whole rig) stays cheap:
/// the feed rebuilds it every period.
#[test]
fn the_opening_window_builds_quickly_on_the_rig() {
    let state = lumacore_objects::demo::fresh_state();
    let plan = state.output_plan();
    let frame = state.output_frame();
    let t = std::time::Instant::now();
    for _ in 0..5 {
        let _ = view(&state.root.show_data, "Admin", Some(&plan), Some(&frame), 0, &Params::default());
    }
    let each = t.elapsed() / 5;
    // Generous for a debug build; the release build is many times faster.
    assert!(each < std::time::Duration::from_millis(400), "{each:?}");
}

/// A value from a preset (decisions 0293): the cell carries the preset and its text follows the
/// window's Show — the value, the link (the preset), or both; the preset bar either way; the hover names
/// both. The rig's Sequence 1 cue 3 links Preset 3 "Split" for Fixture 101.
#[test]
fn a_linked_value_shows_its_preset_by_the_show_setting() {
    let mut state = lumacore_objects::demo::fresh_state();
    run(&mut state, "Goto Sequence 1 Cue 3");
    let key = state.root.show_data.fixture_guid(101).unwrap().to_string();
    let at = |state: &AppState, show: Show| {
        let p = Params { folders: false, programmer_only: false, show, count: MAX_ROWS, ..Params::default() };
        let mut from = 0;
        loop {
            let v = sheet(state, &Params { from, ..p.clone() });
            let c = cell(&v, &key, "Dimmer");
            if c.is_object() || from > v["total"].as_u64().unwrap() as usize {
                return c.clone();
            }
            from += MAX_ROWS;
        }
    };
    let both = at(&state, Show::Both);
    assert_eq!(both["p"]["no"], 3, "{both}");
    assert_eq!(both["p"]["name"], "Split");
    let value = at(&state, Show::Value)["t"].as_str().unwrap().to_string();
    assert_eq!(both["t"], format!("P3 Split · {value}"), "{both}");
    assert_eq!(at(&state, Show::Link)["t"], "P3 Split");
    assert!(both["l"].as_array().unwrap().iter().any(|l| l == "preset"), "the preset bar: {both}");
    assert!(both["src"].as_str().unwrap().contains("Preset 3 \"Split\""), "{both}");
    // A plain value over it in the programmer: no preset.
    run(&mut state, "Fixture 101 At 42");
    let plain = at(&state, Show::Both);
    assert!(plain["p"].is_null() && !plain["t"].as_str().unwrap().contains("P3"), "{plain}");
}

