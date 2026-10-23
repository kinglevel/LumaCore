// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `grammar`.
//!
//! Owns:          the unit tests of grammar.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §15, §30 "12b2", §34 25.9.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::{complete_input, hint_input};
use lumacore_objects::AppState;

#[test]
fn cd_hints_root_from_nested_navigation() {
    let mut state = AppState::new();
    state.navigation_path = vec!["ShowData".to_string()];

    let hint = hint_input("cd roo", "cd roo".len(), &state);
    assert_eq!(hint.as_deref(), Some("t"));
}

#[test]
fn cd_completion_offers_root_from_nested_navigation() {
    let mut state = AppState::new();
    state.navigation_path = vec!["ShowData".to_string()];

    let (_, candidates) = complete_input("cd roo", "cd roo".len(), &state);
    assert!(candidates.iter().any(|candidate| candidate == "root"));
}

#[test]
fn fixture_hint_supports_bare_thru_selection() {
    let state = AppState::new();

    let hint = hint_input("fixture th", "fixture th".len(), &state);
    assert_eq!(hint.as_deref(), Some("ru"));
}

#[test]
fn partial_fixture_verb_still_hints_thru() {
    let state = AppState::new();

    let hint = hint_input("fixtur th", "fixtur th".len(), &state);
    assert_eq!(hint.as_deref(), Some("ru"));
}

#[test]
fn fixture_completion_offers_thru() {
    let state = AppState::new();

    let (_, candidates) = complete_input("fixture th", "fixture th".len(), &state);
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case("thru"))
    );
}
