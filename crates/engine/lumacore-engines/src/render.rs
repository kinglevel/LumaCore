// ██████╗ ███████╗███╗   ██╗██████╗ ███████╗██████╗
// ██╔══██╗██╔════╝████╗  ██║██╔══██╗██╔════╝██╔══██╗
// ██████╔╝█████╗  ██╔██╗ ██║██║  ██║█████╗  ██████╔╝
// ██╔══██╗██╔══╝  ██║╚██╗██║██║  ██║██╔══╝  ██╔══██╗
// ██║  ██║███████╗██║ ╚████║██████╔╝███████╗██║  ██║
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═╝
// FILE: render

//! RenderEngine: the one renderer (decisions §24 "Step 9 hardening",
//! §27). Runs the render thread (60 Hz) and the plan compiler; the line
//! drivers sample its frames.
//!
//! Owns:          RenderEngine, the registry wrapper of the render thread and the plan compiler.
//! Doesn't own:   the render thread itself (lumacore-render), the line drivers that sample its frames (lumacore-lines).
//! Entry points:  RenderEngine.
//! Decisions:     §24 "Step 9 hardening", §27, §34 20.4.

use std::sync::Arc;

use lumacore_render::{EngineHandle, RenderHub, spawn_engine};

use lumacore_wire::{Capability, EngineInfo, EngineStatus};

use crate::registry::{Engine, EngineContext};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

#[derive(Default)]
pub struct RenderEngine {
    status: Option<EngineStatus>,
    handle: Option<EngineHandle>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl RenderEngine {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Engine for RenderEngine {
    fn name(&self) -> &str {
        "render"
    }

    fn requires(&self) -> &[Capability] {
        &[Capability::DmxFramebuffer, Capability::Clock]
    }

    fn status(&self) -> EngineStatus {
        self.status.clone().unwrap_or(EngineStatus::Stopped)
    }

    fn info(&self) -> EngineInfo {
        EngineInfo {
            name: "render".into(),
            status: self.status(),
            requires: self.requires().to_vec(),
            port: None,
        }
    }

    fn start(&mut self, ctx: &EngineContext) -> Result<(), String> {
        if self.is_running() {
            return Ok(());
        }
        let hub: Arc<RenderHub> = ctx
            .render
            .clone()
            .ok_or("EngineContext missing render hub")?;
        // The panic itself is logged by the station's panic hook (report #,
        // file:line, backtrace); the state thread tells the System Monitor
        // what the engine did about it.
        let on_panic = Box::new(|msg: &str| {
            log::error!("render engine panicked: {msg}");
        });
        self.handle = Some(spawn_engine(
            hub,
            ctx.thread_health("Render.Tick"),
            ctx.thread_health("Render.Compiler"),
            on_panic,
        ));
        self.status = Some(EngineStatus::Running);
        log::info!("render engine: started ({} Hz)", lumacore_render::RENDER_HZ);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        if let Some(handle) = &self.handle {
            handle.stop();
        }
        self.status = Some(EngineStatus::Stopped);
        Ok(())
    }

    fn join(&mut self, deadline: std::time::Instant) -> Result<(), String> {
        let threads = self
            .handle
            .as_mut()
            .map(|h| h.take_threads())
            .unwrap_or_default();
        for thread in threads {
            lumacore_base::threads::join_until(thread, deadline)?;
        }
        Ok(())
    }
}
