//! Step 12c (decisions §30 "12c hardening / design locked"): `Self`, the
//! window bar's settings, relative / Default / Toggle on `Set Window`,
//! resets, window presets, JSON export / import, the windows feed.

use super::dispatch;
use crate::self_ref::LineOwner;
use crate::{AppState, DispatchResult};
use lumacore_wire::{FieldInput, UiEvent};
use serde_json::json;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        DispatchResult::Ask(a) => format!("ASK {}", a.title()),
        _ => panic!("'{line}' did not produce output"),
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
    let before: Vec<u32> = state.root.runtime.live_screens.read().unwrap().get(screen).unwrap().windows.keys().copied().collect();
    let out = run(state, &format!("Store Window \"{ty}\""));
    let w = state.root.runtime.live_screens.read().unwrap().get(screen).unwrap().windows.keys().copied().find(|w| !before.contains(w)).unwrap_or_else(|| panic!("{out}"));
    state.current_window = Some(w);
    w
}

fn setting(state: &AppState, w: u32, key: &str) -> Option<serde_json::Value> {
    let store = state.root.runtime.live_screens.read().unwrap();
    store.get(state.current_live_screen.unwrap()).unwrap().windows[&w].settings.get(key).cloned()
}

fn window(state: &AppState, w: u32) -> lumacore_tree::runtime::Window {
    let store = state.root.runtime.live_screens.read().unwrap();
    store.get(state.current_live_screen.unwrap()).unwrap().windows[&w].clone()
}

#[test]
fn self_is_the_focused_window_resolved_before_the_journal() {
    let (mut state, screen) = station();
    let w = add(&mut state, "DMX");
    let out = run(&mut state, "Set Self \"Universe\" + 1");
    assert_eq!(setting(&state, w, "universe"), Some(json!(2)), "{out}");
    assert!(out.contains(&format!("Window {screen}.{w}")), "{out}");
    assert_eq!(state.resolved_line.as_deref(), Some(format!("Set Window {screen}.{w} \"Universe\" + 1").as_str()));
    // Quoted text is never Self.
    let resolved = crate::self_ref::resolve_line(&state, "Label Self \"Self\"").unwrap().unwrap();
    assert_eq!(resolved, format!("Label Window {screen}.{w} \"Self\""));
    assert_eq!(crate::self_ref::resolve_line(&state, "Set Window 1 \"x\" 2").unwrap(), None);
    // Bare `Self` calls the window; EditSetting / Label / Delete route there.
    run(&mut state, "Label Self \"Busk\"");
    assert_eq!(window(&state, w).label.as_deref(), Some("Busk"));
    state.ui_requests.clear();
    run(&mut state, "EditSetting Self");
    assert!(state.ui_requests.iter().any(|e| matches!(e, UiEvent::EditSettings { target, .. } if *target == format!("Window {screen}.{w}"))));
}

#[test]
fn self_without_an_owner_is_e_cmd_016_never_a_guess() {
    let mut state = AppState::new();
    for line in ["EditSetting Self", "Label Self \"Busk\"", "Set Self \"Universe\" 2", "Reset Self", "Self"] {
        let out = run(&mut state, line);
        assert!(out.contains("E-CMD-016") && out.contains("Self has no owner here"), "{line}: {out}");
    }
    // A focused window that's gone isn't one either; macros and plugins
    // aren't owners until their runtimes exist.
    let (mut state, _) = station();
    let w = add(&mut state, "DMX");
    state.line_owner = Some(LineOwner::Macro(4));
    assert!(run(&mut state, "Set Self \"Universe\" 3").contains("E-CMD-016"));
    state.line_owner = Some(LineOwner::Plugin("x".into()));
    assert!(run(&mut state, "Set Self \"Universe\" 3").contains("E-CMD-016"));
    // A keyboard key acts on the focused window, never on itself.
    state.line_owner = Some(LineOwner::Key("user.f5".into()));
    run(&mut state, "Set Self \"Universe\" 3");
    assert_eq!(setting(&state, w, "universe"), Some(json!(3)));
    assert!(crate::errors::ERRORS.iter().any(|e| e.code == "E-CMD-016"));
}

