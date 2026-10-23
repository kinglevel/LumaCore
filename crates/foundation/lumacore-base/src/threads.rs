// ████████╗██╗  ██╗██████╗ ███████╗ █████╗ ██████╗ ███████╗
// ╚══██╔══╝██║  ██║██╔══██╗██╔════╝██╔══██╗██╔══██╗██╔════╝
//    ██║   ███████║██████╔╝█████╗  ███████║██║  ██║███████╗
//    ██║   ██╔══██║██╔══██╗██╔══╝  ██╔══██║██║  ██║╚════██║
//    ██║   ██║  ██║██║  ██║███████╗██║  ██║██████╔╝███████║
//    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝ ╚══════╝
// FILE: threads

//! Waiting for a thread against one deadline (decisions §26: every stop joins every thread
//! against the one shutdown deadline; §34 19.1: it moved down so the engines and the line
//! runtime share it).
//!
//! Owns:          join_until.
//! Doesn't own:   what stops a thread (its owner's stop flag), the deadline (the server's shutdown).
//! Entry points:  join_until.
//! Decisions:     §26, §34 19.1.

use std::thread::JoinHandle;
use std::time::{Duration, Instant};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Wait for a thread to end, at most until `deadline`.
pub fn join_until(handle: JoinHandle<()>, deadline: Instant) -> Result<(), String> {
    while !handle.is_finished() {
        if Instant::now() >= deadline {
            return Err("thread still running at the shutdown deadline".to_string());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    handle
        .join()
        .map_err(|_| "thread ended with a panic".to_string())
}
