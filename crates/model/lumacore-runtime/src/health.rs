//! `Root.Runtime.Health` — every long-lived thread reports how it is doing
//! (`docs/meta/decisions.md` §26).
//!
//! A thread registers once and gets an `Arc<ThreadHealth>`; after that it only
//! stores atomics (no lock, no allocation), so even a render loop may report.
//! Readers (`List Health`, HTTP) take a [`HealthBoard::snapshot`].

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::render_stats::{
    ClockView, LineStats, LineStatsView, MemberClock, PtpStatus, RenderStats,
    RenderStatsView,
};

use serde::{Deserialize, Serialize};

use lumacore_base::clock::now_ms;

/// One thread's live numbers.
#[derive(Debug, Default)]
pub struct ThreadHealth {
    alive: AtomicBool,
    last_activity_ms: AtomicU64,
    iterations: AtomicU64,
    last_loop_us: AtomicU64,
    max_loop_us: AtomicU64,
    queue_depth: AtomicU64,
    /// Work turned away because a bounded queue was full (decisions §31).
    refused: AtomicU64,
    panics: AtomicU64,
    /// How the thread is scheduled (`SCHED_FIFO 70`, `normal — …`), when it said (§31).
    scheduling: Mutex<String>,
}

impl ThreadHealth {
    /// One unit of work done (a job, a frame, a packet) taking `loop_us`.
    pub fn tick(&self, loop_us: u64) {
        self.last_activity_ms.store(now_ms(), Ordering::Relaxed);
        self.iterations.fetch_add(1, Ordering::Relaxed);
        self.last_loop_us.store(loop_us, Ordering::Relaxed);
        self.max_loop_us.fetch_max(loop_us, Ordering::Relaxed);
    }

    /// Still running, nothing to do (keeps `last activity` meaningful for
    /// threads that mostly wait).
    pub fn heartbeat(&self) {
        self.last_activity_ms.store(now_ms(), Ordering::Relaxed);
    }

    pub fn set_queue_depth(&self, depth: usize) {
        self.queue_depth.store(depth as u64, Ordering::Relaxed);
    }

    /// One piece of work refused: its bounded queue was full (decisions §31).
    pub fn refused(&self) {
        self.refused.fetch_add(1, Ordering::Relaxed);
    }

    /// How the OS schedules this thread (set once when it starts, never in a loop).
    pub fn set_scheduling(&self, scheduling: impl Into<String>) {
        *self.scheduling.lock().unwrap_or_else(|e| e.into_inner()) = scheduling.into();
    }

    pub fn panicked(&self) {
        self.panics.fetch_add(1, Ordering::Relaxed);
    }

    /// The thread ended (normally or not).
    pub fn stopped(&self) {
        self.alive.store(false, Ordering::Relaxed);
    }
}

/// Snapshot of one thread for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadHealthView {
    pub name: String,
    pub alive: bool,
    pub last_activity_ms: u64,
    pub iterations: u64,
    pub last_loop_us: u64,
    pub max_loop_us: u64,
    pub queue_depth: u64,
    /// Work turned away because a bounded queue was full (decisions §31).
    #[serde(default)]
    pub refused: u64,
    pub panics: u64,
    /// How the OS schedules it; empty = the thread didn't say (decisions §31).
    #[serde(default)]
    pub scheduling: String,
}

/// All registered threads (`Runtime.Health.Threads`), the render engine
/// (`Runtime.Health.Render`), the lines' send numbers and the session clock
/// (`Runtime.Health.Clock`) — decisions §24 "Step 9 hardening".
#[derive(Debug, Default)]
pub struct HealthBoard {
    threads: Mutex<Vec<(String, Arc<ThreadHealth>)>>,
    /// Shared with the render engine.
    pub render: Arc<RenderStats>,
    /// Output lines' numbers: (line, protocol, stats), set by the state
    /// thread when the drivers change.
    lines: Mutex<Vec<(u32, String, Arc<LineStats>)>>,
    /// On the master: what each member reports about its clock.
    members: Mutex<BTreeMap<String, MemberClock>>,
    /// linuxptp, or why it can't be read (None = not read yet).
    ptp: Mutex<Option<Result<PtpStatus, String>>>,
}

/// Everything in `Runtime.Health`, for display (`List Health`, the Realtime
/// window).
#[derive(Debug, Clone, Serialize)]
pub struct HealthView {
    pub render: RenderStatsView,
    pub lines: Vec<LineStatsView>,
    pub clock: ClockView,
    pub threads: Vec<ThreadHealthView>,
}

