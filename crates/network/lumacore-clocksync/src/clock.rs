//  ██████╗██╗      ██████╗  ██████╗██╗  ██╗
// ██╔════╝██║     ██╔═══██╗██╔════╝██║ ██╔╝
// ██║     ██║     ██║   ██║██║     █████╔╝
// ██║     ██║     ██║   ██║██║     ██╔═██╗
// ╚██████╗███████╗╚██████╔╝╚██████╗██║  ██╗
//  ╚═════╝╚══════╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝
// FILE: clock

//! The session clock sync (decisions §24 "Session clock (researched)":
//! Ableton Link, Snapcast, nqptp, PipeWire, FPP, RFC 5905, chrony, ntpd-rs,
//! statime, linuxptp and game netcode were studied).
//!
//! - **Pings on the UDP command port.** A member sends `Time { t1 }` to its
//!   master; the master's UDP thread stamps `t2` the moment it reads it and
//!   `t3` just before the reply ([`answer`]) — never behind a snapshot on
//!   the stream, never through the state thread. The member reads the reply
//!   at `t4`. Times are session µs on the master, local µs on the member.
//! - **Burst, then steady.** 16 pings 20 ms apart when it starts following a
//!   master (joining, a new master, a step, a gap, 3 outliers in a row),
//!   then one every 500 ms.
//! - **Filter** ([`Filter`]): 32 samples; a sample whose round trip is a 5σ
//!   outlier is dropped once (the second in a row is kept — a real path
//!   change, ntpd-rs); samples above min RTT + max(0.5 ms, ½ min RTT) don't
//!   count. Frequency = the RTT-weighted linear regression of (master − local)
//!   over the window (chrony); offset = the median of the surviving samples,
//!   each moved to now along that slope.
//! - **Discipline** (statime overlay, `lumacore_base::clock`): the session
//!   clock runs at 1 + frequency + correction; correction = offset / 2 s,
//!   at most 0.5 % (2 % while more than 20 ms off). A station ahead slows
//!   down: session time never runs backwards. It steps only once, when it
//!   first locks after joining (before it renders that master's time), or
//!   beyond 1 s (panic, logged).
//! - **Holdover.** 4 pings unanswered (about 3 s), or no master: the clock runs on at
//!   its last frequency. A new master keeps the time it had; members drop
//!   their samples, keep their frequency and burst again (linuxptp).
//! - **Stats** in PTP terms (`Runtime.Health.Clock`); members report theirs
//!   in each ping, so the master lists them all.
//!
//! Owns:          the Time ping answer, the follow target and role, the sync thread, the sample filter (RTT gate, median, regression), the discipline of the session clock, the stats in PTP terms.
//! Doesn't own:   the clock overlay (lumacore_base::clock), who the master is (lumasession), the System Monitor (the caller's warn).
//! Entry points:  answer, follow, set_role, spawn_clock_sync, Filter, Sample, correction_for.
//! Decisions:     §24 "Session clock (researched)", §34 24.2.

use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use lumacore_base::clock;
use lumacore_tree::runtime::{HealthBoard, MemberClock, clock_sync};
use lumacore_wire::{ClockReport, RemoteRequest, TimeReply};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Samples the filter keeps.
const WINDOW: usize = 32;
/// Pings in a burst, and their spacing.
const BURST: u32 = 16;
const BURST_GAP: Duration = Duration::from_millis(20);
/// Steady ping interval.
const STEADY: Duration = Duration::from_millis(500);
/// How long a ping waits for its answer.
const ANSWER_WAIT: Duration = Duration::from_millis(250);
/// Pings in a row without an answer before holdover (about 3 s: each
/// waits `ANSWER_WAIT`, then `STEADY`).
const HOLDOVER_AFTER: u32 = 4;
/// Time to close an offset (the correction rate is offset / this).
const STEER_SECS: i64 = 2;
/// Correction limits (ppb): 0.5 %, and 2 % while more than 20 ms off.
const MAX_CORRECTION_PPB: i64 = 5_000_000;
const CATCH_UP_PPB: i64 = 20_000_000;
const CATCH_UP_BEYOND_US: i64 = 20_000;
/// Frequency limit (ppb): ±500 ppm.
const MAX_FREQUENCY_PPB: i64 = 500_000;
/// Beyond this a locked clock steps (panic).
const PANIC_US: i64 = 1_000_000;
/// Inside this the offset is left alone (µs).
const DEAD_BAND_US: i64 = 100;
/// Stable: this many offsets in a row inside `STABLE_US`.
const STABLE_COUNT: u32 = 10;
const STABLE_US: i64 = 1_000;

