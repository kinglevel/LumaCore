// ██████╗ ██╗   ██╗██╗██╗     ██████╗ ███████╗██████╗
// ██╔══██╗██║   ██║██║██║     ██╔══██╗██╔════╝██╔══██╗
// ██████╔╝██║   ██║██║██║     ██║  ██║█████╗  ██████╔╝
// ██╔══██╗██║   ██║██║██║     ██║  ██║██╔══╝  ██╔══██╗
// ██████╔╝╚██████╔╝██║███████╗██████╔╝███████╗██║  ██║
// ╚═════╝  ╚═════╝ ╚═╝╚══════╝╚═════╝ ╚══════╝╚═╝  ╚═╝
// FILE: builder

//! Server startup and shutdown — one path for `lumacore-server` and the
//! desktop app's embedded server (`docs/meta/decisions.md` §26).
//!
//! [`ServerBuilder::start`] loads the show, binds the ports, starts the state
//! thread (which owns all server state), the network threads around it and
//! the engines (sACN, Art-Net, OSC, Perfetto, HTTP). [`Server::wait`] blocks
//! until a client asks for Shutdown/Restart and then stops everything in
//! order: inputs → state thread → outputs (they say goodbye on the wire) →
//! HTTP → listeners, joining every thread against one shared deadline.
//!
//! Owns:          ServerConfig, ServerBuilder (load the show, bind the ports, start every thread and engine), Server (wait, the ordered stop against SHUTDOWN_DEADLINE), serve, restart_process.
//! Doesn't own:   the state thread (state), the listeners (net), the engines (lumacore-engines), the catalog (the assembler).
//! Entry points:  ServerConfig, ServerBuilder::start, Server::wait, serve, restart_process.
//! Decisions:     §23 "Step 8", §26, "Shutdown", §34 24.

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use lumacore_engines::{EngineContext, EngineRegistry};
use lumacore_wire::Capability;
use lumacore_sysmon::{SERVER_START, sysmon_early, sysmon_elapsed};
use crate::app;
use crate::context::{ServerContext, sysmon};
use crate::state::{self, Periodic, Published, StateHandle};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// How long shutdown waits for all threads together.
pub const SHUTDOWN_DEADLINE: Duration = Duration::from_secs(5);

/// Pause before stopping anything, so the "shutting down / restarting"
/// message reaches the UIs.
const SHUTDOWN_NOTICE: Duration = Duration::from_millis(400);

/// Configuration for starting the server.
#[derive(Clone)]
pub struct ServerConfig {
    pub cmd_port: u16,
    pub shell_port: u16,
    pub discovery_port: u16,
    pub http_port: u16,
    pub stream_port: u16,
    pub no_http: bool,
    /// `--web-root`: the WebUI from disk; `None` = the embedded files (decisions §36 2).
    pub web_root: Option<String>,
    pub show: Option<String>,
    pub log_level: String,
    /// CleanStart at start (see `CommonCli::clean_start`).
    pub clean_start: bool,
    /// Echo every sysmon line to stderr (`--RootMonitor`).
    pub root_monitor: bool,
    pub gdtf_file: Option<String>,
    pub debug: bool,
    pub user: Option<String>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            cmd_port: lumacore_wire::REMOTE_CMD_PORT,
            shell_port: lumacore_wire::REMOTE_SHELL_PORT,
            discovery_port: lumacore_wire::SESSION_DISCOVERY_PORT,
            http_port: lumacore_wire::REMOTE_HTTP_PORT,
            stream_port: lumacore_wire::REMOTE_STREAM_PORT,
            no_http: false,
            web_root: None,
            show: None,
            log_level: "warn".to_string(),
            clean_start: false,
            root_monitor: false,
            gdtf_file: None,
            debug: false,
            user: None,
        }
    }
}

