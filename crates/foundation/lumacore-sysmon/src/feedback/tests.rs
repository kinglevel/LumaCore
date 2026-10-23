// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `feedback`.
//!
//! Owns:          the unit tests of feedback.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12b2", §34 25.10.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn progress_is_active_until_dropped() {
    let p = Progress::start("Saving showfile", category::SHOW, Importance::Important);
    let id = p.id();
    let key = format!("progress.{id}");
    assert!(ACTIVE.lock().unwrap().contains_key(&key));
    drop(p);
    assert!(!ACTIVE.lock().unwrap().contains_key(&key));
}

#[test]
fn status_emits_changes_only_and_user_filter_works() {
    status("test_hi", true, "HI", "◐", Level::Warn);
    status("test_hi", true, "HI", "◐", Level::Warn);
    let active = active_json("anyone");
    assert_eq!(
        active
            .iter()
            .filter(|j| j.contains("\"key\":\"test_hi\""))
            .count(),
        1
    );
    status("test_hi", false, "HI", "◐", Level::Warn);
    assert!(!active_json("anyone").iter().any(|j| j.contains("test_hi")));
}

#[test]
fn importance_orders_by_severity() {
    assert!(
        Importance::Critical > Importance::Important
            && Importance::Important > Importance::Normal
    );
}
