// ███████╗███╗   ██╗ ██████╗ ██╗███╗   ██╗███████╗
// ██╔════╝████╗  ██║██╔════╝ ██║████╗  ██║██╔════╝
// █████╗  ██╔██╗ ██║██║  ███╗██║██╔██╗ ██║█████╗
// ██╔══╝  ██║╚██╗██║██║   ██║██║██║╚██╗██║██╔══╝
// ███████╗██║ ╚████║╚██████╔╝██║██║ ╚████║███████╗
// ╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═╝  ╚═══╝╚══════╝
// FILE: engine

//! The render engine (decisions §24 "Step 9 hardening", §27): one 60 Hz
//! loop on a max-priority OS thread renders the compiled plan into frames;
//! output lines sample the newest frame at their own rates.
//!
//! - The state thread publishes `EngineInput` (show + users' modes +
//!   generations) with [`RenderHub::publish`]; the compiler thread
//!   (`crate::compiler`) turns it into a `Plan` + a pool of frame buffers
//!   (a `Bundle`) and hands it over. The engine swaps bundles at frame
//!   start and sends retired ones back, so it never frees memory.
//! - Every frame renders at the session clock's time (fades), merges DMX
//!   input (`RenderHub::inputs`, published by the line drivers) and is
//!   published in `RenderHub::frame`; `RenderHub::plan` is the plan
//!   that frame came from (readers: the values before masters).
//! - The tick allocates nothing, takes no lock and logs nothing (debug
//!   builds check allocations with `assert_no_alloc`: one is a panic).
//! - A panic holds the last frame (the lines keep sending it) and restarts
//!   the loop; a second panic within 10 s leaves the engine down.
//! - A late frame is never caught up: the next one renders at the real
//!   time; late and skipped frames and overload episodes are counted.
//!
//! Owns:          the 60 Hz render thread, RenderHub (the hand-off between the state thread, the compiler and the lines), Bundle, Retired, EngineHandle, spawn_engine.
//! Doesn't own:   the tick (render), the merge (merge), the RenderEngine wrapper (lumacore-engines render.rs).
//! Entry points:  RenderHub, spawn_engine, EngineHandle, RENDER_HZ, FRAME_PERIOD.
//! Decisions:     §24 "Step 9 hardening", §27, §34 20.4.

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use arrayvec::ArrayVec;
use crossbeam_channel::{Receiver, Sender, bounded};
use lumacore_tree::runtime::{RenderStats, STAGE_COUNT, ThreadHealth};

use crate::compiler::{CompilerHandle, spawn_compiler};
use lumacore_protocols::input::InputLayer;
use crate::plan::Plan;
use crate::merge::EngineInput;
use crate::render::{RenderedFrame, Stages, render_into, stage};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Every driver's input layers, by driver id.
pub type InputFrames = BTreeMap<u64, Arc<Vec<InputLayer>>>;

/// The render rate (decisions §24: fixed 60 Hz, the usual console realtime loop).
pub const RENDER_HZ: u64 = 60;
/// One frame: 1/60 s.
pub const FRAME_PERIOD: Duration = Duration::from_nanos(1_000_000_000 / RENDER_HZ);
/// Frame buffers per plan: the hub, the lines and readers hold a few.
pub const POOL: usize = 6;
/// A second panic within this time leaves the engine down.
pub const PANIC_WINDOW: Duration = Duration::from_secs(10);

/// A compiled plan and frame buffers shaped for it.
pub struct Bundle {
    pub plan: Arc<Plan>,
    pub pool: Vec<Arc<RenderedFrame>>,
}

/// Something the engine let go of; the compiler drops it, so the engine
/// never frees memory (the last reference may be the engine's).
pub enum Retired {
    Bundle(Bundle),
    Plan(Arc<Plan>),
    Frame(Arc<RenderedFrame>),
    Inputs(Arc<InputFrames>),
}

