//! Tests for DiagnosticsSnapshot.

use crate::state::DiagnosticsSnapshot;
use arc_swap::ArcSwap;
use std::sync::Arc;

#[test]
fn snapshot_default_is_empty() {
    let snap = DiagnosticsSnapshot::default();
    assert_eq!(snap.version, "");
    assert_eq!(snap.uptime_secs, 0);
    assert_eq!(snap.fixture_count, 0);
    assert_eq!(snap.universe_count, 0);
    assert_eq!(snap.output_count, 0);
    assert_eq!(snap.session_count, 0);
    assert!(snap.protocols_running.is_empty());
    assert!(snap.sysmon_history.is_empty());
    assert!(snap.threads.is_empty());
}

#[test]
fn snapshot_serialization_roundtrip() {
    let snap = DiagnosticsSnapshot {
        threads: vec![lumacore_tree::runtime::ThreadHealthView {
            name: "State".to_string(),
            alive: true,
            last_activity_ms: 1_000,
            iterations: 44_000,
            last_loop_us: 120,
            max_loop_us: 900,
            queue_depth: 0,
            refused: 0,
            panics: 0,
            scheduling: String::new(),
        }],
        version: "0.1.0".to_string(),
        profile: "release".to_string(),
        target_os: "macos".to_string(),
        target_arch: "aarch64".to_string(),
        git_commit: "abc123".to_string(),
        build_id: "0.1.0/release/macos-aarch64/abc123".to_string(),
        uptime_secs: 3600,
        show_path: Some("library/shows/demo.luma".to_string()),
        session_count: 2,
        fixture_count: 48,
        universe_count: 4,
        output_count: 2,
        protocols_running: [("Sacn".to_string(), 1)].into(),
        sysmon_history: vec!["line1".to_string(), "line2".to_string()],
    };

    let json = serde_json::to_string(&snap).unwrap();
    let deserialized: DiagnosticsSnapshot = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.version, "0.1.0");
    assert_eq!(deserialized.threads[0].iterations, 44_000);
    assert_eq!(deserialized.fixture_count, 48);
    assert_eq!(deserialized.sysmon_history.len(), 2);
    assert_eq!(deserialized.protocols_running["Sacn"], 1);
}

#[test]
fn arcswap_lock_free_read() {
    let snap = Arc::new(DiagnosticsSnapshot {
        version: "0.1.0".to_string(),
        uptime_secs: 100,
        ..Default::default()
    });
    let store = Arc::new(ArcSwap::from(snap));

    // Read from multiple threads concurrently.
    let store_ref = store.clone();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let s = store_ref.clone();
            std::thread::spawn(move || {
                for _ in 0..100 {
                    let loaded = s.load();
                    assert_eq!(loaded.version, "0.1.0");
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn arcswap_update_visible_to_readers() {
    let snap = Arc::new(DiagnosticsSnapshot::default());
    let store = Arc::new(ArcSwap::from(snap));

    // Update the snapshot.
    let new_snap = Arc::new(DiagnosticsSnapshot {
        uptime_secs: 42,
        fixture_count: 100,
        ..Default::default()
    });
    store.store(new_snap);

    // Readers should see the new values.
    let loaded = store.load();
    assert_eq!(loaded.uptime_secs, 42);
    assert_eq!(loaded.fixture_count, 100);
}
