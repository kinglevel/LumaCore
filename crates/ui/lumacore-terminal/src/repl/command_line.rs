//  ██████╗███╗   ███╗██████╗     ██╗     ██╗███╗   ██╗███████╗
// ██╔════╝████╗ ████║██╔══██╗    ██║     ██║████╗  ██║██╔════╝
// ██║     ██╔████╔██║██║  ██║    ██║     ██║██╔██╗ ██║█████╗
// ██║     ██║╚██╔╝██║██║  ██║    ██║     ██║██║╚██╗██║██╔══╝
// ╚██████╗██║ ╚═╝ ██║██████╔╝    ███████╗██║██║ ╚████║███████╗
//  ╚═════╝╚═╝     ╚═╝╚═════╝     ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝
// FILE: cmd_line

//! Remote shell session and ANSI buffer rendering.
//!
//! Owns:          RemoteCommandLine, the ANSI buffer of a remote shell
//! Doesn't own:   the TUI's own input (tui_input)
//! Entry points:  RemoteCommandLine
//! Decisions:     §23 "Step 8", §30 "12f"

use ratatui::prelude::*;

use std::collections::BTreeMap;

use super::TuiApp;
use super::tui_types::{RenderedShellFrame, ViewState};
use lumacore_wire::{RemoteShellKey, UiEvent, UiFocus, Viewport};
use crate::core::AppState;
use crate::core::claims::Claims;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One user's shell session. Its buffers — command line, history, output,
/// suggestions, popups — are shared by every client of that user (each
/// browser screen, the CLI); per-screen state lives in each `ClientView`.
pub struct RemoteCommandLine {
    app: TuiApp,
    user: String,
    width: u16,
    height: u16,
    navigation_path: Vec<String>,
    context: crate::core::ReplContext,
}

/// Per-client (per-screen) state of a shared shell session: which screen
/// it shows, where its focus is, and its terminal viewports, each with its
/// own size and scroll.
#[derive(Debug, Clone)]
pub struct ClientView {
    /// The screen (screen) of the client, if any.
    pub live_screen: Option<u32>,
    /// The focused window in that screen.
    pub window: Option<u32>,
    /// Where navigation keys go.
    pub focus: UiFocus,
    /// Terminal windows, focused (or last focused) first. Empty = one frame
    /// at the client's size, with `main` as its view.
    viewports: Vec<Viewport>,
    views: BTreeMap<u32, ViewState>,
    main: ViewState,
    /// `Login` from this client: move it to that user's session.
    pub switch_user: Option<String>,
    /// A UI client (web / app window, via the WebSocket bridge) — known from
    /// its hello, before it has terminals or a screen.
    pub ui: bool,
    /// The client is on another machine (real address not loopback).
    pub remote: bool,
    /// Its device (decisions §30 "12a3"); None = not a UI device (CLI).
    pub device: Option<String>,
    /// Its shell client id (the station's dialogs for a CLI find it by it).
    pub client: Option<uuid::Uuid>,
    /// `Logout` / `Disconnect` from this client: close this client — not
    /// another window of the same user that happens to render first.
    pub detach: bool,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Default for ClientView {
    fn default() -> Self {
        Self {
            live_screen: None,
            window: None,
            focus: UiFocus::Terminal,
            viewports: Vec::new(),
            views: BTreeMap::new(),
            main: ViewState::focused(),
            switch_user: None,
            ui: false,
            remote: false,
            device: None,
            client: None,
            detach: false,
        }
    }
}

impl ClientView {
    /// Replace the viewport list. Views keep their scroll by id; the first
    /// viewport draws the overlays.
    pub fn set_viewports(&mut self, viewports: Vec<Viewport>) {
        self.views
            .retain(|id, _| viewports.iter().any(|v| v.view == *id));
        for (i, vp) in viewports.iter().enumerate() {
            let view = self.views.entry(vp.view).or_default();
            view.overlays = i == 0;
            view.layers = vp.layers.into();
        }
        self.viewports = viewports;
    }

    pub fn viewports(&self) -> &[Viewport] {
        &self.viewports
    }