/// Shared between the state thread, the compiler, the engine, the line
/// drivers and readers.
pub struct RenderHub {
    /// What the compiler compiles (published by the state thread).
    pub input: ArcSwap<EngineInput>,
    /// The newest rendered frame.
    pub frame: ArcSwap<RenderedFrame>,
    /// The plan `frame` was rendered from.
    pub plan: ArcSwap<Plan>,
    /// DMX input layers, by driver id.
    pub inputs: ArcSwap<InputFrames>,
    /// `Runtime.Health.Render`.
    pub stats: Arc<RenderStats>,
    /// Show generation of the newest published frame.
    rendered: AtomicU64,
    /// Recompile even if the generation didn't change (after a panic).
    pub(crate) force_compile: AtomicBool,
    pub(crate) compile_tx: Sender<()>,
    pub(crate) compile_rx: Receiver<()>,
    pub(crate) bundle_tx: Sender<Bundle>,
    pub(crate) bundle_rx: Receiver<Bundle>,
    pub(crate) retire_tx: Sender<Retired>,
    pub(crate) retire_rx: Receiver<Retired>,
    next_id: AtomicU64,
    /// Start a stopped compiler again (a show load, R1-66).
    pub(crate) restart_compiler: AtomicBool,
    /// Test hook: the next compile panics (R1-66).
    #[cfg(test)]
    pub(crate) panic_next_compile: AtomicBool,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl RenderHub {
    pub fn new(input: EngineInput) -> Arc<Self> {
        Self::with_stats(input, Arc::new(RenderStats::default()))
    }

    pub fn with_stats(input: EngineInput, stats: Arc<RenderStats>) -> Arc<Self> {
        let (compile_tx, compile_rx) = bounded(1);
        let (bundle_tx, bundle_rx) = bounded(4);
        let (retire_tx, retire_rx) = bounded(256);
        stats.rate_hz.store(RENDER_HZ, Ordering::Relaxed);
        let hub = Arc::new(Self {
            input: ArcSwap::from_pointee(input),
            frame: ArcSwap::from_pointee(RenderedFrame::default()),
            plan: ArcSwap::from_pointee(Plan::empty()),
            inputs: ArcSwap::from_pointee(InputFrames::new()),
            stats,
            rendered: AtomicU64::new(0),
            force_compile: AtomicBool::new(false),
            compile_tx,
            compile_rx,
            bundle_tx,
            bundle_rx,
            retire_tx,
            retire_rx,
            next_id: AtomicU64::new(1),
            restart_compiler: AtomicBool::new(false),
            #[cfg(test)]
            panic_next_compile: AtomicBool::new(false),
        });
        let _ = hub.compile_tx.try_send(());
        hub
    }

    /// Publish what to render next; the compiler picks it up.
    pub fn publish(&self, input: EngineInput) {
        self.input.store(Arc::new(input));
        let _ = self.compile_tx.try_send(());
    }

    /// A new id for a line driver (its input layers are keyed by it).
    pub fn driver_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// A driver's current input layers (empty = none).
    pub fn set_inputs(&self, id: u64, layers: Arc<Vec<InputLayer>>) {
        self.inputs.rcu(|all| {
            let mut all = InputFrames::clone(all);
            if layers.is_empty() {
                all.remove(&id);
            } else {
                all.insert(id, layers.clone());
            }
            all
        });
    }

    /// What arrives for a local universe from every input line (HTP), as
    /// received — before it merges with our output (`DMX In`).
    pub fn input_universe(&self, universe: u32) -> Option<[u8; 512]> {
        let inputs = self.inputs.load();
        let mut out: Option<[u8; 512]> = None;
        for l in inputs.values().flat_map(|v| v.iter()).filter(|l| l.universe == universe) {
            let o = out.get_or_insert([0; 512]);
            for (o, v) in o.iter_mut().zip(l.data.iter()) {
                *o = (*o).max(*v);
            }
        }
        out
    }

    /// Start a compiler that stopped after repeated panics again (a show
    /// load; R1-66, Q39).
    pub fn restart_compiler(&self) {
        self.restart_compiler.store(true, Ordering::Relaxed);
        let _ = self.compile_tx.try_send(());
    }

    /// Show generation of the newest published frame.
    pub fn rendered_generation(&self) -> u64 {
        self.rendered.load(Ordering::Acquire)
    }

    /// Wait (at most `timeout`) until a frame from `generation` or later is
    /// published. False if it didn't come (no engine, engine down).
    pub fn wait_rendered(&self, generation: u64, timeout: Duration) -> bool {
        let until = Instant::now() + timeout;
        while self.rendered_generation() < generation {
            if Instant::now() >= until {
                return false;
            }
            thread::sleep(Duration::from_millis(1));
        }
        true
    }
}

/// Reports an engine panic (the station turns it into a report #).
pub type PanicHook = Box<dyn Fn(&str) + Send>;

/// The running engine: the render thread and the compiler.
pub struct EngineHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    compiler: CompilerHandle,
}