#[test]
fn a_bar_line_names_its_window_whatever_has_focus() {
    let (mut state, screen) = station();
    let dmx = add(&mut state, "DMX");
    let other = add(&mut state, "DMX");
    assert_eq!(state.current_window, Some(other));
    state.line_owner = Some(LineOwner::Window { screen, no: dmx });
    run(&mut state, "Set Self \"Universe\" + 1");
    state.line_owner = None;
    assert_eq!(setting(&state, dmx, "universe"), Some(json!(2)));
    assert_eq!(setting(&state, other, "universe"), None, "the focused window is untouched");
}

#[test]
fn a_member_forwards_the_resolved_line() {
    let (mut state, screen) = station();
    let w = add(&mut state, "Groups");
    let (link, rx) = crate::session_link::LinkHandle::test_handle("127.0.0.1:1");
    state.link = Some(link);
    // A line the master runs (Fixture … At) with a Self part.
    let _ = dispatch(&mut state, "Fixture 1 At 50; Set Self \"Columns\" 12");
    // Every statement is its own step: each goes to the master on its own.
    let lines: Vec<String> = std::iter::from_fn(|| rx.try_recv().ok())
        .filter_map(|sent| match sent {
            crate::session_link::LinkOut::Command { line, .. } => Some(line),
            _ => None,
        })
        .collect();
    assert_eq!(lines.first().map(String::as_str), Some("Fixture 1 At 50"));
    assert!(lines.iter().all(|l| !l.contains("Self")), "Self resolved here: {lines:?}");
    let _ = (screen, w);
}

#[test]
fn set_window_takes_relative_default_toggle_and_now() {
    let (mut state, _) = station();
    let dmx = add(&mut state, "DMX");
    run(&mut state, "Set Self \"Universe\" 4");
    run(&mut state, "Set Self \"Universe\" * 2");
    assert_eq!(setting(&state, dmx, "universe"), Some(json!(8)));
    run(&mut state, "Set Self \"Universe\" - 20");
    assert_eq!(setting(&state, dmx, "universe"), Some(json!(1)), "a universe stays ≥ 1");
    assert!(run(&mut state, "Set Self \"Columns\" + abc").contains("Error"));
    assert!(run(&mut state, "Set Self \"Columns\" / 0").contains("E-CMD-008"));
    run(&mut state, "Set Self \"View\" Toggle");
    assert_eq!(setting(&state, dmx, "view"), Some(json!("input")));
    run(&mut state, "Set Self \"Readout\" Decimal");
    let out = run(&mut state, "Reset Self \"Readout\"");
    assert_eq!(setting(&state, dmx, "readout"), None, "{out}");
    assert!(out.contains("(default)"), "{out}");
    assert!(run(&mut state, "Set Self \"Universe\" Toggle").contains("E-CMD-008"));
    // The bar's items: checked against the type, Default = the type's bar.
    assert!(run(&mut state, "Set Self \"WindowBarItems\" \"universe,bogus\"").contains("E-SET-003"));
    run(&mut state, "Set Self \"WindowBarItems\" \"next, prev,universe\"");
    assert_eq!(setting(&state, dmx, "window_bar_items"), Some(json!(["next", "prev", "universe"])));
    run(&mut state, "Reset Self \"WindowBarItems\"");
    assert_eq!(setting(&state, dmx, "window_bar_items"), None);
    run(&mut state, "Set Self \"WindowBar\" Toggle");
    assert_eq!(setting(&state, dmx, "window_bar"), Some(json!(false)));

    // System Monitor: Pause = AutoScroll, Clear = a view-local mark.
    let sm = add(&mut state, "System Monitor");
    run(&mut state, "Set Self \"AutoScroll\" Toggle");
    assert_eq!(setting(&state, sm, "auto_scroll"), Some(json!(false)));
    let before = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
    run(&mut state, "Set Self \"Since\" Now");
    let since = setting(&state, sm, "since").and_then(|v| v.as_u64()).unwrap();
    assert!(since >= before, "a wall-clock mark (ms), it survives a restart");
    run(&mut state, "Set Self \"Search\" \"act 2\"");
    run(&mut state, "Set Self \"Search\" \"\"");
    assert_eq!(setting(&state, sm, "search"), Some(json!("")));
}

