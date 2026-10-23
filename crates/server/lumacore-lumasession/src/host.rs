// ██╗  ██╗ ██████╗ ███████╗████████╗
// ██║  ██║██╔═══██╗██╔════╝╚══██╔══╝
// ███████║██║   ██║███████╗   ██║
// ██╔══██║██║   ██║╚════██║   ██║
// ██║  ██║╚██████╔╝███████║   ██║
// ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝
// FILE: host

//! What LumaSession needs from the station it runs on (decisions §34 19.3: a narrow trait the
//! server implements, 34b's pattern): the state and the session's own fields (one at a time, or
//! both through `split`), the ports and the sender of the request being answered, the discovery
//! cache, the way back onto the state thread, and the station's own work it asks for — the
//! command port's handler, the reply, the network view, the System Monitor, the shells' output.
//!
//! Owns:          SessionHost, what LumaSession needs from the station it runs on.
//! Doesn't own:   its implementation (the server's ServerContext).
//! Entry points:  SessionHost.
//! Decisions:     §34 19.3, 24.2.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use lumacore_net::DiscoveryPeerState;
use lumacore_objects::AppState;
use lumacore_wire::{RemoteReply, RemoteRequest};

use crate::session::LumaSession;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The station a LumaSession runs on (the server's context).
pub trait SessionHost: lumacore_lanes::Host + Sized {
    fn state(&mut self) -> &mut AppState;
    fn state_ref(&self) -> &AppState;
    fn session(&mut self) -> &mut LumaSession;
    fn session_ref(&self) -> &LumaSession;
    /// The state and the session at once.
    fn split(&mut self) -> (&mut AppState, &mut LumaSession);
    /// The UDP command port and the stream port this station listens on.
    fn cmd_port(&self) -> u16;
    fn stream_port(&self) -> u16;
    /// Who sent the command-port request being answered.
    fn request_from(&self) -> Option<SocketAddr>;
    /// What discovery has heard (the listener threads keep it).
    fn discovery_cache(&self) -> &Arc<Mutex<BTreeMap<String, DiscoveryPeerState>>>;
    /// The way onto this station's state thread (None before it runs).
    fn state_handle(&self) -> Option<lumacore_lanes::StateHandle<Self>>;
    /// Bring `Root.Runtime`'s network view up to date.
    fn refresh_runtime(&mut self);
    /// Stop the station.
    fn request_shutdown(&mut self);
    /// The System Monitor lines a command queued.
    fn drain_sysmon_pending(&mut self);
    /// The users with a Shell on this station.
    fn shell_users(&self) -> Vec<String>;
    /// A line in every Shell of `user`; false when it has none.
    fn push_output(&mut self, user: &str, text: &str) -> bool;
    /// A command-port reply carrying this station's facts.
    fn make_reply(&self, ok: bool, message: String) -> RemoteReply;
    /// A command line as `user`, the way the command port runs it.
    fn execute_remote_command(&mut self, user: &str, cmd: &str) -> (RemoteReply, bool);
    /// A command-port request, answered here.
    fn handle_request(&mut self, request: RemoteRequest) -> (RemoteReply, bool);
}
