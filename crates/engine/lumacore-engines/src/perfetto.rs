// ██████╗ ███████╗██████╗ ███████╗███████╗████████╗████████╗ ██████╗
// ██╔══██╗██╔════╝██╔══██╗██╔════╝██╔════╝╚══██╔══╝╚══██╔══╝██╔═══██╗
// ██████╔╝█████╗  ██████╔╝█████╗  █████╗     ██║      ██║   ██║   ██║
// ██╔═══╝ ██╔══╝  ██╔══██╗██╔══╝  ██╔══╝     ██║      ██║   ██║   ██║
// ██║     ███████╗██║  ██║██║     ███████╗   ██║      ██║   ╚██████╔╝
// ╚═╝     ╚══════╝╚═╝  ╚═╝╚═╝     ╚══════╝   ╚═╝      ╚═╝    ╚═════╝
// FILE: perfetto

//! PerfettoEngine: serves Perfetto trace endpoints on a dedicated HTTP port.
//!
//! Runs its own single-threaded tokio runtime on a dedicated OS thread so the
//! trace endpoint is decoupled from the main HTTP engine.
//!
//! Owns:          PerfettoEngine, the trace endpoints on their own port and tokio runtime.
//! Doesn't own:   the trace collector (lumacore-trace), the endpoints' handlers (lumacore-http).
//! Entry points:  PerfettoEngine, DEFAULT_PERFETTO_PORT.
//! Decisions:     §26, §34 20.9.

use std::thread;

use lumacore_wire::{Capability, EngineInfo, EngineStatus};

use crate::registry::{Engine, EngineContext};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Default port for the Perfetto trace endpoint.
pub const DEFAULT_PERFETTO_PORT: u16 = 47657;

pub struct PerfettoEngine {
    port: u16,
    status: EngineStatus,
    _thread: Option<thread::JoinHandle<()>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl PerfettoEngine {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            status: EngineStatus::Stopped,
            _thread: None,
            shutdown_tx: None,
        }
    }
}

impl Engine for PerfettoEngine {
    fn name(&self) -> &str {
        "perfetto"
    }

    fn requires(&self) -> &[Capability] {
        &[Capability::Http]
    }

    fn status(&self) -> EngineStatus {
        self.status.clone()
    }

    fn info(&self) -> EngineInfo {
        EngineInfo {
            name: "perfetto".into(),
            status: self.status(),
            requires: self.requires().to_vec(),
            port: if self.is_running() {
                Some(self.port)
            } else {
                None
            },
        }
    }

    fn start(&mut self, _ctx: &EngineContext) -> Result<(), String> {
        if self.is_running() {
            return Ok(());
        }

        // Initialise the global trace collector (idempotent)
        lumacore_trace::TRACE_COLLECTOR.init();

        let port = self.port;
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        let handle = thread::Builder::new()
            .name("perfetto-engine".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to build tokio runtime for perfetto engine");

                rt.block_on(async move {
                    use axum::Router;
                    use tower_http::cors::{Any, CorsLayer};

                    let cors = CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any);

                    let app = Router::new()
                        .route(
                            "/api/trace",
                            axum::routing::get(lumacore_http::endpoints::trace::get_trace),
                        )
                        .route(
                            "/api/trace/download",
                            axum::routing::get(lumacore_http::endpoints::trace::download_trace),
                        )
                        .layer(cors);

                    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
                    let listener = match tokio::net::TcpListener::bind(addr).await {
                        Ok(l) => l,
                        Err(e) => {
                            log::error!("perfetto engine: failed to bind {addr}: {e}");
                            return;
                        }
                    };

                    log::info!("perfetto engine listening on {addr}");

                    axum::serve(listener, app)
                        .with_graceful_shutdown(async {
                            let _ = shutdown_rx.await;
                        })
                        .await
                        .unwrap_or_else(|e| {
                            log::error!("perfetto engine serve error: {e}");
                        });
                });
            })
            .map_err(|e| format!("failed to spawn perfetto thread: {e}"))?;

        self._thread = Some(handle);
        self.shutdown_tx = Some(shutdown_tx);
        self.status = EngineStatus::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        // We don't join — the thread will exit after the runtime shuts down.
        self._thread = None;
        self.status = EngineStatus::Stopped;
        Ok(())
    }
}
