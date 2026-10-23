// ███████╗███████╗███████╗███████╗██╗ ██████╗ ███╗   ██╗███████╗
// ██╔════╝██╔════╝██╔════╝██╔════╝██║██╔═══██╗████╗  ██║██╔════╝
// ███████╗█████╗  ███████╗███████╗██║██║   ██║██╔██╗ ██║███████╗
// ╚════██║██╔══╝  ╚════██║╚════██║██║██║   ██║██║╚██╗██║╚════██║
// ███████║███████╗███████║███████║██║╚██████╔╝██║ ╚████║███████║
// ╚══════╝╚══════╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝
// FILE: sessions

//! The shell sessions: one per Shell (TCP, the page's /ws/shell, the local console) — its
//! frame, keys, command lines, gestures and dialog answers, kept in order.
//!
//! Owns:          SessionRegistry: every Shell's session, its frames, keys, lines and gestures
//! Doesn't own:   the shell socket (lumacore-shell), the state thread (server)
//! Entry points:  SessionRegistry
//! Decisions:     §23 "Step 8", §30 "12f"

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::time::SystemTime;

use uuid::Uuid;

use crate::core::AppState;
use crate::repl::{ClientView, RemoteCommandLine, RenderedShellFrame};
use lumacore_wire::{RemoteShellKey, UiEvent, UiFocus, Viewport};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

const MIN_SHELL_WIDTH: u16 = 40;
const MIN_SHELL_HEIGHT: u16 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellSizePolicy {
    Smallest,
    Latest,
    Largest,
}

#[derive(Debug, Clone)]
pub struct ClientHandle {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user: String,
    pub remote_addr: SocketAddr,
    pub width: u16,
    pub height: u16,
    pub attached_at: SystemTime,
    pub last_seen_at: SystemTime,
    /// This client's screen: focus, screen, terminal viewports.
    pub view: ClientView,
}

pub struct CommandLine {
    pub id: Uuid,
    pub user: String,
    pub attached_clients: HashSet<Uuid>,
    pub size_policy: ShellSizePolicy,
    pub effective_width: u16,
    pub effective_height: u16,
    pub created_at: SystemTime,
    pub last_attached_at: SystemTime,
    pub last_detached_at: Option<SystemTime>,
    shell: RemoteCommandLine,
}

#[derive(Debug, Clone)]
pub struct AttachResult {
    pub client_id: Uuid,
    pub session_id: Uuid,
    pub reused_existing_session: bool,
}

#[derive(Debug, Clone)]
pub struct RenderResult {
    /// One frame per viewport (`Some(view)`), or one untagged frame.
    pub frames: Vec<(Option<u32>, RenderedShellFrame)>,
    /// Status for the client's command line bar.
    pub status: lumacore_wire::Status,
    pub detached: bool,
    pub running: bool,
    pub exit_code: i32,
}

#[derive(Debug, Clone)]
pub struct CommandLineSnapshot {
    pub id: Uuid,
    pub user: String,
    pub attached_client_ids: Vec<Uuid>,
    pub effective_width: u16,
    pub effective_height: u16,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct ClientHandleSnapshot {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user: String,
    pub remote_addr: SocketAddr,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Default)]
pub struct SessionRegistrySnapshot {
    pub sessions: Vec<CommandLineSnapshot>,
    pub clients: Vec<ClientHandleSnapshot>,
}

