//  ██████╗ ██████╗ ███╗   ██╗████████╗███████╗██╗  ██╗████████╗
// ██╔════╝██╔═══██╗████╗  ██║╚══██╔══╝██╔════╝╚██╗██╔╝╚══██╔══╝
// ██║     ██║   ██║██╔██╗ ██║   ██║   █████╗   ╚███╔╝    ██║
// ██║     ██║   ██║██║╚██╗██║   ██║   ██╔══╝   ██╔██╗    ██║
// ╚██████╗╚██████╔╝██║ ╚████║   ██║   ███████╗██╔╝ ██╗   ██║
//  ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚══════╝╚═╝  ╚═╝   ╚═╝
// FILE: context

//!
//! Owns:          ServerContext (the state, the terminals, the ports, the discovery cache, the engines' info, the LumaSession, the protocol runtimes), refresh_runtime, the discovery beacon, sysmon, the dialogs' housekeeping, the ShellHost and SessionHost impls.
//! Doesn't own:   the state thread (state), LumaSession's own logic (lumacore-lumasession), the shell port (lumacore-shell).
//! Entry points:  ServerContext, sysmon, drain_sysmon_pending.
//! Decisions:     §23 "Step 8", §26, §30 "12b", §34 19.3, 24.2.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use lumacore_terminal::session_manager::SessionRegistry;

use crate::net::sync::sync_session_network_runtime;
use lumacore_net::DiscoveryPeerState;
use lumacore_wire::SessionDiscoveryWire;
use lumacore_objects::build_info as current_build_info;
use lumacore_tree::now_ms as unix_now_ms;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