impl EngineHandle {
    /// Ask both threads to end. `take_threads` gives the threads to join.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
        self.compiler.stop();
    }

    pub fn take_threads(&mut self) -> Vec<JoinHandle<()>> {
        self.thread
            .take()
            .into_iter()
            .chain(self.compiler.take_thread())
            .collect()
    }
}

/// Start the engine (render thread + compiler; `health` and
/// `compiler_health` are their entries on the health board).
pub fn spawn_engine(
    hub: Arc<RenderHub>,
    health: Option<Arc<ThreadHealth>>,
    compiler_health: Option<Arc<ThreadHealth>>,
    on_panic: PanicHook,
) -> EngineHandle {
    let compiler = spawn_compiler(hub.clone(), compiler_health);
    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = stop.clone();
    let thread = thread::Builder::new()
        .name("render_engine".to_string())
        .spawn(move || {
            // Real-time scheduling, reported in Health (decisions §31).
            let granted = crate::realtime::promote_current(crate::realtime::ENGINE_PRIORITY);
            if !granted.starts_with("SCHED_FIFO") {
                log::warn!("render_engine: {granted}");
            }
            if let Some(h) = &health {
                h.set_scheduling(granted);
            }
            supervise(&hub, health.as_deref(), &stop_flag, &on_panic);
            if let Some(h) = &health {
                h.stopped();
            }
            log::info!("render engine: thread exited");
        })
        .expect("should spawn render engine thread");
    EngineHandle {
        stop,
        thread: Some(thread),
        compiler,
    }
}

/// Run the loop; on a panic hold the last frame and restart once.
fn supervise(hub: &RenderHub, health: Option<&ThreadHealth>, stop: &AtomicBool, on_panic: &PanicHook) {
    let mut engine = Engine::default();
    let mut last_panic: Option<Instant> = None;
    loop {
        match catch_unwind(AssertUnwindSafe(|| engine.run(hub, health, stop))) {
            Ok(()) => return,
            Err(payload) => {
                let msg = payload
                    .downcast_ref::<&str>()
                    .map(|s| (*s).to_string())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "panic".to_string());
                hub.stats.panics.fetch_add(1, Ordering::Relaxed);
                if let Some(h) = health {
                    h.panicked();
                }
                on_panic(&msg);
                let again = last_panic.is_some_and(|t| t.elapsed() < PANIC_WINDOW);
                last_panic = Some(Instant::now());
                if again {
                    // Down: the lines keep sending the last frame.
                    hub.stats.down.store(true, Ordering::Relaxed);
                    while !stop.load(Ordering::Relaxed) {
                        thread::sleep(Duration::from_millis(50));
                    }
                    return;
                }
                hub.stats.restarts.fetch_add(1, Ordering::Relaxed);
                // A fresh plan: drop ours and ask the compiler again.
                engine = Engine::default();
                hub.force_compile.store(true, Ordering::Relaxed);
                let _ = hub.compile_tx.try_send(());
            }
        }
    }
}

/// Times the stages of one frame.
struct StageTimer {
    frame_start: Instant,
    current: Option<(usize, Instant)>,
    starts: [u64; STAGE_COUNT],
    durations: [u64; STAGE_COUNT],
    /// The open stage's Tracy zone.
    span: lumacore_trace::HeldZone,
}

impl StageTimer {
    fn new(frame_start: Instant) -> Self {
        Self {
            frame_start,
            current: None,
            starts: [0; STAGE_COUNT],
            durations: [0; STAGE_COUNT],
            span: None,
        }
    }

    fn close(&mut self, now: Instant) {
        if let Some((i, started)) = self.current.take() {
            self.durations[i] += (now - started).as_micros() as u64;
            lumacore_trace::record_complete(lumacore_tree::runtime::STAGES[i], "render", started, now - started);
        }
        self.span = None;
    }

    fn finish(&mut self) {
        self.close(Instant::now());
    }
}

impl Stages for StageTimer {
    fn stage(&mut self, index: usize) {
        let now = Instant::now();
        self.close(now);
        self.starts[index] = (now - self.frame_start).as_micros() as u64;
        self.current = Some((index, now));
        self.span = stage_span(index);
    }
}

