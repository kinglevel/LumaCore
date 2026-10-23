// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `dispatch`: nested lines count one level deeper, and past `MAX_NESTED_DEPTH` they are
//! refused (E-CMD-019) instead of overflowing the stack.
//!
//! Owns:          the unit tests of dispatch.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 16.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn reply(result: DispatchResult) -> crate::Reply {
    match result {
        DispatchResult::Output(r) => r,
        _ => panic!("no output"),
    }
}

#[test]
fn a_nested_line_counts_one_level_and_gives_it_back() {
    let mut state = AppState::new();
    state.dispatch_depth = 3;
    let r = reply(dispatch(&mut state, "Info Error E-CMD-019"));
    assert!(!r.failed, "{}", r.text);
    assert_eq!(state.dispatch_depth, 3);
}

#[test]
fn lines_nested_too_deep_are_refused() {
    let mut state = AppState::new();
    state.dispatch_depth = MAX_NESTED_DEPTH;
    let r = reply(dispatch(&mut state, "Info Error E-CMD-019"));
    assert!(r.failed);
    assert_eq!(r.code.as_deref(), Some("E-CMD-019"), "{}", r.text);
    assert_eq!(state.dispatch_depth, MAX_NESTED_DEPTH);
}