impl From<app::CommonCli> for ServerConfig {
    fn from(mut c: app::CommonCli) -> Self {
        // Several servers on one machine (decisions §21 "6b").
        if let Some(name) = c.station.as_deref() {
            lumacore_luma::station_config::use_station(name);
        }
        #[cfg(debug_assertions)]
        if let Some(seconds) = c.session_timeout {
            lumacore_objects::session_link::set_timeout(seconds);
        }
        if let Some(base) = c.port_base {
            c.cmd_port = base + 3;
            c.shell_port = base + 4;
            c.http_port = base + 6;
            c.stream_port = base + 8;
        }
        Self {
            cmd_port: c.cmd_port,
            shell_port: c.shell_port,
            discovery_port: c.discovery_port,
            http_port: c.http_port,
            stream_port: c.stream_port,
            no_http: c.no_http,
            web_root: c.web_root,
            show: c.show,
            log_level: c.log_level,
            clean_start: c.clean_start,
            root_monitor: false,
            gdtf_file: c.gdtf_file,
            debug: c.debug,
            user: c.user,
        }
    }
}

impl ServerConfig {
    fn common(&self) -> app::CommonCli {
        app::CommonCli {
            gdtf_file: self.gdtf_file.clone(),
            debug: self.debug,
            user: self.user.clone(),
            cmd_port: self.cmd_port,
            shell_port: self.shell_port,
            discovery_port: self.discovery_port,
            http_port: self.http_port,
            stream_port: self.stream_port,
            station: None,
            port_base: None,
            #[cfg(debug_assertions)]
            session_timeout: None,
            no_http: self.no_http,
            web_root: self.web_root.clone(),
            show: self.show.clone(),
            log_level: self.log_level.clone(),
            clean_start: self.clean_start,
        }
    }
}

/// Builds and starts a server.
pub struct ServerBuilder {
    config: ServerConfig,
}