/// A Tracy zone for render stage `index` (`lumacore_runtime::render_stats::STAGES`).
fn stage_span(index: usize) -> lumacore_trace::HeldZone {
    match index {
        0 => lumacore_trace::held_zone!("Plan"),
        1 => lumacore_trace::held_zone!("Inputs"),
        2 => lumacore_trace::held_zone!("Channels"),
        3 => lumacore_trace::held_zone!("Park"),
        4 => lumacore_trace::held_zone!("InputMerge"),
        _ => lumacore_trace::held_zone!("Publish"),
    }
}

/// Per-second maxima, published once a second.
#[derive(Default)]
struct Window {
    frames: u64,
    frame_max: u64,
    jitter_max: u64,
    headroom_min: u64,
    stage_max: [u64; STAGE_COUNT],
    late: bool,
}

/// The engine loop's state (reset on restart).
#[derive(Default)]
struct Engine {
    bundle: Option<Bundle>,
    /// Waiting to go back to the compiler (the channel was full).
    retiring: ArrayVec<Retired, 32>,
    frame_no: u64,
    last_start: Option<Instant>,
    window: Window,
    /// Frames since the last late one (overload ends after a second).
    calm_frames: u64,
}

impl Engine {
    fn run(&mut self, hub: &RenderHub, health: Option<&ThreadHealth>, stop: &AtomicBool) {
        let stats = &*hub.stats;
        // Warm up outside the allocation check: ArcSwap registers this
        // thread's debt slot, the clock its origin, on first use.
        drop((hub.inputs.load(), hub.frame.load(), hub.plan.load()));
        drop(hub.inputs.load_full());
        let _ = lumacore_base::clock::session_ms();
        let mut deadline = Instant::now() + FRAME_PERIOD;
        self.window.headroom_min = u64::MAX;
        while !stop.load(Ordering::Relaxed) {
            let start = Instant::now();
            let mut timer = StageTimer::new(start);
            #[cfg(debug_assertions)]
            let violations = assert_no_alloc::violation_count();
            assert_no_alloc::assert_no_alloc(|| self.tick(hub, &mut timer));
            timer.finish();
            #[cfg(debug_assertions)]
            if assert_no_alloc::violation_count() > violations {
                panic!("the render tick allocated (decisions §24: no allocation in the tick)");
            }
            let work = start.elapsed();
            self.record(stats, health, &timer, start, work, deadline);

            lumacore_trace::tracy_plot!("frame_us", work.as_micros());
            lumacore_trace::tracy_frame_mark!();

            let now = Instant::now();
            if now < deadline {
                spin_sleep::sleep(deadline - now);
                deadline += FRAME_PERIOD;
            } else {
                // Late: never catch up; the next frame renders at the real
                // time.
                let behind = now - deadline;
                stats.late.fetch_add(1, Ordering::Relaxed);
                stats.skipped.fetch_add(
                    (behind.as_nanos() / FRAME_PERIOD.as_nanos()) as u64,
                    Ordering::Relaxed,
                );
                self.window.late = true;
                self.calm_frames = 0;
                if !stats.overload.swap(true, Ordering::Relaxed) {
                    stats.overload_episodes.fetch_add(1, Ordering::Relaxed);
                }
                lumacore_trace::tracy_message!("late_frame");
                deadline = now + FRAME_PERIOD;
            }
        }
    }

// tidy: render-begin (decisions §24: the per-frame path)
    /// One frame: take the newest plan, render, publish. Allocates nothing
    /// and frees nothing.
    fn tick(&mut self, hub: &RenderHub, timer: &mut StageTimer) {
        timer.stage(stage::PLAN);
        while let Ok(newer) = hub.bundle_rx.try_recv() {
            if let Some(old) = self.bundle.replace(newer) {
                self.retire(Retired::Bundle(old));
            }
        }
        let Some(bundle) = self.bundle.as_mut() else {
            return;
        };

        timer.stage(stage::INPUTS);
        let inputs = hub.inputs.load_full();
        let now_ms = lumacore_base::clock::session_ms();
        self.frame_no += 1;
        let frame_no = self.frame_no;

        // A buffer nobody else holds; none free → this frame isn't published.
        let rendered = match bundle.pool.iter_mut().find_map(Arc::get_mut) {
            Some(buf) => {
                buf.frame_no = frame_no;
                let layers = inputs.values().flat_map(|v| v.iter());
                render_into(buf, &bundle.plan, now_ms, layers, timer);
                true
            }
            None => {
                hub.stats.busy.fetch_add(1, Ordering::Relaxed);
                false
            }
        };

        timer.stage(stage::PUBLISH);
        let mut swapped: ArrayVec<Retired, 2> = ArrayVec::new();
        if rendered && let Some(published) = bundle.pool.iter().find(|b| b.frame_no == frame_no) {
            swapped.push(Retired::Frame(hub.frame.swap(published.clone())));
            if !Arc::ptr_eq(&hub.plan.load(), &bundle.plan) {
                swapped.push(Retired::Plan(hub.plan.swap(bundle.plan.clone())));
            }
            hub.rendered.store(bundle.plan.generation, Ordering::Release);
        }
        for r in swapped {
            self.retire(r);
        }
        self.retire(Retired::Inputs(inputs));
        self.flush(hub);
    }

