//! `Root.Runtime.Health.Render` — the render engine's frame numbers
//! (decisions §24 "Step 9 hardening"), like a System Info "Realtime"
//! view: every stage's µs per frame on a frame timeline, frame time,
//! jitter, headroom, late / skipped frames, panics and the plan compiler.
//!
//! The engine writes atomics only (no lock, no allocation). Per-second
//! maxima are kept by the engine and published once a second. Readers take
//! a [`RenderStatsView`].

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

/// The stages of one frame, in order.
pub const STAGES: [&str; 6] = [
    "Plan",
    "Inputs",
    "Channels",
    "Park",
    "InputMerge",
    "Publish",
];
pub const STAGE_COUNT: usize = STAGES.len();

/// One stage's numbers.
#[derive(Debug, Default)]
pub struct StageStats {
    /// Start of the stage in the last frame, µs after the frame began.
    pub start_us: AtomicU64,
    /// Duration in the last frame.
    pub last_us: AtomicU64,
    /// Longest duration during the last full second.
    pub max_us: AtomicU64,
}

/// The render engine's live numbers.
#[derive(Debug, Default)]
pub struct RenderStats {
    pub stages: [StageStats; STAGE_COUNT],
    /// Render rate (Hz).
    pub rate_hz: AtomicU64,
    pub frames: AtomicU64,
    /// Work in the last frame (all stages).
    pub frame_us: AtomicU64,
    /// Longest frame during the last full second.
    pub frame_max_us: AtomicU64,
    /// Largest |interval − period| during the last full second.
    pub jitter_us: AtomicU64,
    /// Time left before the deadline after the last frame.
    pub headroom_us: AtomicU64,
    /// Least headroom during the last full second.
    pub headroom_min_us: AtomicU64,
    /// Frames that missed their deadline, and frames skipped because of it.
    pub late: AtomicU64,
    pub skipped: AtomicU64,
    /// In an overload episode (late frames within the last second).
    pub overload: AtomicBool,
    pub overload_episodes: AtomicU64,
    /// Frames that found every frame buffer still in use (not published).
    pub busy: AtomicU64,
    pub panics: AtomicU64,
    pub restarts: AtomicU64,
    /// Stopped after repeated panics; the lines hold the last frame.
    pub down: AtomicBool,
    /// The show generation of the plan being rendered.
    pub plan_generation: AtomicU64,
    /// Last compile time of the channel tier (0 = reused) and value tier.
    pub compile_channel_us: AtomicU64,
    pub compile_value_us: AtomicU64,
    pub compiles: AtomicU64,
    /// Compiles that panicked (the last plan kept; R1-66).
    pub compile_panics: AtomicU64,
    /// The first input generation that failed to compile while the stage
    /// still shows an older plan (0 = the plan is current; R1-66, Q39).
    pub stale_since: AtomicU64,
    /// The compiler stopped after 3 panics within 60 s; the stage keeps the
    /// last plan until a show load or a restart.
    pub compiler_down: AtomicBool,
    /// Channel slots and universes in the plan.
    pub channels: AtomicU64,
    pub universes: AtomicU64,
    /// Every frame's numbers, the last [`FRAME_RING`] of them (the Realtime
    /// window's `health.render` feed, decisions §30 "12b2 hardening").
    pub ring: FrameRing,
}

/// Frames the per-frame ring keeps: a little over 10 s at 60 Hz.
pub const FRAME_RING: usize = 640;

/// One frame's numbers, as the ring hands them out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrameSample {
    /// The frame's number (1, 2, …; `RenderStats::frames` after it).
    pub seq: u64,
    /// Session clock when the frame started (ms).
    pub t_ms: u64,
    pub frame_us: u64,
    pub headroom_us: u64,
    pub jitter_us: u64,
    /// The frame ended after its deadline.
    pub late: bool,
    pub stages: [u64; STAGE_COUNT],
}

/// One slot: a seqlock (`seq` is `u64::MAX` while the engine writes it).
#[derive(Debug, Default)]
struct FrameSlot {
    seq: AtomicU64,
    t_ms: AtomicU64,
    frame_us: AtomicU64,
    headroom_us: AtomicU64,
    jitter_us: AtomicU64,
    late: AtomicBool,
    stages: [AtomicU64; STAGE_COUNT],
}

/// A ring of the last [`FRAME_RING`] frames: one writer (the render engine,
/// atomics only — no lock, no allocation), any number of readers that take
/// what they haven't seen yet. A slot overwritten while it is read is
/// skipped (the reader sees a gap, never a torn frame).
#[derive(Debug)]
pub struct FrameRing {
    slots: Box<[FrameSlot]>,
    /// The newest frame written (0 = none yet).
    head: AtomicU64,
}

