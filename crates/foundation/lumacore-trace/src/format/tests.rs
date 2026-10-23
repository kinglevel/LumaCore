// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests for Chrome Trace Event JSON serialization.
//!
//! Owns:          the unit tests of format.rs (the Chrome Trace Event JSON)
//! Doesn't own:   the ring buffer (collector/tests.rs)
//! Entry points:  the #[test] fns
//! Decisions:     §23 "Step 8"

use crate::collector::TraceEvent;
use crate::format::event_to_json;

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[test]
fn complete_event_has_duration() {
    let event = TraceEvent {
        name: "render_frame",
        category: "gfx",
        phase: 'X',
        timestamp_us: 1000,
        duration_us: 500,
        pid: 1,
        tid: 2,
    };
    let json = event_to_json(&event);

    assert_eq!(json["name"], "render_frame");
    assert_eq!(json["cat"], "gfx");
    assert_eq!(json["ph"], "X");
    assert_eq!(json["ts"], 1000);
    assert_eq!(json["dur"], 500);
    assert_eq!(json["pid"], 1);
    assert_eq!(json["tid"], 2);
}

#[test]
fn instant_event_has_no_duration() {
    let event = TraceEvent {
        name: "marker",
        category: "debug",
        phase: 'i',
        timestamp_us: 2000,
        duration_us: 0,
        pid: 1,
        tid: 3,
    };
    let json = event_to_json(&event);

    assert_eq!(json["name"], "marker");
    assert_eq!(json["ph"], "i");
    // Instant events should NOT have a "dur" field.
    assert!(
        json.get("dur").is_none(),
        "instant event should not have dur field"
    );
}

#[test]
fn begin_event_has_no_duration() {
    let event = TraceEvent {
        name: "scope",
        category: "runtime",
        phase: 'B',
        timestamp_us: 3000,
        duration_us: 0,
        pid: 1,
        tid: 4,
    };
    let json = event_to_json(&event);

    assert_eq!(json["ph"], "B");
    assert!(json.get("dur").is_none());
}

#[test]
fn end_event_has_no_duration() {
    let event = TraceEvent {
        name: "scope",
        category: "runtime",
        phase: 'E',
        timestamp_us: 4000,
        duration_us: 0,
        pid: 1,
        tid: 4,
    };
    let json = event_to_json(&event);

    assert_eq!(json["ph"], "E");
    assert!(json.get("dur").is_none());
}

#[test]
fn json_is_valid_perfetto_format() {
    let event = TraceEvent {
        name: "dmx_output",
        category: "io",
        phase: 'X',
        timestamp_us: 0,
        duration_us: 22727,
        pid: 100,
        tid: 200,
    };
    let json = event_to_json(&event);

    // All required Perfetto fields must be present.
    assert!(json["name"].is_string());
    assert!(json["cat"].is_string());
    assert!(json["ph"].is_string());
    assert!(json["ts"].is_number());
    assert!(json["pid"].is_number());
    assert!(json["tid"].is_number());

    // The JSON must be serializable to a string (roundtrip).
    let serialized = serde_json::to_string(&json).unwrap();
    let _: serde_json::Value = serde_json::from_str(&serialized).unwrap();
}
