// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `election`.
//!
//! Owns:          the unit tests of election.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §21 "6c", "6d", §34 19.3.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn c(name: &str, priority: StationPriority, applied: u64, online: u64) -> Candidate {
    Candidate {
        name: name.into(),
        station_id: name.into(),
        priority,
        applied,
        online_since_ms: online,
        ip: String::new(),
        stream_port: 0,
        me: false,
    }
}

#[test]
fn priority_then_results_then_online_time_then_id() {
    use StationPriority::*;
    assert!(ranks_above(&c("a", High, 0, 9_000_000), &c("b", Normal, 99, 0)));
    assert!(ranks_above(&c("a", Normal, 10, 9_000_000), &c("b", Normal, 9, 0)));
    // Online time counts in whole 5 s buckets.
    assert!(ranks_above(&c("b", Normal, 10, 0), &c("a", Normal, 10, 6000)));
    assert!(ranks_above(&c("a", Normal, 10, 4000), &c("b", Normal, 10, 0)));
    assert!(ranks_above(&c("b", Normal, 10, 4999), &c("a", Normal, 10, 5000)));
}
