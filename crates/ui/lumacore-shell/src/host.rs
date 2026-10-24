// ██╗  ██╗ ██████╗ ███████╗████████╗
// ██║  ██║██╔═══██╗██╔════╝╚══██╔══╝
// ███████║██║   ██║███████╗   ██║
// ██╔══██║██║   ██║╚════██║   ██║
// ██║  ██║╚██████╔╝███████║   ██║
// ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝
// FILE: host

//! What the shell port needs from the context it serves (decisions §34 24.2): the terminals with
//! the state they act on, the network view to refresh when a client comes or goes, a way to ask
//! for shutdown, and the System Monitor. The station's `ServerContext` is the one host. Every
//! gesture a client sends is one of the provided methods, run on the state thread.
//!
//! Owns:          ShellHost and its provided methods (attach, render, keys, gestures, taps, UI commands, viewports, focus, screen, scroll).
//! Doesn't own:   the terminals themselves (lumacore-terminal), the state thread (lumacore-lanes), the network view (the server).
//! Entry points:  ShellHost.
//! Decisions:     §23 "Step 8", §30 "12b", "12e hardening", §34 24.2.

use std::net::SocketAddr;

use lumacore_objects::AppState;
use lumacore_terminal::session_manager::{AttachResult, RenderResult, SessionRegistry};
use lumacore_wire::{RemoteShellKey, UiEvent, UiFocus, Viewport};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The context a shell port serves.
pub trait ShellHost: lumacore_lanes::Host + Sized {
    /// The terminals and the state they act on, both at once.
    fn split(&mut self) -> (&mut SessionRegistry, &mut AppState);
    /// Bring `Root.Runtime`'s network view up to date (a client came or went).
    fn refresh_runtime(&mut self);
    /// A shell session asked the station to stop.
    fn request_shutdown(&mut self);
    /// One System Monitor line.
    fn sysmon(&self, subsystem: &str, message: impl AsRef<str>);
    /// The System Monitor lines a command queued.
    fn drain_sysmon_pending(&mut self);

    fn sessions(&mut self) -> &mut SessionRegistry {
        self.split().0
    }

    fn state(&mut self) -> &mut AppState {
        self.split().1
    }

    fn attach_shell_client(&mut self, user: String, remote_addr: SocketAddr, width: u16, height: u16) -> AttachResult {
        let (sessions, state) = self.split();
        sessions.attach_shell_client(user, remote_addr, width, height, state)
    }

    fn render_shell_for_client(&mut self, client_id: uuid::Uuid) -> Result<RenderResult, String> {
        let (sessions, state) = self.split();
        sessions.render_for_client(client_id, state)
    }

    fn set_shell_viewports(&mut self, client_id: uuid::Uuid, viewports: Vec<Viewport>) -> Result<(), String> {
        self.sessions().set_viewports_for_client(client_id, viewports)
    }

    fn handle_shell_key(&mut self, client_id: uuid::Uuid, key: RemoteShellKey) -> Result<Vec<UiEvent>, String> {
        let (sessions, state) = self.split();
        sessions.handle_key_for_client(client_id, state, key)
    }

    fn set_shell_live_screen(&mut self, client_id: uuid::Uuid, no: Option<u32>) -> Result<(), String> {
        self.sessions().set_live_screen_for_client(client_id, no)
    }

    /// A window of the client's screen scrolled (§30 "12e hardening"): kept on the window,
    /// quietly.
    fn set_shell_scroll(&mut self, client_id: uuid::Uuid, window: u32, row: f64) -> Result<(), String> {
        let (sessions, state) = self.split();
        let no = sessions.live_screen_of(client_id).ok_or("no screen")?;
        let mut store = state.root.runtime.live_screens.write().map_err(|_| "screens unavailable".to_string())?;
        let scroll = lumacore_tree::runtime::WindowScroll { h: 0.0, v: row.max(0.0) };
        store.set_scroll(no, window, scroll)
    }

    fn shell_ui_command(&mut self, client_id: uuid::Uuid, cmd: &str, window: Option<u32>) -> Result<Vec<UiEvent>, String> {
        let (sessions, state) = self.split();
        sessions.ui_command_for_client(client_id, state, cmd, window)
    }

    fn shell_key_action(
        &mut self,
        client_id: uuid::Uuid,
        key: &str,
        gesture: &str,
        meow: bool,
        window: Option<u32>,
    ) -> Result<Vec<UiEvent>, String> {
        let (sessions, state) = self.split();
        sessions.key_action_for_client(client_id, state, key, gesture, meow, window)
    }

    /// A keyboard key's Hold / Release gesture (the press came as a key).
    fn shell_key_gesture(&mut self, client_id: uuid::Uuid, key: RemoteShellKey, gesture: &str) -> Result<Vec<UiEvent>, String> {
        let (sessions, state) = self.split();
        sessions.key_gesture_for_client(client_id, state, key, gesture)
    }

    fn shell_tap(&mut self, client_id: uuid::Uuid, tap: lumacore_terminal::Tap) -> Result<Vec<UiEvent>, String> {
        let (sessions, state) = self.split();
        sessions.tap_for_client(client_id, state, tap)
    }

    fn set_shell_focus(&mut self, client_id: uuid::Uuid, focus: UiFocus, window: Option<u32>) -> Result<(), String> {
        self.sessions().set_focus_for_client(client_id, focus, window)
    }
}
