// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The keys line in, reply out: the core keys can't be bound, a key line's flags and bare
//! numbers are checked on save, a binding is a Keys Oops step.
//!
//! Owns:          the tests of the keys' words.
//! Doesn't own:   the checks' unit tests (lines/tests.rs, bind/tests.rs).
//! Entry points:  the #[test] fns.
//! Decisions:     §17, §29 "11c", §30 "12a3", §33 21.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match lumacore_objects::commands::dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => panic!("'{line}' did not produce output"),
    }
}

#[test]
fn core_keys_are_f9_to_f12_and_cant_be_bound() {
    let mut state = AppState::new();
    let out = run(&mut state, "Bind F9 `List Fixture`");
    assert!(out.contains("E-KEY-005"), "{out}");
    assert!(run(&mut state, "Bind Alt-F12 `List Fixture`").contains("E-KEY-005"));
    assert!(
        !run(&mut state, "Bind F5 `List Fixture`").contains("Error"),
        "F1–F8 are free"
    );
    let keys = lumacore_tree::builtin_ui_keys();
    let lock = keys.values().find(|k| k.id == "desklock").unwrap();
    assert_eq!(lock.press, "DeskLock");
    assert_eq!(lock.meow.press, "Toggle NormalTheme");
    assert_eq!(
        lock.meow.release_after_hold, "Toggle NormalTheme",
        "held = while held"
    );
    let vanilla = lumacore_base::keybindings::VANILLA;
    for key in ["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8"] {
        assert!(!vanilla.iter().any(|(k, ..)| *k == key), "{key} stays free");
    }
}

#[test]
fn a_key_line_with_an_unknown_flag_is_refused_on_save() {
    let mut state = AppState::new();
    let out = run(&mut state, "Bind F5 `Store Group 3 /Foo`");
    assert!(out.contains("isn't an option"), "{out}");
    assert!(!run(&mut state, "Bind F5 `Store Group 3 /Overwrite`").contains("isn't an option"));
}

#[test]
fn a_stored_line_names_its_object() {
    let problems = crate::check_line("3 At 50");
    assert!(
        problems
            .iter()
            .any(|p| p.as_ref().is_err_and(|e| e.contains("E-SYN-008"))),
        "{problems:?}"
    );
    assert!(crate::check_line("Fixture 3 At 50").is_empty());
}

/// A binding is an Oops step of its own category (decisions §29 "11c").
#[test]
fn bind_is_a_keys_oops_step() {
    let mut state = AppState::new();
    run(&mut state, "Bind F5 `List Fixture`");
    let undo = state.undo_stack("Admin").expect("Admin's list");
    let newest = undo.frames.back().expect("a step");
    assert_eq!(newest.category, lumacore_tree::OopsCategory::Keys);
}

fn user_key(state: &AppState, id: &str) -> lumacore_tree::UiKey {
    state.root.show_data.profile_for_user(state.effective_user_name()).unwrap().content.ui_keys.values().find(|k| k.id == id).cloned().unwrap()
}

/// A letter types in KBD (decisions 0295): it binds only in a mode where keys are words, and
/// several lines are a press cycle.
#[test]
fn a_letter_binds_in_a_mode_as_a_press_cycle() {
    use lumacore_tree::keybindings::Layer;
    let mut state = lumacore_objects::demo::fresh_state();
    assert!(run(&mut state, "Bind \"E\" `Edit`").contains("E-KEY-006"));
    assert!(run(&mut state, "Bind \"E\" `Edit` /Shortcuts \"KBD\"").contains("E-KEY-006"));
    let out = run(&mut state, "Bind \"E\" `Edit` `EditSetting` /Shortcuts \"PRG\"");
    assert!(out.contains("user.prg.e"), "{out}");
    let key = user_key(&state, "user.prg.e");
    assert_eq!((key.press.as_str(), key.cycle.as_slice()), ("Edit", ["EditSetting".to_string()].as_slice()));
    assert_eq!(crate::lookup::bindings_in(&state, Layer::Programmer)["E"].ui_key, "user.prg.e");
    assert!(!crate::lookup::bindings_in(&state, Layer::Keyboard).contains_key("E"));
    // Ctrl and Alt combinations bind in every mode.
    assert!(!run(&mut state, "Bind \"Ctrl-E\" `Edit`").contains("E-KEY"));
}