impl Default for FrameRing {
    fn default() -> Self {
        Self {
            slots: (0..FRAME_RING).map(|_| FrameSlot::default()).collect(),
            head: AtomicU64::new(0),
        }
    }
}

impl FrameRing {
    /// Store a frame (the engine; `s.seq` grows by one per frame).
    pub fn push(&self, s: &FrameSample) {
        let slot = &self.slots[(s.seq % FRAME_RING as u64) as usize];
        slot.seq.store(u64::MAX, Ordering::Relaxed);
        std::sync::atomic::fence(Ordering::Release);
        slot.t_ms.store(s.t_ms, Ordering::Relaxed);
        slot.frame_us.store(s.frame_us, Ordering::Relaxed);
        slot.headroom_us.store(s.headroom_us, Ordering::Relaxed);
        slot.jitter_us.store(s.jitter_us, Ordering::Relaxed);
        slot.late.store(s.late, Ordering::Relaxed);
        for (a, v) in slot.stages.iter().zip(s.stages) {
            a.store(v, Ordering::Relaxed);
        }
        slot.seq.store(s.seq, Ordering::Release);
        self.head.store(s.seq, Ordering::Release);
    }

    /// The newest frame's number (0 = none).
    pub fn head(&self) -> u64 {
        self.head.load(Ordering::Acquire)
    }

    /// The oldest frame the ring may still hold.
    pub fn oldest(&self) -> u64 {
        self.head().saturating_sub(FRAME_RING as u64 - 1).max(1)
    }

    /// Every frame after `after` the ring still holds, oldest first, into
    /// `out`. A frame missing from the result (before the first or between
    /// two) is lost: the caller reports it as a gap.
    pub fn read_after(&self, after: u64, out: &mut Vec<FrameSample>) {
        let head = self.head();
        if head == 0 || after >= head {
            return;
        }
        // Leave the slot the engine writes next alone.
        let from = (after + 1).max(head.saturating_sub(FRAME_RING as u64 - 2)).max(1);
        for seq in from..=head {
            let slot = &self.slots[(seq % FRAME_RING as u64) as usize];
            if slot.seq.load(Ordering::Acquire) != seq {
                continue;
            }
            let mut s = FrameSample {
                seq,
                t_ms: slot.t_ms.load(Ordering::Relaxed),
                frame_us: slot.frame_us.load(Ordering::Relaxed),
                headroom_us: slot.headroom_us.load(Ordering::Relaxed),
                jitter_us: slot.jitter_us.load(Ordering::Relaxed),
                late: slot.late.load(Ordering::Relaxed),
                stages: [0; STAGE_COUNT],
            };
            for (v, a) in s.stages.iter_mut().zip(&slot.stages) {
                *v = a.load(Ordering::Relaxed);
            }
            std::sync::atomic::fence(Ordering::Acquire);
            if slot.seq.load(Ordering::Relaxed) == seq {
                out.push(s);
            }
        }
    }
}

#[cfg(test)]
mod ring_tests {
    use super::*;

    fn frame(seq: u64) -> FrameSample {
        FrameSample { seq, t_ms: seq * 16, frame_us: seq, late: seq.is_multiple_of(7), ..Default::default() }
    }

    #[test]
    fn the_ring_hands_out_what_a_reader_has_not_seen() {
        let ring = FrameRing::default();
        let mut out = Vec::new();
        ring.read_after(0, &mut out);
        assert!(out.is_empty());
        for seq in 1..=10 {
            ring.push(&frame(seq));
        }
        ring.read_after(4, &mut out);
        assert_eq!(out.iter().map(|s| s.seq).collect::<Vec<_>>(), (5..=10).collect::<Vec<_>>());
        assert_eq!(out[2], frame(7));
    }

    #[test]
    fn a_reader_past_the_ring_gets_only_what_it_holds() {
        let ring = FrameRing::default();
        let n = FRAME_RING as u64 * 3;
        for seq in 1..=n {
            ring.push(&frame(seq));
        }
        let mut out = Vec::new();
        ring.read_after(5, &mut out);
        assert_eq!(out.last().map(|s| s.seq), Some(n));
        assert!(out[0].seq > 5 + FRAME_RING as u64, "the lost frames are a gap");
        assert!(out.windows(2).all(|w| w[1].seq == w[0].seq + 1));
        assert!(out.len() >= FRAME_RING - 2);
    }
}