pub struct SessionRegistry {
    command_lines: HashMap<Uuid, CommandLine>,
    clients: HashMap<Uuid, ClientHandle>,
    user_to_session: HashMap<String, Uuid>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self {
            command_lines: HashMap::new(),
            clients: HashMap::new(),
            user_to_session: HashMap::new(),
        }
    }

    pub fn command_line_count(&self) -> usize {
        self.command_lines.len()
    }

    pub fn attached_client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn attach_shell_client(
        &mut self,
        user: String,
        remote_addr: SocketAddr,
        width: u16,
        height: u16,
        state: &mut AppState,
    ) -> AttachResult {
        let now = SystemTime::now();
        let requested_width = width.max(MIN_SHELL_WIDTH);
        let requested_height = height.max(MIN_SHELL_HEIGHT);

        let (session_id, reused_existing_session) =
            self.session_for_user(&user, requested_width, requested_height, state);

        let client_id = Uuid::new_v4();
        let mut view = ClientView::default();
        view.client = Some(client_id);
        self.clients.insert(
            client_id,
            ClientHandle {
                id: client_id,
                session_id,
                user: user.clone(),
                remote_addr,
                width: requested_width,
                height: requested_height,
                attached_at: now,
                last_seen_at: now,
                view,
            },
        );

        if let Some(session) = self.command_lines.get_mut(&session_id) {
            session.attached_clients.insert(client_id);
            session.last_attached_at = now;
            session.last_detached_at = None;
            session.shell.prepare_for_attach();
        }
        self.recalculate_session_size(session_id);

        AttachResult {
            client_id,
            session_id,
            reused_existing_session,
        }
    }

    /// The user's shell session, created if needed. Returns `(id, reused)`.
    fn session_for_user(
        &mut self,
        user: &str,
        width: u16,
        height: u16,
        state: &mut AppState,
    ) -> (Uuid, bool) {
        let user_key = user.to_ascii_lowercase();
        if let Some(id) = self.user_to_session.get(&user_key).copied() {
            if self.command_lines.contains_key(&id) {
                return (id, true);
            }
            self.user_to_session.remove(&user_key);
        }
        let now = SystemTime::now();
        let session_id = Uuid::new_v4();
        let shell = RemoteCommandLine::new(user.to_string(), state, width, height);
        self.command_lines.insert(
            session_id,
            CommandLine {
                id: session_id,
                user: user.to_string(),
                attached_clients: HashSet::new(),
                size_policy: ShellSizePolicy::Smallest,
                effective_width: width,
                effective_height: height,
                created_at: now,
                last_attached_at: now,
                last_detached_at: None,
                shell,
            },
        );
        self.user_to_session.insert(user_key, session_id);
        (session_id, false)
    }

    /// Users with an attached client on this station, sorted (the master
    /// learns who is logged in where — decisions §21 "6b").
    pub fn users(&self) -> Vec<String> {
        let mut users: Vec<String> = self
            .command_lines
            .values()
            .filter(|s| !s.attached_clients.is_empty())
            .map(|s| s.user.clone())
            .filter(|u| !u.eq_ignore_ascii_case(crate::core::INTERNAL_ROOT_USER))
            .collect();
        users.sort();
        users.dedup();
        users
    }

    /// Print `text` in `user`'s Shell. False when the user has none here.
    pub fn push_output(&mut self, user: &str, text: &str) -> bool {
        let Some(id) = self.user_to_session.get(&user.to_ascii_lowercase()) else {
            return false;
        };
        match self.command_lines.get_mut(id) {
            Some(session) => {
                session.shell.push_output(text);
                true
            }
            None => false,
        }
    }

    /// The user a client works as.
    pub fn client_user(&self, client_id: Uuid) -> Option<&str> {
        self.clients.get(&client_id).map(|c| c.user.as_str())
    }

    /// A `Login` from this client asked to switch user, if any.
    pub fn take_user_switch(&mut self, client_id: Uuid) -> Option<String> {
        self.clients.get_mut(&client_id)?.view.switch_user.take()
    }

    /// Move a client to `user`'s shell session (after `Login`): it gets that
    /// user's command line, history and programmer. Its screen is dropped — the
    /// UI rejoins one of the new user's screens. Returns the new session id.
    pub fn move_client_to_user(
        &mut self,
        client_id: Uuid,
        user: &str,
        state: &mut AppState,
    ) -> Result<Uuid, String> {
        let (old_session, width, height) = {
            let c = self
                .clients
                .get(&client_id)
                .ok_or_else(|| "unknown shell client".to_string())?;
            (c.session_id, c.width, c.height)
        };
        let (new_session, _) = self.session_for_user(user, width, height, state);
        if new_session == old_session {
            return Ok(new_session);
        }
        if let Some(session) = self.command_lines.get_mut(&old_session) {
            session.attached_clients.remove(&client_id);
            session.last_detached_at = Some(SystemTime::now());
        }
        if let Some(client) = self.clients.get_mut(&client_id) {
            client.session_id = new_session;
            client.user = user.to_string();
            client.view.live_screen = None;
            client.view.window = None;
        }
        if let Some(session) = self.command_lines.get_mut(&new_session) {
            session.attached_clients.insert(client_id);
            session.last_attached_at = SystemTime::now();
            session.last_detached_at = None;
            session.shell.prepare_for_attach();
        }
        self.recalculate_session_size(old_session);
        self.prune_closed_session_if_idle(old_session);
        self.recalculate_session_size(new_session);
        Ok(new_session)
    }

    pub fn detach_client(&mut self, client_id: Uuid) -> Option<ClientHandle> {
        let client = self.clients.remove(&client_id)?;
        if let Some(session) = self.command_lines.get_mut(&client.session_id) {
            session.attached_clients.remove(&client_id);
            session.last_detached_at = Some(SystemTime::now());
        }
        self.recalculate_session_size(client.session_id);
        self.prune_closed_session_if_idle(client.session_id);
        Some(client)
    }

    pub fn resize_client(
        &mut self,
        client_id: Uuid,
        width: u16,
        height: u16,
    ) -> Result<(), String> {
        let session_id = {
            let client = self
                .clients
                .get_mut(&client_id)
                .ok_or_else(|| "unknown shell client".to_string())?;
            client.width = width.max(MIN_SHELL_WIDTH);
            client.height = height.max(MIN_SHELL_HEIGHT);
            client.last_seen_at = SystemTime::now();
            client.session_id
        };
        self.recalculate_session_size(session_id);
        Ok(())
    }

    /// The client and the shared session it is attached to.
    fn client_and_session(
        &mut self,
        client_id: Uuid,
    ) -> Result<(&mut ClientHandle, &mut CommandLine), String> {
        let client = self
            .clients
            .get_mut(&client_id)
            .ok_or_else(|| "unknown shell client".to_string())?;
        let session = self
            .command_lines
            .get_mut(&client.session_id)
            .ok_or_else(|| "unknown shell session".to_string())?;
        Ok((client, session))
    }

    pub fn handle_key_for_client(
        &mut self,
        client_id: Uuid,
        state: &mut AppState,
        key: RemoteShellKey,
    ) -> Result<Vec<UiEvent>, String> {
        let (client, session) = self.client_and_session(client_id)?;
        client.last_seen_at = SystemTime::now();
        Ok(session.shell.handle_key(&mut client.view, state, key))
    }

    pub fn set_live_screen_for_client(
        &mut self,
        client_id: Uuid,
        no: Option<u32>,
    ) -> Result<(), String> {
        self.client_and_session(client_id)?.0.view.live_screen = no;
        Ok(())
    }

    /// The screen the client shows, if it said.
    pub fn live_screen_of(&mut self, client_id: Uuid) -> Option<u32> {
        self.client_and_session(client_id).ok()?.0.view.live_screen
    }

    /// The client is a UI (web / app window).
    pub fn mark_ui_client(&mut self, client_id: Uuid) {
        if let Some(c) = self.clients.get_mut(&client_id) {
            c.view.ui = true;
        }
    }

    /// The client is on another machine: in-session `Login` checks Remote
    /// Login (decisions §19).
    /// The client's device (decisions §30 "12a3").
    /// The clients that can hold something (an echo's `Top On`, decisions §32): attached, on a
    /// device that isn't locked. A holder missing here has gone away.
    pub fn holders(&self, state: &AppState) -> std::collections::HashSet<String> {
        self.clients
            .values()
            .filter(|c| !c.view.device.as_ref().is_some_and(|d| crate::core::devices::device_locked(state, d)))
            .map(|c| c.id.to_string())
            .collect()
    }

    pub fn set_client_device(&mut self, client_id: Uuid, device: String) {
        if let Some(c) = self.clients.get_mut(&client_id) {
            c.view.device = Some(device);
        }
    }

    pub fn mark_remote_client(&mut self, client_id: Uuid) {
        if let Some(c) = self.clients.get_mut(&client_id) {
            c.view.remote = true;
        }
    }

    pub fn set_viewports_for_client(
        &mut self,
        client_id: Uuid,
        viewports: Vec<Viewport>,
    ) -> Result<(), String> {
        self.client_and_session(client_id)?
            .0
            .view
            .set_viewports(viewports);
        Ok(())
    }

    pub fn tap_for_client(
        &mut self,
        client_id: Uuid,
        state: &mut AppState,
        object: &str,
        no: u32,
        hold: bool,
        action: Option<&str>,
        window: Option<u32>,
    ) -> Result<Vec<UiEvent>, String> {
        let (client, session) = self.client_and_session(client_id)?;
        Ok(session
            .shell
            .handle_tap(&mut client.view, state, object, no, hold, action, window))
    }

    pub fn ui_command_for_client(
        &mut self,
        client_id: Uuid,
        state: &mut AppState,
        cmd: &str,
        window: Option<u32>,
    ) -> Result<Vec<UiEvent>, String> {
        let (client, session) = self.client_and_session(client_id)?;
        Ok(session.shell.run_ui_command(&mut client.view, state, cmd, window))
    }

    /// A continuous control's line (decisions §30 "12f"): run quietly as this client's user.
    pub fn gesture_for_client(
        &mut self,
        client_id: Uuid,
        state: &mut AppState,
        gesture: crate::Gesture,
    ) -> Result<Vec<UiEvent>, String> {
        let (client, session) = self.client_and_session(client_id)?;
        client.last_seen_at = SystemTime::now();
        Ok(session.shell.run_gesture(&mut client.view, state, gesture))
    }

    pub fn answer_dialog_for_client(
        &mut self,
        client_id: Uuid,
        state: &mut AppState,
        answer: &lumacore_wire::DialogAnswer,
    ) -> Result<Vec<UiEvent>, String> {
        let (client, session) = self.client_and_session(client_id)?;
        client.last_seen_at = SystemTime::now();
        Ok(session.shell.answer_dialog(&mut client.view, state, answer))
    }

    pub fn suggest_for_client(
        &mut self,
        client_id: Uuid,
        state: &mut AppState,
        seq: u64,
        line: Option<String>,
        cursor: Option<u32>,
        surface: lumacore_wire::Surface,
    ) -> Result<lumacore_wire::Suggestions, String> {
        let (client, session) = self.client_and_session(client_id)?;
        Ok(session.shell.suggest(&mut client.view, state, seq, line, cursor, surface))
    }

    pub fn edit_line_for_client(
        &mut self,
        client_id: Uuid,
        state: &mut AppState,
        start: u32,
        end: u32,
        text: &str,
        expect: Option<&str>,
    ) -> Result<(), String> {
        let (client, session) = self.client_and_session(client_id)?;
        client.last_seen_at = SystemTime::now();
        session.shell.edit_line(&mut client.view, state, start, end, text, expect);
        Ok(())
    }

    pub fn pause_dialog_for_client(
        &mut self,
        client_id: Uuid,
        state: &mut AppState,
        id: &str,
    ) -> Result<(), String> {
        let (client, session) = self.client_and_session(client_id)?;
        session.shell.pause_dialog(&mut client.view, state, id);
        Ok(())
    }

    /// A client of `to` (decisions §30 "12b"), to run a dialog's default
    /// line in, or to know it is still there.
    pub fn client_for(&self, to: &crate::core::dialogs::Audience) -> Option<Uuid> {
        use crate::core::dialogs::Audience;
        self.clients
            .values()
            .find(|c| match to {
                Audience::Device(d) => c.view.device.as_deref() == Some(d.as_str()),
                Audience::Client(id) => c.id == *id,
                Audience::User(u) => c.user.eq_ignore_ascii_case(u),
                Audience::Local | Audience::Station => false,
            })
            .map(|c| c.id)
    }

    /// A UI key pressed / double-pressed / held on `client_id`'s UI.
    pub fn key_action_for_client(
        &mut self,
        client_id: Uuid,
        state: &mut AppState,
        key: &str,
        gesture: &str,
        meow: bool,
        window: Option<u32>,
    ) -> Result<Vec<UiEvent>, String> {
        let (client, session) = self.client_and_session(client_id)?;
        client.last_seen_at = SystemTime::now();
        Ok(session
            .shell
            .key_action(&mut client.view, state, key, gesture, meow, window))
    }

    pub fn key_gesture_for_client(
        &mut self,
        client_id: Uuid,
        state: &mut AppState,
        key: RemoteShellKey,
        gesture: &str,
    ) -> Result<Vec<UiEvent>, String> {
        let (client, session) = self.client_and_session(client_id)?;
        client.last_seen_at = SystemTime::now();
        Ok(session.shell.key_gesture(&mut client.view, state, key, gesture))
    }

    pub fn set_focus_for_client(
        &mut self,
        client_id: Uuid,
        focus: UiFocus,
        window: Option<u32>,
    ) -> Result<(), String> {
        let view = &mut self.client_and_session(client_id)?.0.view;
        view.focus = focus;
        view.window = window;
        Ok(())
    }

    /// Render the session for one client: a frame per terminal viewport, or a
    /// single frame at the client's own size — clients no longer shrink each
    /// other.
    pub fn render_for_client(
        &mut self,
        client_id: Uuid,
        state: &mut AppState,
    ) -> Result<RenderResult, String> {
        let session_id = self
            .clients
            .get(&client_id)
            .map(|client| client.session_id)
            .ok_or_else(|| "unknown shell client".to_string())?;
        self.recalculate_session_size(session_id);
        let (client, session) = self.client_and_session(client_id)?;
        let size = (client.width, client.height);
        let frames = session
            .shell
            .render_client(&mut client.view, state, Some(size));
        let status = session.shell.status(&mut client.view, state);
        // The client that ran Logout (see `ClientView::detach`).
        let detached =
            std::mem::take(&mut client.view.detach) | session.shell.take_detach_request();
        let running = session.shell.is_running();
        let exit_code = session.shell.exit_code();
        Ok(RenderResult {
            frames,
            status,
            detached,
            running,
            exit_code,
        })
    }

    pub fn session_summary(&self, session_id: Uuid) -> Option<String> {
        let session = self.command_lines.get(&session_id)?;
        Some(format!(
            "session={} user={} clients={} size={}x{} policy={:?}",
            session.id,
            session.user,
            session.attached_clients.len(),
            session.effective_width,
            session.effective_height,
            session.size_policy,
        ))
    }

    pub fn snapshot(&self) -> SessionRegistrySnapshot {
        let mut sessions: Vec<CommandLineSnapshot> = self
            .command_lines
            .values()
            .map(|session| CommandLineSnapshot {
                id: session.id,
                user: session.user.clone(),
                attached_client_ids: session.attached_clients.iter().copied().collect(),
                effective_width: session.effective_width,
                effective_height: session.effective_height,
                running: session.shell.is_running(),
            })
            .collect();
        sessions.sort_by(|a, b| a.user.cmp(&b.user));

        let mut clients: Vec<ClientHandleSnapshot> = self
            .clients
            .values()
            .map(|client| ClientHandleSnapshot {
                id: client.id,
                session_id: client.session_id,
                user: client.user.clone(),
                remote_addr: client.remote_addr,
                width: client.width,
                height: client.height,
            })
            .collect();
        clients.sort_by_key(|client| client.id);

        SessionRegistrySnapshot { sessions, clients }
    }

    fn prune_closed_session_if_idle(&mut self, session_id: Uuid) {
        let should_remove = self
            .command_lines
            .get(&session_id)
            .map(|session| session.attached_clients.is_empty() && !session.shell.is_running())
            .unwrap_or(false);
        if should_remove && let Some(session) = self.command_lines.remove(&session_id) {
            self.user_to_session
                .remove(&session.user.to_ascii_lowercase());
        }
    }

    fn recalculate_session_size(&mut self, session_id: Uuid) {
        let Some((policy, current_width, current_height, attached_clients)) =
            self.command_lines.get(&session_id).map(|session| {
                (
                    session.size_policy,
                    session.effective_width,
                    session.effective_height,
                    session.attached_clients.iter().copied().collect::<Vec<_>>(),
                )
            })
        else {
            return;
        };

        let (width, height) = select_size(
            policy,
            current_width,
            current_height,
            attached_clients
                .into_iter()
                .filter_map(|client_id| self.clients.get(&client_id))
                .collect(),
        );

        if let Some(session) = self.command_lines.get_mut(&session_id) {
            session.effective_width = width;
            session.effective_height = height;
            session.shell.resize(width, height);
        }
    }
}