/// State indices (`lumacore_runtime::render_stats::CLOCK_STATES`).
const UNLOCKED: usize = 0;
const ACQUIRING: usize = 1;
const LOCKED: usize = 2;
const STABLE: usize = 3;
const HOLDOVER: usize = 4;

// *************************************************************************************************
// Master side
// *************************************************************************************************

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Answer a member's ping at the socket (`t2` was taken when it was read)
/// and keep its report for `Runtime.Health.Clock`.
#[allow(clippy::too_many_arguments, reason = "the four timestamps and who sent them")]
pub fn answer(
    socket: &UdpSocket,
    addr: SocketAddr,
    board: &HealthBoard,
    seq: u64,
    t1: u64,
    t2: u64,
    from: &str,
    report: Option<ClockReport>,
) {
    let master = clock_sync().role.load(Ordering::Relaxed) == 1;
    let reply = TimeReply {
        seq,
        t1,
        t2,
        t3: clock::session_us(),
        master,
    };
    if let Ok(bytes) = serde_json::to_vec(&reply) {
        let _ = socket.send_to(&bytes, addr);
    }
    if let (true, Some(r)) = (master, report) {
        board.set_member_clock(
            from,
            Some(MemberClock {
                station: from.to_string(),
                state: r.state,
                offset_us: r.offset_us,
                path_delay_us: r.path_delay_us,
                jitter_us: r.jitter_us,
                frequency_ppb: r.frequency_ppb,
                last_sync_ms: lumacore_tree::now_ms(),
            }),
        );
    }
}

// *************************************************************************************************
// Member side
// *************************************************************************************************
/// The master a member follows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The master's command port.
    pub addr: SocketAddr,
    pub master: String,
    pub epoch: u64,
    /// This station's name (the master lists its report under it).
    pub from: String,
}

#[derive(Default)]
struct Shared {
    target: Mutex<Option<Target>>,
    stop: AtomicBool,
}

fn shared() -> &'static Arc<Shared> {
    static SHARED: OnceLock<Arc<Shared>> = OnceLock::new();
    SHARED.get_or_init(|| Arc::new(Shared::default()))
}

/// Follow `target` (a member's link connected) or nobody (left, link lost:
/// holdover).
pub fn follow(target: Option<Target>) {
    let s = shared();
    *s.target.lock().unwrap_or_else(|e| e.into_inner()) = target.clone();
    if let Some(t) = target {
        set_role(2);
        *clock_sync().master.lock().unwrap_or_else(|e| e.into_inner()) = t.master;
        clock_sync().epoch.store(t.epoch, Ordering::Relaxed);
    }
}

/// This station's role for `Runtime.Health.Clock` (0 standalone, 1 master,
/// 2 member), kept by the state thread.
pub fn set_role(role: u8) {
    let sync = clock_sync();
    if sync.role.swap(role, Ordering::Relaxed) != role && role != 2 {
        // A master or a standalone station is its own reference: it runs on
        // at the frequency it had (a new master keeps its time).
        let state = sync.state.load(Ordering::Relaxed) as usize;
        if matches!(state, ACQUIRING | LOCKED | STABLE) {
            sync.set_state(HOLDOVER);
        }
        sync.correction_ppb.store(0, Ordering::Relaxed);
        clock::set_rate_ppb(sync.frequency_ppb.load(Ordering::Relaxed));
    }
}