/// A running server.
pub struct Server {
    state: StateHandle,
    state_thread: JoinHandle<ServerContext>,
    io_threads: Vec<(&'static str, JoinHandle<()>)>,
    engines: EngineRegistry,
    stop: Arc<AtomicBool>,
}

/// How the server ended.
pub struct Ended {
    /// `Restart` was asked for: re-execute the program.
    pub restart: bool,
}

impl ServerBuilder {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Load the show, bind the ports and start every thread and engine.
    pub fn start(self) -> Result<Server, String> {
        let config = self.config;
        SERVER_START.get_or_init(Instant::now);
        app::init_server_logging(&config.log_level);
        // System Monitor log: library/logs/sysmon-YYYY-MM-DD.log (7 days).
        lumacore_sysmon::start_log_file(format!("{}/logs", lumacore_luma::station_config::dir()));
        lumacore_sysmon::panic_report::install();
        crate::require_catalog()?;
        startup_banner();

        // ── State ────────────────────────────────────────────────────────
        match &config.show {
            Some(show) => sysmon_early("Show", format!("Loading show file '{show}'")),
            None => sysmon_early("Show", "No show file specified — starting empty"),
        }
        let (mut state, show_path) = app::build_state(&config.common()).map_err(|e| {
            sysmon_early("Show", format!("[ERROR] {e}"));
            e
        })?;
        log_show(&state, config.show.is_some());
        // Marks the station running; after an unclean stop with a newer
        // recovery copy it asks once (decisions §30 "12d part 5").
        lumacore_objects::recovery::startup(&mut state);

        // ── Sockets ──────────────────────────────────────────────────────
        let socket = lumacore_net::socket::bind_server_socket_at(config.cmd_port)
            .map_err(|e| format!("Failed to bind UDP {}: {e}", config.cmd_port))?;
        sysmon_early(
            "Network",
            format!("UDP command listener bound on 0.0.0.0:{}", config.cmd_port),
        );
        let shell_listener = lumacore_net::socket::bind_shell_listener_at(config.shell_port)
            .map_err(|e| format!("Failed to bind TCP {}: {e}", config.shell_port))?;
        sysmon_early(
            "Network",
            format!("TCP shell listener bound on 0.0.0.0:{}", config.shell_port),
        );
        let stream_listener = lumacore_lumasession::stream::bind_stream_listener_at(config.stream_port)
            .map_err(|e| format!("Failed to bind TCP {}: {e}", config.stream_port))?;
        sysmon_early(
            "Network",
            format!(
                "TCP LumaSession stream listener bound on 0.0.0.0:{}",
                config.stream_port
            ),
        );

        lumacore_sysmon::set_echo(config.root_monitor);
        let mut ctx = ServerContext::new(
            state,
            show_path,
            config.root_monitor,
            config.cmd_port,
            config.shell_port,
            config.discovery_port,
        );
        ctx.stream_port = config.stream_port;
        ctx.state.lumasession_ports.2 = config.stream_port;
        ctx.state.switch_to_internal_root();
        for nic in &ctx.state.root.station_settings.nic_addresses {
            sysmon_early("Server", format!("NIC: {nic}"));
        }

        let health = ctx.state.root.runtime.health.clone();
        let published = Published::new(
            ctx.state.root.show_data.clone(),
            ctx.state.root.runtime.health.render.clone(),
        );
        // Which interfaces serve LumaSession / Shell / HTTP (decisions §11):
        // computed from the station settings now, kept by the state thread.
        ctx.services.store(std::sync::Arc::new(
            lumacore_lumasession::ServiceAccess::compute(
                &ctx.state.root.station_settings,
                &lumacore_net::nics::list(),
            ),
        ));
        let mut published = published;
        published.services = ctx.services.clone();
        let services = ctx.services.clone();
        let (inbound_tx, inbound_rx) =
            crossbeam_channel::bounded::<lumacore_base::InboundEvent>(256);

        // ── HTTP (published into the tree before the state thread owns it) ─
        #[cfg(feature = "http")]
        let http = if config.no_http {
            sysmon_early("HTTP", "HTTP engine disabled (--no-http)");
            None
        } else {
            Some(http_setup(&config, &mut ctx))
        };
        #[cfg(not(feature = "http"))]
        let _ = &config.no_http;

        // ── State thread ─────────────────────────────────────────────────
        let periodic = periodic_tasks(
            #[cfg(feature = "http")]
            http.as_ref().map(|h| h.diagnostics.clone()),
        );
        let (state_handle, state_thread) =
            state::spawn(ctx, inbound_rx, periodic, published.clone());

        // ── Network threads ──────────────────────────────────────────────
        let stop = Arc::new(AtomicBool::new(false));
        let mut io_threads = vec![
            (
                "udp",
                crate::net::spawn::spawn_udp_server(
                    socket,
                    state_handle.clone(),
                    health.clone(),
                    stop.clone(),
                    services.clone(),
                ),
            ),
            (
                "shell-listener",
                crate::net::spawn::spawn_shell_server(
                    shell_listener,
                    state_handle.clone(),
                    health.clone(),
                    stop.clone(),
                    services.clone(),
                ),
            ),
            (
                "stream-listener",
                lumacore_lumasession::stream::spawn_stream_server(
                    stream_listener,
                    state_handle.clone(),
                    health.clone(),
                    stop.clone(),
                    services.clone(),
                ),
            ),
        ];
        // The drives watcher (decisions §38): the OS's mounted volumes, every 2 s.
        io_threads.push((
            "drives",
            crate::net::spawn::spawn_drives_watcher(state_handle.clone(), stop.clone()),
        ));
        // The session clock follows the master's command port; linuxptp is
        // read when it runs (decisions §24 "Session clock").
        io_threads.push((
            "clock-sync",
            lumacore_clocksync::spawn_clock_sync(stop.clone()),
        ));
        io_threads.push((
            "ptp-reader",
            lumacore_net::ptp::spawn_ptp_reader(health.clone(), stop.clone()),
        ));
        for thread in crate::net::spawn::spawn_session_discovery(
            config.discovery_port,
            state_handle.clone(),
            health.clone(),
            stop.clone(),
            services.clone(),
        ) {
            io_threads.push(("discovery", thread));
        }

        // ── Engines ──────────────────────────────────────────────────────
        let caps: HashSet<Capability> = [
            Capability::Network,
            Capability::Clock,
            Capability::ShowData,
            Capability::DmxFramebuffer,
            Capability::CommandDispatch,
            #[cfg(feature = "http")]
            Capability::Http,
        ]
        .into_iter()
        .collect();
        let engine_ctx = EngineContext {
            show_data: None,
            render: Some(published.render.clone()),
            command_tx: Some(inbound_tx.clone()),
            health: Some(health.clone()),
        };
        // The render engine first: it stops last, after the outputs said
        // goodbye (stop_all goes in reverse order).
        let mut engines = EngineRegistry::new(caps, engine_ctx);
        engines.register(Box::new(lumacore_engines::render::RenderEngine::new()));
        #[cfg(feature = "perfetto")]
        engines.register(Box::new(lumacore_engines::perfetto::PerfettoEngine::new(
            lumacore_wire::REMOTE_PERFETTO_PORT,
        )));
        #[cfg(feature = "http")]
        if let Some(http) = http {
            let console = Arc::new(crate::net::console_bridge::ServerConsoleBridge::new(
                state_handle.clone(),
                published.clone(),
                health.clone(),
            ));
            engines.register(Box::new(lumacore_engines::http::HttpUiEngine::new(
                lumacore_engines::http::HttpConfig {
                    addr: http.addr,
                    web_root: http.web_root,
                    shell_port: config.shell_port,
                    diagnostics: http.diagnostics,
                    sysmon_tx: http.sysmon_tx,
                    show: published.show.clone(),
                    live_screens: http.live_screens,
                    console,
                },
            )));
        }
        for (name, result) in engines.start_all() {
            match result {
                Ok(()) => sysmon_early("Engine", format!("'{name}' started")),
                Err(e) => sysmon_early("Engine", format!("'{name}' skipped: {e}")),
            }
        }
        let info = engines.engine_info();
        // Protocol runtimes start from show content (decisions §11): the
        // state thread reconciles them after every show change.
        let hub = published.render.clone();
        let protocol_health = health.clone();
        let protocol_inbound = inbound_tx;
        let link_handle = state_handle.clone();
        state_handle.send(move |ctx| {
            ctx.state_handle = Some(link_handle);
            ctx.engine_info = info;
            ctx.protocols = Some(lumacore_lines::ProtocolRuntimes::new(
                hub.clone(),
                protocol_inbound,
                Some(protocol_health),
            ));
            ctx.state.playback = Some(hub);
        });

        sysmon_early(
            "Server",
            "****************************************************",
        );
        sysmon_early("Server", "*  Server ready");
        sysmon_early(
            "Server",
            "****************************************************",
        );

        Ok(Server {
            state: state_handle,
            state_thread,
            io_threads,
            engines,
            stop,
        })
    }
}

impl Server {
    /// Reach the state thread.
    pub fn state(&self) -> &StateHandle {
        &self.state
    }