#[test]
fn reset_self_puts_the_type_defaults_back_as_one_step() {
    let (mut state, _) = station();
    let w = add(&mut state, "DMX");
    run(&mut state, "Label Self \"Front\"");
    run(&mut state, "Set Self \"Universe\" 3");
    run(&mut state, "Set Self \"WindowBarItems\" \"first\"");
    super::screens::sync(&mut state);
    let steps = state.undo_stack("Admin").map_or(0, |s| s.frames.len());
    let out = run(&mut state, "Reset Self");
    assert!(out.contains("back to the DMX defaults"), "{out}");
    let win = window(&state, w);
    assert!(win.settings.is_empty());
    assert_eq!(win.label.as_deref(), Some("Front"), "the label stays");
    super::screens::sync(&mut state);
    assert_eq!(state.undo_stack("Admin").map_or(0, |s| s.frames.len()), steps + 1, "one Oops step");
    // A bare Reset is still the programmer's.
    assert!(!run(&mut state, "Reset").contains("Error"));
}

#[test]
fn resetting_a_profiles_windows_asks_when_more_than_one() {
    let (mut state, _) = station();
    let a = add(&mut state, "DMX");
    let b = add(&mut state, "DMX");
    run(&mut state, &format!("Set Window {a} \"Universe\" 3"));
    run(&mut state, &format!("Set Window {b} \"Universe\" 4"));
    super::screens::sync(&mut state);
    match dispatch(&mut state, "Reset UserProfile 2 Window /Type DMX") {
        DispatchResult::Ask(ask) => {
            assert_eq!(ask.title(), "Reset 2 DMX windows?");
            assert_eq!(ask.primary_line().as_deref(), Some("Reset UserProfile Admin Window /Type dmx /NoConfirmation"));
        }
        _ => panic!("no ask"),
    }
    assert_eq!(setting(&state, a, "universe"), Some(json!(3)), "nothing before the answer");
    let out = run(&mut state, "Reset UserProfile Admin Window /Type DMX /NoConfirmation");
    assert!(out.contains("2 DMX window(s)"), "{out}");
    assert_eq!(setting(&state, a, "universe"), None);
    assert_eq!(setting(&state, b, "universe"), None);
    let saved = &state.root.show_data.user_profiles[&2].content.screen_configs;
    assert!(saved.values().flat_map(|c| c.screens.values()).flat_map(|s| s.windows.values()).filter(|w| w.kind == "dmx").all(|w| w.settings.is_empty()));
}

