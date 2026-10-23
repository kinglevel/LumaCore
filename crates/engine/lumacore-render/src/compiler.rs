//  ██████╗ ██████╗ ███╗   ███╗██████╗ ██╗██╗     ███████╗██████╗
// ██╔════╝██╔═══██╗████╗ ████║██╔══██╗██║██║     ██╔════╝██╔══██╗
// ██║     ██║   ██║██╔████╔██║██████╔╝██║██║     █████╗  ██████╔╝
// ██║     ██║   ██║██║╚██╔╝██║██╔═══╝ ██║██║     ██╔══╝  ██╔══██╗
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║     ██║███████╗███████╗██║  ██║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝
// FILE: compiler

//! The plan compiler thread (decisions §24 "Step 9 hardening").
//!
//! The state thread publishes an `EngineInput` into the hub and pokes this
//! thread; it compiles the newest input into a `Plan` (the channel tier is
//! reused while the patch is unchanged), builds a pool of frame buffers
//! shaped for it and hands both to the engine as a `Bundle`. Bursts
//! coalesce: only the newest input is compiled. The engine never frees a
//! plan or a buffer: it sends retired bundles back here, and they are
//! dropped on this thread.
//!
//! A compile that panics (R1-66, Q39) is caught: the engine keeps rendering
//! the last plan, `RenderStats::compile_panics` counts it, `stale_since`
//! names the first generation the stage doesn't show yet (latched until a
//! compile works; the state thread tells the System Monitor once per
//! episode) and that input generation is not tried again — the next
//! published one is. Three panics within 60 s stop the compiler
//! (`compiler_down`): the last plan stays on stage — never a blackout —
//! until a show load (`RenderHub::restart_compiler`) or a restart. The
//! thread is `Render.Compiler` on the health board.
//!
//! Owns:          the plan compiler thread: CompilerHandle, spawn_compiler, bundle, bundle_cached.
//! Doesn't own:   the plan (plan), the tick (render), publishing the input (engine::RenderHub).
//! Entry points:  spawn_compiler, CompilerHandle, bundle, bundle_cached.
//! Decisions:     §24 "Step 9 hardening", Q39.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use lumacore_tree::runtime::ThreadHealth;
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{RecvTimeoutError, SendTimeoutError};

use crate::engine::{Bundle, POOL, RenderHub};
use crate::plan::Plan;
use crate::render::RenderedFrame;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The running compiler.
pub struct CompilerHandle {
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

impl CompilerHandle {
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn take_thread(&mut self) -> Option<JoinHandle<()>> {
        self.thread.take()
    }
}

/// Start the compiler thread (`health`: its entry on the health board,
/// `Render.Compiler`).
pub fn spawn_compiler(hub: Arc<RenderHub>, health: Option<Arc<ThreadHealth>>) -> CompilerHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let thread = std::thread::Builder::new()
        .name("plan_compiler".to_string())
        .spawn(move || {
            run(&hub, &flag, health.as_deref());
            if let Some(h) = &health {
                h.stopped();
            }
        })
        .expect("should spawn plan compiler thread");
    CompilerHandle {
        stop,
        thread: Some(thread),
    }
}

/// Compile one bundle for `input`, after `previous` (the plan rendering
/// now: cue fades start from where its channels are).
pub fn bundle(input: &crate::merge::EngineInput, previous: Option<&Arc<Plan>>) -> Bundle {
    bundle_cached(input, previous, &mut crate::merge::SequenceCache::default())
}

/// [`bundle`] with the compiler thread's sequence cache (decisions §31).
pub fn bundle_cached(
    input: &crate::merge::EngineInput,
    previous: Option<&Arc<Plan>>,
    cache: &mut crate::merge::SequenceCache<u32>,
) -> Bundle {
    let now = lumacore_base::clock::session_ms();
    let plan = Arc::new(Plan::compile_cached(
        input,
        previous.map(|p| &p.channel),
        previous.map(|p| p.as_ref()),
        now,
        cache,
    ));
    let pool = (0..POOL)
        .map(|_| Arc::new(RenderedFrame::for_tier(&plan.channel)))
        .collect();
    Bundle { plan, pool }
}