    /// Block until a client asks for Shutdown or Restart, then shut down.
    pub fn wait(self) -> Ended {
        let Server {
            state,
            state_thread,
            io_threads,
            mut engines,
            stop,
        } = self;
        drop(state);
        let ctx = state_thread.join();
        let deadline = Instant::now() + SHUTDOWN_NOTICE + SHUTDOWN_DEADLINE;

        let mut ctx = match ctx {
            Ok(ctx) => ctx,
            Err(_) => {
                // The state thread itself died (its jobs are guarded, so this
                // is a bug in the loop). Still stop outputs and threads.
                log::error!("state thread ended with a panic");
                std::thread::sleep(SHUTDOWN_NOTICE);
                stop.store(true, Ordering::Relaxed);
                let late = engines.stop_all(deadline);
                report_late(&late);
                return Ended { restart: false };
            }
        };
        let sessions = ctx.sessions.command_line_count();
        // Leave the servers lists at once (§30 "12b2 hardening"): the mDNS
        // goodbye here; the discovery sender sends its `leaving` beacon as
        // soon as the state thread is gone.
        if let Some(mdns) = ctx.mdns.take() {
            mdns.withdraw();
        }
        std::thread::sleep(SHUTDOWN_NOTICE);

        // Inputs and outputs first (goodbye packets), then engines (the
        // render engine, HTTP), then listeners.
        let mut late = ctx
            .protocols
            .take()
            .map(|mut p| p.stop_all(deadline))
            .unwrap_or_default();
        late.extend(engines.stop_all(deadline));
        if sessions > 0 {
            sysmon(
                &ctx,
                "Shutdown",
                format!("Closing {sessions} shell session(s)"),
            );
        }
        stop.store(true, Ordering::Relaxed);
        for (name, thread) in io_threads {
            if let Err(e) = lumacore_base::threads::join_until(thread, deadline) {
                late.push(format!("{name} ({e})"));
            }
        }
        report_late(&late);
        sysmon(&ctx, "Shutdown", "Listeners closed");
        sysmon(
            &ctx,
            "Shutdown",
            format!("Server shut down after {}", sysmon_elapsed()),
        );
        lumacore_luma::backups::stopped();
        Ended {
            restart: ctx.state.restart_requested,
        }
    }
}

fn report_late(late: &[String]) {
    if !late.is_empty() {
        sysmon_early(
            "Shutdown",
            format!(
                "[WARN] still running at the {}s deadline: {}",
                SHUTDOWN_DEADLINE.as_secs(),
                late.join(", ")
            ),
        );
    }
}

/// Start the server and run it until Shutdown (exit) or Restart (re-exec).
/// The one entry point of `lumacore-server` and `lumacore-app --server`.
pub fn serve(config: ServerConfig) -> ! {
    // Signals are watched from the start: one during startup (a slow show
    // load) ends the process without leaving the `.starting` marker, so the
    // next start doesn't take it for a crash (R1-17).
    #[cfg(unix)]
    let ready = watch_signals();
    let server = match ServerBuilder::new(config).start() {
        Ok(server) => server,
        Err(e) => {
            sysmon_early("Server", format!("[ERROR] {e}"));
            std::process::exit(1);
        }
    };
    #[cfg(unix)]
    if let Some(ready) = ready {
        let _ = ready.set(server.state().clone());
    }
    if server.wait().restart {
        restart_process();
    }
    std::process::exit(0);
}

/// SIGTERM, SIGINT and SIGHUP stop the server like `Shutdown /NoSave`, after
/// an autosave copy of unsaved changes: outputs say goodbye and every thread
/// joins against `SHUTDOWN_DEADLINE`. A second signal exits at once
/// (decisions §23 "App ↔ local server hardening").
///
/// Returned: where `serve` puts the state handle once the server runs.
/// Before that (startup, the show load) a signal removes the `.starting`
/// marker and exits — a stop asked for is no crash.
#[cfg(unix)]
fn watch_signals() -> Option<std::sync::Arc<std::sync::OnceLock<StateHandle>>> {
    use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
    let mut signals = match signal_hook::iterator::Signals::new([SIGTERM, SIGINT, SIGHUP]) {
        Ok(s) => s,
        Err(e) => {
            sysmon_early("Server", format!("[WARN] signals not handled: {e}"));
            return None;
        }
    };
    let ready: std::sync::Arc<std::sync::OnceLock<StateHandle>> = Default::default();
    let state = ready.clone();
    let spawned = std::thread::Builder::new()
        .name("signals".to_string())
        .spawn(move || {
            let mut seen = false;
            for signal in signals.forever() {
                let name = match signal {
                    SIGTERM => "SIGTERM",
                    SIGINT => "SIGINT",
                    _ => "SIGHUP",
                };
                let Some(state) = state.get() else {
                    // tidy: drive-ok (the signal thread, before the server runs)
                    let _ = lumacore_luma::drive::remove_file(format!("{}/.starting", lumacore_luma::station_config::dir()));
                    sysmon_early("Shutdown", format!("[WARN] {name} while starting: exiting"));
                    std::process::exit(128 + signal);
                };
                if seen {
                    sysmon_early("Shutdown", format!("[WARN] {name} again: exiting now"));
                    std::process::exit(130);
                }
                seen = true;
                state.send(move |ctx| shutdown_on_signal(ctx, name));
            }
        });
    if let Err(e) = spawned {
        sysmon_early("Server", format!("[WARN] signals not handled: {e}"));
        return None;
    }
    Some(ready)
}

/// The state-thread side of a stop signal.
#[cfg(unix)]
fn shutdown_on_signal(ctx: &mut ServerContext, signal: &str) {
    let kept = lumacore_objects::session_link::autosave_copy(&ctx.state, "signal");
    lumacore_sysmon::feedback::message(
        lumacore_sysmon::Level::Alert,
        lumacore_sysmon::feedback::category::POWER,
        lumacore_sysmon::feedback::Importance::Critical,
        "Server shutting down",
        &format!("by {signal}"),
        None,
    );
    sysmon(
        ctx,
        "Shutdown",
        match kept {
            Ok(Some(path)) => format!("{signal}: shutting down (unsaved changes kept in {path})"),
            Ok(None) => format!("{signal}: shutting down"),
            // A stop signal still stops (R1-16), but says what was lost.
            Err(e) => format!("[ERROR] {signal}: shutting down; the unsaved changes couldn't be kept ({e})"),
        },
    );
    ctx.state.restart_requested = false;
    ctx.shutdown_requested = true;
}

/// Timed work on the state thread.
fn periodic_tasks(
    #[cfg(feature = "http")] diagnostics: Option<
        Arc<arc_swap::ArcSwap<lumacore_http::state::DiagnosticsSnapshot>>,
    >,
) -> Vec<Periodic> {
    #[allow(unused_mut, reason = "the http feature adds a task")]
    let mut tasks = vec![
        Periodic::new("housekeeping", Duration::from_millis(50), |ctx| {
            lumacore_objects::commands::screens::sync(&mut ctx.state);
            lumacore_objects::commands::screenshot::tick(&mut ctx.state);
            lumacore_cmd_showfile::autosave_tick(&mut ctx.state);
            let holders = ctx.sessions.holders(&ctx.state);
            lumacore_objects::commands::playback::echo_tick(&mut ctx.state, &holders);
            crate::context::dialog_tick(ctx);
            lumacore_objects::show_feedback::publish_statuses(&ctx.state);
            // Background work that finished: its follow-up lines run as
            // their users (R1-70, Q42).
            for done in lumacore_luma::background::take() {
                let (reply, _) = crate::net::handler::execute_remote_command(ctx, &done.user, &done.line);
                sysmon(ctx, "Background", format!("{}: {}", done.user, lumacore_objects::text::strip_ansi(&reply.message)));
            }
            crate::context::drain_sysmon_pending(ctx);
        }),
        // The Devices holder lost from the session (decisions §34 "34g4 patch entry hardened" 5):
        // gone for 5 s — not a page reload — their claim goes and the draft is discarded.
        Periodic::new("patch holder", Duration::from_secs(1), {
            let mut watch = lumacore_objects::claims::HolderWatch::default();
            move |ctx| {
                use lumacore_objects::claims::Claims;
                // A member holds a replica of the claims: its master decides.
                if ctx.state.link.is_some() {
                    watch = Default::default();
                    return;
                }
                let present: Vec<String> = crate::state::presence(ctx).into_keys().collect();
                // An Identify whose user is gone (or that ran 5 minutes) ends here too (§34 round 3).
                lumacore_objects::objects::identify::tick(&mut ctx.state, &present.iter().cloned().collect());
                let holder = ctx.state.root.runtime.claims.values().find(|c| c.label == "Devices").map(|c| c.user.clone());
                if watch.lost(holder.as_deref(), &present, std::time::Instant::now()) {
                    ctx.state.release_lost_devices(&present);
                }
            }
        }),
        Periodic::new("purr", Duration::from_secs(30), |ctx| {
            sysmon(ctx, "Server", "purrrrrrrrrr");
        }),
    ];
    #[cfg(feature = "http")]
    if let Some(diagnostics) = diagnostics {
        tasks.push(Periodic::new(
            "diagnostics",
            Duration::from_secs(1),
            move |ctx| {
                diagnostics.store(Arc::new(diagnostics_snapshot(ctx)));
            },
        ));
    }
    tasks
}

fn startup_banner() {
    let build_info = lumacore_objects::build_info();
    sysmon_early(
        "Server",
        format!(
            "LumaCore server v{}/{}/{}-{}",
            build_info.version, build_info.profile, build_info.target_os, build_info.target_arch,
        ),
    );
    sysmon_early("Server", format!("Build {}", build_info.build_id));
    sysmon_early(
        "Server",
        format!(
            "OS: {} {} ({})",
            std::env::consts::OS,
            os_version(),
            std::env::consts::ARCH,
        ),
    );
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    sysmon_early(
        "Server",
        format!("Host: {hostname} ({})", local_ip_address()),
    );
    sysmon_early("Server", format!("PID {}", std::process::id()));
}

fn log_show(state: &lumacore_objects::AppState, loaded: bool) {
    use lumacore_tree::TreeNode;
    let show = &state.root.show_data;
    if loaded {
        sysmon_early(
            "Show",
            format!(
                "Loaded: {} fixtures, {} universes, {} lines",
                show.devices.dmx.patch.fixtures.len(),
                show.devices.dmx.patch.universes.len(),
                show.protocols.all_lines().count()
            ),
        );
    }
    for (protocol, lines) in &state.protocols_running {
        sysmon_early(protocol, format!("Output started: {} line(s)", (*lines).max(1)));
    }
    if state.protocols_running.is_empty() && show.protocols.all_lines().any(|(_, _, l)| l.sends()) {
        sysmon_early("Output", "No output line sends on this station");
    }
    let names: Vec<String> = show
        .user_profiles
        .values()
        .filter_map(|p| p.name().map(str::to_string))
        .collect();
    if !names.is_empty() {
        sysmon_early(
            "Show",
            format!("User profiles: {} ({})", names.len(), names.join(", ")),
        );
    }
}

#[cfg(feature = "http")]
struct HttpParts {
    addr: std::net::SocketAddr,
    web_root: Option<std::path::PathBuf>,
    diagnostics: Arc<arc_swap::ArcSwap<lumacore_http::state::DiagnosticsSnapshot>>,
    sysmon_tx: tokio::sync::broadcast::Sender<String>,
    live_screens: lumacore_tree::runtime::SharedScreens,
}

/// Everything the HTTP engine needs, and its entry in the tree.
#[cfg(feature = "http")]
fn http_setup(config: &ServerConfig, ctx: &mut ServerContext) -> HttpParts {
    let (sysmon_tx, _) = tokio::sync::broadcast::channel::<String>(512);
    {
        // The sysmon and feedback feeds (/ws/feeds): every line as JSON.
        let tx = sysmon_tx.clone();
        lumacore_sysmon::forward_json(move |json| {
            let _ = tx.send(json);
            true
        });
    }
    lumacore_trace::TRACE_COLLECTOR.init();
    let web_root = config.web_root.as_ref().map(std::path::PathBuf::from).and_then(|dir| {
        if dir.exists() {
            Some(dir)
        } else {
            sysmon_early(
                "HTTP",
                format!("Web root '{}' not found, serving the built-in WebUI", dir.display()),
            );
            None
        }
    });
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.http_port));
    let entry = lumacore_engines::http_ui_engine_state(addr, web_root.as_deref());
    ctx.state.root.ui_engines.publish(entry);
    ctx.http_port = Some(config.http_port);
    ctx.mdns = lumacore_net::mdns::advertise(
        &ctx.state.root.station_settings.station_name,
        &ctx.state.local_settings.station_id,
        config.http_port,
        ctx.shell_port,
    );
    sysmon_early(
        "HTTP",
        format!("HTTP engine listening on 0.0.0.0:{}", config.http_port),
    );
    HttpParts {
        addr,
        web_root,
        diagnostics: Arc::new(arc_swap::ArcSwap::from_pointee(Default::default())),
        sysmon_tx,
        live_screens: ctx.state.root.runtime.live_screens.clone(),
    }
}