/// Start the sync thread (once per process; it idles without a target). `warn` takes the one
/// line it says when the clock had to step while locked (the station's System Monitor).
pub fn spawn_clock_sync(stop: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("clock-sync".to_string())
        .spawn(move || run(&stop))
        .expect("should spawn clock sync thread")
}

/// One measurement: local midpoint, master − local at that moment, round
/// trip (µs).
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    pub local: u64,
    pub delta: i64,
    pub rtt: u64,
}

/// What the filter makes of its window.
#[derive(Debug, Clone, Copy, Default)]
pub struct Estimate {
    /// master − local now (µs).
    pub delta_now: i64,
    pub frequency_ppb: i64,
    pub skew_ppb: u64,
    pub jitter_us: u64,
    pub path_delay_us: u64,
    pub used: usize,
}

/// The sample filter.
#[derive(Debug, Default)]
pub struct Filter {
    samples: VecDeque<Sample>,
    prev_outlier: bool,
    /// The last frequency (kept across masters and gaps).
    pub frequency_ppb: i64,
}

impl Filter {
    /// Forget the samples (a new master, a step); the frequency stays.
    pub fn restart(&mut self) {
        self.samples.clear();
        self.prev_outlier = false;
    }

    /// Add a sample; None when it was dropped as an outlier.
    pub fn add(&mut self, s: Sample, now_local: u64) -> Option<Estimate> {
        if self.samples.len() >= 8 {
            let n = self.samples.len() as f64;
            let mean = self.samples.iter().map(|x| x.rtt as f64).sum::<f64>() / n;
            let var = self
                .samples
                .iter()
                .map(|x| (x.rtt as f64 - mean).powi(2))
                .sum::<f64>()
                / n;
            let sd = var.sqrt().max(50.0);
            if s.rtt as f64 > mean + 5.0 * sd && !self.prev_outlier {
                self.prev_outlier = true;
                return None;
            }
        }
        self.prev_outlier = false;
        if self.samples.len() == WINDOW {
            self.samples.pop_front();
        }
        self.samples.push_back(s);
        Some(self.estimate(now_local))
    }

    fn estimate(&mut self, now_local: u64) -> Estimate {
        let min_rtt = self.samples.iter().map(|x| x.rtt).min().unwrap_or(0);
        let gate = min_rtt + (min_rtt / 2).max(500);
        let used: Vec<Sample> = self.samples.iter().copied().filter(|x| x.rtt <= gate).collect();

        // RTT-weighted regression of delta over local time (chrony): the
        // slope is the frequency. Needs a few samples over a second or more.
        let spread = used.iter().map(|x| x.rtt).max().unwrap_or(0).saturating_sub(min_rtt).max(100) as f64;
        let weights: Vec<f64> = used
            .iter()
            .map(|x| 1.0 / (1.0 + (x.rtt - min_rtt) as f64 / spread).powi(2))
            .collect();
        let span = match (used.first(), used.last()) {
            (Some(a), Some(b)) => b.local.saturating_sub(a.local),
            _ => 0,
        };
        let mut skew_ppb = 0;
        if used.len() >= 4 && span >= 1_000_000 {
            let w: f64 = weights.iter().sum();
            let x0 = used[0].local as f64;
            let xm = used.iter().zip(&weights).map(|(s, w)| (s.local as f64 - x0) * w).sum::<f64>() / w;
            let ym = used.iter().zip(&weights).map(|(s, w)| s.delta as f64 * w).sum::<f64>() / w;
            let sxx: f64 = used.iter().zip(&weights).map(|(s, w)| w * (s.local as f64 - x0 - xm).powi(2)).sum();
            let sxy: f64 = used
                .iter()
                .zip(&weights)
                .map(|(s, w)| w * (s.local as f64 - x0 - xm) * (s.delta as f64 - ym))
                .sum();
            if sxx > 0.0 {
                let slope = sxy / sxx;
                let resid: f64 = used
                    .iter()
                    .zip(&weights)
                    .map(|(s, w)| {
                        let fit = ym + slope * (s.local as f64 - x0 - xm);
                        w * (s.delta as f64 - fit).powi(2)
                    })
                    .sum::<f64>()
                    / (used.len() as f64 - 2.0).max(1.0);
                skew_ppb = ((resid / sxx).sqrt() * 1e9) as u64;
                self.frequency_ppb = ((slope * 1e9) as i64).clamp(-MAX_FREQUENCY_PPB, MAX_FREQUENCY_PPB);
            }
        }
        let slope = self.frequency_ppb as f64 / 1e9;

        // Every sample moved to now along the slope; the median is the offset.
        let mut now: Vec<i64> = used
            .iter()
            .map(|s| s.delta + (slope * now_local.saturating_sub(s.local) as f64) as i64)
            .collect();
        now.sort_unstable();
        let delta_now = now.get(now.len() / 2).copied().unwrap_or(0);
        let mut dev: Vec<u64> = now.iter().map(|d| (d - delta_now).unsigned_abs()).collect();
        dev.sort_unstable();
        Estimate {
            delta_now,
            frequency_ppb: self.frequency_ppb,
            skew_ppb,
            jitter_us: dev.get(dev.len() / 2).copied().unwrap_or(0),
            path_delay_us: min_rtt / 2,
            used: used.len(),
        }
    }
}