/// Panics within this long, this many times, stop the compiler (Q39).
const PANIC_WINDOW: Duration = Duration::from_secs(60);
const PANICS_TO_STOP: usize = 3;

fn run(hub: &RenderHub, stop: &AtomicBool, health: Option<&ThreadHealth>) {
    let mut compiled: Option<u64> = None;
    let mut last: Option<Arc<Plan>> = None;
    // What each running sequence offers, kept between compiles (decisions §31).
    let mut sequences = crate::merge::SequenceCache::default();
    let mut panics: std::collections::VecDeque<Instant> = std::collections::VecDeque::new();
    while !stop.load(Ordering::Relaxed) {
        // Retired bundles are freed here, never on the engine thread.
        while hub.retire_rx.try_recv().is_ok() {}
        match hub.compile_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(()) | Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        let stats = &hub.stats;
        if hub.restart_compiler.swap(false, Ordering::Relaxed) {
            panics.clear();
            stats.compiler_down.store(false, Ordering::Relaxed);
            compiled = None;
        }
        if stats.compiler_down.load(Ordering::Relaxed) {
            if let Some(h) = health {
                h.heartbeat();
            }
            continue;
        }
        let forced = hub.force_compile.swap(false, Ordering::Relaxed);
        let input = hub.input.load_full();
        if compiled == Some(input.generation) && !forced {
            if let Some(h) = health {
                h.heartbeat();
            }
            continue;
        }
        lumacore_trace::trace_zone!("plan_compile");
        let started = Instant::now();
        let compiled_now = catch_unwind(AssertUnwindSafe(|| {
            #[cfg(test)]
            if hub.panic_next_compile.swap(false, Ordering::Relaxed) {
                panic!("test: the plan compiler panics once");
            }
            bundle_cached(&input, last.as_ref(), &mut sequences)
        }));
        // This generation is done either way: a panic isn't retried in a
        // loop; the engine keeps the last plan until the next input.
        compiled = Some(input.generation);
        let b = match compiled_now {
            Ok(b) => b,
            Err(_) => {
                // The station's panic hook logs it (report #, backtrace).
                stats.compile_panics.fetch_add(1, Ordering::Relaxed);
                if stats.stale_since.load(Ordering::Relaxed) == 0 {
                    stats.stale_since.store(input.generation.max(1), Ordering::Relaxed);
                }
                let now = Instant::now();
                panics.push_back(now);
                while panics.front().is_some_and(|t| now.duration_since(*t) > PANIC_WINDOW) {
                    panics.pop_front();
                }
                if panics.len() >= PANICS_TO_STOP {
                    stats.compiler_down.store(true, Ordering::Relaxed);
                }
                if let Some(h) = health {
                    h.panicked();
                }
                continue;
            }
        };
        if let Some(h) = health {
            h.tick(started.elapsed().as_micros() as u64);
        }
        stats
            .compile_channel_us
            .store(b.plan.compile_us.0, Ordering::Relaxed);
        stats
            .compile_value_us
            .store(b.plan.compile_us.1, Ordering::Relaxed);
        stats.compiles.fetch_add(1, Ordering::Relaxed);
        stats.stale_since.store(0, Ordering::Relaxed);
        last = Some(b.plan.clone());
        // The engine takes bundles every frame; if it isn't running, the
        // newest one waits in the channel and older ones are dropped here.
        let mut pending = b;
        loop {
            match hub.bundle_tx.send_timeout(pending, Duration::from_millis(100)) {
                Ok(()) => break,
                Err(SendTimeoutError::Disconnected(_)) => return,
                Err(SendTimeoutError::Timeout(b)) => {
                    if stop.load(Ordering::Relaxed) {
                        return;
                    }
                    // Make room: the oldest waiting bundle is stale.
                    let _ = hub.bundle_rx.try_recv();
                    pending = b;
                }
            }
        }
    }
}
