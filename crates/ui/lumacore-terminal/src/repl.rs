// ██████╗ ███████╗██████╗ ██╗
// ██╔══██╗██╔════╝██╔══██╗██║
// ██████╔╝█████╗  ██████╔╝██║
// ██╔══██╗██╔══╝  ██╔═══╝ ██║
// ██║  ██║███████╗██║     ███████╗
// ╚═╝  ╚═╝╚══════╝╚═╝     ╚══════╝
// FILE: repl

//! Ratatui-based TUI for LumaCore terminal.
//!
//! Full-screen alternate-screen UI with three zones:
//!   - Path bar (top): context path, claims, status indicators
//!   - Output area (middle): scrollable command output + watch panels
//!   - Input line (bottom): prompt + text input with tab completion
//!
//! Replaces the previous rustyline-based REPL.
//!
//! Owns:          TuiApp, the shell loop, the remote and local clients' frames
//! Doesn't own:   drawing (tui_draw), keys (tui_input), commands (tui_commands)
//! Entry points:  TuiApp, run_tui, ClientView, RenderedShellFrame
//! Decisions:     §23 "Step 8", §30 "12b"

pub mod clipboard;
mod command_line;
mod tui_commands;
mod tui_dialog;
mod tui_draw;
mod tui_input;
mod tui_look;
mod tui_nav;
mod tui_types;

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::*;
use tui_textarea::{CursorMove, TextArea};

use crate::core::session_log::SessionLog;
use crate::core::AppState;

// Re-export public items so external paths remain unchanged.
pub use command_line::{ClientView, RemoteCommandLine};
use tui_types::{CompletionState, DialogUi};
pub use tui_types::{Layers, RenderedShellFrame, ViewState, parse_key_name};
use lumacore_wire::{REMOTE_CMD_PORT, UiEvent, UiFocus};
use crate::core::navigation::Navigation;

// *************************************************************************************************
// TUI Application
// *************************************************************************************************

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One gesture under way: its log line is written at the end.
struct GestureLog {
    label: String,
    from: String,
    to: String,
    /// An error was shown already (once per gesture).
    told: bool,
}

struct TuiApp {
    /// Input text area widget.
    textarea: TextArea<'static>,
    /// Output line buffer (ANSI strings, one per line).
    output: Vec<String>,
    /// Command history.
    history: Vec<String>,
    /// Current position in history (None = new input).
    history_idx: Option<usize>,
    /// Saved input when browsing history.
    saved_input: String,
    /// Session log writer.
    session_log: Option<SessionLog>,
    /// Whether the app is still running.
    running: bool,
    /// Exit code to return.
    exit_code: i32,
    /// Active tab completion state.
    completion: Option<CompletionState>,
    /// Confirmation dialog state.
    /// This terminal's place in the open dialog (decisions §30 "12b").
    dialog_ui: Option<DialogUi>,
    /// Keyboard bindings (key → UiKey id), rebuilt when the show changes.
    keys: KeyCache,
    /// Gestures under way (decisions §30 "12f"): what moves, from where, to where.
    gestures: std::collections::HashMap<String, GestureLog>,
    /// Watch output cache (refreshed periodically).
    watch_output: Vec<String>,
    /// UDP command bridge for external tools.
    remote_listener: Option<RemoteCommandListener>,
    /// Last shortcut key + timestamp for double-tap detection.
    last_shortcut: Option<(char, std::time::Instant)>,
    /// Navigation path history for Alt-Left/Alt-Right (cd history).
    nav_history: Vec<Vec<String>>,
    /// Current position in nav_history.
    nav_history_idx: Option<usize>,
    /// Undo stack for command-line edits.
    input_undo: Vec<String>,
    /// Redo stack for command-line edits.
    input_redo: Vec<String>,
    /// Last input snapshot (to detect changes for undo).
    last_input_snapshot: String,
    /// History search prefix (for substring matching with Up/Down).
    history_search_prefix: Option<String>,
    /// Whether the shell requested a detach without destroying session state.
    detach_requested: bool,
    /// NAV mode: the tree, the inspector, marks, filters and the gesture log (decisions 0291).
    nav: tui_nav::Nav,
    /// The view key handling acts on (scroll, page size): the local terminal,
    /// or the focused terminal window of a remote client.
    view: ViewState,
    /// Where navigation keys go (terminal or a UI panel).
    focus: UiFocus,
    /// Navigation events routed to the focused panel, drained by the session.
    ui_outbox: Vec<UiEvent>,
    /// Backspace timing for repeat / hold detection.
    backspace: tui_input::BackspaceTracker,
}