/// The correction rate (ppb) for an offset of `error` µs (positive = the
/// master is ahead).
pub fn correction_for(error: i64) -> i64 {
    if error.abs() <= DEAD_BAND_US {
        return 0;
    }
    let limit = if error.abs() > CATCH_UP_BEYOND_US {
        CATCH_UP_PPB
    } else {
        MAX_CORRECTION_PPB
    };
    // error µs over STEER_SECS s = error / STEER_SECS µs per s = that many ppm.
    (error * 1_000 / STEER_SECS).clamp(-limit, limit)
}

/// What this station tells its master.
fn report() -> ClockReport {
    let s = clock_sync();
    ClockReport {
        state: s.state_label().to_string(),
        offset_us: s.offset_us.load(Ordering::Relaxed),
        path_delay_us: s.path_delay_us.load(Ordering::Relaxed),
        jitter_us: s.jitter_us.load(Ordering::Relaxed),
        frequency_ppb: s.frequency_ppb.load(Ordering::Relaxed),
        last_sync_ms: s.last_sync_ms.load(Ordering::Relaxed),
    }
}

fn record_step(delta: i64) {
    let s = clock_sync();
    clock::step_us(delta);
    s.steps.fetch_add(1, Ordering::Relaxed);
    s.last_step_ms.store(lumacore_tree::now_ms(), Ordering::Relaxed);
    s.last_step_us.store(delta, Ordering::Relaxed);
}

/// Ask `target` once; the sample, or None (no answer).
fn ping(socket: &UdpSocket, target: &Target, seq: u64) -> Option<Sample> {
    let l1 = clock::local_us();
    let request = RemoteRequest::Time {
        seq,
        t1: l1,
        from: target.from.clone(),
        report: Some(report()),
    };
    let bytes = serde_json::to_vec(&request).ok()?;
    socket.send_to(&bytes, target.addr).ok()?;
    let mut buf = [0u8; 512];
    loop {
        let (n, from) = socket.recv_from(&mut buf).ok()?;
        let l4 = clock::local_us();
        if from != target.addr {
            continue;
        }
        let Ok(reply) = serde_json::from_slice::<TimeReply>(&buf[..n]) else {
            continue;
        };
        if reply.seq != seq {
            // A late answer to an earlier ping: keep waiting for ours.
            continue;
        }
        let held = reply.t3.saturating_sub(reply.t2);
        let rtt = l4.saturating_sub(l1).saturating_sub(held);
        let master_mid = (reply.t2 + reply.t3) / 2;
        let local_mid = (l1 + l4) / 2;
        return Some(Sample {
            local: local_mid,
            delta: master_mid as i64 - local_mid as i64,
            rtt,
        });
    }
}

