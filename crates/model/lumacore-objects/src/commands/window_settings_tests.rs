//! Window settings (decisions §30 "12b2"): `EditSetting Window n` gives typed
//! fields with one tab per section, the DataPool is a GUID link with the
//! follow row, named numbers (`Fit`) and Default font size.

use super::dispatch;
use crate::AppState;
use crate::DispatchResult;
use lumacore_wire::{FieldInput, UiEvent};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// A station with Admin's screen holding one Groups window; returns its number.
fn with_groups_window() -> (AppState, u32) {
    let mut state = AppState::new();
    let screen = state.root.runtime.live_screens.write().unwrap().join_or_create("Admin", None);
    state.current_live_screen = Some(screen);
    let out = run(&mut state, "Store Window \"Groups\"");
    let w = state.root.runtime.live_screens.read().unwrap().get(screen).unwrap().windows.iter()
        .find(|(_, win)| win.kind == "groups").map(|(n, _)| *n).unwrap_or_else(|| panic!("{out}"));
    (state, w)
}

fn fields(state: &mut AppState, w: u32) -> Vec<lumacore_wire::SettingField> {
    state.ui_requests.clear();
    run(state, &format!("EditSetting Window {w}"));
    state.ui_requests.iter().find_map(|e| match e {
        UiEvent::EditSettings { fields, .. } => Some(fields.clone()),
        _ => None,
    }).expect("an EditSettings event")
}

#[test]
fn a_pool_window_has_three_tabs_and_typed_cells() {
    // (plus the Window bar tab every window has since 12c)
    let (mut state, w) = with_groups_window();
    let f = fields(&mut state, w);
    let mut sections: Vec<&str> = f.iter().map(|f| f.section.as_str()).collect();
    sections.dedup();
    assert_eq!(sections, ["Pool", "Layout", "Colours", "Window bar"]);
    let dp = f.iter().find(|f| f.name == "DataPool").unwrap();
    assert_eq!(dp.input, FieldInput::Link);
    assert_eq!(dp.value, "Selected DataPool");
    let pick = dp.pick.as_ref().unwrap();
    assert_eq!(pick.follow.as_deref(), Some("Selected DataPool"));
    assert!(!pick.items.is_empty(), "the show's datapools");
    let cols = f.iter().find(|f| f.name == "Columns").unwrap();
    assert!(matches!(&cols.input, FieldInput::Number { named, .. } if named == &["Fit"]));
    let font = f.iter().find(|f| f.name == "Font size").unwrap();
    assert_eq!(font.value, "Default");
    assert!(dp.line.starts_with("Set Window ") && dp.line.ends_with(" \"Datapool\" {value}"), "{}", dp.line);
}

#[test]
fn the_datapool_is_stored_by_guid_and_selected_follows() {
    let (mut state, w) = with_groups_window();
    let (no, guid) = state.root.show_data.datapools.iter().next().map(|(n, d)| (*n, d.meta.identity.guid.to_string())).unwrap();
    let out = run(&mut state, &format!("Set Window {w} \"DataPool\" {no}"));
    let stored = |state: &AppState| {
        let store = state.root.runtime.live_screens.read().unwrap();
        let s = store.get(state.current_live_screen.unwrap()).unwrap();
        s.windows[&w].settings.get("datapool").cloned()
    };
    assert_eq!(stored(&state), Some(serde_json::json!(guid)), "{out}");
    let f = fields(&mut state, w);
    assert!(f.iter().find(|f| f.name == "DataPool").unwrap().value.starts_with(&no.to_string()));
    run(&mut state, &format!("Set Window {w} \"DataPool\" Selected"));
    assert_eq!(stored(&state), Some(serde_json::json!("selected")));
    run(&mut state, &format!("Set Window {w} \"Columns\" Fit"));
    let f = fields(&mut state, w);
    assert_eq!(f.iter().find(|f| f.name == "Columns").unwrap().value, "Fit");
    assert!(run(&mut state, &format!("Set Window {w} \"EmptyPaint\" \"pink\"")).contains("E-SET-003"));
}

