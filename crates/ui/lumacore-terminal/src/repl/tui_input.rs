// ██╗███╗   ██╗██████╗ ██╗   ██╗████████╗
// ██║████╗  ██║██╔══██╗██║   ██║╚══██╔══╝
// ██║██╔██╗ ██║██████╔╝██║   ██║   ██║
// ██║██║╚██╗██║██╔═══╝ ██║   ██║   ██║
// ██║██║ ╚████║██║     ╚██████╔╝   ██║
// ╚═╝╚═╝  ╚═══╝╚═╝      ╚═════╝    ╚═╝
// FILE: input

//! TUI input handling methods for `TuiApp`.
//!
//! Owns:          TuiApp's keys: editing, completion, history, core keys
//! Doesn't own:   what a key line runs (lumacore-cmd-keys)
//! Entry points:  TuiApp::handle_key
//! Decisions:     §17, §30 "12b", §30 "12b2"

use crossterm::event::{Event, KeyCode, KeyEvent as CtKeyEvent, KeyEventKind, KeyModifiers};

use std::time::{Duration, Instant};

use super::tui_types::try_expand_abbreviation;
use lumacore_wire::{MoveDir, UiEvent, UiFocus};
use super::{TuiApp, farewell_string, make_textarea, next_word_from_hint};
use crate::core::AppState;
use crate::core::app_state::ShortcutMode;
use crate::core::navigation::Navigation;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Backspace presses closer together than this count as auto-repeat when the
/// client can't say (plain terminals don't report repeat).
const REPEAT_GAP: Duration = Duration::from_millis(150);
/// A reset is ignored this soon after Backspace erased a character, so
/// erasing a typed command never overshoots into the programmer.
const ERASE_GUARD: Duration = Duration::from_millis(400);
/// Holding Backspace this long in NAV jumps to the top level.
const HOLD_TO_ROOT: Duration = Duration::from_millis(1000);

#[derive(Default)]
pub(super) struct BackspaceTracker {
    last_at: Option<Instant>,
    press_started: Option<Instant>,
    last_erase_at: Option<Instant>,
    hold_fired: bool,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl BackspaceTracker {
    /// Record a press; returns whether it is an auto-repeat.
    fn press(&mut self, repeat: Option<bool>, now: Instant) -> bool {
        let is_repeat = repeat.unwrap_or_else(|| {
            self.last_at
                .is_some_and(|t| now.duration_since(t) < REPEAT_GAP)
        });
        self.last_at = Some(now);
        if !is_repeat {
            self.press_started = Some(now);
            self.hold_fired = false;
        }
        is_repeat
    }

    fn erased_recently(&self, now: Instant) -> bool {
        self.last_erase_at
            .is_some_and(|t| now.duration_since(t) < ERASE_GUARD)
    }

    fn held_long_enough(&self, now: Instant) -> bool {
        self.press_started
            .is_some_and(|t| now.duration_since(t) >= HOLD_TO_ROOT)
    }
}

impl TuiApp {
    // ── Focus routing ────────────────────────────────────────────────

    pub(super) fn set_focus(&mut self, focus: UiFocus) {
        self.focus = focus;
    }

    /// Events for the client's UI: from key routing, plus any that commands
    /// queued on the state (e.g. `Window 5` focusing a window).
    pub(super) fn take_ui_events(&mut self, state: &mut AppState) -> Vec<UiEvent> {
        let mut events = std::mem::take(&mut self.ui_outbox);
        events.append(&mut state.ui_requests);
        events.extend(self.armed_event(state));
        events
    }

