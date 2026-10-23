// ██╗     ██╗███╗   ██╗███████╗███████╗
// ██║     ██║████╗  ██║██╔════╝██╔════╝
// ██║     ██║██╔██╗ ██║█████╗  ███████╗
// ██║     ██║██║╚██╗██║██╔══╝  ╚════██║
// ███████╗██║██║ ╚████║███████╗███████║
// ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝
// FILE: lines

//! Drivers per (protocol, network) (decisions §11 "Step 4b/4c hardening",
//! §24 "Step 9 hardening"). A driver holds the sockets of one NIC and runs
//! on its own thread:
//!
//! - **Output:** each line samples the engine's newest frame at its own
//!   `Rate` (Hz; never above what an Art-Net node reports it can take) and
//!   sends its routed universes — `Full` every time, or `Changes` (a change
//!   at once, 3 repeats, then a keep-alive every second). `PacketGap`
//!   spaces universes for old nodes; `Sync` sends ArtSync / an E1.31 sync
//!   packet after the line's universes. Packets are preallocated: the send
//!   allocates nothing (checked in debug builds).
//! - **Input:** each input line's protocol universes, merged over their
//!   sources into layers the engine merges into our output.
//!
//! The plans are swapped live without restarting the driver; a different
//! NIC means a new driver.
//!
//! The protocol is a `Box<dyn DmxDriver>` its catalog row opened (decisions §34 22): this file
//! never names one.
//!
//! Owns:          the driver thread per (protocol, NIC): NetworkDriver, DriverShared; the send loop and the input loop.
//! Doesn't own:   what a line sends (lumacore_protocols::LinePlan), the packets (the protocol crates), the frames (lumacore_render).
//! Entry points:  NetworkDriver, DriverShared.
//! Decisions:     §11 "Step 4b/4c hardening", §24 "Step 9 hardening", §34 22.

use std::collections::BTreeSet;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use lumacore_protocols::LineSend;
use lumacore_tree::runtime::LineStats;

use lumacore_protocols::input::{InputLayer, InputMerger, InputPlan, Packet, SourceInfo};
use lumacore_protocols::{DmxDriver, LinePlan, Node};
use lumacore_render::engine::{FRAME_PERIOD, RenderHub};
use lumacore_render::render::RenderedFrame;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Shared between a driver thread and the state thread (lock-free).
#[derive(Default)]
pub struct DriverShared {
    pub plan: ArcSwap<Vec<LinePlan>>,
    pub inputs: ArcSwap<Vec<InputPlan>>,
    /// The last socket error, cleared when it works again.
    pub error: ArcSwap<Option<String>>,
    /// The nodes the protocol finds on this network (Art-Net: ArtPoll).
    pub nodes: ArcSwap<Vec<Node>>,
    /// Input layers live now (one per input line and local universe).
    pub layers: ArcSwap<Vec<InputLayer>>,
    /// Input sources heard now.
    pub sources: ArcSwap<Vec<SourceInfo>>,
    /// Send numbers per output line.
    pub lines: ArcSwap<Vec<(u32, Arc<LineStats>)>>,
    /// Packets sent.
    pub sent: AtomicU64,
    /// DMX packets received.
    pub received: AtomicU64,
    /// A socket that shares its protocol port (`SO_REUSEPORT`) couldn't be
    /// pinned to the NIC: unicast may be lost to another program on the
    /// port (R1-54; shown by `Info Output` / `Info Input`).
    pub unpinned: AtomicBool,
}

