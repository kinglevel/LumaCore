// ██╗     ██╗███╗   ██╗███████╗███████╗    ████████╗███████╗███████╗████████╗███████╗
// ██║     ██║████╗  ██║██╔════╝██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║     ██║██╔██╗ ██║█████╗  ███████╗       ██║   █████╗  ███████╗   ██║   ███████╗
// ██║     ██║██║╚██╗██║██╔══╝  ╚════██║       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ███████╗██║██║ ╚████║███████╗███████║       ██║   ███████╗███████║   ██║   ███████║
// ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: lines tests

//! Tests of `lines`.
//!
//! Owns:          the unit tests of lines.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §11 "Step 4b/4c hardening", §24 "Step 9 hardening".

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::{frame_cadence, frames_per_send, period_of};

#[test]
fn a_rate_spans_whole_engine_frames() {
    assert_eq!(frames_per_send(period_of(60)), 1);
    assert_eq!(frames_per_send(period_of(30)), 2);
    assert_eq!(frames_per_send(period_of(20)), 3);
    assert_eq!(frames_per_send(period_of(44)), 2);
    assert_eq!(frames_per_send(period_of(1)), 60);
}

#[test]
fn doubled_and_skipped_frames_are_counted() {
    // A 30 Hz line: every 2nd frame is on time.
    assert_eq!(frame_cadence(10, 12, 2), (0, 0));
    assert_eq!(frame_cadence(10, 10, 2), (1, 0), "the same frame again");
    assert_eq!(frame_cadence(10, 11, 2), (0, 0), "early is not a skip");
    assert_eq!(frame_cadence(10, 13, 2), (0, 1), "one frame jumped over");
    // Nothing to compare: no engine, a first send, an engine restart.
    assert_eq!(frame_cadence(0, 5, 2), (0, 0));
    assert_eq!(frame_cadence(5, 0, 2), (0, 0));
    assert_eq!(frame_cadence(50, 3, 2), (0, 0));
}
