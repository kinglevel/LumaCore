// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests for PerfettoGuard RAII trace recording.
//!
//! These tests use the global TRACE_COLLECTOR. Since tests run in parallel,
//! we check relative changes rather than absolute counts.
//!
//! Owns:          the unit tests of guard.rs (PerfettoGuard)
//! Doesn't own:   the ring buffer (collector/tests.rs)
//! Entry points:  the #[test] fns
//! Decisions:     §23 "Step 8"

use crate::collector::TRACE_COLLECTOR;
use crate::guard::PerfettoGuard;

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[test]
fn guard_records_event_on_drop() {
    TRACE_COLLECTOR.init();
    let before = TRACE_COLLECTOR.len();

    {
        let _guard = PerfettoGuard::new("test_zone", "test");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    let after = TRACE_COLLECTOR.len();
    assert!(
        after > before,
        "PerfettoGuard should record an event on drop"
    );

    let events = TRACE_COLLECTOR.snapshot(None);
    let found = events
        .iter()
        .any(|e| e.name == "test_zone" && e.category == "test" && e.phase == 'X');
    assert!(found, "should find 'test_zone' event in snapshot");
}

#[test]
fn guard_measures_duration_accurately() {
    TRACE_COLLECTOR.init();

    {
        let _guard = PerfettoGuard::new("sleep_5ms", "bench");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let events = TRACE_COLLECTOR.snapshot(None);
    let event = events.iter().find(|e| e.name == "sleep_5ms").unwrap();
    assert!(
        event.duration_us >= 4000,
        "expected >= 4ms, got {}µs",
        event.duration_us
    );
    assert!(
        event.duration_us < 50_000,
        "expected < 50ms, got {}µs",
        event.duration_us
    );
}

#[test]
fn guard_nested_records_both() {
    TRACE_COLLECTOR.init();
    let before = TRACE_COLLECTOR.len();

    {
        let _outer = PerfettoGuard::new("nested_outer", "test");
        {
            let _inner = PerfettoGuard::new("nested_inner", "test");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    let after = TRACE_COLLECTOR.len();
    assert!(
        after >= before + 2,
        "nested guards should record 2 events, before={before} after={after}"
    );

    let events = TRACE_COLLECTOR.snapshot(None);
    assert!(
        events.iter().any(|e| e.name == "nested_inner"),
        "should find inner event"
    );
    assert!(
        events.iter().any(|e| e.name == "nested_outer"),
        "should find outer event"
    );
}

#[test]
fn guard_records_correct_pid() {
    TRACE_COLLECTOR.init();

    {
        let _guard = PerfettoGuard::new("pid_check", "test");
    }

    let events = TRACE_COLLECTOR.snapshot(None);
    let event = events.iter().find(|e| e.name == "pid_check").unwrap();
    assert_eq!(event.pid, std::process::id());
}