impl HealthBoard {
    /// Register a thread by name (`State`, `Udp`, `Render.sACN`, …). Calling it
    /// again with the same name (a restarted thread) reuses the entry.
    pub fn register(&self, name: &str) -> Arc<ThreadHealth> {
        let mut threads = self.threads.lock().unwrap_or_else(|e| e.into_inner());
        let health = match threads.iter().find(|(n, _)| n == name) {
            Some((_, h)) => h.clone(),
            None => {
                let h = Arc::new(ThreadHealth::default());
                threads.push((name.to_string(), h.clone()));
                h
            }
        };
        health.alive.store(true, Ordering::Relaxed);
        health.heartbeat();
        health
    }

    pub fn set_lines(&self, lines: Vec<(u32, String, Arc<LineStats>)>) {
        *self.lines.lock().unwrap_or_else(|e| e.into_inner()) = lines;
    }

    /// A member's clock report (master side); `None` drops it.
    pub fn set_member_clock(&self, station: &str, clock: Option<MemberClock>) {
        let mut members = self.members.lock().unwrap_or_else(|e| e.into_inner());
        match clock {
            Some(c) => {
                members.insert(station.to_string(), c);
            }
            None => {
                members.remove(station);
            }
        }
    }

    pub fn set_ptp(&self, reading: Result<PtpStatus, String>) {
        *self.ptp.lock().unwrap_or_else(|e| e.into_inner()) = Some(reading);
    }

    pub fn clock_view(&self) -> ClockView {
        let c = crate::render_stats::clock_sync();
        let ptp = self.ptp.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let l = |a: &std::sync::atomic::AtomicU64| a.load(Ordering::Relaxed);
        let li = |a: &std::sync::atomic::AtomicI64| a.load(Ordering::Relaxed);
        ClockView {
            role: c.role_label(),
            state: c.state_label(),
            master: c.master.lock().unwrap_or_else(|e| e.into_inner()).clone(),
            epoch: l(&c.epoch),
            offset_us: li(&c.offset_us),
            path_delay_us: l(&c.path_delay_us),
            jitter_us: l(&c.jitter_us),
            frequency_ppb: li(&c.frequency_ppb),
            skew_ppb: l(&c.skew_ppb),
            correction_ppb: li(&c.correction_ppb),
            samples_used: l(&c.samples_used),
            samples_rejected: l(&c.samples_rejected),
            last_sync_ms: l(&c.last_sync_ms),
            steps: l(&c.steps),
            last_step_ms: l(&c.last_step_ms),
            last_step_us: li(&c.last_step_us),
            // A member not heard for 5 s is gone (left, or lost).
            members: self
                .members
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .values()
                .filter(|m| m.last_sync_ms + 5_000 >= now_ms())
                .cloned()
                .collect(),
            ptp_note: match &ptp {
                Some(Err(e)) => Some(e.clone()),
                None => Some("not read yet".to_string()),
                Some(Ok(_)) => None,
            },
            ptp: ptp.and_then(Result::ok),
        }
    }

    pub fn view(&self) -> HealthView {
        HealthView {
            render: self.render.snapshot(),
            lines: self
                .lines
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .iter()
                .map(|(line, protocol, s)| s.view(*line, protocol))
                .collect(),
            clock: self.clock_view(),
            threads: self.snapshot(),
        }
    }

    pub fn snapshot(&self) -> Vec<ThreadHealthView> {
        let threads = self.threads.lock().unwrap_or_else(|e| e.into_inner());
        threads
            .iter()
            .map(|(name, h)| ThreadHealthView {
                name: name.clone(),
                alive: h.alive.load(Ordering::Relaxed),
                last_activity_ms: h.last_activity_ms.load(Ordering::Relaxed),
                iterations: h.iterations.load(Ordering::Relaxed),
                last_loop_us: h.last_loop_us.load(Ordering::Relaxed),
                max_loop_us: h.max_loop_us.load(Ordering::Relaxed),
                queue_depth: h.queue_depth.load(Ordering::Relaxed),
                refused: h.refused.load(Ordering::Relaxed),
                panics: h.panics.load(Ordering::Relaxed),
                scheduling: h.scheduling.lock().unwrap_or_else(|e| e.into_inner()).clone(),
            })
            .collect()
    }
}

/// Marks the thread stopped when dropped — also when it unwinds.
pub struct ThreadGuard(pub Arc<ThreadHealth>);

impl Drop for ThreadGuard {
    fn drop(&mut self) {
        self.0.stopped();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_ticks_and_stops() {
        let board = HealthBoard::default();
        let h = board.register("State");
        h.tick(120);
        h.tick(80);
        h.set_queue_depth(3);
        {
            let _guard = ThreadGuard(board.register("State"));
        }
        let snap = board.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].iterations, 2);
        assert_eq!(snap[0].max_loop_us, 120);
        assert_eq!(snap[0].last_loop_us, 80);
        assert_eq!(snap[0].queue_depth, 3);
        assert!(!snap[0].alive);
    }
}