    /// Front stage of key handling: decides whether a navigation key belongs to
    /// the focused panel, and applies the per-mode Backspace / Esc rules.
    /// Returns true when the key was consumed.
    ///
    /// Typing never stops here — characters always reach the command line.
    fn route_key(&mut self, key: CtKeyEvent, repeat: Option<bool>, state: &mut AppState) -> bool {
        if key.modifiers != KeyModifiers::NONE {
            return false;
        }
        let panel = matches!(self.focus, UiFocus::Panel { .. });
        let input_empty = self.get_input().is_empty();

        match key.code {
            KeyCode::Backspace => self.route_backspace(repeat, state, panel, input_empty),
            // The chain: popups first (handled before routing), then focus back
            // to the terminal. Esc only closes; it never clears the command line
            // (an armed `Assign Tag 1 At` waits until you clear it, decisions 0297).
            KeyCode::Esc => {
                if panel {
                    self.focus = UiFocus::Terminal;
                    self.ui_outbox.push(UiEvent::FocusTerminal);
                    true
                } else {
                    false
                }
            }
            _ if !panel => false,
            KeyCode::Up => self.emit_move(MoveDir::Up),
            KeyCode::Down => self.emit_move(MoveDir::Down),
            KeyCode::Left => self.emit_move(MoveDir::Left),
            KeyCode::Right => self.emit_move(MoveDir::Right),
            KeyCode::PageUp => self.emit_move(MoveDir::PageUp),
            KeyCode::PageDown => self.emit_move(MoveDir::PageDown),
            KeyCode::Home => self.emit_move(MoveDir::Home),
            KeyCode::End => self.emit_move(MoveDir::End),
            KeyCode::Enter if input_empty => {
                self.ui_outbox
                    .push(if state.shortcut_mode == ShortcutMode::Navigator {
                        UiEvent::Edit
                    } else {
                        UiEvent::Activate
                    });
                true
            }
            // Suggestions can't be browsed while a panel owns the arrows, so
            // Enter executes what was typed instead of accepting a suggestion.
            KeyCode::Enter => {
                self.completion = None;
                false
            }
            KeyCode::Delete => {
                self.ui_outbox.push(UiEvent::Delete);
                true
            }
            _ => false,
        }
    }

    fn emit_move(&mut self, dir: MoveDir) -> bool {
        self.ui_outbox.push(UiEvent::Move { dir });
        true
    }

    /// Backspace per shortcut mode:
    /// - text on the line: erase (terminal focus) / nothing (panel focus)
    /// - KBD: plain erase, never reset
    /// - PRG: `reset` one stage per fresh press, never on auto-repeat
    /// - NAV: up one level; held → top level
    fn route_backspace(
        &mut self,
        repeat: Option<bool>,
        state: &mut AppState,
        panel: bool,
        input_empty: bool,
    ) -> bool {
        let now = Instant::now();
        let is_repeat = self.backspace.press(repeat, now);

        if !input_empty {
            if panel {
                return true;
            }
            self.backspace.last_erase_at = Some(now);
            return false; // textarea erases the character
        }

        match state.shortcut_mode {
            ShortcutMode::Keyboard => {
                if panel {
                    self.ui_outbox.push(UiEvent::Erase);
                }
            }
            ShortcutMode::Programmer => {
                if !is_repeat && !self.backspace.erased_recently(now) {
                    self.completion = None;
                    self.execute_command(state, "reset");
                }
            }
            ShortcutMode::Navigator => {
                if is_repeat {
                    if !self.backspace.hold_fired && self.backspace.held_long_enough(now) {
                        self.backspace.hold_fired = true;
                        if panel {
                            self.ui_outbox.push(UiEvent::Root);
                        } else {
                            self.nav_cd(state, "cd /");
                        }
                    }
                } else if !self.backspace.erased_recently(now) {
                    if panel {
                        self.ui_outbox.push(UiEvent::Back);
                    } else {
                        self.nav_cd(state, "cd ..");
                    }
                }
            }
        }
        true
    }

    // ── Event handling ───────────────────────────────────────────────