    /// Queue something to be freed by the compiler.
    fn retire(&mut self, r: Retired) {
        // A full queue (the compiler stalled) drops here, rarely.
        let _ = self.retiring.try_push(r);
    }

    /// Hand queued things back to the compiler.
    fn flush(&mut self, hub: &RenderHub) {
        while let Some(r) = self.retiring.pop() {
            if let Err(e) = hub.retire_tx.try_send(r) {
                let _ = self.retiring.try_push(e.into_inner());
                break;
            }
        }
    }

// tidy: render-end
    /// Store this frame's numbers (atomics only).
    fn record(
        &mut self,
        stats: &RenderStats,
        health: Option<&ThreadHealth>,
        timer: &StageTimer,
        start: Instant,
        work: Duration,
        deadline: Instant,
    ) {
        let work_us = work.as_micros() as u64;
        let headroom = deadline
            .checked_duration_since(start + work)
            .map_or(0, |d| d.as_micros() as u64);
        let jitter = self.last_start.map_or(0, |last| {
            let interval = (start - last).as_micros() as i64;
            (interval - FRAME_PERIOD.as_micros() as i64).unsigned_abs()
        });
        self.last_start = Some(start);

        for (i, s) in stats.stages.iter().enumerate() {
            s.start_us.store(timer.starts[i], Ordering::Relaxed);
            s.last_us.store(timer.durations[i], Ordering::Relaxed);
            self.window.stage_max[i] = self.window.stage_max[i].max(timer.durations[i]);
        }
        let seq = stats.frames.fetch_add(1, Ordering::Relaxed) + 1;
        // Every frame into the ring (the Realtime window's graphs): atomics only.
        stats.ring.push(&lumacore_tree::runtime::FrameSample {
            seq,
            t_ms: lumacore_base::clock::session_ms(),
            frame_us: work_us,
            headroom_us: headroom,
            jitter_us: jitter,
            late: start + work > deadline,
            stages: timer.durations,
        });
        stats.frame_us.store(work_us, Ordering::Relaxed);
        stats.headroom_us.store(headroom, Ordering::Relaxed);
        if let Some(b) = &self.bundle {
            stats
                .plan_generation
                .store(b.plan.generation, Ordering::Relaxed);
            stats
                .channels
                .store(b.plan.channel.channels.len() as u64, Ordering::Relaxed);
            stats
                .universes
                .store(b.plan.channel.universes.len() as u64, Ordering::Relaxed);
        }
        if let Some(h) = health {
            h.tick(work_us);
        }

        let w = &mut self.window;
        w.frames += 1;
        w.frame_max = w.frame_max.max(work_us);
        w.jitter_max = w.jitter_max.max(jitter);
        w.headroom_min = w.headroom_min.min(headroom);
        self.calm_frames += 1;
        if w.frames >= RENDER_HZ {
            stats.frame_max_us.store(w.frame_max, Ordering::Relaxed);
            stats.jitter_us.store(w.jitter_max, Ordering::Relaxed);
            stats.headroom_min_us.store(w.headroom_min, Ordering::Relaxed);
            for (s, max) in stats.stages.iter().zip(w.stage_max) {
                s.max_us.store(max, Ordering::Relaxed);
            }
            if self.calm_frames >= RENDER_HZ {
                stats.overload.store(false, Ordering::Relaxed);
            }
            *w = Window {
                headroom_min: u64::MAX,
                ..Window::default()
            };
        }
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