fn run(stop: &AtomicBool) {
    let Ok(socket) = UdpSocket::bind(("0.0.0.0", 0)) else {
        log::error!("clock sync: no UDP socket");
        return;
    };
    let _ = socket.set_read_timeout(Some(ANSWER_WAIT));
    let sync = clock_sync();
    let mut filter = Filter::default();
    let mut following: Option<Target> = None;
    let mut seq = 0u64;
    let mut burst = 0u32;
    let mut misses = 0u32;
    let mut outliers = 0u32;
    let mut locked_once = false;
    let mut calm = 0u32;

    while !stop.load(Ordering::Relaxed) && !shared().stop.load(Ordering::Relaxed) {
        let target = shared().target.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let Some(target) = target else {
            if following.take().is_some() {
                if locked_once {
                    hold(&filter);
                } else {
                    sync.set_state(UNLOCKED);
                }
            }
            thread::sleep(Duration::from_millis(100));
            continue;
        };
        if following.as_ref() != Some(&target) {
            // A new master (or the first): drop the samples, keep the
            // frequency, burst. Joining a session steps once when it locks.
            if following.as_ref().is_none_or(|f| f.addr != target.addr || f.master != target.master) {
                locked_once = false;
            }
            following = Some(target.clone());
            filter.restart();
            burst = BURST;
            calm = 0;
            sync.set_state(ACQUIRING);
        }

        seq += 1;
        match ping(&socket, &target, seq) {
            None => {
                misses += 1;
                if misses >= HOLDOVER_AFTER && sync.state.load(Ordering::Relaxed) as usize != HOLDOVER {
                    hold(&filter);
                }
            }
            Some(sample) => {
                misses = 0;
                let now_local = clock::local_us();
                match filter.add(sample, now_local) {
                    None => {
                        sync.samples_rejected.fetch_add(1, Ordering::Relaxed);
                        outliers += 1;
                        if outliers >= 3 {
                            outliers = 0;
                            burst = BURST;
                        }
                    }
                    Some(e) => {
                        outliers = 0;
                        sync.samples_used.fetch_add(1, Ordering::Relaxed);
                        // master − our session clock, now.
                        let error = e.delta_now - clock::offset_us();
                        let ready = burst <= 1 || e.used >= 8;
                        if !locked_once && ready {
                            // First lock after joining: one step, before this
                            // master's time is rendered.
                            if error.abs() > DEAD_BAND_US {
                                record_step(error);
                            }
                            locked_once = true;
                            clock::set_rate_ppb(e.frequency_ppb);
                            sync.set_state(LOCKED);
                        } else if locked_once && error.abs() > PANIC_US {
                            lumacore_sysmon::sysmon_early("Clock", format!("[WARN] session clock {error:+} µs off its master: stepped"));
                            record_step(error);
                            filter.restart();
                            burst = BURST;
                            sync.set_state(ACQUIRING);
                        } else if locked_once {
                            let correction = correction_for(error);
                            clock::set_rate_ppb(e.frequency_ppb + correction);
                            sync.correction_ppb.store(correction, Ordering::Relaxed);
                            calm = if error.abs() <= STABLE_US { calm + 1 } else { 0 };
                            sync.set_state(if calm >= STABLE_COUNT { STABLE } else { LOCKED });
                        }
                        sync.offset_us.store(error, Ordering::Relaxed);
                        sync.path_delay_us.store(e.path_delay_us, Ordering::Relaxed);
                        sync.jitter_us.store(e.jitter_us, Ordering::Relaxed);
                        sync.frequency_ppb.store(e.frequency_ppb, Ordering::Relaxed);
                        sync.skew_ppb.store(e.skew_ppb, Ordering::Relaxed);
                        sync.last_sync_ms.store(lumacore_tree::now_ms(), Ordering::Relaxed);
                    }
                }
            }
        }
        let wait = if burst > 0 {
            burst -= 1;
            BURST_GAP
        } else {
            STEADY
        };
        thread::sleep(wait);
    }
}

/// Holdover: run on at the last frequency, no correction.
fn hold(filter: &Filter) {
    let sync = clock_sync();
    clock::set_rate_ppb(filter.frequency_ppb);
    sync.correction_ppb.store(0, Ordering::Relaxed);
    sync.set_state(HOLDOVER);
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
