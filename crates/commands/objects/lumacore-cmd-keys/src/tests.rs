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