    pub(super) fn handle_event(&mut self, event: Event, state: &mut AppState) {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return;
            }
            self.handle_key(key, state);
        }
    }

    pub(super) fn handle_key(&mut self, key: CtKeyEvent, state: &mut AppState) {
        self.handle_key_with_repeat(key, None, state);
    }

    pub(super) fn handle_key_with_repeat(
        &mut self,
        key: CtKeyEvent,
        repeat: Option<bool>,
        state: &mut AppState,
    ) {
        // A dialog of the station for this client (a terminal: CLI or the
        // station's console) takes every key while it is open.
        if self.dialog_key(key, state) {
            return;
        }

        // In NAV your own NAV bindings come before NAV's keys (decisions 0295).
        if state.shortcut_mode == ShortcutMode::Navigator && self.bound_key_pressed(state, &key) {
            return;
        }

        // NAV's keys while its tree is up and the command line is NAV's (decisions 0291).
        if self.nav_key(key, state) {
            return;
        }

        if self.route_key(key, repeat, state) {
            return;
        }

        // Keyboard bindings: key → UiKey (§17), in the screen's shortcut mode (0295): PRG's
        // letters are LumaCore's keys there, and a key's next press cycles its word. Inside an
        // open quote a plain key types.
        if self.bound_key_pressed(state, &key) {
            return;
        }

        match (key.code, key.modifiers) {
            // Submit the line as typed (decisions §30 "12b2": Tab takes a
            // suggestion, Enter never does) -- or Meow when input is empty
            (KeyCode::Enter, _) => {
                self.completion = None;
                if self.get_input().is_empty() {
                    self.execute_command(state, "meow");
                } else {
                    // Try abbreviation expansion before submitting
                    let input = self.get_input();
                    if let Some(expanded) = try_expand_abbreviation(&input) {
                        self.set_input(&expanded);
                    }
                    self.submit_command(state);
                }
            }

            // Space -- try abbreviation expansion
            (KeyCode::Char(' '), KeyModifiers::NONE) => {
                self.completion = None;
                // Snapshot for undo before modification
                self.snapshot_input();
                self.textarea.input(key);

                // Check if the word before the space is an abbreviation
                let input = self.get_input();
                let before_space = input.trim_end();
                if let Some(expanded) = try_expand_abbreviation(before_space) {
                    self.set_input(&format!("{expanded} "));
                }
                self.refresh_live_completion(state);
            }

            // Tab: quicksheet when empty, completion when typing
            (KeyCode::Tab, _) => {
                if self.get_input().is_empty() {
                    self.execute_command(state, "quicksheet");
                } else {
                    self.tab_complete(state);
                }
            }

            // History navigation -- substring search
            (KeyCode::Up, KeyModifiers::NONE) => {
                if let Some(ref mut comp) = self.completion {
                    if comp.index == 0 {
                        comp.index = comp.items.len().saturating_sub(1);
                    } else {
                        comp.index -= 1;
                    }
                } else {
                    self.completion = None;
                    self.history_up_search();
                }
            }
            (KeyCode::Down, KeyModifiers::NONE) => {
                if let Some(ref mut comp) = self.completion {
                    comp.index = (comp.index + 1) % comp.items.len().max(1);
                } else {
                    self.completion = None;
                    self.history_down_search();
                }
            }

            // Scroll output
            (KeyCode::PageUp, _) => {
                self.view.scroll = self.view.scroll.saturating_add(self.page_size());
            }
            (KeyCode::PageDown, _) => {
                self.view.scroll = self.view.scroll.saturating_sub(self.page_size());
            }

            // Undo (Ctrl-Z)
            (KeyCode::Char('z'), KeyModifiers::CONTROL) => {
                self.undo_input();
            }

            // Redo (Ctrl-Y)
            (KeyCode::Char('y'), KeyModifiers::CONTROL) => {
                self.redo_input();
            }

            // Alt-Left: navigate back in cd history
            (KeyCode::Left, KeyModifiers::ALT) => {
                self.nav_back(state);
            }

            // Alt-Right: navigate forward in cd history / accept next word of ghost hint
            (KeyCode::Right, KeyModifiers::ALT) => {
                let input = self.get_input();
                let cursor_col = self.textarea.cursor().1;
                if cursor_col >= input.chars().count() {
                    // Cursor at end -- accept next word of ghost hint
                    if let Some(hint) = self.ghost(state) {
                        let next_word = next_word_from_hint(&hint);
                        if !next_word.is_empty() {
                            let new_input = format!("{input}{next_word}");
                            self.set_input(&new_input);
                            self.completion = None;
                            return;
                        }
                    }
                }
                // Fallback: navigate forward in cd history
                self.nav_forward(state);
            }

            // Cancel
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.completion = None;
                let input = self.get_input();
                if input.is_empty() {
                    self.add_output("(use 'shutdown' or 'turnoff' to quit LumaCore)");
                } else {
                    self.snapshot_input();
                    self.textarea = make_textarea("");
                }
            }

            // Exit
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                self.add_output(&farewell_string());
                self.running = false;
                self.exit_code = 0;
            }

            // Escape: close the suggestion list
            (KeyCode::Esc, _) => {
                self.completion = None;
            }

            // Left arrow: Previous when input is empty + Programmer mode, else normal cursor movement
            (KeyCode::Left, KeyModifiers::NONE) => {
                if state.shortcut_mode == ShortcutMode::Programmer && self.get_input().is_empty() {
                    self.execute_command(state, "previous");
                } else {
                    self.textarea.input(key);
                    self.refresh_live_completion(state);
                }
            }

            // Right arrow: Next when input is empty + Programmer mode, accept ghost hint at end, else cursor movement
            (KeyCode::Right, KeyModifiers::NONE) => {
                let input = self.get_input();
                if state.shortcut_mode == ShortcutMode::Programmer && input.is_empty() {
                    self.execute_command(state, "next");
                } else {
                    let cursor_col = self.textarea.cursor().1;
                    if cursor_col >= input.chars().count() {
                        // Cursor at end -- try to accept ghost hint (full)
                        if let Some(hint) = self.ghost(state) {
                            let new_input = format!("{input}{hint}");
                            self.set_input(&new_input);
                            self.completion = None;
                        }
                    } else {
                        // Cursor in middle -- normal right arrow movement
                        self.textarea.input(key);
                        self.refresh_live_completion(state);
                    }
                }
            }

            // All other keys: pass to textarea (with undo snapshot)
            _ => {
                self.completion = None;
                // Reset history search when user types
                if let KeyCode::Char(_) = key.code {
                    self.history_search_prefix = None;
                    self.snapshot_input();
                }
                self.textarea.input(key);
                self.refresh_live_completion(state);
            }
        }
    }

    // ── Input helpers ────────────────────────────────────────────────

    pub(super) fn get_input(&self) -> String {
        self.textarea.lines().first().cloned().unwrap_or_default()
    }

    pub(super) fn set_input(&mut self, text: &str) {
        self.textarea = make_textarea(text);
    }

    // ── Undo/Redo ────────────────────────────────────────────────────

    /// Snapshot the current input for undo (only if it changed).
    pub(super) fn snapshot_input(&mut self) {
        let current = self.get_input();
        if current != self.last_input_snapshot {
            self.input_undo.push(self.last_input_snapshot.clone());
            self.input_redo.clear();
            self.last_input_snapshot = current;
            // Cap undo stack
            if self.input_undo.len() > 100 {
                self.input_undo.remove(0);
            }
        }
    }

    fn undo_input(&mut self) {
        if let Some(prev) = self.input_undo.pop() {
            let current = self.get_input();
            self.input_redo.push(current);
            self.last_input_snapshot = prev.clone();
            self.set_input(&prev);
        }
    }

    fn redo_input(&mut self) {
        if let Some(next) = self.input_redo.pop() {
            let current = self.get_input();
            self.input_undo.push(current);
            self.last_input_snapshot = next.clone();
            self.set_input(&next);
        }
    }

    // ── History (substring search) ──────────────────────────────────

    fn history_up_search(&mut self) {
        if self.history.is_empty() {
            return;
        }

        // On first Up press, capture the current input as the search prefix
        if self.history_search_prefix.is_none() {
            let input = self.get_input();
            self.history_search_prefix = Some(input.clone());
            self.saved_input = input;
        }

        let prefix = self.history_search_prefix.clone().unwrap_or_default();
        let start = match self.history_idx {
            Some(0) => return, // at oldest
            Some(idx) => idx - 1,
            None => self.history.len() - 1,
        };

        // Search backwards for a history entry containing the prefix (substring match)
        for i in (0..=start).rev() {
            if prefix.is_empty()
                || self.history[i]
                    .to_lowercase()
                    .contains(&prefix.to_lowercase())
            {
                self.history_idx = Some(i);
                let cmd = self.history[i].clone();
                self.set_input(&cmd);
                return;
            }
        }
    }

    fn history_down_search(&mut self) {
        match self.history_idx {
            None => {} // no history active
            Some(idx) => {
                let prefix = self.history_search_prefix.clone().unwrap_or_default();

                // Search forward for next matching entry
                for i in (idx + 1)..self.history.len() {
                    if prefix.is_empty()
                        || self.history[i]
                            .to_lowercase()
                            .contains(&prefix.to_lowercase())
                    {
                        self.history_idx = Some(i);
                        let cmd = self.history[i].clone();
                        self.set_input(&cmd);
                        return;
                    }
                }

                // No more matches -- restore saved input
                self.history_idx = None;
                self.history_search_prefix = None;
                let saved = self.saved_input.clone();
                self.set_input(&saved);
            }
        }
    }

    // ── Navigation history (cd back/forward) ─────────────────────────

    pub(super) fn push_nav_history(&mut self, path: Vec<String>) {
        // Truncate any forward history
        if let Some(idx) = self.nav_history_idx {
            self.nav_history.truncate(idx + 1);
        }
        self.nav_history.push(path);
        self.nav_history_idx = Some(self.nav_history.len() - 1);

        // Cap history
        if self.nav_history.len() > 50 {
            self.nav_history.remove(0);
            self.nav_history_idx = self.nav_history_idx.map(|i| i.saturating_sub(1));
        }
    }

    fn nav_back(&mut self, state: &mut AppState) {
        if let Some(idx) = self.nav_history_idx
            && idx > 0
        {
            self.nav_history_idx = Some(idx - 1);
            state.navigation_path = self.nav_history[idx - 1].clone();
            let path = state.path_string();
            self.add_output(&format!(
                "{}",
                nu_ansi_term::Style::new()
                    .dimmed()
                    .paint(format!("cd {path}"))
            ));
        }
    }

    fn nav_forward(&mut self, state: &mut AppState) {
        if let Some(idx) = self.nav_history_idx
            && idx + 1 < self.nav_history.len()
        {
            self.nav_history_idx = Some(idx + 1);
            state.navigation_path = self.nav_history[idx + 1].clone();
            let path = state.path_string();
            self.add_output(&format!(
                "{}",
                nu_ansi_term::Style::new()
                    .dimmed()
                    .paint(format!("cd {path}"))
            ));
        }
    }

    /// Append a keyword to the command line with correct spacing.
    /// A bound key, pressed: true when it acted. A plain key inside an open quote types.
    fn bound_key_pressed(&mut self, state: &mut AppState, key: &CtKeyEvent) -> bool {
        let plain = matches!(key.code, KeyCode::Char(_)) && !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        if plain && self.get_input().matches('"').count() % 2 == 1 {
            return false;
        }
        match self.bound_key(state, key) {
            Some((id, meow)) => self.press_bound_key(state, &id, meow),
            None => false,
        }
    }
}