/// A stage for display.
#[derive(Debug, Clone, Serialize)]
pub struct StageView {
    pub name: &'static str,
    pub start_us: u64,
    pub last_us: u64,
    pub max_us: u64,
}

/// The render numbers for display.
#[derive(Debug, Clone, Serialize)]
pub struct RenderStatsView {
    pub rate_hz: u64,
    pub period_us: u64,
    pub frames: u64,
    pub frame_us: u64,
    pub frame_max_us: u64,
    pub jitter_us: u64,
    pub headroom_us: u64,
    pub headroom_min_us: u64,
    pub late: u64,
    pub skipped: u64,
    pub overload: bool,
    pub overload_episodes: u64,
    pub busy: u64,
    pub panics: u64,
    pub restarts: u64,
    pub down: bool,
    pub plan_generation: u64,
    pub compile_channel_us: u64,
    pub compile_value_us: u64,
    pub compiles: u64,
    pub compile_panics: u64,
    pub stale_since: u64,
    pub compiler_down: bool,
    pub channels: u64,
    pub universes: u64,
    pub stages: Vec<StageView>,
}

impl RenderStats {
    pub fn snapshot(&self) -> RenderStatsView {
        let l = |a: &AtomicU64| a.load(Ordering::Relaxed);
        let rate = l(&self.rate_hz);
        RenderStatsView {
            rate_hz: rate,
            period_us: 1_000_000u64.checked_div(rate).unwrap_or(0),
            frames: l(&self.frames),
            frame_us: l(&self.frame_us),
            frame_max_us: l(&self.frame_max_us),
            jitter_us: l(&self.jitter_us),
            headroom_us: l(&self.headroom_us),
            headroom_min_us: l(&self.headroom_min_us),
            late: l(&self.late),
            skipped: l(&self.skipped),
            overload: self.overload.load(Ordering::Relaxed),
            overload_episodes: l(&self.overload_episodes),
            busy: l(&self.busy),
            panics: l(&self.panics),
            restarts: l(&self.restarts),
            down: self.down.load(Ordering::Relaxed),
            plan_generation: l(&self.plan_generation),
            compile_channel_us: l(&self.compile_channel_us),
            compile_value_us: l(&self.compile_value_us),
            compiles: l(&self.compiles),
            compile_panics: l(&self.compile_panics),
            stale_since: l(&self.stale_since),
            compiler_down: self.compiler_down.load(Ordering::Relaxed),
            channels: l(&self.channels),
            universes: l(&self.universes),
            stages: STAGES
                .iter()
                .zip(&self.stages)
                .map(|(name, s)| StageView {
                    name,
                    start_us: l(&s.start_us),
                    last_us: l(&s.last_us),
                    max_us: l(&s.max_us),
                })
                .collect(),
        }
    }
}

/// One output line's send numbers (per driver, per line).
#[derive(Debug, Default)]
pub struct LineStats {
    /// Configured rate and the rate actually used (capped by nodes), Hz.
    pub rate_hz: AtomicU64,
    pub effective_hz: AtomicU64,
    /// Measured send rate over the last second (frames sent).
    pub actual_hz: AtomicU64,
    /// Frames and packets sent, send errors.
    pub frames: AtomicU64,
    pub packets: AtomicU64,
    pub errors: AtomicU64,
    /// Time to send one frame of this line (µs), last and max (last second).
    pub send_us: AtomicU64,
    pub send_max_us: AtomicU64,
    /// Sends that took the same engine frame as the send before (decisions §31 "Line
    /// timing": the line's own timer drifted against the engine's).
    pub doubled: AtomicU64,
    /// Engine frames a send jumped over beyond what its rate skips anyway (a 30 Hz line
    /// takes every 2nd frame; one that jumps 3 counts 1).
    pub skipped: AtomicU64,
}

/// A line's numbers for display.
#[derive(Debug, Clone, Serialize)]
pub struct LineStatsView {
    pub line: u32,
    pub protocol: String,
    pub rate_hz: u64,
    pub effective_hz: u64,
    pub actual_hz: u64,
    pub frames: u64,
    pub packets: u64,
    pub errors: u64,
    pub send_us: u64,
    pub send_max_us: u64,
    pub doubled: u64,
    pub skipped: u64,
}

impl LineStats {
    pub fn view(&self, line: u32, protocol: &str) -> LineStatsView {
        let l = |a: &AtomicU64| a.load(Ordering::Relaxed);
        LineStatsView {
            line,
            protocol: protocol.to_string(),
            rate_hz: l(&self.rate_hz),
            effective_hz: l(&self.effective_hz),
            actual_hz: l(&self.actual_hz),
            frames: l(&self.frames),
            packets: l(&self.packets),
            errors: l(&self.errors),
            send_us: l(&self.send_us),
            send_max_us: l(&self.send_max_us),
            doubled: l(&self.doubled),
            skipped: l(&self.skipped),
        }
    }
}