struct RemoteCommandListener {
    socket: UdpSocket,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl RemoteCommandListener {
    fn bind() -> Option<Self> {
        let socket = match UdpSocket::bind(("0.0.0.0", REMOTE_CMD_PORT)) {
            Ok(socket) => socket,
            Err(err) => {
                log::warn!("remote command listener unavailable on UDP {REMOTE_CMD_PORT}: {err}");
                return None;
            }
        };
        if let Err(err) = socket.set_nonblocking(true) {
            log::warn!("remote command listener could not enter nonblocking mode: {err}");
            return None;
        }
        Some(Self { socket })
    }

    fn drain(&self) -> Vec<(tui_commands::RemoteCommandPacket, SocketAddr)> {
        let mut packets = Vec::new();
        let mut buf = [0u8; 4096];

        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    match serde_json::from_slice::<tui_commands::RemoteCommandPacket>(&buf[..len]) {
                        Ok(packet) => packets.push((packet, addr)),
                        Err(err) => self.send_reply(addr, false, format!("invalid packet: {err}")),
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => {
                    log::warn!("remote command receive error: {err}");
                    break;
                }
            }
        }

        packets
    }

    fn send_reply(&self, addr: SocketAddr, ok: bool, message: String) {
        let reply = tui_commands::RemoteCommandReply { ok, message };
        match serde_json::to_vec(&reply) {
            Ok(bytes) => {
                let _ = self.socket.send_to(&bytes, addr);
            }
            Err(err) => log::warn!("remote command reply serialization failed: {err}"),
        }
    }
}

fn make_textarea(initial: &str) -> TextArea<'static> {
    let mut ta = TextArea::new(vec![initial.to_string()]);
    ta.set_cursor_line_style(Style::default());
    ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    ta.set_block(Block::default());
    if !initial.is_empty() {
        ta.move_cursor(CursorMove::End);
    }
    ta
}

/// The keyboard layer of decisions §17 (key → UiKey), resolved for the
/// acting user: debug > profile > vanilla. Rebuilt whenever the show, the
/// user or debug mode changes, so `Bind` takes effect at once.
pub(super) struct KeyCache {
    stamp: Option<(u64, String, bool)>,
    map: HashMap<(KeyCode, u8), String>,
    /// The held modifier acting as Meow (profile `meow_key`).
    meow: KeyModifiers,
    /// Last bound key pressed, for DoublePress timing.
    pub(super) last: Option<(String, std::time::Instant)>,
}

impl Default for KeyCache {
    fn default() -> Self {
        Self {
            stamp: None,
            map: HashMap::new(),
            meow: KeyModifiers::ALT,
            last: None,
        }
    }
}

impl KeyCache {
    fn refresh(&mut self, state: &AppState) {
        let stamp = (
            state.show_generation,
            state.effective_user_name().to_string(),
            state.debug_mode,
        );
        if self.stamp.as_ref() == Some(&stamp) {
            return;
        }
        self.map = lumacore_cmd_keys::bindings(state)
            .iter()
            .filter_map(|(name, b)| {
                parse_key_name(name).map(|(code, mods)| ((code, mods.bits()), b.ui_key.clone()))
            })
            .collect();
        self.meow = match state
            .root
            .show_data
            .profile_for_user(&stamp.1)
            .map(|p| p.settings.preferences.meow_key.to_ascii_lowercase())
            .as_deref()
        {
            Some("alt") => KeyModifiers::ALT,
            Some("ctrl") => KeyModifiers::CONTROL,
            _ => KeyModifiers::SHIFT,
        };
        self.stamp = Some(stamp);
    }
}

