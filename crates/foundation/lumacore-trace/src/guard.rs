//  ██████╗ ██╗   ██╗ █████╗ ██████╗ ██████╗
// ██╔════╝ ██║   ██║██╔══██╗██╔══██╗██╔══██╗
// ██║  ███╗██║   ██║███████║██████╔╝██║  ██║
// ██║   ██║██║   ██║██╔══██║██╔══██╗██║  ██║
// ╚██████╔╝╚██████╔╝██║  ██║██║  ██║██████╔╝
//  ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝
// FILE: guard

//! PerfettoGuard: RAII guard that records a complete trace event on drop.
//!
//! Owns:          PerfettoGuard
//! Doesn't own:   the collector (collector)
//! Entry points:  PerfettoGuard::new
//! Decisions:     §23 "Step 8"

use std::time::Instant;

use crate::collector::TRACE_COLLECTOR;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// RAII guard for Perfetto trace events.
///
/// Records a complete ('X') trace event when dropped, measuring the
/// duration from creation to drop.
pub struct PerfettoGuard {
    name: &'static str,
    category: &'static str,
    start: Instant,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl PerfettoGuard {
    /// Create a new guard that will record a trace event on drop.
    pub fn new(name: &'static str, category: &'static str) -> Self {
        Self {
            name,
            category,
            start: Instant::now(),
        }
    }
}

impl Drop for PerfettoGuard {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        TRACE_COLLECTOR.record_complete(self.name, self.category, self.start, duration);
    }
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
