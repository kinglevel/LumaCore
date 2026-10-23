//  ██████╗ ██████╗ ██╗     ██╗     ███████╗ ██████╗████████╗ ██████╗ ██████╗
// ██╔════╝██╔═══██╗██║     ██║     ██╔════╝██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗
// ██║     ██║   ██║██║     ██║     █████╗  ██║        ██║   ██║   ██║██████╔╝
// ██║     ██║   ██║██║     ██║     ██╔══╝  ██║        ██║   ██║   ██║██╔══██╗
// ╚██████╗╚██████╔╝███████╗███████╗███████╗╚██████╗   ██║   ╚██████╔╝██║  ██║
//  ╚═════╝ ╚═════╝ ╚══════╝╚══════╝╚══════╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝
// FILE: collector

//! TraceCollector: ring buffer of trace events for Perfetto export.
//!
//! Owns:          TRACE_COLLECTOR, TraceCollector, TraceEvent, the ring buffer
//! Doesn't own:   the JSON form (format), the guard (guard), the macros (macros)
//! Entry points:  TRACE_COLLECTOR, TraceCollector::{init, record_complete, try_record_complete, snapshot}
//! Decisions:     §23 "Step 8", §24 "Step 9 hardening"

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Instant;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Global trace collector instance.
pub static TRACE_COLLECTOR: TraceCollector = TraceCollector::new();

/// Maximum events in the ring buffer (~12.5 MB at ~125 bytes/event).
const MAX_EVENTS: usize = 100_000;

/// A single trace event in Chrome Trace Event format.
#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub name: &'static str,
    pub category: &'static str,
    /// 'X' = complete, 'B' = begin, 'E' = end, 'i' = instant.
    pub phase: char,
    /// Microseconds since server start.
    pub timestamp_us: u64,
    /// Duration in microseconds (for phase 'X').
    pub duration_us: u64,
    pub pid: u32,
    pub tid: u64,
}

/// Ring buffer collecting trace events for Perfetto export.
pub struct TraceCollector {
    events: Mutex<Option<TraceCollectorInner>>,
}

struct TraceCollectorInner {
    buffer: VecDeque<TraceEvent>,
    start: Instant,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl TraceCollector {
    const fn new() -> Self {
        Self {
            events: Mutex::new(None),
        }
    }

    /// Create an isolated collector for testing (not the global singleton).
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        Self::new()
    }

    /// Initialize the collector. Safe to call multiple times — only the first
    /// call takes effect (subsequent calls are no-ops).
    pub fn init(&self) {
        let mut guard = self.events.lock().unwrap();
        if guard.is_none() {
            *guard = Some(TraceCollectorInner {
                buffer: VecDeque::with_capacity(MAX_EVENTS),
                start: Instant::now(),
            });
        }
    }

    /// Record a trace event.
    pub fn record(&self, event: TraceEvent) {
        if let Ok(mut guard) = self.events.lock()
            && let Some(ref mut inner) = *guard
        {
            if inner.buffer.len() >= MAX_EVENTS {
                inner.buffer.pop_front();
            }
            inner.buffer.push_back(event);
        }
    }

    /// Record a complete event (phase 'X') with name, category, and duration.
    pub fn record_complete(
        &self,
        name: &'static str,
        category: &'static str,
        start: Instant,
        duration: std::time::Duration,
    ) {
        if let Ok(guard) = self.events.lock()
            && let Some(ref inner) = *guard
        {
            // Use checked_duration_since to handle the case where
            // the collector was (re-)initialized after the guard was created.
            let timestamp_us = start
                .checked_duration_since(inner.start)
                .map(|d| d.as_micros() as u64)
                .unwrap_or(0);
            drop(guard);
            self.record(TraceEvent {
                name,
                category,
                phase: 'X',
                timestamp_us,
                duration_us: duration.as_micros() as u64,
                pid: std::process::id(),
                tid: thread_id(),
            });
        }
    }

    /// Record a complete event without waiting: skipped when the collector
    /// is busy. The render tick uses this (decisions §24: it never blocks);
    /// the ring is preallocated, so recording allocates nothing.
    pub fn try_record_complete(
        &self,
        name: &'static str,
        category: &'static str,
        start: Instant,
        duration: std::time::Duration,
    ) {
        if let Ok(mut guard) = self.events.try_lock()
            && let Some(inner) = guard.as_mut()
        {
            let timestamp_us = start
                .checked_duration_since(inner.start)
                .map(|d| d.as_micros() as u64)
                .unwrap_or(0);
            if inner.buffer.len() >= MAX_EVENTS {
                inner.buffer.pop_front();
            }
            inner.buffer.push_back(TraceEvent {
                name,
                category,
                phase: 'X',
                timestamp_us,
                duration_us: duration.as_micros() as u64,
                pid: std::process::id(),
                tid: thread_id(),
            });
        }
    }

    /// Get a snapshot of events, optionally filtered to the last N seconds.
    pub fn snapshot(&self, last_seconds: Option<u64>) -> Vec<TraceEvent> {
        let guard = match self.events.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        let inner = match guard.as_ref() {
            Some(i) => i,
            None => return Vec::new(),
        };

        match last_seconds {
            Some(secs) => {
                let now_us = inner.start.elapsed().as_micros() as u64;
                let cutoff = now_us.saturating_sub(secs * 1_000_000);
                inner
                    .buffer
                    .iter()
                    .filter(|e| e.timestamp_us >= cutoff)
                    .cloned()
                    .collect()
            }
            None => inner.buffer.iter().cloned().collect(),
        }
    }

    /// Number of events currently stored.
    pub fn len(&self) -> usize {
        self.events
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|i| i.buffer.len()))
            .unwrap_or(0)
    }

    /// Whether the collector is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Current time in microseconds since collector init.
    pub fn now_us(&self) -> u64 {
        self.events
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|i| i.start.elapsed().as_micros() as u64))
            .unwrap_or(0)
    }
}

fn thread_id() -> u64 {
    // Use the thread ID hash as a stable numeric identifier.
    let id = std::thread::current().id();
    // Thread IDs in Rust don't expose a numeric value directly,
    // so we use the Debug format and hash it.
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish()
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