impl TuiApp {
    /// The UiKey a key event is bound to, and whether Meow was held: an exact
    /// binding wins (`Alt-S`); otherwise the key without the Meow modifier.
    pub(super) fn bound_key(
        &mut self,
        state: &AppState,
        key: &crossterm::event::KeyEvent,
    ) -> Option<(String, bool)> {
        self.keys.refresh(state);
        if let Some(id) = self.keys.map.get(&(key.code, key.modifiers.bits())) {
            return Some((id.clone(), false));
        }
        let meow = self.keys.meow;
        if !key.modifiers.contains(meow) {
            return None;
        }
        self.keys
            .map
            .get(&(key.code, (key.modifiers - meow).bits()))
            .map(|id| (id.clone(), true))
    }

    fn new(state: &AppState) -> Self {
        let mut app = Self {
            textarea: make_textarea(""),
            output: Vec::new(),
            history: Vec::new(),
            history_idx: None,
            saved_input: String::new(),
            session_log: SessionLog::open(),
            running: true,
            exit_code: 0,
            completion: None,
            dialog_ui: None,
            keys: KeyCache::default(),
            gestures: std::collections::HashMap::new(),
            watch_output: Vec::new(),
            remote_listener: RemoteCommandListener::bind(),
            last_shortcut: None,
            nav_history: vec![Vec::new()], // start with empty path
            nav_history_idx: Some(0),
            input_undo: Vec::new(),
            input_redo: Vec::new(),
            last_input_snapshot: String::new(),
            history_search_prefix: None,
            detach_requested: false,
            nav: tui_nav::Nav::default(),
            view: ViewState::focused(),
            focus: UiFocus::Terminal,
            ui_outbox: Vec::new(),
            backspace: tui_input::BackspaceTracker::default(),
        };

        // Startup output
        app.add_output(&lumacore_cmd_station::splash());
        if state.debug_mode {
            app.add_output(
                &nu_ansi_term::Color::Yellow
                    .bold()
                    .paint("  *** DEBUG MODE ENABLED ***")
                    .to_string(),
            );
        }
        app.add_output(&greeting_string());

        app
    }

    fn run(&mut self, terminal: &mut ratatui::DefaultTerminal, state: &mut AppState) -> i32 {
        loop {
            self.handle_remote_commands(state);

            // Refresh watches before drawing
            if !state.watches().is_empty() {
                self.refresh_watches(state);
            } else if !self.watch_output.is_empty() {
                self.watch_output.clear();
            }

            // Draw (NAV's tree and inspector follow the show first)
            self.nav_refresh(state);
            let _ = terminal.draw(|frame| self.draw(frame, state));

            // Calculate poll timeout
            let timeout = if state.watches().is_empty() {
                Duration::from_millis(100)
            } else {
                let min_ms = state
                    .watches()
                    .iter()
                    .map(|w| w.interval_ms)
                    .min()
                    .unwrap_or(200);
                Duration::from_millis(min_ms as u64)
            };

            // Poll for events -- drain ALL pending events, deduplicate rapid repeats
            if event::poll(timeout).unwrap_or(false) {
                // Collect all pending events
                let mut events: Vec<Event> = Vec::new();
                if let Ok(evt) = event::read() {
                    events.push(evt);
                }
                while event::poll(Duration::ZERO).unwrap_or(false) {
                    if let Ok(evt) = event::read() {
                        events.push(evt);
                    }
                }

                // Deduplicate: collapse consecutive identical key presses that
                // map to the same keybinding command (e.g. rapid Delete -> "reset").
                // Keep at most 1 of each consecutive run of identical bound keys.
                let mut deduped: Vec<Event> = Vec::with_capacity(events.len());
                for evt in events {
                    let dominated = if let Event::Key(key) = &evt {
                        if key.kind == KeyEventKind::Press {
                            if let Some(prev) = deduped.last() {
                                prev == &evt && self.bound_key(state, key).is_some()
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    // Also deduplicate bare Delete key (hardcoded reset)
                    let dominated = dominated || {
                        if let Event::Key(key) = &evt {
                            key.kind == KeyEventKind::Press
                                && key.code == KeyCode::Delete
                                && key.modifiers == KeyModifiers::NONE
                                && (deduped.last() == Some(&evt))
                        } else {
                            false
                        }
                    };
                    if !dominated {
                        deduped.push(evt);
                    }
                }

                for evt in deduped {
                    if !self.running || self.detach_requested {
                        break;
                    }
                    self.handle_event(evt, state);
                }
            }

            // This console is its own client: a copy goes to its terminal as OSC 52.
            let said = clipboard::write_to_own_terminal([&mut self.ui_outbox, &mut state.ui_requests], &mut std::io::stdout());
            for line in said {
                if state.shortcut_mode == crate::core::app_state::ShortcutMode::Navigator {
                    self.nav.say(line);
                } else {
                    self.add_output(&line);
                }
            }

            if !self.running || self.detach_requested {
                break;
            }
        }

        self.exit_code
    }
}

// *************************************************************************************************
// Helper functions
// *************************************************************************************************
/// Extract the next word from a ghost hint string (up to and including trailing space or separator).
fn next_word_from_hint(hint: &str) -> &str {
    let trimmed = hint.trim_start();
    let leading = hint.len() - trimmed.len();

    if trimmed.is_empty() {
        return hint; // just whitespace -- accept it all
    }

    // Find the end of the next word
    let word_end = trimmed
        .find(|c: char| c.is_whitespace() || c == '.')
        .map(|pos| pos + 1) // include the separator
        .unwrap_or(trimmed.len());

    &hint[..leading + word_end]
}

// *************************************************************************************************
// Entry point
// *************************************************************************************************
/// Run the interactive TUI REPL until the user exits.
///
/// Returns an exit code: 0 for normal exit.
pub fn run_repl(state: &mut AppState) -> i32 {
    // Install panic hook to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original_hook(info);
    }));

