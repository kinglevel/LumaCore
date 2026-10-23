// ██████╗ ██╗███╗   ██╗██████╗     ████████╗███████╗███████╗████████╗███████╗
// ██╔══██╗██║████╗  ██║██╔══██╗    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██████╔╝██║██╔██╗ ██║██║  ██║       ██║   █████╗  ███████╗   ██║   ███████╗
// ██╔══██╗██║██║╚██╗██║██║  ██║       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██████╔╝██║██║ ╚████║██████╔╝       ██║   ███████╗███████║   ██║   ███████║
// ╚═════╝ ╚═╝╚═╝  ╚═══╝╚═════╝        ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: bind tests

//! Tests of `bind`.
//!
//! Owns:          the unit tests of bind.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §17, §30 "12a3", §33 91.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use crate::lookup::bindings;

#[test]
fn bind_makes_a_user_key_and_takes_effect_at_once() {
    let mut state = AppState::new();
    let out = bind(&mut state, &["F6", "`List Fixture`"]);
    assert!(out.contains("user.f6"), "{out}");
    let b = &bindings(&state)["F6"];
    assert_eq!(b.ui_key, "user.f6");
    let key = find(&state, "user.f6").unwrap();
    assert_eq!(key.press, "List Fixture");
    // A UiKey id binds directly.
    bind(&mut state, &["F7", "Blind"]);
    assert_eq!(bindings(&state)["F7"].ui_key, "blind");
    // Unknown command lines are refused.
    assert!(bind(&mut state, &["F8", "`Frobnicate`"]).contains("E-KEY-001"));
    // Quoted text is never a line (decisions §33 91).
    assert!(bind(&mut state, &["F8", "\"List Fixture\""]).contains("E-SYN-014"));
}