/// The diagnostics HTTP serves (`/api/health`, `/api/metrics`, `/api/info`):
/// built once a second on the state thread.
#[cfg(feature = "http")]
fn diagnostics_snapshot(ctx: &ServerContext) -> lumacore_http::state::DiagnosticsSnapshot {
    let build_info = &ctx.state.root.diagnostics.build_info;
    let uptime_secs = SERVER_START
        .get()
        .map(|s| s.elapsed().as_secs())
        .unwrap_or(0);

    lumacore_http::state::DiagnosticsSnapshot {
        threads: ctx.state.root.runtime.health.snapshot(),
        version: build_info.version.clone(),
        profile: build_info.profile.clone(),
        target_os: build_info.target_os.clone(),
        target_arch: build_info.target_arch.clone(),
        git_commit: build_info.git_commit.clone(),
        build_id: build_info.build_id.clone(),
        uptime_secs,
        show_path: ctx.show_path.clone(),
        session_count: ctx.sessions.command_line_count() as u32,
        fixture_count: ctx.state.root.show_data.devices.dmx.patch.fixtures.len(),
        universe_count: ctx.state.root.show_data.devices.dmx.patch.universes.len(),
        output_count: ctx.state.root.show_data.protocols.all_lines().filter(|(_, _, l)| l.sends()).count(),
        protocols_running: ctx.state.protocols_running.clone(),
        sysmon_history: lumacore_sysmon::sysmon_history_snapshot(),
    }
}