    let mut terminal = ratatui::init();
    let mut app = TuiApp::new(state);
    let code = app.run(&mut terminal, state);
    ratatui::restore();

    code
}

fn greeting_string() -> String {
    format!(
        "{}\n  =^..^=  Meow! Ready to light things up.\n  Type {} for available commands.",
        nu_ansi_term::Style::new().bold().paint(
            "╔══════════════════════════════════════╗\n\
             ║  LumaCore v0.1.0                     ║\n\
             ║  Terminal-first DMX lighting console ║\n\
             ╚══════════════════════════════════════╝"
        ),
        nu_ansi_term::Color::Cyan.paint("help")
    )
}

/// The prompt for a command line `width` columns wide (decisions §30 "12e
/// hardening"): the first form that leaves typing room (12 columns, or half
/// the line when it is wider) — `[user@host] scope> `, `scope> `,
/// `‹end of scope> ` (`‹` = cut, as in the line), `> `. The `>` is never cut.
pub(super) fn fit_prompt(user: &str, host: &str, scope: &str, width: u16) -> String {
    let w = width as usize;
    let room = w.saturating_sub(12.min(w).max(w / 2));
    let full = format!("[{user}@{host}] {scope}> ");
    if full.chars().count() <= room {
        return full;
    }
    let short = format!("{scope}> ");
    if short.chars().count() <= room {
        return short;
    }
    if room >= 6 {
        let keep = room - 3;
        let skip = scope.chars().count().saturating_sub(keep);
        let tail: String = scope.chars().skip(skip).collect();
        return format!("‹{tail}> ");
    }
    "> ".to_string()
}

/// The prompt of this shell's line on a command line `width` wide.
pub(super) fn prompt_for(state: &AppState, width: u16) -> String {
    let host = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| state.root.station_settings.station_name.clone());
    // Where you are and your default object (decisions §33 21; `User [Fixture] >`).
    let scope = format!("{} [{}]", state.scope_label(), crate::core::objects::default_object(state).keyword);
    fit_prompt(state.effective_user_name(), &host, &scope, width)
}

/// The first character of a line `len` characters long that shows in `w`
/// columns with the cursor at `cursor`: the line scrolls sideways and keeps
/// the cursor a few columns from the right edge (readline's
/// horizontal-scroll-mode, on by itself for a one-row terminal; §30 "12e
/// hardening"). One column past the end is room for the cursor.
pub(super) fn input_offset(len: usize, cursor: usize, w: usize) -> usize {
    let total = len + 1;
    if w == 0 || total <= w {
        return 0;
    }
    let margin = 4.min((w - 1) / 3);
    cursor.saturating_sub(w - 1 - margin).min(total - w)
}

fn farewell_string() -> String {
    "  =^..^=  Prrrrr... shutting down. Good night!".to_string()
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