pub struct ServerContext {
    pub state: lumacore_objects::AppState,
    pub show_path: Option<String>,
    pub root_monitor: bool,
    pub sessions: SessionRegistry,
    pub cmd_port: u16,
    pub shell_port: u16,
    pub discovery_port: u16,
    /// HTTP / UI engine port, once the HTTP engine runs (advertised in the
    /// discovery beacon and over mDNS).
    pub http_port: Option<u16>,
    pub(crate) discovery_cache: Arc<Mutex<BTreeMap<String, DiscoveryPeerState>>>,
    /// Engine info snapshot for discovery beacons (updated by server binary).
    pub engine_info: Vec<lumacore_wire::EngineInfo>,
    /// Set to true when any client issues a shutdown command.
    /// The main loop checks this flag each iteration.
    pub shutdown_requested: bool,
    /// Known station UDP endpoints for replication broadcast.
    pub station_endpoints: BTreeMap<String, SocketAddr>,
    /// Protocol runtimes started from show content (decisions §11); None
    /// until the server sets them up.
    pub protocols: Option<lumacore_lines::ProtocolRuntimes>,
    /// Where the UDP request being handled came from.
    pub request_from: Option<SocketAddr>,
    /// Publish the `StationView` after this job, not at the next tick
    /// (a logout must take effect for HTTP at once).
    pub(crate) publish_station_now: bool,
    /// Which interfaces serve what (shared with the listener threads).
    pub services: lumacore_lumasession::Services,
    /// The LumaSession stream port (decisions §21 "6b").
    pub stream_port: u16,
    /// A way to send jobs to this state thread (link threads use it).
    pub state_handle: Option<crate::state::StateHandle>,
    /// This station's LumaSession: journal, members, replies, election, deferred work (decisions
    /// §34 19.3: one struct, reached by `lumacore_lumasession` through `SessionHost`).
    pub lumasession: lumacore_lumasession::LumaSession,
    /// This station's mDNS record while it is advertised; withdrawn on a
    /// clean shutdown (`Server::wait`, §30 "12b2 hardening").
    pub(crate) mdns: Option<lumacore_net::mdns::Advertisement>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl ServerContext {
    pub fn new(
        state: lumacore_objects::AppState,
        show_path: Option<String>,
        root_monitor: bool,
        cmd_port: u16,
        shell_port: u16,
        discovery_port: u16,
    ) -> Self {
        let mut state = state;
        state.lumasession_ports = (cmd_port, shell_port, lumacore_wire::REMOTE_STREAM_PORT);
        let build_info = current_build_info();
        state.root.diagnostics.build_info = build_info.clone();
        state
            .root
            .diagnostics
            .status_center
            .push(format!("build: {}", build_info.build_id));
        Self {
            state,
            show_path,
            root_monitor,
            sessions: SessionRegistry::new(),
            cmd_port,
            shell_port,
            discovery_port,
            http_port: None,
            discovery_cache: Arc::new(Mutex::new(BTreeMap::new())),
            engine_info: Vec::new(),
            shutdown_requested: false,
            station_endpoints: BTreeMap::new(),
            protocols: None,
            request_from: None,
            publish_station_now: false,
            services: Arc::new(arc_swap::ArcSwap::from_pointee(
                lumacore_lumasession::ServiceAccess::default(),
            )),
            stream_port: lumacore_wire::REMOTE_STREAM_PORT,
            state_handle: None,
            lumasession: Default::default(),
            mdns: None,
        }
    }

    /// Bring `Root.Runtime`'s network view (listeners, discovered stations,
    /// connections, users' clients) up to date.
    pub(crate) fn refresh_runtime(&mut self) {
        let discovered_peers = self
            .discovery_cache
            .lock()
            .map(|cache| cache.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        sync_session_network_runtime(
            &mut self.state,
            &self.sessions,
            (self.cmd_port, self.shell_port, self.stream_port),
            self.http_port,
            discovered_peers,
        );
        self.state.root.runtime.luma_sessions.active_cmd_clients = 0;
        self.state.root.runtime.luma_sessions.active_shell_clients =
            self.sessions.attached_client_count() as u32;
        self.state.root.runtime.network.active_output_lines = self.state.protocols_running.values().sum();
    }

    pub(crate) fn local_discovery_beacon(&self) -> SessionDiscoveryWire {
        SessionDiscoveryWire {
            station_name: self.state.root.station_settings.station_name.clone(),
            authority: self
                .state
                .root
                .runtime
                .joined_sessions
                .values()
                .find(|joined| joined.connected)
                .map(|joined| joined.target.clone())
                .unwrap_or_else(|| self.state.root.station_settings.station_name.clone()),
            cmd_port: self.cmd_port,
            shell_port: self.shell_port,
            stream_port: self.stream_port,
            session_count: self.state.root.runtime.users.len() as u32,
            active_shell_clients: self.sessions.attached_client_count() as u32,
            build_id: self.state.root.diagnostics.build_info.build_id.clone(),
            sent_unix_ms: unix_now_ms(),
            engines: self.engine_info.clone(),
            station_priority: self
                .state
                .root
                .station_settings
                .station_priority
                .to_string(),
            http_port: self.http_port,
            station_id: self.state.local_settings.station_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            show: self.state.current_show.clone(),
            lumasession: self.state.root.station_settings.lumasession.clone(),
            location: self.state.root.station_settings.location.clone(),
            invite: self.state.root.station_settings.invite,
            kind: lumacore_wire::StationKind::Server,
            client_protocol: lumacore_wire::CLIENT_PROTOCOL,
            session_protocol: lumacore_wire::SESSION_PROTOCOL,
            status: lumacore_objects::session_link::status_word(&self.state).to_string(),
            group: lumacore_wire::discovery_group(self.state.root.station_settings.multicast_alternative)
                .to_string(),
            // A master's own epoch and journal tip; a member's master's epoch
            // and what it applied (the election's inputs, §21 "6c").
            epoch: if self.state.link.is_none()
                && !self.state.root.runtime.luma_sessions.members.is_empty()
            {
                self.lumasession.journal.epoch()
            } else {
                self.state.session_epoch
            },
            applied: if self.state.link.is_some() {
                self.state.last_applied.map_or(0, |(_, seq)| seq)
            } else {
                self.lumasession.journal.tip_seq()
            },
            online_since_ms: self.state.online_since_ms,
            show_status: self.state.show_status.to_string(),
            // Only the discovery sender's last beacon on shutdown says so.
            leaving: false,
        }
    }
}

/// LumaSession's host (decisions §34 19.3): the state and the session at once, this station's
/// ports, and the station's own work the session asks for.
impl lumacore_lumasession::SessionHost for ServerContext {
    fn state(&mut self) -> &mut lumacore_objects::AppState {
        &mut self.state
    }
    fn state_ref(&self) -> &lumacore_objects::AppState {
        &self.state
    }
    fn session(&mut self) -> &mut lumacore_lumasession::LumaSession {
        &mut self.lumasession
    }
    fn session_ref(&self) -> &lumacore_lumasession::LumaSession {
        &self.lumasession
    }
    fn split(&mut self) -> (&mut lumacore_objects::AppState, &mut lumacore_lumasession::LumaSession) {
        (&mut self.state, &mut self.lumasession)
    }
    fn cmd_port(&self) -> u16 {
        self.cmd_port
    }
    fn stream_port(&self) -> u16 {
        self.stream_port
    }
    fn request_from(&self) -> Option<SocketAddr> {
        self.request_from
    }
    fn discovery_cache(&self) -> &Arc<Mutex<BTreeMap<String, DiscoveryPeerState>>> {
        &self.discovery_cache
    }
    fn state_handle(&self) -> Option<crate::state::StateHandle> {
        self.state_handle.clone()
    }
    fn refresh_runtime(&mut self) {
        ServerContext::refresh_runtime(self);
    }
    fn request_shutdown(&mut self) {
        self.shutdown_requested = true;
    }
    fn drain_sysmon_pending(&mut self) {
        drain_sysmon_pending(self);
    }
    fn shell_users(&self) -> Vec<String> {
        self.sessions.users()
    }
    fn push_output(&mut self, user: &str, text: &str) -> bool {
        self.sessions.push_output(user, text)
    }
    fn make_reply(&self, ok: bool, message: String) -> lumacore_wire::RemoteReply {
        crate::net::handler::make_reply(self, ok, message)
    }
    fn execute_remote_command(&mut self, user: &str, cmd: &str) -> (lumacore_wire::RemoteReply, bool) {
        crate::net::handler::execute_remote_command(self, user, cmd)
    }
    fn handle_request(&mut self, request: lumacore_wire::RemoteRequest) -> (lumacore_wire::RemoteReply, bool) {
        crate::net::handler::handle_request(self, request)
    }
}

/// The shell port's host (decisions §34 24.2): its clients' terminals and the state they act on.
impl lumacore_shell::ShellHost for ServerContext {
    fn split(&mut self) -> (&mut lumacore_terminal::session_manager::SessionRegistry, &mut lumacore_objects::AppState) {
        (&mut self.sessions, &mut self.state)
    }
    fn refresh_runtime(&mut self) {
        ServerContext::refresh_runtime(self);
    }
    fn request_shutdown(&mut self) {
        self.shutdown_requested = true;
    }
    fn sysmon(&self, subsystem: &str, message: impl AsRef<str>) {
        sysmon(self, subsystem, message);
    }
    fn drain_sysmon_pending(&mut self) {
        drain_sysmon_pending(self);
    }
}

/// Emit a sysmon diagnostic line (log, monitor clients, and stderr when
/// `--RootMonitor` is on).
pub fn sysmon(ctx: &ServerContext, subsystem: &str, message: impl AsRef<str>) {
    lumacore_sysmon::sysmon_line(subsystem, message, ctx.root_monitor);
}

/// Drain sysmon messages queued by the command dispatch layer.
pub(crate) fn drain_sysmon_pending(ctx: &mut ServerContext) {
    let pending: Vec<(String, String)> = std::mem::take(&mut ctx.state.sysmon_pending);
    for (subsystem, message) in pending {
        sysmon(ctx, &subsystem, message);
    }
}

/// The station's dialogs (decisions §30 "12b", housekeeping ~50 ms): a
/// countdown that ended presses its default button — the line runs in a
/// client of the dialog's audience as that client's user —, and a dialog
/// whose client has been gone a while counts as cancelled.
pub(crate) fn dialog_tick(ctx: &mut ServerContext) {
    if ctx.state.dialogs.is_empty() {
        return;
    }
    for due in lumacore_objects::dialogs::take_due(&mut ctx.state) {
        let Some(line) = due.line else {
            continue;
        };
        if let Some(client) = ctx.sessions.client_for(&due.to) {
            let _ = ctx
                .sessions
                .ui_command_for_client(client, &mut ctx.state, &line, None);
        }
    }
    let sessions = &ctx.sessions;
    lumacore_objects::dialogs::forget_gone(&mut ctx.state, |to| sessions.client_for(to).is_some());
    // A waiting login whose client went away with its dialog ends with it (decisions 0290).
    lumacore_objects::consent::forget_gone(&mut ctx.state);
}