/// A running driver.
pub struct NetworkDriver {
    pub shared: Arc<DriverShared>,
    hub: Arc<RenderHub>,
    id: u64,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl NetworkDriver {
    /// Start sending `plan` / taking in `inputs`.
    pub fn start(
        hub: Arc<RenderHub>,
        sender: Box<dyn DmxDriver>,
        plan: Vec<LinePlan>,
        inputs: Vec<InputPlan>,
        thread_name: &str,
        health: Option<Arc<lumacore_tree::runtime::ThreadHealth>>,
    ) -> Self {
        let shared = Arc::new(DriverShared::default());
        shared.plan.store(Arc::new(plan));
        shared.inputs.store(Arc::new(inputs));
        let id = hub.driver_id();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_shared = shared.clone();
        let thread_hub = hub.clone();
        let thread_stop = stop.clone();
        let thread = std::thread::Builder::new()
            .name(thread_name.to_string())
            .spawn(move || {
                // Just below the engine it takes frames from (decisions §31).
                let granted = lumacore_render::realtime::promote_current(lumacore_render::realtime::DRIVER_PRIORITY);
                if let Some(h) = &health {
                    h.set_scheduling(granted);
                }
                Driver {
                    sender,
                    shared: &thread_shared,
                    hub: &thread_hub,
                    id,
                    merger: InputMerger::new(),
                    packets: Vec::with_capacity(256),
                    lines: Vec::new(),
                    plan_seen: None,
                    nodes_checked: None,
                    alloc_reported: false,
                    health: health.clone(),
                }
                .run(&thread_stop);
                if let Some(h) = &health {
                    h.stopped();
                }
            })
            .expect("should spawn protocol driver thread");
        Self {
            shared,
            hub,
            id,
            stop,
            thread: Some(thread),
        }
    }

    pub fn set_plan(&self, plan: Vec<LinePlan>, inputs: Vec<InputPlan>) {
        if **self.shared.plan.load() != plan {
            self.shared.plan.store(Arc::new(plan));
        }
        if **self.shared.inputs.load() != inputs {
            self.shared.inputs.store(Arc::new(inputs));
        }
    }

    /// Stop: the driver says goodbye and ends. Returns the thread to join.
    pub fn stop(&mut self) -> Option<JoinHandle<()>> {
        self.stop.store(true, Ordering::Relaxed);
        self.thread.take()
    }
}

impl Drop for NetworkDriver {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.hub.set_inputs(self.id, Arc::new(Vec::new()));
    }
}

/// Changes mode: repeats after a change, and the keep-alive (Art-Net 4 and
/// E1.31 both recommend 800–1000 ms).
const REPEATS: u8 = 3;
const KEEP_ALIVE: Duration = Duration::from_secs(1);
/// A line is due this early (keeps a 30 Hz line on every second frame).
const DUE_SLACK: Duration = Duration::from_millis(2);

/// One routed universe of a line.
struct RouteState {
    local: u32,
    wire: u16,
    /// Where it sends (Art-Net: precomputed from the nodes, so the send allocates nothing).
    targets: Vec<Ipv4Addr>,
    last: [u8; 512],
    last_sent: Option<Instant>,
    repeats: u8,
}

/// One output line's schedule.
struct LineState {
    plan: LinePlan,
    period: Duration,
    next_due: Instant,
    stats: Arc<LineStats>,
    routes: Vec<RouteState>,
    window_start: Instant,
    window_frames: u64,
    window_send_max: u64,
    /// The engine frame the last send took (0 = none yet).
    last_frame_no: u64,
}

/// Engine frames a line's period spans: 2 at 30 Hz, 3 at 20 Hz, 2 at 44 Hz (it takes
/// every 1st or 2nd frame, so a jump of 2 is still on time).
fn frames_per_send(period: Duration) -> u64 {
    let ratio = period.as_secs_f64() / FRAME_PERIOD.as_secs_f64();
    // Periods are whole nanoseconds, so 1 Hz is 60.000002 frames: round off the dust first.
    (ratio - 1e-3).ceil().max(1.0) as u64
}

/// What one send says about a line's timing against the engine (decisions §31 "Line
/// timing"): `(doubled, skipped)` from the frame it took, the one before, and
/// [`frames_per_send`]. No engine frame yet, a first send or an engine restart (the counter
/// went back) count nothing.
fn frame_cadence(previous: u64, current: u64, step: u64) -> (u64, u64) {
    if previous == 0 || current == 0 || current < previous {
        return (0, 0);
    }
    if current == previous {
        return (1, 0);
    }
    (0, (current - previous).saturating_sub(step))
}

struct Driver<'a> {
    sender: Box<dyn DmxDriver>,
    shared: &'a DriverShared,
    hub: &'a RenderHub,
    id: u64,
    merger: InputMerger,
    /// Received packets, reused every loop (no allocation per packet).
    packets: Vec<Packet>,
    lines: Vec<LineState>,
    plan_seen: Option<Arc<Vec<LinePlan>>>,
    nodes_checked: Option<Instant>,
    alloc_reported: bool,
    /// `Runtime.Health.Threads.Line.<protocol>-<ip>`.
    health: Option<Arc<lumacore_tree::runtime::ThreadHealth>>,
}

/// Hz → period.
fn period_of(hz: u64) -> Duration {
    Duration::from_nanos(1_000_000_000 / hz.clamp(1, 60))
}

