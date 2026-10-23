// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `monitor`.
//!
//! Owns:          the unit tests of monitor.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §26, §30 "12b2", §34 25.10.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn level_comes_from_the_prefix() {
    assert_eq!(split_level("[ERROR] boom"), (Level::Error, "boom"));
    assert_eq!(split_level("[WARN] careful"), (Level::Warn, "careful"));
    assert_eq!(split_level("all good"), (Level::Info, "all good"));
}

#[test]
fn ring_keeps_the_newest_and_subscribers_see_new_lines() {
    let (_, rx) = subscribe();
    for i in 0..(RING_SIZE + 10) {
        record("Test", &format!("line {i}"));
    }
    // Other tests write to the same global ring in parallel: look at ours.
    let h = history();
    assert!(h.len() <= RING_SIZE);
    let last_ours = h.iter().rev().find(|l| l.subsystem == "Test").unwrap();
    assert!(
        last_ours
            .message
            .ends_with(&format!("line {}", RING_SIZE + 9))
    );
    let first = std::iter::from_fn(|| rx.recv_timeout(Duration::from_secs(1)).ok())
        .find(|l| l.subsystem == "Test");
    assert!(first.is_some());
    let e = record("Test", "[ERROR] bad");
    assert_eq!(e.level, Level::Error);
    assert!(e.text().contains("[ERROR] bad"));
}

#[test]
fn old_log_files_are_pruned() {
    let dir = std::env::temp_dir().join(format!("sysmon-prune-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for d in ["2026-10-01", "2026-10-03", "2026-10-04", "2026-10-10"] {
        std::fs::write(dir.join(format!("sysmon-{d}.log")), "x").unwrap();
    }
    std::fs::write(dir.join("other.log"), "x").unwrap();
    prune_logs(&dir, "2026-10-10");
    let mut left: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    left.sort();
    assert_eq!(
        left,
        [
            "other.log",
            "sysmon-2026-10-04.log",
            "sysmon-2026-10-10.log"
        ]
    );
    let _ = std::fs::remove_dir_all(&dir);
}