/// Unbind takes your binding away (LumaCore's comes back); with none of yours it turns LumaCore's
/// off on your profile, and again brings it back.
#[test]
fn unbind_turns_lumacores_key_off_and_back() {
    use lumacore_tree::keybindings::Layer;
    let mut state = lumacore_objects::demo::fresh_state();
    let prg = |s: &AppState| crate::lookup::bindings_in(s, Layer::Programmer).get("E").map(|b| b.ui_key.clone());
    assert_eq!(prg(&state).as_deref(), Some("prg.e"));
    assert!(run(&mut state, "Unbind \"E\" /Shortcuts \"PRG\"").contains("off on this profile"));
    assert_eq!(prg(&state), None);
    assert!(run(&mut state, "Unbind \"E\" /Shortcuts \"PRG\"").contains("is back"));
    assert_eq!(prg(&state).as_deref(), Some("prg.e"));
    assert!(run(&mut state, "Unbind \"F6\"").contains("E-KEY-003"));
    assert!(run(&mut state, "Unbind \"F10\"").contains("E-KEY-005"));
}

/// A profile key's lines are its properties: Press takes a cycle, Types the presses that type;
/// a LumaCore key is locked.
#[test]
fn a_keys_press_cycle_and_its_typing_presses_are_set_line_by_line() {
    let mut state = lumacore_objects::demo::fresh_state();
    run(&mut state, "Bind \"F5\" `List Fixture`");
    let out = run(&mut state, "Set UIKey \"user.f5\" \"Press\" `Copy` `Paste` `Insert`");
    assert!(!out.contains("E-"), "{out}");
    let out = run(&mut state, "Set UIKey \"user.f5\" \"Types\" 1 Thru 2");
    assert!(!out.contains("E-"), "{out}");
    let key = user_key(&state, "user.f5");
    assert_eq!(key.press_rows(false).iter().map(|r| (r.line.as_str(), r.types)).collect::<Vec<_>>(), [("Copy", true), ("Paste", true), ("Insert", false)]);
    assert!(run(&mut state, "Set UIKey \"user.f5\" \"Types\" 4").contains("E-KEY-007"));
    assert!(run(&mut state, "Set UIKey \"user.f5\" \"Press\" `Frobnicate`").contains("E-KEY-001"));
    assert!(run(&mut state, "Set UIKey \"user.f5\" \"MeowPress\" `Cut`").contains("Cut"));
    assert_eq!(user_key(&state, "user.f5").meow.press, "Cut");
    assert!(run(&mut state, "Set UIKey \"prg.e\" \"Press\" `Oops`").contains("E-KEY-007"));
}

/// The Programmer's keys are LumaCore's own rows (decisions 0295): E cycles Edit › EditSetting,
/// a dot then a dot runs Zero, Delete clears level by level and held clears all.
#[test]
fn the_programmers_keys_are_lumacores_rows() {
    let keys = lumacore_tree::builtin_ui_keys();
    let key = |id: &str| keys.values().find(|k| k.id == id).unwrap().clone();
    let rows = |id: &str| key(id).press_rows(false).into_iter().map(|r| (r.line, r.types)).collect::<Vec<_>>();
    assert_eq!(rows("prg.e"), [("Edit".to_string(), true), ("EditSetting".to_string(), true)]);
    assert_eq!(rows("prg.alt-a"), [("Assign".to_string(), true), ("Label".to_string(), true), ("Paint".to_string(), true)]);
    assert_eq!(rows("prg.dot"), [(".".to_string(), true), ("Zero".to_string(), false)]);
    assert_eq!(rows("prg.delete").len(), 3);
    assert_eq!(key("prg.delete").hold, "ResetAll");
    assert_eq!(key("prg.p").press_rows(true)[0].line, "FeatureGroup");
}