impl Driver<'_> {
    fn run(mut self, stop: &AtomicBool) {
        while !stop.load(Ordering::Relaxed) {
            let now = Instant::now();
            let mut first_error = None;
            self.follow_plan(now);
            self.send_due(now, &mut first_error);
            self.receive(&mut first_error);
            if let Some(h) = &self.health {
                h.tick(now.elapsed().as_micros() as u64);
            }
            if **self.shared.error.load() != first_error {
                self.shared.error.store(Arc::new(first_error));
            }
            // Wake for the next due line, at least once a frame (input).
            let next = self
                .lines
                .iter()
                .map(|l| l.next_due)
                .min()
                .unwrap_or(now + FRAME_PERIOD)
                .min(Instant::now() + FRAME_PERIOD);
            let now = Instant::now();
            if next > now {
                spin_sleep::sleep(next - now);
            }
        }
        self.hub.set_inputs(self.id, Arc::new(Vec::new()));
        // Dropping the sender ends its streams (sACN stream-terminated).
    }

    /// Rebuild the line schedule when the plan (or the nodes' rates)
    /// changes. Allocates; never inside the send.
    fn follow_plan(&mut self, now: Instant) {
        let plan = self.shared.plan.load_full();
        let nodes_due = self.nodes_checked.is_none_or(|t| now - t >= Duration::from_secs(1));
        let changed = self.plan_seen.as_ref().is_none_or(|p| !Arc::ptr_eq(p, &plan));
        if !changed && !nodes_due {
            return;
        }
        self.nodes_checked = Some(now);
        let nodes: Vec<Node> = self.sender.nodes();
        if changed {
            let mut lines = Vec::with_capacity(plan.len());
            let mut stats = Vec::with_capacity(plan.len());
            for p in plan.iter() {
                // Keep a line's schedule and numbers across plan edits.
                let old = self.lines.iter().position(|l| l.plan.line == p.line);
                let old = old.map(|i| self.lines.swap_remove(i));
                let line_stats = old
                    .as_ref()
                    .map_or_else(|| Arc::new(LineStats::default()), |o| o.stats.clone());
                let routes = p
                    .routes
                    .iter()
                    .map(|&(local, wire)| {
                        let prev = old
                            .as_ref()
                            .and_then(|o| o.routes.iter().find(|r| r.local == local && r.wire == wire));
                        RouteState {
                            local,
                            wire,
                            targets: Vec::new(),
                            last: prev.map_or([0; 512], |r| r.last),
                            last_sent: prev.and_then(|r| r.last_sent),
                            repeats: prev.map_or(REPEATS, |r| r.repeats),
                        }
                    })
                    .collect();
                stats.push((p.line, line_stats.clone()));
                lines.push(LineState {
                    plan: p.clone(),
                    period: period_of(u64::from(p.rate_hz)),
                    next_due: old.as_ref().map_or(now, |o| o.next_due),
                    stats: line_stats,
                    routes,
                    window_start: now,
                    window_frames: 0,
                    window_send_max: 0,
                    last_frame_no: old.as_ref().map_or(0, |o| o.last_frame_no),
                });
            }
            self.lines = lines;
            self.shared.lines.store(Arc::new(stats));
            self.sender.plan(&plan);
            self.plan_seen = Some(plan);
        }
        // Targets per route and the nodes' rate caps (a protocol without targets or nodes
        // keeps 60).
        for line in &mut self.lines {
            let mut cap = 60u64;
            for r in &mut line.routes {
                r.targets = self.sender.targets(r.wire, line.plan.unicast);
                for n in nodes.iter().filter(|n| r.targets.contains(&n.ip)) {
                    if let Some(hz) = n.refresh_hz {
                        cap = cap.min(u64::from(hz));
                    }
                }
            }
            let streams: Vec<(Ipv4Addr, u16)> = line
                .routes
                .iter()
                .flat_map(|r| r.targets.iter().map(|&ip| (ip, r.wire)))
                .collect();
            self.sender.prepare(&streams);
            let rate = u64::from(line.plan.rate_hz).clamp(1, 60);
            let effective = rate.min(cap.max(1));
            line.period = period_of(effective);
            line.stats.rate_hz.store(rate, Ordering::Relaxed);
            line.stats.effective_hz.store(effective, Ordering::Relaxed);
        }
    }

    /// Send every due line from the newest frame.
    fn send_due(&mut self, now: Instant, first_error: &mut Option<String>) {
        if self.lines.iter().all(|l| l.next_due > now + DUE_SLACK) {
            return;
        }
        let frame = self.hub.frame.load_full();
        let mut sent_total = 0u64;
        let mut error: Option<std::io::ErrorKind> = None;
        for i in 0..self.lines.len() {
            if self.lines[i].next_due > now + DUE_SLACK {
                continue;
            }
            let started = Instant::now();
            #[cfg(debug_assertions)]
            let violations = assert_no_alloc::violation_count();
            let (packets, failed) = assert_no_alloc::assert_no_alloc(|| {
                send_line(self.sender.as_mut(), &mut self.lines[i], &frame, now)
            });
            #[cfg(debug_assertions)]
            if assert_no_alloc::violation_count() > violations && !self.alloc_reported {
                self.alloc_reported = true;
                log::error!(
                    "line {}: the send allocated (decisions §24: no allocation in the send)",
                    self.lines[i].plan.line
                );
            }
            let line = &mut self.lines[i];
            if let Some(kind) = failed {
                error.get_or_insert(kind);
                line.stats.errors.fetch_add(1, Ordering::Relaxed);
            }
            let took = started.elapsed().as_micros() as u64;
            let (doubled, skipped) = frame_cadence(line.last_frame_no, frame.frame_no, frames_per_send(line.period));
            line.last_frame_no = frame.frame_no;
            if doubled > 0 {
                line.stats.doubled.fetch_add(doubled, Ordering::Relaxed);
            }
            if skipped > 0 {
                line.stats.skipped.fetch_add(skipped, Ordering::Relaxed);
            }
            line.stats.frames.fetch_add(1, Ordering::Relaxed);
            line.stats.packets.fetch_add(packets, Ordering::Relaxed);
            line.stats.send_us.store(took, Ordering::Relaxed);
            line.window_frames += 1;
            line.window_send_max = line.window_send_max.max(took);
            if now - line.window_start >= Duration::from_secs(1) {
                line.stats
                    .actual_hz
                    .store(line.window_frames, Ordering::Relaxed);
                line.stats
                    .send_max_us
                    .store(line.window_send_max, Ordering::Relaxed);
                line.window_start = now;
                line.window_frames = 0;
                line.window_send_max = 0;
            }
            sent_total += packets;
            line.next_due += line.period;
            if line.next_due + DUE_SLACK < now {
                // Fell behind: no catch-up.
                line.next_due = now + line.period;
            }
        }
        if let Some(kind) = error {
            first_error.get_or_insert(format!("send: {kind}"));
        }
        self.shared.sent.fetch_add(sent_total, Ordering::Relaxed);
        self.sender.after_send();
    }

    fn receive(&mut self, first_error: &mut Option<String>) {
        let inputs = self.shared.inputs.load_full();
        let wires: BTreeSet<u16> = inputs
            .iter()
            .flat_map(|l| l.routes.iter().map(|&(_, w)| w))
            .collect();
        let now = Instant::now();
        let mut packets = std::mem::take(&mut self.packets);
        packets.clear();
        if let Some(e) = self.sender.receive(&wires, &mut packets) {
            first_error.get_or_insert(e);
        }
        if let Some(pinned) = self.sender.pinned() {
            self.shared.unpinned.store(!pinned, Ordering::Relaxed);
        }
        let nodes = self.sender.nodes();
        if **self.shared.nodes.load() != nodes {
            self.shared.nodes.store(Arc::new(nodes));
        }
        self.shared
            .received
            .fetch_add(packets.iter().filter(|p| !p.is_sync).count() as u64, Ordering::Relaxed);
        self.merger.ingest(&packets, now);
        self.packets = packets;
        if let Some(layers) = self.merger.layers(&inputs, now) {
            let layers = Arc::new(layers);
            self.shared.layers.store(layers.clone());
            self.hub.set_inputs(self.id, layers);
        }
        let sources = self.merger.sources();
        if **self.shared.sources.load() != sources {
            self.shared.sources.store(Arc::new(sources));
        }
    }
}

