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

use super::tui_types::{FieldKind, NavPopup, PopupField, try_expand_abbreviation};
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

/// A single visible entry in the NAV tree view.
pub(super) struct NavEntry {
    /// Display name ("Fixtures", "1", etc.)
    pub name: String,
    /// Full dot-path ("Root.ShowData.Devices.Dmx.Patch.Fixtures")
    pub dot_path: String,
    /// Nesting depth (0 = direct child of navigation_path)
    pub depth: usize,
    /// Whether this node has children (determines +/- vs blank)
    pub has_children: bool,
    /// Whether this node is currently expanded
    pub is_expanded: bool,
}

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
            // to the terminal, then clear the unexecuted command line.
            KeyCode::Esc => {
                if panel {
                    self.focus = UiFocus::Terminal;
                    self.ui_outbox.push(UiEvent::FocusTerminal);
                    true
                } else if self.completion.is_none() && !input_empty {
                    self.snapshot_input();
                    self.set_input("");
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

    /// Change directory from the navigator and reset its tree view.
    fn nav_cd(&mut self, state: &mut AppState, cmd: &str) {
        self.execute_command(state, cmd);
        self.nav_cursor = 0;
        self.nav_expanded.clear();
        self.nav_flat_entries.clear();
        self.render_nav_listing(state);
    }

    /// Object tap: with a pending command on the line (`Store`, `Delete`,
    /// `Label` …) the tapped object completes it and it executes
    /// (`Store` + tap → `Store ViewButton 4`); on an empty line the object's
    /// default action runs (`ViewButton 4` = call/recall). Tap-and-hold stores.
    pub(super) fn handle_tap(
        &mut self,
        state: &mut AppState,
        object: &str,
        no: u32,
        hold: bool,
        action: Option<&str>,
    ) {
        self.completion = None;
        let input = self.get_input();
        let input = input.trim_end();

        // A UI client (it has a screen) gets the Store View dialog — a
        // station ask (decisions §30 "12b") — instead of an immediate store;
        // terminal-only sessions store directly.
        use crate::core::commands::registry::{self, Word};
        let is_view = crate::core::objects::find(object).is_some_and(|o| {
            std::ptr::eq(o, &crate::core::commands::views::VIEW)
                || std::ptr::eq(o, &crate::core::commands::views::VIEW_BUTTON)
        });
        let pending_store = matches!(
            registry::resolve(input),
            Ok(Word::Command(c)) if c.object_verb == Some(crate::core::objects::Verb::Store)
        );
        let has_ui = state.current_live_screen.is_some();
        if is_view && has_ui && (hold || pending_store) {
            if !input.is_empty() {
                self.snapshot_input();
                self.set_input("");
            }
            let button =
                crate::core::objects::find(object).is_some_and(|o| std::ptr::eq(o, &crate::core::commands::views::VIEW_BUTTON));
            match crate::core::commands::views::store_ask(state, button, no) {
                Ok(ask) => {
                    crate::core::dialogs::open(state, ask);
                }
                Err(e) => self.add_output(&crate::core::errors::render_code(&e, "E-CMD-010")),
            }
            return;
        }

        // A pending Move / Copy / Assign takes the tapped object, then
        // waits for its target (`Move Group 3` + tap 7 → `… At 7`), the usual console behaviour.
        let first = input.split_whitespace().next().unwrap_or("");
        let wants_target = matches!(
            registry::resolve(first),
            Ok(Word::Command(c)) if matches!(
                c.object_verb,
                Some(crate::core::objects::Verb::Move
                    | crate::core::objects::Verb::Copy
                    | crate::core::objects::Verb::Assign)
            )
        );
        let words = input.split_whitespace().count();
        let has_at = input.split_whitespace().any(|w| w.eq_ignore_ascii_case("at"));
        if hold {
            self.run_ui_command(state, &format!("store {object} {no}"));
        } else if input.is_empty() {
            let line = match action {
                None => format!("{object} {no}"),
                Some(template) => template.replace("{obj}", &format!("{object} {no}")),
            };
            if !line.trim().is_empty() {
                self.run_ui_command(state, &line);
            }
        } else if wants_target && words == 1 {
            self.snapshot_input();
            self.set_input(&format!("{input} {object} {no}"));
        } else if wants_target && !has_at {
            // A target of another kind names itself (`Assign Sequence 3` + a view button tap →
            // `… At ViewButton 6`, decisions 0287); the same kind is a bare number.
            let pending = input.split_whitespace().nth(1).and_then(crate::core::objects::find);
            let tapped = crate::core::objects::find(object);
            let target = match (pending, tapped) {
                (Some(p), Some(t)) if !std::ptr::eq(p, t) => format!("{} {no}", t.keyword),
                _ => no.to_string(),
            };
            self.snapshot_input();
            self.set_input(&format!("{input} At {target}"));
            self.submit_command(state);
        } else {
            self.snapshot_input();
            self.set_input(&format!("{input} {object} {no}"));
            self.submit_command(state);
        }
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

        // NAV popup intercept: all keys go to popup handler when open
        if self.nav_popup.is_some() {
            self.handle_popup_key(key, state);
            return;
        }

        if self.route_key(key, repeat, state) {
            return;
        }

        // Keyboard bindings first: key → UiKey (§17).
        if let Some((id, meow)) = self.bound_key(state, &key)
        {
            self.press_bound_key(state, &id, meow);
            return;
        }

        // Programmer mode: object/noun keys append to command line (work with any input)
        if state.shortcut_mode == ShortcutMode::Programmer
            && let KeyCode::Char(ch) = key.code
            && (key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT)
        {
            let now = std::time::Instant::now();
            let is_double = self
                .last_shortcut
                .map(|(prev_ch, prev_time)| {
                    prev_ch == ch.to_ascii_lowercase()
                        && now.duration_since(prev_time).as_millis() < 500
                })
                .unwrap_or(false);

            let handled = match ch.to_ascii_lowercase() {
                // ── Object/noun keys: append to command line ──
                'a' => {
                    if is_double {
                        self.last_shortcut = None;
                        self.execute_command(state, "at 100");
                    } else {
                        self.last_shortcut = Some(('a', now));
                        self.append_keyword("at");
                    }
                    true
                }
                't' => {
                    self.append_keyword("thru");
                    true
                }
                'f' => {
                    self.append_keyword("fixture");
                    true
                }
                'g' => {
                    self.append_keyword("group");
                    true
                }
                's' => {
                    self.append_keyword("store");
                    true
                }
                'u' => {
                    self.append_keyword("update");
                    true
                }
                'o' => {
                    self.append_keyword("off");
                    true
                }
                'p' => {
                    self.append_keyword("preset");
                    true
                }
                'e' => {
                    self.append_keyword("edit");
                    true
                }
                'i' => {
                    self.append_keyword("if");
                    true
                }
                // ── Action keys: execute immediately (only on empty input) ──
                'h' if self.get_input().is_empty() => {
                    self.execute_command(state, "highlight");
                    true
                }
                'b' if self.get_input().is_empty() => {
                    self.execute_command(state, "blind");
                    true
                }
                'l' if self.get_input().is_empty() => {
                    self.execute_command(state, "list");
                    true
                }
                _ => false,
            };

            if handled {
                return;
            }
        }

        // Navigator mode: tree browsing when input is empty
        if state.shortcut_mode == ShortcutMode::Navigator
            && self.get_input().is_empty()
        {
            // Navigator arrow keys + Enter + Delete (tree view)
            match key.code {
                KeyCode::Up if key.modifiers == KeyModifiers::NONE => {
                    self.rebuild_nav_flat_entries(state);
                    let count = self.nav_flat_entries.len();
                    if count > 0 {
                        if self.nav_cursor == 0 {
                            self.nav_cursor = count - 1;
                        } else {
                            self.nav_cursor -= 1;
                        }
                        self.render_nav_listing(state);
                    }
                    return;
                }
                KeyCode::Down if key.modifiers == KeyModifiers::NONE => {
                    self.rebuild_nav_flat_entries(state);
                    let count = self.nav_flat_entries.len();
                    if count > 0 {
                        self.nav_cursor = (self.nav_cursor + 1) % count;
                        self.render_nav_listing(state);
                    }
                    return;
                }
                KeyCode::Right if key.modifiers == KeyModifiers::NONE => {
                    self.rebuild_nav_flat_entries(state);
                    if let Some(entry) = self.nav_flat_entries.get(self.nav_cursor)
                        && entry.has_children
                        && !entry.is_expanded
                    {
                        let dp = entry.dot_path.clone();
                        self.nav_expanded.insert(dp);
                        self.render_nav_listing(state);
                    }
                    return;
                }
                KeyCode::Left if key.modifiers == KeyModifiers::NONE => {
                    self.rebuild_nav_flat_entries(state);
                    let action = self.nav_flat_entries.get(self.nav_cursor).map(|entry| {
                        if entry.has_children && entry.is_expanded {
                            LeftAction::Collapse(entry.dot_path.clone())
                        } else if entry.depth > 0 {
                            LeftAction::JumpToParent(entry.depth)
                        } else {
                            LeftAction::CdUp
                        }
                    });
                    match action {
                        Some(LeftAction::Collapse(dp)) => {
                            let prefix = format!("{}.", dp);
                            self.nav_expanded.retain(|p| !p.starts_with(&prefix));
                            self.nav_expanded.remove(&dp);
                            self.render_nav_listing(state);
                        }
                        Some(LeftAction::JumpToParent(cur_depth)) => {
                            // Scan backwards for first entry at depth - 1
                            for i in (0..self.nav_cursor).rev() {
                                if self.nav_flat_entries[i].depth == cur_depth - 1 {
                                    self.nav_cursor = i;
                                    break;
                                }
                            }
                            self.render_nav_listing(state);
                        }
                        Some(LeftAction::CdUp) => self.nav_cd(state, "cd .."),
                        None => {}
                    }
                    return;
                }
                KeyCode::Enter => {
                    self.rebuild_nav_flat_entries(state);
                    if !self.nav_flat_entries.is_empty() {
                        self.nav_popup = self.build_nav_popup(state);
                    }
                    return;
                }
                KeyCode::Delete if key.modifiers == KeyModifiers::NONE => {
                    if let Some(name) = self.nav_selected_child(state) {
                        if Self::is_structural_object(&name) {
                            self.add_output(&format!("Cannot delete structural object '{name}'"));
                        } else {
                            self.set_input(&format!("delete {name}"));
                        }
                    }
                    return;
                }
                _ => {}
            }
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

    // ── Navigator mode helpers ──────────────────────────────────────

    /// Build the base dot-path for the current navigation path.
    fn nav_base_dot_path(&self, state: &AppState) -> String {
        crate::core::tree_path::dot(&state.navigation_path)
    }

    /// Check whether a node at the given dot-path has any children.
    fn node_has_children(dot_path: &str, state: &AppState) -> bool {
        use crate::core::{nav_children_at, vfs};
        if !nav_children_at(dot_path).is_empty() {
            return true;
        }
        if let Some(dyn_children) = vfs::dynamic_children_at(dot_path, &state.root)
            && !dyn_children.is_empty()
        {
            return true;
        }
        false
    }

    /// Recursively collect children into `nav_flat_entries`.
    fn collect_children(&mut self, parent_dot_path: &str, depth: usize, state: &AppState) {
        use crate::core::{nav_children_at, vfs};

        let static_children = nav_children_at(parent_dot_path);
        for name in &static_children {
            let dot_path = format!("{}.{}", parent_dot_path, name);
            let has_children = Self::node_has_children(&dot_path, state);
            let is_expanded = has_children && self.nav_expanded.contains(&dot_path);
            self.nav_flat_entries.push(NavEntry {
                name: name.clone(),
                dot_path: dot_path.clone(),
                depth,
                has_children,
                is_expanded,
            });
            if is_expanded {
                self.collect_children(&dot_path, depth + 1, state);
            }
        }

        if let Some(dyn_children) = vfs::dynamic_children_at(parent_dot_path, &state.root) {
            for dc in dyn_children {
                let clean_name = dc
                    .display_name
                    .split("  ")
                    .next()
                    .unwrap_or(&dc.display_name)
                    .to_string();
                let dot_path = format!("{}.{}", parent_dot_path, clean_name);
                let has_children = Self::node_has_children(&dot_path, state);
                let is_expanded = has_children && self.nav_expanded.contains(&dot_path);
                self.nav_flat_entries.push(NavEntry {
                    name: clean_name,
                    dot_path: dot_path.clone(),
                    depth,
                    has_children,
                    is_expanded,
                });
                if is_expanded {
                    self.collect_children(&dot_path, depth + 1, state);
                }
            }
        }
    }

    /// Rebuild the flattened entry list from the tree state.
    fn rebuild_nav_flat_entries(&mut self, state: &AppState) {
        self.nav_flat_entries.clear();
        let base = self.nav_base_dot_path(state);
        self.collect_children(&base, 0, state);
    }

    /// Get the name of the currently selected child (if any).
    fn nav_selected_child(&mut self, state: &AppState) -> Option<String> {
        self.rebuild_nav_flat_entries(state);
        self.nav_flat_entries
            .get(self.nav_cursor)
            .map(|e| e.name.clone())
    }

    /// Render the navigator tree listing to output.
    pub(super) fn render_nav_listing(&mut self, state: &AppState) {
        self.output.clear();
        self.view.scroll = 0;
        self.rebuild_nav_flat_entries(state);

        if self.nav_flat_entries.is_empty() {
            self.add_output("  (empty)");
            return;
        }

        // Clamp cursor
        if self.nav_cursor >= self.nav_flat_entries.len() {
            self.nav_cursor = self.nav_flat_entries.len() - 1;
        }

        let path = state.path_string();
        let dim = nu_ansi_term::Style::new().dimmed();
        let cyan = nu_ansi_term::Color::Cyan;
        let mut lines = vec![format!("{}", dim.paint(format!("  {path}/")))];

        for (i, entry) in self.nav_flat_entries.iter().enumerate() {
            let cursor = if i == self.nav_cursor { "►" } else { " " };
            let indent = "  ".repeat(entry.depth);
            let indicator = if entry.has_children && entry.is_expanded {
                "- "
            } else if entry.has_children {
                "+ "
            } else {
                "  "
            };
            let styled_name = if i == self.nav_cursor {
                cyan.bold().paint(entry.name.as_str()).to_string()
            } else {
                entry.name.clone()
            };
            lines.push(format!("  {cursor} {indent}{indicator}{styled_name}"));
        }
        self.add_output(&lines.join("\n"));

        // Auto-center scroll so cursor stays visible
        let total_lines = self.output.len();
        let cursor_line = 1 + self.nav_cursor; // 1 for header
        let visible = self.view.output_height as usize;
        if total_lines > visible {
            let lines_below = total_lines.saturating_sub(cursor_line + 1);
            self.view.scroll = lines_below.saturating_sub(visible / 2);
        }
    }

    /// Append a keyword to the command line with correct spacing.
    fn append_keyword(&mut self, keyword: &str) {
        let current = self.get_input();
        let sep = if current.is_empty() || current.ends_with(' ') {
            ""
        } else {
            " "
        };
        self.set_input(&format!("{current}{sep}{keyword} "));
    }

    // ── NAV popup ─────────────────────────────────────────────────────

    /// Build a property popup for the currently selected NAV entry.
    fn build_nav_popup(&self, state: &AppState) -> Option<NavPopup> {
        use crate::core::vfs;

        let entry = self.nav_flat_entries.get(self.nav_cursor)?;
        let dot_path = entry.dot_path.clone();
        let node = vfs::resolve_node(&dot_path, &state.root);

        let mut fields = Vec::new();
        let title;

        match node {
            Some(vfs::NodeView::Object {
                json,
                node_type,
                id,
                guid,
                name,
            }) => {
                title = name.clone().unwrap_or_else(|| format!("{node_type} {id}"));
                // Add metadata fields
                fields.push(PopupField {
                    label: "Type".to_string(),
                    value: node_type.to_string(),
                    kind: FieldKind::ReadOnly,
                });
                fields.push(PopupField {
                    label: "ID".to_string(),
                    value: id.to_string(),
                    kind: FieldKind::ReadOnly,
                });
                fields.push(PopupField {
                    label: "GUID".to_string(),
                    value: guid,
                    kind: FieldKind::ReadOnly,
                });
                if let Some(n) = &name {
                    fields.push(PopupField {
                        label: "Name".to_string(),
                        value: n.clone(),
                        kind: FieldKind::ReadOnly,
                    });
                }
                // Extract JSON top-level keys
                if let serde_json::Value::Object(map) = &json {
                    for (k, v) in map {
                        // Skip fields we already show
                        if matches!(k.as_str(), "id" | "guid" | "name") {
                            continue;
                        }
                        let value_str = match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Null => "(null)".to_string(),
                            other => other.to_string(),
                        };
                        fields.push(PopupField {
                            label: k.clone(),
                            value: value_str,
                            kind: FieldKind::ReadOnly,
                        });
                    }
                }
            }
            Some(vfs::NodeView::Structural { name, children }) => {
                title = name;
                for child in &children {
                    fields.push(PopupField {
                        label: child.clone(),
                        value: String::new(),
                        kind: FieldKind::ReadOnly,
                    });
                }
            }
            Some(vfs::NodeView::FileInfo {
                name,
                size,
                is_dir,
                extension,
                ..
            }) => {
                title = name;
                fields.push(PopupField {
                    label: "Size".to_string(),
                    value: format!("{size} bytes"),
                    kind: FieldKind::ReadOnly,
                });
                fields.push(PopupField {
                    label: "Directory".to_string(),
                    value: is_dir.to_string(),
                    kind: FieldKind::ReadOnly,
                });
                if let Some(ext) = extension {
                    fields.push(PopupField {
                        label: "Extension".to_string(),
                        value: ext,
                        kind: FieldKind::ReadOnly,
                    });
                }
            }
            None => {
                title = entry.name.clone();
                fields.push(PopupField {
                    label: "Path".to_string(),
                    value: dot_path.clone(),
                    kind: FieldKind::ReadOnly,
                });
            }
        }

        // Enrich known editable fields
        self.enrich_editable_fields(&dot_path, &mut fields, state);

        if fields.is_empty() {
            fields.push(PopupField {
                label: "Info".to_string(),
                value: "(no properties)".to_string(),
                kind: FieldKind::ReadOnly,
            });
        }

        Some(NavPopup {
            title,
            fields,
            cursor: 0,
            scroll: 0,
            editing: None,
        })
    }

    /// Map known dot-paths to editable FieldKinds.
    fn enrich_editable_fields(&self, dot_path: &str, fields: &mut [PopupField], _state: &AppState) {
        let path_lower = dot_path.to_lowercase();

        // StationSettings fields
        if path_lower.ends_with("stationsettings") {
            for field in fields.iter_mut() {
                match field.label.as_str() {
                    "station_name" => {
                        field.kind = FieldKind::Text {
                            cmd_prefix: "Set Station \"Name\"".to_string(),
                        };
                    }
                    "station_priority" => {
                        field.kind = FieldKind::Enum {
                            cmd_prefix: "Set Station \"Priority\"".to_string(),
                            options: lumacore_cmd_station::PRIORITY_CHOICES.iter().map(|c| c.to_string()).collect(),
                            current_index: lumacore_cmd_station::PRIORITY_CHOICES
                                .iter()
                                .position(|c| c.eq_ignore_ascii_case(field.value.trim_matches('"')))
                                .unwrap_or(3),
                        };
                    }
                    _ => {}
                }
            }
        }

        // UserProfile fields
        if path_lower.contains("userprofiles.") {
            // The profile's number, from the path (`…UserProfiles.2`).
            let profile = path_lower.rsplit("userprofiles.").next().and_then(|r| r.split('.').next()).unwrap_or("").to_string();
            for field in fields.iter_mut() {
                match field.label.as_str() {
                    "clear_buffer" => {
                        let current = field.value.contains("true");
                        field.kind = FieldKind::Toggle {
                            cmd_prefix: format!("Set UserProfile {profile} \"ClearBuffer\""),
                            current,
                        };
                    }
                    "show_descriptions" => {
                        let current = field.value.contains("true");
                        field.kind = FieldKind::Toggle {
                            cmd_prefix: format!("Set UserProfile {profile} \"ShowDescriptions\""),
                            current,
                        };
                    }
                    "verbosity" => {
                        let current_index = match field.value.trim_matches('"') {
                            "Verbose" => 1,
                            "Debug" => 2,
                            _ => 0,
                        };
                        field.kind = FieldKind::Enum {
                            cmd_prefix: format!("Set UserProfile {profile} \"Verbosity\""),
                            options: vec![
                                "Summary".to_string(),
                                "Verbose".to_string(),
                                "Debug".to_string(),
                            ],
                            current_index,
                        };
                    }
                    _ => {}
                }
            }
        }
    }

    /// Handle keyboard input when the NAV popup is open.
    fn handle_popup_key(&mut self, key: CtKeyEvent, state: &mut AppState) {
        match key.code {
            KeyCode::Esc => {
                if self.nav_popup.as_ref().is_some_and(|p| p.editing.is_some()) {
                    // Cancel editing
                    if let Some(popup) = &mut self.nav_popup {
                        popup.editing = None;
                    }
                } else {
                    // Close popup
                    self.nav_popup = None;
                }
            }
            KeyCode::Up => {
                if let Some(popup) = &mut self.nav_popup
                    && popup.cursor > 0
                {
                    popup.cursor -= 1;
                    if popup.cursor < popup.scroll {
                        popup.scroll = popup.cursor;
                    }
                }
            }
            KeyCode::Down => {
                if let Some(popup) = &mut self.nav_popup
                    && popup.cursor + 1 < popup.fields.len()
                {
                    popup.cursor += 1;
                    // Scroll if cursor goes below visible area (approx 20 rows visible)
                    let visible = 20usize;
                    if popup.cursor >= popup.scroll + visible {
                        popup.scroll = popup.cursor.saturating_sub(visible - 1);
                    }
                }
            }
            KeyCode::Enter => {
                // Extract what we need before taking nav_popup
                let action = self.nav_popup.as_ref().and_then(|popup| {
                    let field = popup.fields.get(popup.cursor)?;
                    match &field.kind {
                        FieldKind::ReadOnly => None,
                        FieldKind::Text { cmd_prefix } => {
                            if let Some(editing) = popup.editing.as_ref() {
                                // Confirm edit
                                // A text value is quoted (decisions §15 "Quoting").
                                let val = editing.replace('"', "");
                                Some(PopupAction::Execute(format!("{cmd_prefix} \"{val}\"")))
                            } else {
                                // Start editing
                                Some(PopupAction::StartEdit(field.value.clone()))
                            }
                        }
                        FieldKind::Toggle {
                            cmd_prefix,
                            current,
                        } => {
                            let new_val = if *current { "Off" } else { "On" };
                            Some(PopupAction::Execute(format!("{cmd_prefix} {new_val}")))
                        }
                        FieldKind::Enum {
                            cmd_prefix,
                            options,
                            current_index,
                        } => {
                            let next = (current_index + 1) % options.len();
                            let val = &options[next];
                            Some(PopupAction::Execute(format!("{cmd_prefix} {val}")))
                        }
                    }
                });

                match action {
                    Some(PopupAction::Execute(cmd)) => {
                        let saved_cursor = self.nav_popup.as_ref().map(|p| p.cursor).unwrap_or(0);
                        self.nav_popup = None;
                        self.execute_command(state, &cmd);
                        // Rebuild popup at same cursor position
                        self.rebuild_nav_flat_entries(state);
                        if let Some(mut popup) = self.build_nav_popup(state) {
                            popup.cursor = saved_cursor.min(popup.fields.len().saturating_sub(1));
                            self.nav_popup = Some(popup);
                        }
                    }
                    Some(PopupAction::StartEdit(val)) => {
                        if let Some(popup) = &mut self.nav_popup {
                            popup.editing = Some(val);
                        }
                    }
                    None => {}
                }
            }
            KeyCode::Char(ch) => {
                if let Some(popup) = &mut self.nav_popup
                    && let Some(editing) = &mut popup.editing
                {
                    editing.push(ch);
                }
            }
            KeyCode::Backspace => {
                if let Some(popup) = &mut self.nav_popup
                    && let Some(editing) = &mut popup.editing
                {
                    editing.pop();
                }
            }
            _ => {}
        }
    }

    /// Whether an object name is a structural/locked node that cannot be copied/moved/deleted.
    fn is_structural_object(name: &str) -> bool {
        // tidy: not-keywords (tree node names)
        matches!(
            name,
            "Root"
                | "RuntimeState"
                | "ShowData"
                | "Devices"
                | "Dmx"
                | "Metadata"
                | "Protocols"
                | "DataPools"
                | "UserProfiles"
                | "Playback"
                | "Patch"
                | "StationSettings"
                | "Drives"
        )
    }
}

/// Helper enum for Left arrow action dispatch (avoids borrow issues).
enum LeftAction {
    Collapse(String),
    JumpToParent(usize),
    CdUp,
}

/// Helper enum for popup Enter action dispatch (avoids borrow issues).
enum PopupAction {
    Execute(String),
    StartEdit(String),
}