    /// The view key handling acts on (scroll keys, page size).
    fn active_mut(&mut self) -> &mut ViewState {
        match self.viewports.first() {
            Some(vp) => self.views.entry(vp.view).or_insert_with(ViewState::focused),
            None => &mut self.main,
        }
    }
}

impl RemoteCommandLine {
    pub fn new(user: String, state: &mut AppState, width: u16, height: u16) -> Self {
        let app = TuiApp::new(state);
        Self {
            app,
            user,
            width: width.max(40),
            height: height.max(8),
            navigation_path: Vec::new(),
            context: crate::core::ReplContext::Fixture,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width.max(40);
        self.height = height.max(8);
    }

    /// Handle one key from `client`; returns navigation events for its
    /// focused panel.
    pub fn handle_key(
        &mut self,
        client: &mut ClientView,
        state: &mut AppState,
        key: RemoteShellKey,
    ) -> Vec<UiEvent> {
        self.with_client(client, state, |app, state| {
            let event = pressed(app, state, &key);
            app.handle_key_with_repeat(event, key.repeat, state);
            app.take_ui_events(state)
        })
    }

    /// A command issued by the client's UI (e.g. a dialog's OK): echoed and
    /// executed like a typed command.
    pub fn run_ui_command(
        &mut self,
        client: &mut ClientView,
        state: &mut AppState,
        cmd: &str,
        window: Option<u32>,
    ) -> Vec<UiEvent> {
        self.with_client(client, state, |app, state| {
            // A window bar / menu / settings line names its window: `Self`
            // (decisions §30 "12c").
            state.line_owner = window
                .zip(state.current_live_screen)
                .map(|(no, screen)| crate::core::self_ref::LineOwner::Window { screen, no });
            app.run_ui_command(state, cmd);
            state.line_owner = None;
            app.take_ui_events(state)
        })
    }

    /// A continuous control's line (decisions §30 "12f", the gesture channel): dispatched as
    /// this user like any line, but with no echo, history, Usage or journal per move; the end
    /// files one entry naming what moved. A refusal is told once per gesture.
    pub fn run_gesture(
        &mut self,
        client: &mut ClientView,
        state: &mut AppState,
        g: crate::Gesture,
    ) -> Vec<UiEvent> {
        use crate::core::gesture::GestureLine;
        self.with_client(client, state, |app, state| {
            let entry = app.gestures.entry(g.id.clone()).or_insert_with(|| super::GestureLog {
                label: g.label.clone().unwrap_or_else(|| g.line.clone()),
                from: g.from.clone().unwrap_or_else(|| "?".into()),
                to: String::new(),
                told: false,
            });
            if let Some(v) = &g.value {
                entry.to = v.clone();
            }
            let end = g.phase == lumacore_wire::GesturePhase::End;
            state.gesture = Some(if end {
                let to = if entry.to.is_empty() { g.line.clone() } else { entry.to.clone() };
                GestureLine::End { id: g.id.clone(), summary: format!("Fader {}: {} → {to}", entry.label, entry.from) }
            } else {
                GestureLine::Move { id: g.id.clone() }
            });
            state.line_owner = g
                .window
                .zip(state.current_live_screen)
                .map(|(no, screen)| crate::core::self_ref::LineOwner::Window { screen, no });
            let result = crate::core::commands::dispatch(state, g.line.trim());
            state.line_owner = None;
            state.gesture = None;
            let failed = matches!(&result, crate::core::DispatchResult::Output(r) if r.failed);
            let entry = app.gestures.get_mut(&g.id).expect("just inserted");
            if failed && !entry.told {
                entry.told = true;
                if let crate::core::DispatchResult::Output(r) = &result {
                    state.ui_requests.push(UiEvent::Notice {
                        text: crate::core::text::strip_ansi(&r.to_string()).trim_start_matches("Error:").trim().to_string(),
                        level: "error".into(),
                    });
                }
            }
            if end {
                app.gestures.remove(&g.id);
            }
            app.take_ui_events(state)
        })
    }

    /// The client answers one of the station's dialogs (decisions §30
    /// "12b"): closed everywhere, then the button's line runs here as this
    /// user, echoed like a typed command.
    pub fn answer_dialog(
        &mut self,
        client: &mut ClientView,
        state: &mut AppState,
        answer: &lumacore_wire::DialogAnswer,
    ) -> Vec<UiEvent> {
        self.with_client(client, state, |app, state| {
            let who = crate::core::dialogs::Who::current(state);
            match crate::core::dialogs::answer(state, &who, answer) {
                Ok(Some(line)) => app.run_ui_command(state, &line),
                Ok(None) => {}
                Err(e) => app.add_output(&e),
            }
            app.take_ui_events(state)
        })
    }

    /// Suggestions for `line` at `cursor` (chars), or for the session's own
    /// command line where it is now (decisions §30 "12b2"), as this client's
    /// user sees them.
    pub fn suggest(
        &mut self,
        client: &mut ClientView,
        state: &mut AppState,
        seq: u64,
        line: Option<String>,
        cursor: Option<u32>,
        surface: lumacore_wire::Surface,
    ) -> lumacore_wire::Suggestions {
        self.with_client(client, state, |app, state| {
            let mut s = app.suggestions(state, line, cursor.map(|c| c as usize), surface);
            s.seq = seq;
            s
        })
    }

    /// Take a suggestion: chars `start..end` of the command line become
    /// `text` — unless the line is no longer `expect`.
    pub fn edit_line(
        &mut self,
        client: &mut ClientView,
        state: &mut AppState,
        start: u32,
        end: u32,
        text: &str,
        expect: Option<&str>,
    ) -> Vec<UiEvent> {
        // Its events too: a line edited into (or out of) `Assign Tag 1 At` arms the pools.
        self.with_client(client, state, |app, state| {
            app.edit_line(start as usize, end as usize, text, expect);
            app.take_ui_events(state)
        })
    }

    /// A touch on a dialog with a countdown: it waits now.
    pub fn pause_dialog(&mut self, client: &mut ClientView, state: &mut AppState, id: &str) {
        self.with_client(client, state, |app, state| {
            let who = crate::core::dialogs::Who::current(state);
            if let Some(line) = crate::core::dialogs::pause(state, &who, id) {
                app.run_ui_command(state, &line);
            }
        });
    }

    /// A UI key of the client's UI: runs its command (see `run_key_action`).
    pub fn key_action(
        &mut self,
        client: &mut ClientView,
        state: &mut AppState,
        key: &str,
        gesture: &str,
        meow: bool,
        window: Option<u32>,
    ) -> Vec<UiEvent> {
        self.with_client(client, state, |app, state| {
            // A key in a window bar acts on that window; from the keyboard on
            // the focused one, never on the key (decisions §30 "12c").
            state.line_owner = Some(match window.zip(state.current_live_screen) {
                Some((no, screen)) => crate::core::self_ref::LineOwner::Window { screen, no },
                None => crate::core::self_ref::LineOwner::Key(key.to_string()),
            });
            app.run_key_action(state, key, gesture, meow);
            state.line_owner = None;
            app.take_ui_events(state)
        })
    }

    /// A keyboard key held or released: the bound UI key's gesture.
    pub fn key_gesture(
        &mut self,
        client: &mut ClientView,
        state: &mut AppState,
        key: RemoteShellKey,
        gesture: &str,
    ) -> Vec<UiEvent> {
        self.with_client(client, state, |app, state| {
            let key = pressed(app, state, &key);
            if let Some((id, meow)) = app.bound_key(state, &key) {
                app.run_key_action(state, &id, gesture, meow);
            }
            app.take_ui_events(state)
        })
    }

    /// A UI object was tapped (e.g. a view button): completes the pending
    /// command line with it, or runs its default action. Returns UI events
    /// (e.g. open the Store View dialog).
    pub fn handle_tap(
        &mut self,
        client: &mut ClientView,
        state: &mut AppState,
        tap: crate::Tap,
    ) -> Vec<UiEvent> {
        self.with_client(client, state, |app, state| {
            // The line a tap makes is its window's, like a window bar's (`Self`).
            state.line_owner = tap
                .window
                .zip(state.current_live_screen)
                .map(|(no, screen)| crate::core::self_ref::LineOwner::Window { screen, no });
            app.handle_tap(state, &tap);
            state.line_owner = None;
            app.take_ui_events(state)
        })
    }

    /// Render the session at its own size with its own view (CLI clients).
    pub fn render(&mut self, state: &mut AppState) -> RenderedShellFrame {
        let mut client = ClientView::default();
        self.render_client(&mut client, state, None)
            .pop()
            .map(|(_, f)| f)
            .unwrap_or(RenderedShellFrame {
                ansi: String::new(),
            })
    }

    /// Stacked rendering: one frame per viewport of `client` (or a single
    /// frame at `size` / the session size when it has none), all composited
    /// from the same shared buffers. The focused viewport is drawn last so
    /// size-dependent state (page size) follows it.
    pub fn render_client(
        &mut self,
        client: &mut ClientView,
        state: &mut AppState,
        size: Option<(u16, u16)>,
    ) -> Vec<(Option<u32>, RenderedShellFrame)> {
        let (width, height) = size.unwrap_or((self.width, self.height));
        let jobs: Vec<(Option<u32>, u16, u16)> = if client.viewports.is_empty() {
            vec![(None, width, height)]
        } else {
            client
                .viewports
                .iter()
                .rev()
                .map(|v| (Some(v.view), v.width, v.height))
                .collect()
        };
        self.with_client(client, state, |app, state| {
            if !state.watches().is_empty() {
                app.refresh_watches(state);
            } else if !app.watch_output.is_empty() {
                app.watch_output.clear();
            }
            // NAV's tree and inspector follow the show (another station's edit lights up).
            app.nav_refresh(state);
        });
        let mut frames = Vec::with_capacity(jobs.len());
        for (id, w, h) in jobs {
            let mut view = match id {
                Some(id) => client.views.remove(&id).unwrap_or_default(),
                None => std::mem::replace(&mut client.main, ViewState::focused()),
            };
            let frame = self.with_client(client, state, |app, state| {
                let backend = ratatui::backend::TestBackend::new(w.max(20), h.max(1));
                let mut terminal = Terminal::new(backend).expect("test backend");
                let _ = terminal.draw(|frame| app.compose(frame, state, &mut view));
                let buffer = terminal.backend().buffer().clone();
                RenderedShellFrame {
                    ansi: buffer_to_ansi(&buffer),
                }
            });
            match id {
                Some(id) => {
                    client.views.insert(id, view);
                }
                None => client.main = view,
            }
            frames.push((id, frame));
        }
        frames
    }

    /// Status for the client's command line bar (the status icons): only
    /// what is active is meant to be shown.
    pub fn status(&mut self, client: &mut ClientView, state: &mut AppState) -> lumacore_wire::Status {
        use crate::core::app_state::ShortcutMode;
        self.with_client(client, state, |_, state| {
            let user = state.effective_user_name().to_string();
            let programmer = state.current_programmer();
            let walk = programmer.filter(|p| p.is_walking()).map(|p| {
                let total = p.selection_pool.as_ref().map_or(0, |pool| pool.len());
                [p.walk_index + 1, total]
            });
            let claims = state
                .claims_in_order()
                .into_iter()
                .map(|(object, owner, editor)| lumacore_wire::ClaimStatus {
                    object,
                    mine: owner == user,
                    owner,
                    editor,
                })
                .collect();
            // The station's questions for this client (decisions §30 "12b"):
            // its lines' confirms, claim asks to its user, the merge decision.
            let (dialogs, dialogs_closed) = crate::core::dialogs::for_client(
                state,
                &crate::core::dialogs::Who::current(state),
            );
            lumacore_wire::Status {
                mode: match state.shortcut_mode {
                    ShortcutMode::Keyboard => "KBD",
                    ShortcutMode::Programmer => "PRG",
                    ShortcutMode::Navigator => "NAV",
                }
                .to_string(),
                // Station statuses (BO, PARK, sACN, WATCH, DEBUG) are feedback
                // status events (`feedback::publish_statuses`); this snapshot
                // keeps what differs per user / client.
                walk,
                // This user's mode objects that are on (decisions §16).
                modes: state
                    .root
                    .runtime
                    .user_ref(&user)
                    .map(|u| u.modes.on().map(|m| m.name().to_string()).collect())
                    .unwrap_or_default(),
                selected: programmer.map_or(0, |p| p.selected_fixtures.len()),
                claims,
                show: lumacore_cmd_showfile::show_info(state),
                // LumaSession health and a decision to make (§21 "6c").
                lumasession: crate::core::session_link::status_json(state),
                device: crate::core::devices::status(state),
                dialogs,
                dialogs_closed,
                suggestions: lumacore_suggest::mode(state).label().to_string(),
                hold_follows: state
                    .root
                    .show_data
                    .profile_for_user(&user)
                    .is_none_or(|p| p.settings.preferences.hold_follows),
                attributes_waiting: state
                    .root
                    .show_data
                    .profile_for_user(&user)
                    .filter(|p| !p.locked)
                    .map_or(0, |p| p.content.attributes.waiting().len() as u32),
                strip: strip(state, &user),
                user,
            }
        })
    }

    /// Add text to the session's output, as if a command had printed it (a
    /// reply that arrives later from the master — decisions §21 "6b").
    pub fn push_output(&mut self, text: &str) {
        self.app.add_output(text);
    }

    pub fn is_running(&self) -> bool {
        self.app.running
    }

    pub fn exit_code(&self) -> i32 {
        self.app.exit_code
    }

    pub fn take_detach_request(&mut self) -> bool {
        let requested = self.app.detach_requested;
        self.app.detach_requested = false;
        requested
    }

    pub fn prepare_for_attach(&mut self) {
        self.app.detach_requested = false;
    }

    /// Run `f` with the session's state as `client` sees it: acting user,
    /// navigation path, its screen, focus and active view swapped in.
    fn with_client<R>(
        &mut self,
        client: &mut ClientView,
        state: &mut AppState,
        f: impl FnOnce(&mut TuiApp, &mut AppState) -> R,
    ) -> R {
        let previous_navigation_path = state.navigation_path.clone();
        let previous_context = state.context.clone();
        let previous_acting_user = state.acting_user.clone();
        let previous_live_screen = state.current_live_screen;
        let previous_ui_client = state.ui_client;
        let previous_remote_client = state.remote_client;
        let previous_device = state.current_device.take();
        let previous_client = state.current_client.take();
        let previous_window = state.current_window;
        let previous_reply_to = std::mem::replace(
            &mut state.reply_to,
            crate::core::session_link::ReplyTo::Shell(self.user.clone()),
        );

        let _ = state.begin_acting_as_existing_user(&self.user);
        state.navigation_path = self.navigation_path.clone();
        state.context = self.context.clone();
        state.current_live_screen = client.live_screen;
        state.ui_client =
            client.ui || client.live_screen.is_some() || !client.viewports.is_empty();
        state.current_window = client.window;
        state.remote_client = client.remote;
        state.current_device = client.device.clone();
        state.current_client = client.client;
        state.ui_requests.clear();
        state.pending_user_switch = None;
        self.app.set_focus(client.focus.clone());
        let active = std::mem::take(client.active_mut());
        let parked = std::mem::replace(&mut self.app.view, active);

        let result = f(&mut self.app, state);
        if let Some(user) = state.pending_user_switch.take() {
            client.switch_user = Some(user);
        }
        if self.app.detach_requested {
            self.app.detach_requested = false;
            client.detach = true;
        }

        *client.active_mut() = std::mem::replace(&mut self.app.view, parked);
        client.focus = self.app.focus.clone();
        self.navigation_path = state.navigation_path.clone();
        self.context = state.context.clone();

        state.navigation_path = previous_navigation_path;
        state.context = previous_context;
        state.acting_user = previous_acting_user;
        state.current_live_screen = previous_live_screen;
        state.ui_client = previous_ui_client;
        state.remote_client = previous_remote_client;
        state.current_device = previous_device;
        state.current_client = previous_client;
        state.current_window = previous_window;
        state.reply_to = previous_reply_to;
        state.ui_requests.clear();

        result
    }
}

/// The top bar's facts for `user` (decisions top-bar-status-strip): the station, the user's
/// profile, DataPool and selected sequence, and how many steps their Oops can undo.
fn strip(state: &AppState, user: &str) -> lumacore_wire::StripStatus {
    let show = &state.root.show_data;
    let named = |no: u32, name: Option<&str>| match name.filter(|n| !n.is_empty()) {
        Some(n) => format!("{no} {n}"),
        None => no.to_string(),
    };
    let dp = lumacore_state::pools::user_datapool(show, user);
    lumacore_wire::StripStatus {
        station: state.root.station_settings.station_name.clone(),
        profile: show.profile_for_user(user).and_then(|p| p.meta.name()).unwrap_or_default().to_string(),
        datapool: named(dp, show.datapools.get(&dp).and_then(|d| d.meta.name())),
        sequence: show
            .user(user)
            .and_then(|u| u.selected.sequence)
            .and_then(|g| show.sequence_by_guid(g))
            .map(|(_, no, s)| named(no, s.meta.name()))
            .unwrap_or_default(),
        oops: state.undo_stack(user).map_or(0, |s| s.frames.len() as u32),
    }
}

fn buffer_to_ansi(buffer: &ratatui::buffer::Buffer) -> String {
    // Clear before each frame so terminal shrink does not leave stale cells behind.
    let mut out = String::from("\x1b[?25l\x1b[2J");
    let area = buffer.area;

    for y in 0..area.height {
        out.push_str(&format!("\x1b[{};1H", y + 1));
        let mut current_fg = None;
        let mut current_bg = None;
        let mut current_mod = Modifier::empty();

        for x in 0..area.width {
            let cell = &buffer[(x, y)];
            if current_fg != Some(cell.fg)
                || current_bg != Some(cell.bg)
                || current_mod != cell.modifier
            {
                out.push_str("\x1b[0m");
                out.push_str(&ansi_style(cell.fg, cell.bg, cell.modifier));
                current_fg = Some(cell.fg);
                current_bg = Some(cell.bg);
                current_mod = cell.modifier;
            }
            out.push_str(cell.symbol());
        }

        out.push_str("\x1b[0m");
    }

    out.push_str("\x1b[0m");
    out.push_str("\x1b[H\x1b[?25h");
    out
}

fn ansi_style(fg: Color, bg: Color, modifier: Modifier) -> String {
    let mut parts: Vec<String> = Vec::new();

    if modifier.contains(Modifier::BOLD) {
        parts.push("1".to_string());
    }
    if modifier.contains(Modifier::DIM) {
        parts.push("2".to_string());
    }
    if modifier.contains(Modifier::ITALIC) {
        parts.push("3".to_string());
    }
    if modifier.contains(Modifier::UNDERLINED) {
        parts.push("4".to_string());
    }
    if modifier.contains(Modifier::REVERSED) {
        parts.push("7".to_string());
    }

    if let Some(code) = ansi_color_code(fg, false) {
        parts.push(code);
    }
    if let Some(code) = ansi_color_code(bg, true) {
        parts.push(code);
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!("\x1b[{}m", parts.join(";"))
    }
}

fn ansi_color_code(color: Color, background: bool) -> Option<String> {
    let base = if background { 40 } else { 30 };
    let bright_base = if background { 100 } else { 90 };

    match color {
        Color::Reset => None,
        Color::Black => Some(base.to_string()),
        Color::Red => Some((base + 1).to_string()),
        Color::Green => Some((base + 2).to_string()),
        Color::Yellow => Some((base + 3).to_string()),
        Color::Blue => Some((base + 4).to_string()),
        Color::Magenta => Some((base + 5).to_string()),
        Color::Cyan => Some((base + 6).to_string()),
        Color::Gray => Some((base + 7).to_string()),
        Color::DarkGray => Some(bright_base.to_string()),
        Color::LightRed => Some((bright_base + 1).to_string()),
        Color::LightGreen => Some((bright_base + 2).to_string()),
        Color::LightYellow => Some((bright_base + 3).to_string()),
        Color::LightBlue => Some((bright_base + 4).to_string()),
        Color::LightMagenta => Some((bright_base + 5).to_string()),
        Color::LightCyan => Some((bright_base + 6).to_string()),
        Color::White => Some((bright_base + 7).to_string()),
        Color::Indexed(n) => Some(format!("{};5;{}", if background { 48 } else { 38 }, n)),
        Color::Rgb(r, g, b) => Some(format!(
            "{};2;{};{};{}",
            if background { 48 } else { 38 },
            r,
            g,
            b
        )),
    }
}

/// The key a client's event presses (decisions 0295): with Alt or Ctrl held, the key where it sits
/// (`base`) when that one is bound — a Mac's Option+A types `å`, Option+E is a dead key — else
/// the key as typed, so Option still types its characters and AltGr its signs.
pub(super) fn pressed(app: &mut TuiApp, state: &AppState, key: &RemoteShellKey) -> crossterm::event::KeyEvent {
    use lumacore_wire::shell::modifiers as wire;
    let typed = super::tui_types::to_crossterm(key);
    if let Some(b) = key.base
        && key.modifiers & (wire::ALT | wire::CTRL) != 0
    {
        let placed = crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Char(b), typed.modifiers);
        if app.bound_key(state, &placed).is_some() {
            return placed;
        }
    }
    typed
}
