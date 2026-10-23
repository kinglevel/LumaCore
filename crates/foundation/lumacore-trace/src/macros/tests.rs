// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests
//! Tests of `macros`: every form compiles where it is used, and the zone lives to the end of its
//! scope.
//!
//! Owns:          the unit tests of macros.rs
//! Doesn't own:   the ring buffer (collector/tests.rs)
//! Entry points:  the #[test] fns
//! Decisions:     §34 20

use std::time::{Duration, Instant};

use crate::macros::{HeldZone, record_complete};

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[test]
fn every_macro_expands_in_a_caller() {
    crate::trace_zone!("test_zone");
    crate::tracy_frame_mark!();
    crate::tracy_plot!("test_plot", 3_u32);
    crate::tracy_message!("test_message");
    let _span = crate::tracy_zone!("test_span");
    let held: HeldZone = crate::held_zone!("test_held");
    assert!(held.is_none() || cfg!(feature = "tracy"));
    record_complete("test_complete", "test", Instant::now(), Duration::from_micros(5));
}

#[cfg(feature = "perfetto")]
#[test]
fn a_trace_zone_records_when_its_scope_ends() {
    crate::TRACE_COLLECTOR.init();
    let before = crate::TRACE_COLLECTOR.snapshot(None).iter().filter(|e| e.name == "scoped_zone").count();
    {
        crate::trace_zone!("scoped_zone");
        let during = crate::TRACE_COLLECTOR.snapshot(None).iter().filter(|e| e.name == "scoped_zone").count();
        assert_eq!(during, before, "the zone is still open");
    }
    let after = crate::TRACE_COLLECTOR.snapshot(None).iter().filter(|e| e.name == "scoped_zone").count();
    assert_eq!(after, before + 1);
}