#[test]
fn a_profiles_preferences_are_typed_settings_on_six_tabs() {
    let mut state = AppState::new();
    state.ui_requests.clear();
    run(&mut state, "EditSetting UserProfile 2");
    let f = state.ui_requests.iter().find_map(|e| match e {
        UiEvent::EditSettings { fields, .. } => Some(fields.clone()),
        _ => None,
    }).expect("an EditSettings event");
    let mut sections: Vec<&str> = f.iter().map(|f| f.section.as_str()).collect();
    sections.sort();
    sections.dedup();
    // Meta: the facts every show object has (decisions §12 "Tags hardened" 8).
    assert_eq!(sections, ["Attributes", "Command line", "Input & keys", "Lock screen", "Look", "MVR import", "Meta", "Oops", "Update"]);
    let meow = f.iter().find(|f| f.name == "MeowKey").unwrap();
    assert_eq!(meow.value, "Shift", "Shift is the default Meow key");
    assert!(matches!(&meow.input, FieldInput::Choice { options } if options.len() == 3));
    let out = run(&mut state, "Set UserProfile 2 \"MeowKey\" Alt");
    assert_eq!(state.root.show_data.user_profiles[&2].settings.preferences.meow_key, "Alt", "{out}");
    run(&mut state, "Set UserProfile 2 \"OopsConfirmation\" Main");
    assert_eq!(state.root.show_data.user_profiles[&2].settings.preferences.oops.confirmation, lumacore_tree::OopsConfirmation::Main);
    // The Theme, its Paint and the LockScreen are the profile's children, owner first (§33 60).
    let base = f.iter().find(|f| f.name == "Base").unwrap();
    assert_eq!(base.line, "Set UserProfile 2 Theme \"Base\" {value}");
    assert_eq!(base.section, "Look");
    let accent = f.iter().find(|f| f.name == "Accent").unwrap();
    assert_eq!(accent.line, "Set UserProfile 2 Theme Paint \"Accent\" {value}");
    assert_eq!(f.iter().find(|f| f.name == "AutoLock").unwrap().section, "Lock screen");
    let out = run(&mut state, &base.line.replace("{value}", "Night"));
    assert_eq!(state.root.show_data.user_profiles[&2].settings.theme.preset, "Night", "{out}");
}

/// `Theme` typed alone opens your own theme's settings (decisions §33 63).
#[test]
fn a_theme_typed_alone_opens_its_settings() {
    let mut state = AppState::new();
    state.ui_requests.clear();
    let out = run(&mut state, "Theme");
    let f = state.ui_requests.iter().find_map(|e| match e {
        UiEvent::EditSettings { fields, .. } => Some(fields.clone()),
        _ => None,
    }).unwrap_or_else(|| panic!("{out}"));
    assert_eq!(f.iter().find(|f| f.name == "Base").unwrap().line, "Set Theme \"Base\" {value}");
    assert!(f.iter().any(|f| f.line == "Set Theme Paint \"Accent\" {value}"));
    // Owner first: another profile's theme.
    state.ui_requests.clear();
    let out = run(&mut state, "EditSetting UserProfile 2 Theme");
    let title = state.ui_requests.iter().find_map(|e| match e {
        UiEvent::EditSettings { title, .. } => Some(title.clone()),
        _ => None,
    }).unwrap_or_else(|| panic!("{out}"));
    assert_eq!(title, "UserProfile 2");
}

#[test]
fn a_text_setting_clears_with_empty_quotes() {
    let mut state = AppState::new();
    let screen = state.root.runtime.live_screens.write().unwrap().join_or_create("Admin", None);
    state.current_live_screen = Some(screen);
    run(&mut state, "Store Window \"System Monitor\"");
    let w = state.root.runtime.live_screens.read().unwrap().get(screen).unwrap().windows.iter()
        .find(|(_, win)| win.kind == "sysmon").map(|(n, _)| *n).unwrap();
    run(&mut state, &format!("Set Window {w} \"subsystems\" \"Echo\""));
    let out = run(&mut state, &format!("Set Window {w} \"subsystems\" \"\""));
    let store = state.root.runtime.live_screens.read().unwrap();
    assert_eq!(store.get(screen).unwrap().windows[&w].settings.get("subsystems"), Some(&serde_json::json!("")), "{out}");
}

/// `Set Screen "EncoderBar"` is saved on the screen config (and survives the live → saved copy);
/// mode, group, page and layer change only the live screen (decisions §30 "12f").
#[test]
fn set_screen_drives_the_encoder_bar() {
    use lumacore_tree::runtime::{EncoderLayer, EncoderMode};
    let mut state = AppState::new();
    let no = state.root.runtime.live_screens.write().unwrap().join_or_create("Admin", None);
    state.current_live_screen = Some(no);
    crate::commands::screens::sync(&mut state);
    let steps = |s: &AppState| s.undo_stack("Admin").map_or(0, |u| u.frames.len());
    let before = steps(&state);
    assert!(run(&mut state, "Set Screen \"EncoderMode\" Masters").contains("Masters"));
    assert!(run(&mut state, "Set Screen \"EncoderGroup\" 3").contains("group 3"));
    let page = run(&mut state, "Set Screen \"EncoderPage\" Next");
    assert!(page.contains("page 2"), "{page}");
    assert!(run(&mut state, "Set Screen \"EncoderLayer\" Next").contains("Fade"));
    assert_eq!(steps(&state), before, "switching the bar is not an Oops step");
    let e = state.root.runtime.live_screens.read().unwrap().get(no).unwrap().encoder.clone();
    assert_eq!((e.mode, e.group, e.page, e.layer), (EncoderMode::Masters, 3, 2, EncoderLayer::Fade));
    assert!(run(&mut state, "Set Screen \"EncoderMode\" \"Party\"").contains("E-SET-003"));
    assert!(run(&mut state, "Set Screen \"EncoderBar\" Off").contains("off"));
    crate::commands::screens::sync(&mut state);
    let profile = state.root.show_data.profile_for_user("Admin").unwrap();
    let saved = profile.content.screen_configs.values().next().unwrap().screens.get(&1).unwrap();
    assert_eq!(saved.encoder_bar.on, Some(false), "kept across the live → saved copy");
}