#[test]
fn a_window_preset_stores_loads_and_survives_the_show_file() {
    let (mut state, _) = station();
    let a = add(&mut state, "DMX");
    run(&mut state, "Set Self \"Universe\" 3");
    run(&mut state, "Set Self \"Readout\" Decimal");
    run(&mut state, "Label Self \"Big\"");
    let out = run(&mut state, "Store WindowPreset \"Big DMX\"");
    assert!(out.contains("DMX preset 1 \"Big DMX\" stored"), "{out}");
    let guid = state.root.show_data.user_profiles[&2].content.window_presets["dmx"][&1].meta.identity.guid;
    assert_eq!(window(&state, a).preset, Some(guid), "the window is the preset's copy");

    let b = add(&mut state, "DMX");
    let out = run(&mut state, "WindowPreset 1");
    assert!(out.contains("→ Window"), "{out}");
    let wb = window(&state, b);
    assert_eq!(wb.settings.get("universe"), Some(&json!(3)));
    assert_eq!(wb.label.as_deref(), Some("Big"));
    assert_eq!(wb.preset, Some(guid));
    // A copy: changing the window doesn't touch the preset.
    run(&mut state, "Set Self \"Universe\" 9");
    assert_eq!(state.root.show_data.user_profiles[&2].content.window_presets["dmx"][&1].settings["universe"], json!(3));
    state.ui_requests.clear();
    run(&mut state, "EditSetting Self");
    let subtitle = state.ui_requests.iter().find_map(|e| match e {
        UiEvent::EditSettings { subtitle, .. } => Some(subtitle.clone()),
        _ => None,
    });
    assert_eq!(subtitle.as_deref(), Some("from preset 1 \"Big DMX\" · changed"));

    // Overwriting asks; /Overwrite keeps the identity.
    assert_eq!(run(&mut state, "Store WindowPreset 1"), "ASK Overwrite preset 1 \"Big DMX\"?");
    run(&mut state, "Store WindowPreset 1 /Overwrite");
    let p = &state.root.show_data.user_profiles[&2].content.window_presets["dmx"][&1];
    assert_eq!((p.meta.identity.guid, p.settings["universe"].clone(), p.meta.name().unwrap_or_default()), (guid, json!(9), "Big DMX"));
    // Bare words ask (the pick list).
    assert_eq!(run(&mut state, "Call WindowPreset"), "ASK Load preset · DMX");
    assert_eq!(run(&mut state, "Store WindowPreset"), "ASK Save preset · DMX");
    run(&mut state, "Label WindowPreset 1 \"Patch check\"");
    assert!(run(&mut state, "List WindowPreset").contains("Patch check"));
    assert!(run(&mut state, "List WindowPreset /Type \"System Monitor\"").contains("No window presets"));
    assert!(run(&mut state, "UserProfile Admin WindowPreset 1").contains("→ Window"), "owner first");

    // The show file keeps them (format 10).
    let (bytes, _) = lumacore_luma::show_file::encode(&state.root.show_data).unwrap();
    let back = lumacore_luma::show_file::from_bytes(&bytes).unwrap();
    assert_eq!(back.user_profiles[&2].content.window_presets, state.root.show_data.user_profiles[&2].content.window_presets);
    assert!(back.identity().duplicates.is_empty());

    run(&mut state, "Delete WindowPreset 1 /NoConfirmation");
    assert!(state.root.show_data.user_profiles[&2].content.window_presets.is_empty());
    // No window, no type: a preset line says which it needs.
    state.current_window = None;
    assert!(run(&mut state, "Delete WindowPreset 1 /nc").contains("E-CMD-006"));
}

