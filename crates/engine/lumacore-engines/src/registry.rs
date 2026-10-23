// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// FILE: registry

//! Runtime engine system for LumaCore.
//!
//! Engines are optional, capability-gated subsystems that can be started and
//! stopped at runtime.  System services (shell, discovery) are NOT engines —
//! they always run.
//!
//! Each engine declares the platform capabilities it requires.  The registry
//! checks availability before starting an engine and provides an `EngineContext`
//! with shared resources (the render hub, health, …).
//!
//! Owns:          the Engine trait, EngineContext, EngineRegistry (capability-gated start, stop, join against one deadline), PublishedShow, the HTTP engine's tree entry.
//! Doesn't own:   the engines themselves (render, http, perfetto), what the server registers (lumacore-server).
//! Entry points:  Engine, EngineContext, EngineRegistry, PublishedShow, http_ui_engine_state.
//! Decisions:     §26, §34 19.1, 20.4, 24.2.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use crossbeam_channel::Sender;
use lumacore_base::InboundEvent;
use lumacore_render::RenderHub;
use lumacore_tree::tree::{ShowData, UIEngineState};
use arc_swap::ArcSwap;

use lumacore_wire::{Capability, EngineInfo, EngineStatus};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The show the state thread publishes after every change (decisions §26): engines and the
/// HTTP feeds read it, never the state (§34 19.1: the alias lives here).
pub type PublishedShow = Arc<ArcSwap<ShowData>>;

/// Resources provided to engines at start time.
///
/// All fields are `Option` so existing engines continue to work unchanged.
/// New engines pull the resources they need and fail cleanly if missing.
pub struct EngineContext {
    /// Show data snapshot for engines that need fixture/universe info.
    pub show_data: Option<Arc<ShowData>>,
    /// The render engine's hub: output engines attach their drivers here.
    pub render: Option<Arc<RenderHub>>,
    /// Channel for input engines to push events into the command path.
    pub command_tx: Option<Sender<InboundEvent>>,
    /// Where engine threads report (`Runtime.Health.Threads`).
    pub health: Option<Arc<lumacore_tree::runtime::HealthBoard>>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl EngineContext {
    /// Register an engine thread for health reporting (None without a board).
    pub fn thread_health(&self, name: &str) -> Option<Arc<lumacore_tree::runtime::ThreadHealth>> {
        self.health.as_ref().map(|board| board.register(name))
    }
}

/// A runtime engine that can be started and stopped.
pub trait Engine: Send + 'static {
    /// Human-readable name (lowercase, e.g. `"perfetto"`, `"sacn"`).
    fn name(&self) -> &str;

    /// Capabilities this engine requires to run.
    fn requires(&self) -> &[Capability];

    /// Current status.
    fn status(&self) -> EngineStatus;

    /// Snapshot for discovery advertisement.
    fn info(&self) -> EngineInfo;

    /// Start the engine.  Called only when all required capabilities are available.
    fn start(&mut self, ctx: &EngineContext) -> Result<(), String>;

    /// Stop the engine and release resources. Signals its threads; `join`
    /// waits for them.
    fn stop(&mut self) -> Result<(), String>;

    /// Wait (at most until `deadline`) for the threads `stop` signalled to
    /// end. Err names what is still running.
    fn join(&mut self, _deadline: Instant) -> Result<(), String> {
        Ok(())
    }

    /// Convenience: is the engine currently running?
    fn is_running(&self) -> bool {
        matches!(self.status(), EngineStatus::Running)
    }
}

/// Registry that owns engines, checks capabilities, and manages lifecycle.
pub struct EngineRegistry {
    engines: Vec<Box<dyn Engine>>,
    available: HashSet<Capability>,
    context: EngineContext,
}

impl EngineRegistry {
    pub fn new(caps: HashSet<Capability>, ctx: EngineContext) -> Self {
        Self {
            engines: Vec::new(),
            available: caps,
            context: ctx,
        }
    }

    /// Register an engine.  Does not start it.
    pub fn register(&mut self, engine: Box<dyn Engine>) {
        self.engines.push(engine);
    }

    /// Can the named engine be started on this platform?
    pub fn can_start(&self, name: &str) -> bool {
        self.engines
            .iter()
            .find(|m| m.name() == name)
            .map(|m| m.requires().iter().all(|c| self.available.contains(c)))
            .unwrap_or(false)
    }

    /// Start a single engine by name.
    pub fn start(&mut self, name: &str) -> Result<(), String> {
        let idx = self
            .engines
            .iter()
            .position(|m| m.name() == name)
            .ok_or_else(|| format!("unknown engine '{name}'"))?;

        let engine = &self.engines[idx];
        let missing: Vec<Capability> = engine
            .requires()
            .iter()
            .filter(|c| !self.available.contains(c))
            .copied()
            .collect();
        if !missing.is_empty() {
            return Err(format!("missing capabilities: {:?}", missing));
        }

        self.engines[idx].start(&self.context)
    }

    /// Stop a single engine by name.
    pub fn stop(&mut self, name: &str) -> Result<(), String> {
        let idx = self
            .engines
            .iter()
            .position(|m| m.name() == name)
            .ok_or_else(|| format!("unknown engine '{name}'"))?;
        self.engines[idx].stop()
    }

    /// Start all engines whose capabilities are satisfied.
    /// Returns `(name, result)` for every registered engine.
    pub fn start_all(&mut self) -> Vec<(String, Result<(), String>)> {
        let mut results = Vec::new();
        for i in 0..self.engines.len() {
            let name = self.engines[i].name().to_string();
            let missing: Vec<Capability> = self.engines[i]
                .requires()
                .iter()
                .filter(|c| !self.available.contains(c))
                .copied()
                .collect();
            let result = if missing.is_empty() {
                self.engines[i].start(&self.context)
            } else {
                Err(format!("missing capabilities: {:?}", missing))
            };
            results.push((name, result));
        }
        results
    }

    /// Stop all running engines (in reverse registration order), then wait
    /// for their threads until `deadline`. Returns the engines that did not
    /// finish in time, with what is still running.
    pub fn stop_all(&mut self, deadline: Instant) -> Vec<String> {
        let mut stopped = Vec::new();
        for (i, engine) in self.engines.iter_mut().enumerate().rev() {
            if engine.is_running() {
                if let Err(e) = engine.stop() {
                    log::warn!("engine '{}' stop failed: {e}", engine.name());
                }
                stopped.push(i);
            }
        }
        let mut late = Vec::new();
        for i in stopped {
            let engine = &mut self.engines[i];
            if let Err(e) = engine.join(deadline) {
                late.push(format!("{} ({e})", engine.name()));
            }
        }
        late
    }

    /// Collect info from all engines (for discovery beacons).
    pub fn engine_info(&self) -> Vec<EngineInfo> {
        self.engines.iter().map(|m| m.info()).collect()
    }
}

/// Tree entry for the HTTP/WebSocket UI engine, published at `Root.UIEngines`.
pub fn http_ui_engine_state(
    addr: std::net::SocketAddr,
    web_root: Option<&std::path::Path>,
) -> UIEngineState {
    UIEngineState {
        name: "HTTP".to_string(),
        kind: "http".to_string(),
        status: "Running".to_string(),
        address: addr.to_string(),
        port: Some(addr.port()),
        web_root: Some(web_root.map_or_else(|| "built-in".to_string(), |p| p.display().to_string())),
        ..Default::default()
    }
}
