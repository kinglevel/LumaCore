// ██╗  ██╗████████╗████████╗██████╗
// ██║  ██║╚══██╔══╝╚══██╔══╝██╔══██╗
// ███████║   ██║      ██║   ██████╔╝
// ██╔══██║   ██║      ██║   ██╔═══╝
// ██║  ██║   ██║      ██║   ██║
// ╚═╝  ╚═╝   ╚═╝      ╚═╝   ╚═╝
// FILE: http

//! HttpEngine as a registry engine: REST, the two WebSockets and the web UI on
//! its own tokio runtime (`lumacore-http`). It reads the show the state thread
//! publishes and runs commands through the console bridge — never the state
//! itself (decisions.md §26).
//!
//! Owns:          HttpUiEngine and HttpConfig, the registry wrapper of the HTTP engine.
//! Doesn't own:   REST, the WebSockets and the feeds (lumacore-http), the console bridge (the server).
//! Entry points:  HttpUiEngine, HttpConfig.
//! Decisions:     §26, §30 "12b2", §34 20.4.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use arc_swap::ArcSwap;
use lumacore_http::state::DiagnosticsSnapshot;
use lumacore_http::{ConsoleBridge, HttpHandle};

use lumacore_wire::{Capability, EngineInfo, EngineStatus};

use crate::registry::{Engine, EngineContext};
use crate::PublishedShow;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Everything the HTTP engine serves from.
pub struct HttpConfig {
    pub addr: SocketAddr,
    pub web_root: Option<PathBuf>,
    pub shell_port: u16,
    pub diagnostics: Arc<ArcSwap<DiagnosticsSnapshot>>,
    pub sysmon_tx: tokio::sync::broadcast::Sender<String>,
    pub show: PublishedShow,
    pub live_screens: lumacore_tree::runtime::SharedScreens,
    pub console: Arc<dyn ConsoleBridge>,
}

pub struct HttpUiEngine {
    config: HttpConfig,
    status: EngineStatus,
    handle: Option<HttpHandle>,
    /// Shown in `Runtime.Health` while the server runs.
    health: Option<Arc<lumacore_tree::runtime::ThreadHealth>>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl HttpUiEngine {
    pub fn new(config: HttpConfig) -> Self {
        Self {
            config,
            status: EngineStatus::Stopped,
            handle: None,
            health: None,
        }
    }
}

impl Engine for HttpUiEngine {
    fn name(&self) -> &str {
        "http"
    }

    fn requires(&self) -> &[Capability] {
        &[Capability::Http, Capability::Network]
    }

    fn status(&self) -> EngineStatus {
        self.status.clone()
    }

    fn info(&self) -> EngineInfo {
        EngineInfo {
            name: "http".into(),
            status: self.status(),
            requires: self.requires().to_vec(),
            port: Some(self.config.addr.port()),
        }
    }

    fn start(&mut self, ctx: &EngineContext) -> Result<(), String> {
        if self.is_running() {
            return Ok(());
        }
        let c = &self.config;
        self.handle = Some(lumacore_http::HttpEngine::start(
            c.addr,
            c.diagnostics.clone(),
            c.sysmon_tx.clone(),
            c.web_root.clone(),
            c.shell_port,
            c.show.clone(),
            c.live_screens.clone(),
            c.console.clone(),
        ));
        self.health = ctx.thread_health("Http");
        self.status = EngineStatus::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        if let Some(handle) = self.handle.as_mut() {
            handle.stop();
        }
        self.status = EngineStatus::Stopped;
        Ok(())
    }

    fn join(&mut self, deadline: Instant) -> Result<(), String> {
        let joined = match self.handle.take().and_then(|mut h| h.take_thread()) {
            Some(thread) => lumacore_base::threads::join_until(thread, deadline),
            None => Ok(()),
        };
        if let Some(health) = self.health.take() {
            health.stopped();
        }
        joined
    }
}
