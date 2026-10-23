//! The session clock (decisions §24 "Step 9 hardening", "Session clock
//! (researched)").
//!
//! One clock per station, an overlay on the monotonic `Instant` (statime's
//! `OverlayClock`): local µs started at the Unix time the process started
//! and then only move with `Instant`; session time is
//!
//! ```text
//! session = base + (local − anchor) × (1 + rate)
//! ```
//!
//! where `rate` = frequency (the drift against the master) + correction
//! (closing the offset). Changing the rate re-anchors at the current session
//! time, so the clock is continuous; the rate is kept within ±5 %, so it never
//! runs backwards. Only `step` jumps (before the first render after joining,
//! or a panic beyond 1 s). A standalone station or a master just runs at its
//! last rate — a new master keeps the time it had.
//!
//! Reading takes no lock and allocates nothing (the render tick reads it
//! every frame): the three numbers are published under a sequence counter.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering, fence};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// The rate never leaves this (parts per billion): ±5 %.
pub const MAX_RATE_PPB: i64 = 50_000_000;

/// Where local time starts: the `Instant` and Unix µs at the first read.
static ORIGIN: OnceLock<(Instant, u64)> = OnceLock::new();

/// The overlay, published under `SEQ` (odd while being written).
static SEQ: AtomicU64 = AtomicU64::new(0);
static ANCHOR_LOCAL: AtomicU64 = AtomicU64::new(0);
static BASE_SESSION: AtomicU64 = AtomicU64::new(0);
static RATE_PPB: AtomicI64 = AtomicI64::new(0);
/// Writers take turns (the sync thread, a step); readers never wait.
static WRITING: AtomicU64 = AtomicU64::new(0);

fn origin() -> (Instant, u64) {
    *ORIGIN.get_or_init(|| {
        let unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_micros() as u64);
        (Instant::now(), unix)
    })
}

/// Wall-clock milliseconds since the epoch: stamps, labels and log times — never for fades,
/// which run on the session clock (`session_ms`). The one copy (decisions §34 25.14).
pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// This station's monotonic clock, in µs (Unix time at process start +
/// elapsed).
pub fn local_us() -> u64 {
    let (at, unix) = origin();
    unix + at.elapsed().as_micros() as u64
}

/// (anchor, base, rate) as one consistent read.
fn overlay() -> (u64, u64, i64) {
    loop {
        let s1 = SEQ.load(Ordering::Acquire);
        if s1 & 1 == 1 {
            std::hint::spin_loop();
            continue;
        }
        let anchor = ANCHOR_LOCAL.load(Ordering::Relaxed);
        let base = BASE_SESSION.load(Ordering::Relaxed);
        let rate = RATE_PPB.load(Ordering::Relaxed);
        fence(Ordering::Acquire);
        if SEQ.load(Ordering::Relaxed) == s1 {
            return (anchor, base, rate);
        }
    }
}

/// Session time at local time `local`.
fn session_at(local: u64, (anchor, base, rate): (u64, u64, i64)) -> u64 {
    if anchor == 0 && base == 0 {
        // Never adjusted: session time is local time.
        return local;
    }
    let elapsed = local.saturating_sub(anchor) as i128;
    let adjusted = elapsed + elapsed * rate as i128 / 1_000_000_000;
    (base as i128 + adjusted).max(0) as u64
}

fn publish(anchor: u64, base: u64, rate: i64) {
    while WRITING
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        std::hint::spin_loop();
    }
    let s = SEQ.load(Ordering::Relaxed);
    SEQ.store(s + 1, Ordering::Relaxed);
    fence(Ordering::Release);
    ANCHOR_LOCAL.store(anchor, Ordering::Relaxed);
    BASE_SESSION.store(base, Ordering::Relaxed);
    RATE_PPB.store(rate, Ordering::Relaxed);
    SEQ.store(s + 2, Ordering::Release);
    WRITING.store(0, Ordering::Release);
}

/// The session clock, in µs.
pub fn session_us() -> u64 {
    session_at(local_us(), overlay())
}

/// The session clock, in ms.
pub fn session_ms() -> u64 {
    session_us() / 1000
}

/// Session time minus local time (µs): how far this station's session clock
/// has been moved from its own monotonic clock.
pub fn offset_us() -> i64 {
    let local = local_us();
    session_at(local, overlay()) as i64 - local as i64
}

/// The current rate (ppb): frequency + correction.
pub fn rate_ppb() -> i64 {
    overlay().2
}

/// Run at `rate` ppb from now on (clamped to ±5 %), continuing from the
/// current session time.
pub fn set_rate_ppb(rate: i64) {
    let local = local_us();
    let now = session_at(local, overlay());
    publish(local, now, rate.clamp(-MAX_RATE_PPB, MAX_RATE_PPB));
}

/// Jump the session clock by `delta` µs (either way), keeping the rate.
/// Only for the first lock after joining and for a panic beyond 1 s.
pub fn step_us(delta: i64) {
    let local = local_us();
    let (a, b, rate) = overlay();
    let now = session_at(local, (a, b, rate)) as i64;
    publish(local, (now + delta).max(0) as u64, rate);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuous_monotonic_and_rate() {
        let a = session_us();
        set_rate_ppb(10_000_000); // +1 %
        let b = session_us();
        assert!(b >= a, "continuous at a rate change");
        let (l0, s0) = (local_us(), session_us());
        std::thread::sleep(std::time::Duration::from_millis(50));
        let (l1, s1) = (local_us(), session_us());
        let ratio = (s1 - s0) as f64 / (l1 - l0) as f64;
        assert!((ratio - 1.01).abs() < 0.002, "ratio {ratio}");
        // Clamped: never backwards.
        set_rate_ppb(-2_000_000_000);
        assert_eq!(rate_ppb(), -MAX_RATE_PPB);
        let c = session_us();
        assert!(session_us() >= c);
        set_rate_ppb(0);
        let before = session_us();
        step_us(1_000_000);
        assert!(session_us() >= before + 1_000_000);
        step_us(-1_000_000);
    }
}