fn select_size(
    policy: ShellSizePolicy,
    current_width: u16,
    current_height: u16,
    clients: Vec<&ClientHandle>,
) -> (u16, u16) {
    if clients.is_empty() {
        return (
            current_width.max(MIN_SHELL_WIDTH),
            current_height.max(MIN_SHELL_HEIGHT),
        );
    }

    match policy {
        ShellSizePolicy::Smallest => (
            clients
                .iter()
                .map(|client| client.width)
                .min()
                .unwrap_or(current_width)
                .max(MIN_SHELL_WIDTH),
            clients
                .iter()
                .map(|client| client.height)
                .min()
                .unwrap_or(current_height)
                .max(MIN_SHELL_HEIGHT),
        ),
        ShellSizePolicy::Largest => (
            clients
                .iter()
                .map(|client| client.width)
                .max()
                .unwrap_or(current_width)
                .max(MIN_SHELL_WIDTH),
            clients
                .iter()
                .map(|client| client.height)
                .max()
                .unwrap_or(current_height)
                .max(MIN_SHELL_HEIGHT),
        ),
        ShellSizePolicy::Latest => {
            let latest = clients
                .into_iter()
                .max_by_key(|client| client.last_seen_at)
                .expect("clients already checked non-empty");
            (
                latest.width.max(MIN_SHELL_WIDTH),
                latest.height.max(MIN_SHELL_HEIGHT),
            )
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