// tidy: render-begin (decisions §24: the per-frame path)
/// Send one line's universes from `frame`. Allocates nothing. Returns the
/// packets sent and the first error.
fn send_line(
    sender: &mut dyn DmxDriver,
    line: &mut LineState,
    frame: &RenderedFrame,
    now: Instant,
) -> (u64, Option<std::io::ErrorKind>) {
    let mut packets = 0u64;
    let mut error = None;
    let changes = line.plan.send == LineSend::Changes;
    let gap = Duration::from_micros(u64::from(line.plan.packet_gap_us));
    let mut first = true;
    for r in &mut line.routes {
        let Some(data) = frame.get(r.local) else {
            continue;
        };
        if changes {
            if *data != r.last {
                r.repeats = REPEATS;
            } else if r.repeats > 0 {
                r.repeats -= 1;
            } else if r.last_sent.is_some_and(|t| now - t < KEEP_ALIVE) {
                continue;
            }
        }
        if !first && !gap.is_zero() {
            spin_sleep::sleep(gap);
        }
        first = false;
        let result = sender.send(r.wire, data, &line.plan, &r.targets);
        match result {
            Ok(n) => packets += n,
            Err(e) => {
                error.get_or_insert(e.kind());
            }
        }
        r.last = *data;
        r.last_sent = Some(now);
    }
    if line.plan.sync
        && packets > 0
        && let Err(e) = sender.sync(&line.plan)
    {
        error.get_or_insert(e.kind());
    }
    (packets, error)
}
// tidy: render-end

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