#[test]
fn views_screenconfigs_and_presets_round_trip_as_json() {
    let (mut state, _) = station();
    let _ = std::fs::remove_dir_all(super::ui_files::dir(""));
    add(&mut state, "DMX");
    run(&mut state, "Set Self \"Universe\" 5");
    run(&mut state, "Store WindowPreset \"Json DMX\"");
    let out = run(&mut state, "Export WindowPreset 1");
    assert!(out.contains("windowpresets/Json DMX.json"), "{out}");
    let text = std::fs::read_to_string(format!("{}/Json DMX.json", super::ui_files::dir("windowpresets"))).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!((v["kind"].as_str(), v["format"].as_u64(), v["type"].as_str()), (Some("WindowPreset"), Some(u64::from(lumacore_tree::FORMAT_VERSION)), Some("dmx")));
    let out = run(&mut state, "Import WindowPreset \"Json DMX\"");
    assert!(out.contains("preset 2") && out.contains("a copy: new GUIDs"), "{out}");
    let pool = &state.root.show_data.user_profiles[&2].content.window_presets["dmx"];
    assert_ne!(pool[&1].meta.identity.guid, pool[&2].meta.identity.guid);
    assert_eq!(pool[&2].settings["universe"], json!(5));

    run(&mut state, "Store View 3 \"Busk\"");
    assert!(run(&mut state, "Export View 3").contains("views/Busk.json"));
    let out = run(&mut state, "Import View \"Busk\"");
    assert!(out.contains("View 4") && out.contains("copy"), "{out}");
    let views = &state.root.show_data.user_profiles[&2].content.views;
    assert_eq!(views[&4].screens.len(), views[&3].screens.len());

    assert!(state.root.show_data.identity().duplicates.is_empty());
    // Another kind or format is refused.
    std::fs::write(format!("{}/Old.json", super::ui_files::dir("views")), r#"{"kind":"View","format":3}"#).unwrap();
    assert!(run(&mut state, "Import View \"Old\"").contains("E-SHOW-002"));
    assert!(run(&mut state, "Import View \"../x\"").contains("E-CMD-012"));
}

#[test]
fn the_settings_window_has_keys_resets_and_the_bar_list() {
    let (mut state, screen) = station();
    let w = add(&mut state, "DMX");
    run(&mut state, "Set Self \"Readout\" Hex");
    state.ui_requests.clear();
    run(&mut state, &format!("EditSetting Window {w}"));
    let (title, fields) = state.ui_requests.iter().find_map(|e| match e {
        UiEvent::EditSettings { title, fields, .. } => Some((title.clone(), fields.clone())),
        _ => None,
    }).unwrap();
    assert_eq!(title, format!("Window {screen}.{w} · DMX"));
    let readout = fields.iter().find(|f| f.key == "readout").unwrap();
    assert!(readout.changed);
    assert_eq!(readout.reset, format!("Reset Window {screen}.{w} \"Readout\""));
    assert!(!fields.iter().find(|f| f.key == "universe").unwrap().changed);
    let bar = fields.iter().find(|f| f.key == "window_bar_items").unwrap();
    assert_eq!(bar.section, "Window bar");
    assert_eq!(bar.value, "universe,prev,next,readout");
    let FieldInput::List { items } = &bar.input else { panic!("{:?}", bar.input) };
    let next = items.iter().find(|i| i.id == "next").unwrap();
    assert_eq!((next.kind.as_str(), next.line.as_str()), ("action", "Set Self \"Universe\" + 1"));
    assert!(items.iter().any(|i| i.id == "readout" && i.kind == "setting"));
}

#[test]
fn bare_selects_and_a_bare_profile_ask_with_the_pick_list() {
    let mut state = AppState::new();
    assert_eq!(run(&mut state, "Select DataPool"), "ASK Select DataPool");
    match dispatch(&mut state, "Select DataPool") {
        DispatchResult::Ask(ask) => {
            let line = ask.primary_line().unwrap();
            assert!(line.starts_with("Select DataPool "), "{line}");
            assert!(matches!(&ask.view().inputs[0].kind, lumacore_wire::InputKind::Link { pick } if !pick.items.is_empty()));
        }
        _ => panic!("no ask"),
    }
    match dispatch(&mut state, "Store User \"Phone\" /Profile") {
        DispatchResult::Ask(ask) => {
            // The name keeps its quotes in the answer's line (issue 07).
            assert_eq!(ask.primary_line().as_deref(), Some("Store User \"Phone\" /Profile New"), "the first row: their own copy of Default");
        }
        _ => panic!("no ask"),
    }
}

#[test]
fn page_words_reach_only_a_ui_window() {
    let mut state = AppState::new();
    run(&mut state, "Store User \"LD\" /Rights Program");
    state.user = "LD".into();
    assert!(run(&mut state, "Sidebars").contains("E-CMD-010"));
    state.ui_client = true;
    run(&mut state, "FullScreen");
    assert!(state.ui_requests.iter().any(|e| matches!(e,
        lumacore_wire::UiEvent::Page { what, on: None } if what == "fullscreen")));
}
