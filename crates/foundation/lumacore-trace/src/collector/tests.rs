// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests for the TraceCollector ring buffer.
//!
//! Owns:          the unit tests of collector.rs (the ring buffer)
//! Doesn't own:   the guard (guard/tests.rs), the JSON form (format/tests.rs)
//! Entry points:  the #[test] fns
//! Decisions:     §23 "Step 8"

use crate::collector::{TraceCollector, TraceEvent};
use std::time::Instant;

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

fn make_event(name: &'static str, ts: u64, dur: u64) -> TraceEvent {
    TraceEvent {
        name,
        category: "test",
        phase: 'X',
        timestamp_us: ts,
        duration_us: dur,
        pid: 1,
        tid: 1,
    }
}

#[test]
fn collector_uninitialized_returns_empty() {
    let collector = TraceCollector::new_for_test();
    assert!(collector.is_empty());
    assert_eq!(collector.len(), 0);
    assert!(collector.snapshot(None).is_empty());
}

#[test]
fn collector_record_before_init_is_noop() {
    let collector = TraceCollector::new_for_test();
    collector.record(make_event("noop", 0, 100));
    assert_eq!(collector.len(), 0);
}

#[test]
fn collector_init_and_record() {
    let collector = TraceCollector::new_for_test();
    collector.init();
    assert!(collector.is_empty());

    collector.record(make_event("first", 0, 100));
    assert_eq!(collector.len(), 1);

    collector.record(make_event("second", 100, 200));
    assert_eq!(collector.len(), 2);

    let events = collector.snapshot(None);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].name, "first");
    assert_eq!(events[1].name, "second");
}

#[test]
fn collector_snapshot_preserves_event_fields() {
    let collector = TraceCollector::new_for_test();
    collector.init();
    collector.record(TraceEvent {
        name: "render",
        category: "gfx",
        phase: 'X',
        timestamp_us: 5000,
        duration_us: 1200,
        pid: 42,
        tid: 99,
    });

    let events = collector.snapshot(None);
    assert_eq!(events.len(), 1);
    let e = &events[0];
    assert_eq!(e.name, "render");
    assert_eq!(e.category, "gfx");
    assert_eq!(e.phase, 'X');
    assert_eq!(e.timestamp_us, 5000);
    assert_eq!(e.duration_us, 1200);
    assert_eq!(e.pid, 42);
    assert_eq!(e.tid, 99);
}

#[test]
fn collector_ring_buffer_overflow() {
    let collector = TraceCollector::new_for_test();
    collector.init();

    // Fill beyond capacity — oldest events should be dropped.
    // Use a smaller test: record 100_001 events, verify len is 100_000.
    for i in 0..100_001u64 {
        collector.record(make_event("tick", i, 1));
    }

    assert_eq!(collector.len(), 100_000);

    // The first event should have been evicted — first remaining is ts=1.
    let events = collector.snapshot(None);
    assert_eq!(events[0].timestamp_us, 1);
    assert_eq!(events.last().unwrap().timestamp_us, 100_000);
}

#[test]
fn collector_snapshot_time_filter() {
    let collector = TraceCollector::new_for_test();
    collector.init();

    // Simulate events at various timestamps (microseconds).
    // now_us will be close to 0 since we just initialized.
    collector.record(make_event("old", 0, 10));
    collector.record(make_event("recent", 500_000, 10)); // 0.5s ago (if now ~0.5s)

    // Without filter, get all events.
    let all = collector.snapshot(None);
    assert_eq!(all.len(), 2);

    // With a very large window, get all events.
    let all_wide = collector.snapshot(Some(3600));
    assert_eq!(all_wide.len(), 2);
}

#[test]
fn collector_record_complete_sets_fields() {
    let collector = TraceCollector::new_for_test();
    collector.init();

    let start = Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(2));
    let duration = start.elapsed();

    collector.record_complete("sleep_test", "test", start, duration);

    let events = collector.snapshot(None);
    assert_eq!(events.len(), 1);
    let e = &events[0];
    assert_eq!(e.name, "sleep_test");
    assert_eq!(e.category, "test");
    assert_eq!(e.phase, 'X');
    assert!(
        e.duration_us >= 1000,
        "expected >= 1ms duration, got {}µs",
        e.duration_us
    );
    assert_eq!(e.pid, std::process::id());
}

#[test]
fn collector_now_us_increases() {
    let collector = TraceCollector::new_for_test();
    collector.init();

    let t1 = collector.now_us();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let t2 = collector.now_us();
    assert!(t2 > t1, "now_us should increase over time");
}

#[test]
fn collector_concurrent_recording() {
    let collector = TraceCollector::new_for_test();
    collector.init();

    // Spawn multiple threads recording events concurrently.
    let collector_ref = &collector;
    std::thread::scope(|s| {
        for thread_idx in 0..4 {
            s.spawn(move || {
                for i in 0..250 {
                    collector_ref.record(make_event(
                        "concurrent",
                        (thread_idx * 1000 + i) as u64,
                        1,
                    ));
                }
            });
        }
    });

    assert_eq!(collector.len(), 1000);
}

#[test]
fn try_record_complete_drops_when_contended() {
    let collector = TraceCollector::new_for_test();
    collector.init();
    {
        let _held = collector.events.lock().unwrap();
        collector.try_record_complete("busy", "test", Instant::now(), std::time::Duration::from_micros(1));
    }
    assert!(collector.is_empty(), "a busy collector drops the event instead of waiting");
    collector.try_record_complete("free", "test", Instant::now(), std::time::Duration::from_micros(1));
    assert_eq!(collector.len(), 1);
}