/// Get OS version string.
fn os_version() -> String {
    let kernel = || {
        std::process::Command::new("uname")
            .arg("-r")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    };
    if cfg!(target_os = "macos") {
        format!("macOS {}", kernel())
    } else if cfg!(target_os = "linux") {
        format!("Linux {}", kernel())
    } else {
        std::env::consts::OS.to_string()
    }
}

/// Determine local IP address via UDP connect trick.
fn local_ip_address() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            s.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

/// Re-execute the running program with its arguments (Restart). A binary
/// rebuilt in the meantime is picked up; the startup show loads again. Sockets
/// are close-on-exec, so the ports are free for the new process.
pub fn restart_process() -> ! {
    let exe = std::env::current_exe().expect("current executable");
    // A CleanStart happens once: the restarted process loads its startup show.
    let args: Vec<std::ffi::OsString> = std::env::args_os()
        .skip(1)
        .filter(|a| a != "--clean-start")
        .collect();
    sysmon_early("Server", format!("restarting {}", exe.display()));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(&exe).args(&args).exec();
        eprintln!("restart failed: {err}");
        std::process::exit(1);
    }
    #[cfg(not(unix))]
    {
        match std::process::Command::new(&exe).args(&args).spawn() {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("restart failed: {e}");
                std::process::exit(1);
            }
        }
    }
}