/// The session clock's lock state (PTP servo states, plus Holdover). A
/// master or standalone station shows "Reference".
pub const CLOCK_STATES: [&str; 5] = ["Unlocked", "Acquiring", "Locked", "Stable", "Holdover"];

/// This station's session clock sync (`Runtime.Health.Clock`), in PTP/NTP
/// terms (decisions §24 "Session clock (researched)"). Written by the clock
/// sync thread and the UDP port, atomics only (the master's name behind a
/// small lock).
#[derive(Debug, Default)]
pub struct ClockSync {
    /// 0 standalone, 1 master, 2 member.
    pub role: std::sync::atomic::AtomicU8,
    /// Index into [`CLOCK_STATES`].
    pub state: std::sync::atomic::AtomicU8,
    /// offsetFromMaster: master minus us, filtered (µs).
    pub offset_us: std::sync::atomic::AtomicI64,
    /// meanPathDelay: half the gated round trip (µs).
    pub path_delay_us: AtomicU64,
    /// Spread of the gated offsets (µs).
    pub jitter_us: AtomicU64,
    /// Frequency against the master (ppb) and its uncertainty (skew).
    pub frequency_ppb: std::sync::atomic::AtomicI64,
    pub skew_ppb: AtomicU64,
    /// The correction rate closing the offset (ppb).
    pub correction_ppb: std::sync::atomic::AtomicI64,
    pub samples_used: AtomicU64,
    pub samples_rejected: AtomicU64,
    pub last_sync_ms: AtomicU64,
    pub steps: AtomicU64,
    pub last_step_ms: AtomicU64,
    pub last_step_us: std::sync::atomic::AtomicI64,
    /// The master followed and its epoch.
    pub master: std::sync::Mutex<String>,
    pub epoch: AtomicU64,
}

/// A member's clock as its master sees it (reported in its Time pings).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MemberClock {
    pub station: String,
    pub state: String,
    pub offset_us: i64,
    pub path_delay_us: u64,
    pub jitter_us: u64,
    pub frequency_ppb: i64,
    pub last_sync_ms: u64,
}

/// linuxptp's view (read-only, `pmc`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PtpStatus {
    pub offset_ns: i64,
    pub path_delay_ns: i64,
    pub grandmaster: String,
    pub gm_present: bool,
    pub port_state: String,
    pub steps_removed: u32,
    pub read_ms: u64,
}

/// The clock numbers for display.
#[derive(Debug, Clone, Serialize)]
pub struct ClockView {
    pub role: &'static str,
    pub state: &'static str,
    pub master: String,
    pub epoch: u64,
    pub offset_us: i64,
    pub path_delay_us: u64,
    pub jitter_us: u64,
    pub frequency_ppb: i64,
    pub skew_ppb: u64,
    pub correction_ppb: i64,
    pub samples_used: u64,
    pub samples_rejected: u64,
    pub last_sync_ms: u64,
    pub steps: u64,
    pub last_step_ms: u64,
    pub last_step_us: i64,
    /// On the master: each member's report.
    pub members: Vec<MemberClock>,
    pub ptp: Option<PtpStatus>,
    /// Why there is no PTP reading.
    pub ptp_note: Option<String>,
}

/// This process's clock sync numbers (one station per process; the
/// session clock itself is `lumacore_base::clock`).
pub fn clock_sync() -> &'static ClockSync {
    static CLOCK: std::sync::OnceLock<ClockSync> = std::sync::OnceLock::new();
    CLOCK.get_or_init(ClockSync::default)
}

impl ClockSync {
    /// The lock state; a master (or standalone station) is the reference
    /// itself and runs on at the frequency it had.
    pub fn state_label(&self) -> &'static str {
        if self.role.load(Ordering::Relaxed) != 2 {
            return "Reference";
        }
        CLOCK_STATES
            .get(usize::from(self.state.load(Ordering::Relaxed)))
            .copied()
            .unwrap_or("Unlocked")
    }

    pub fn set_state(&self, state: usize) {
        self.state.store(state as u8, Ordering::Relaxed);
    }

    pub fn role_label(&self) -> &'static str {
        match self.role.load(Ordering::Relaxed) {
            1 => "Master",
            2 => "Member",
            _ => "Standalone",
        }
    }
}
