// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `realtime`.
//!
//! Owns:          the unit tests of realtime.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §24, §31 "Thread priority".

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

/// Granted or not, the answer says which (it never panics, never empty).
#[test]
fn promote_reports_what_it_got() {
    let got = std::thread::spawn(|| promote_current(DRIVER_PRIORITY)).join().unwrap();
    assert!(got.starts_with("SCHED_FIFO") || got.starts_with("normal") || got.starts_with("highest"), "{got}");
}

#[test]
fn an_out_of_range_priority_is_refused_by_name() {
    if cfg!(target_os = "linux") {
        assert!(promote_current(200).contains("out of range"));
    }
}
