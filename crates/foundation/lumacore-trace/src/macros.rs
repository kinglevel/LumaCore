// ███╗   ███╗ █████╗  ██████╗██████╗  ██████╗ ███████╗
// ████╗ ████║██╔══██╗██╔════╝██╔══██╗██╔═══██╗██╔════╝
// ██╔████╔██║███████║██║     ██████╔╝██║   ██║███████╗
// ██║╚██╔╝██║██╔══██║██║     ██╔══██╗██║   ██║╚════██║
// ██║ ╚═╝ ██║██║  ██║╚██████╗██║  ██║╚██████╔╝███████║
// ╚═╝     ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝
// FILE: macros
//! The trace macros, one home for every crate (decisions §34 20): a zone to the end of the
//! caller's scope (`trace_zone!`, Tracy live + Perfetto offline), Tracy's frame mark, plots and
//! messages, and a zone held in a field (`held_zone!`). Which backend records is this crate's
//! `tracy` / `perfetto` feature, never the caller's: each macro has a recording form and an empty
//! one, so a crate that zones only depends on this crate and the top crate turns the features on.
//! The empty forms expand to nothing (or `()` / `None`): the render tick stays allocation-free.
//!
//! Owns:          trace_zone!, tracy_zone!, perfetto_zone!, tracy_plot!, tracy_message!,
//!                tracy_frame_mark!, held_zone!, HeldZone, record_complete
//! Doesn't own:   the ring buffer (collector), the Perfetto engine (the station)
//! Entry points:  the macros (at the crate root), HeldZone, record_complete
//! Decisions:     §24 "Step 9 hardening", §34 20

use std::time::{Duration, Instant};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A zone kept in a struct field: it ends when the field is replaced or dropped (a render
/// stage ends when the next begins).
#[cfg(feature = "tracy")]
pub type HeldZone = Option<tracy_client::Span>;
/// A zone kept in a struct field (no Tracy in this build: always `None`).
#[cfg(not(feature = "tracy"))]
pub type HeldZone = Option<()>;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// One complete event for the Perfetto trace (a stage timed by hand); nothing without the
/// `perfetto` feature. Never blocks: a busy collector drops the event.
#[inline]
pub fn record_complete(name: &'static str, category: &'static str, start: Instant, duration: Duration) {
    #[cfg(feature = "perfetto")]
    crate::TRACE_COLLECTOR.try_record_complete(name, category, start, duration);
    #[cfg(not(feature = "perfetto"))]
    let _ = (name, category, start, duration);
}

/// A zone in both backends to the end of the caller's scope.
#[macro_export]
macro_rules! trace_zone {
    ($name:expr) => {
        // Statements, not a block: the guards live to the end of the caller's scope.
        let _tracy_zone = $crate::tracy_zone!($name);
        let _perfetto_zone = $crate::perfetto_zone!($name);
    };
}

/// A Tracy span guard (`()` without Tracy).
#[cfg(feature = "tracy")]
#[macro_export]
macro_rules! tracy_zone {
    ($name:expr) => {
        $crate::tracy_client::span!($name)
    };
}
/// A Tracy span guard (`()` without Tracy).
#[cfg(not(feature = "tracy"))]
#[macro_export]
macro_rules! tracy_zone {
    ($name:expr) => {
        ()
    };
}

/// A Perfetto guard in the "runtime" category (`()` without Perfetto).
#[cfg(feature = "perfetto")]
#[macro_export]
macro_rules! perfetto_zone {
    ($name:expr) => {
        $crate::PerfettoGuard::new($name, "runtime")
    };
}
/// A Perfetto guard in the "runtime" category (`()` without Perfetto).
#[cfg(not(feature = "perfetto"))]
#[macro_export]
macro_rules! perfetto_zone {
    ($name:expr) => {
        ()
    };
}

/// A zone for a [`HeldZone`] field (a literal name; `None` while no Tracy client runs).
#[cfg(feature = "tracy")]
#[macro_export]
macro_rules! held_zone {
    ($name:literal) => {
        $crate::tracy_client::Client::running().map(|_| $crate::tracy_client::span!($name))
    };
}
/// A zone for a [`HeldZone`] field (always `None` without Tracy).
#[cfg(not(feature = "tracy"))]
#[macro_export]
macro_rules! held_zone {
    ($name:literal) => {
        None
    };
}

/// A frame boundary in Tracy's frame view.
#[cfg(feature = "tracy")]
#[macro_export]
macro_rules! tracy_frame_mark {
    () => {
        if let Some(client) = $crate::tracy_client::Client::running() {
            client.frame_mark();
        }
    };
}
/// A frame boundary in Tracy's frame view (nothing without Tracy).
#[cfg(not(feature = "tracy"))]
#[macro_export]
macro_rules! tracy_frame_mark {
    () => {};
}

/// A named value in Tracy's plot view.
#[cfg(feature = "tracy")]
#[macro_export]
macro_rules! tracy_plot {
    ($name:expr, $value:expr) => {
        $crate::tracy_client::plot!($name, $value as f64);
    };
}
/// A named value in Tracy's plot view (nothing without Tracy; the value isn't evaluated).
#[cfg(not(feature = "tracy"))]
#[macro_export]
macro_rules! tracy_plot {
    ($name:expr, $value:expr) => {};
}

/// A message marker in Tracy's timeline.
#[cfg(feature = "tracy")]
#[macro_export]
macro_rules! tracy_message {
    ($text:expr) => {
        if let Some(client) = $crate::tracy_client::Client::running() {
            client.message($text, 0);
        }
    };
}
/// A message marker in Tracy's timeline (nothing without Tracy).
#[cfg(not(feature = "tracy"))]
#[macro_export]
macro_rules! tracy_message {
    ($text:expr) => {};
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
