//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// FILE: commands

//! TUI command execution methods for `TuiApp`.
//!
//! Owns:          TuiApp's command execution
//! Doesn't own:   drawing, keys
//! Entry points:  TuiApp::execute_command
//! Decisions:     §23 "Step 8"

use ratatui::prelude::*;
use serde::{Deserialize, Serialize};

use super::tui_types::CompletionState;
use crate::core::DispatchResult;
use super::{TuiApp, farewell_string, make_textarea};
use crate::core::AppState;
use crate::core::app_state::ShortcutMode;
use crate::core::commands::dispatch;
use crate::core::claims::Claims;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

#[derive(Debug, Deserialize)]
pub(super) struct RemoteCommandPacket {
    pub(super) user: String,
    pub(super) cmd: String,
}

#[derive(Debug, Serialize)]
pub(super) struct RemoteCommandReply {
    pub(super) ok: bool,
    pub(super) message: String,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl TuiApp {
    // ── Command processing ───────────────────────────────────────────

    pub(super) fn submit_command(&mut self, state: &mut AppState) {
        let input = self.get_input().trim().to_string();
        self.textarea = make_textarea("");
        self.view.scroll = 0;
        // Reset undo state for new command
        self.input_undo.clear();
        self.input_redo.clear();
        self.last_input_snapshot = String::new();
        self.history_search_prefix = None;

        if input.is_empty() {
            return;
        }
        // Passwords (`Login "LD" "pw"`) never reach history, echo or logs.
        let shown = crate::core::passwords::mask_secrets(&input);

        // Add to history (skip for admin)
        if !state.is_admin() {
            self.history.push(shown.clone());
        }
        self.history_idx = None;

        // Log
        if let Some(log) = self.session_log.as_mut() {
            log.log_command(&shown);
        }

        // Transient prompt: add a dim copy of the command to output for scrollback
        let _hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| state.root.station_settings.station_name.clone());
        let _scope_str = state.context.to_string();
        let transient = format!(
            "{}",
            nu_ansi_term::Style::new()
                .dimmed()
                .paint(format!("> {shown}"))
        );
        self.add_output(&transient);

        self.process_commands_logged(state, &input, true);
    }

    pub(super) fn execute_command(&mut self, state: &mut AppState, cmd: &str) {
        self.process_commands_logged(state, cmd, false);
    }

    /// Run a command that came from the UI (a tap, a dialog's OK) as if it had
    /// been typed: echoed in the output, kept in history, then executed. The
    /// usual console behaviour — pop-ups and taps issue real commands that show up in the
    /// command line history. The line being typed is left alone.
    pub(super) fn run_ui_command(&mut self, state: &mut AppState, cmd: &str) {
        let cmd = cmd.trim();
        if cmd.is_empty() {
            return;
        }
        let shown = crate::core::passwords::mask_secrets(cmd);
        if !state.is_admin() {
            self.history.push(shown.clone());
        }
        self.history_idx = None;
        if let Some(log) = self.session_log.as_mut() {
            log.log_command(&shown);
        }
        self.view.scroll = 0;
        self.add_output(
            &nu_ansi_term::Style::new()
                .dimmed()
                .paint(format!("> {shown}"))
                .to_string(),
        );
        self.process_commands_logged(state, cmd, true);
    }

    /// A UI key (`menu`, `plugin.x.y`, `user.z`) reported a gesture (press,
    /// release, hold, release-after-hold, double), Meow held or not: run its
    /// command line like a UI command (echoed, logged). Results of anything
    /// but a press are reported as a notice; a press only on error. No
    /// double-press line = a second press. Decisions §17.
    pub(super) fn run_key_action(
        &mut self,
        state: &mut AppState,
        key: &str,
        gesture: &str,
        meow: bool,
    ) {
        use lumacore_wire::UiEvent;
        use lumacore_tree::ui_keys::Gesture;

        let notice = |state: &mut AppState, text: String, error: bool| {
            state.ui_requests.push(UiEvent::Notice {
                text,
                level: if error { "error" } else { "info" }.into(),
            });
        };
        let Some(gesture) = Gesture::parse(gesture) else {
            return notice(state, format!("Unknown gesture '{gesture}'"), true);
        };
        let Some(def) = lumacore_cmd_keys::find(state, key) else {
            return notice(state, format!("No UI key '{key}'"), true);
        };
        let (gesture, cmd) = lumacore_cmd_keys::command_for(&def, gesture, meow);
        if cmd.trim().is_empty() {
            return;
        }
        if gesture == Gesture::Double && def.double_replaces_press {
            state.ui_requests.push(UiEvent::CloseMenu);
        }
        let before = self.output.len();
        // A stored line: Delete doesn't ask (R1 review).
        state.running_stored_line = true;
        self.run_ui_command(state, &cmd);
        state.running_stored_line = false;
        // The command's output (after its echo line): first line → notice.
        let first = self
            .output
            .get(before + 1..)
            .unwrap_or_default()
            .iter()
            .map(|l| crate::core::commands::sysmon::strip_ansi_for_sysmon(l))
            .find(|l| !l.trim().is_empty());
        if let Some(line) = first {
            // Success feedback comes from the command itself (e.g. SaveShow's
            // progress + "Show saved" flash); errors are reported here.
            if line.starts_with("Error:") {
                notice(
                    state,
                    line.trim_start_matches("Error:").trim().to_string(),
                    true,
                );
            }
        }
    }

    /// A keyboard key bound to UiKey `id` was pressed (decisions §17). The
    /// terminal only sees presses: a second press of the same key inside the
    /// profile's double-click time is the DoublePress. Release / Hold need the
    /// app or web UI.
    pub(super) fn press_bound_key(&mut self, state: &mut AppState, id: &str, meow: bool) {
        use lumacore_tree::ui_keys::Gesture;
        let Some(def) = lumacore_cmd_keys::find(state, id) else {
            self.output.push(crate::core::errors::render_code(
                format!("The key is bound to UIKey '{id}', which doesn't exist."),
                "E-KEY-004",
            ));
            return;
        };
        let window = state
            .root
            .show_data
            .profile_for_user(state.effective_user_name())
            .map_or(lumacore_tree::ui_keys::DEFAULT_DOUBLE_CLICK_MS, |p| {
                p.settings.preferences.double_click_ms
            });
        let now = std::time::Instant::now();
        let has_double = !def.command_for(Gesture::Double, meow).is_empty();
        let double = has_double
            && self.keys.last.as_ref().is_some_and(|(last, t)| {
                last.eq_ignore_ascii_case(id)
                    && now.duration_since(*t).as_millis() < u128::from(window)
            });
        self.keys.last = (!double).then(|| (id.to_string(), now));
        let gesture = if double {
            Gesture::Double
        } else {
            Gesture::Press
        };
        let (_, cmd) = lumacore_cmd_keys::command_for(&def, gesture, meow);
        if !cmd.trim().is_empty() {
            self.execute_command(state, &cmd);
        }
    }

    pub(super) fn execute_remote_command(
        &mut self,
        state: &mut AppState,
        user: &str,
        cmd: &str,
    ) -> RemoteCommandReply {
        let input = cmd.trim().to_string();
        if input.is_empty() {
            return RemoteCommandReply {
                ok: false,
                message: "command is empty".to_string(),
            };
        }

        let Some(acting_user) = state.begin_acting_as_existing_user(user) else {
            return RemoteCommandReply {
                ok: false,
                message: format!("unknown user '{user}'"),
            };
        };
        self.add_output(
            &nu_ansi_term::Style::new()
                .dimmed()
                .paint(format!("udp[{acting_user}]> {input}"))
                .to_string(),
        );

        let nav_before = state.navigation_path.clone();
        let commands: Vec<&str> = input.split(';').map(|s| s.trim()).collect();
        let mut outputs = Vec::new();
        let mut error = None;

        'commands: for command in commands {
            if command.is_empty() {
                continue;
            }

            let result = dispatch(state, command);
            crate::core::commands::sysmon::log_command(state, command, &result, Some("udp"));
            match result {
                DispatchResult::Output(output) => {
                    if output == "\x1B[2J\x1B[1;1H" {
                        self.output.clear();
                        let msg =
                            crate::core::text::render_success("Terminal buffer cleared");
                        self.add_output(&msg);
                        outputs.push(msg);
                    } else if !output.is_empty() {
                        if let Some(log) = self.session_log.as_mut() {
                            log.log_output(&output);
                        }
                        self.add_output(&output);
                        outputs.push(String::from(output));
                    }
                }
                DispatchResult::Exit => {
                    let farewell = farewell_string();
                    self.add_output(&farewell);
                    outputs.push(farewell);
                    self.running = false;
                    self.exit_code = 0;
                    break 'commands;
                }
                DispatchResult::Logout => {
                    self.detach_requested = true;
                    break 'commands;
                }
                DispatchResult::ContextSwitch(ctx) => {
                    state.context = ctx;
                }
                DispatchResult::Ask(ask) => {
                    error = Some(ask.refusal());
                    break 'commands;
                }
            }
        }

        if state.navigation_path != nav_before {
            self.push_nav_history(state.navigation_path.clone());
        }

        state.clear_acting_user();

        if let Some(message) = error {
            RemoteCommandReply { ok: false, message }
        } else {
            let message = outputs
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            RemoteCommandReply {
                ok: true,
                message: if message.is_empty() {
                    "command accepted".to_string()
                } else {
                    message
                },
            }
        }
    }

    /// Run `input` (`;`-separated). `log`: a command the user issued (typed
    /// or from a UI dialog) — each one gets a System Monitor `Cmd` line.
    pub(super) fn process_commands_logged(&mut self, state: &mut AppState, input: &str, log: bool) {
        // Save nav path and mode before commands (for cd history + NAV auto-render)
        let nav_before = state.navigation_path.clone();
        let mode_before = state.shortcut_mode;

        let commands: Vec<&str> = input.split(';').map(|s| s.trim()).collect();

        for (i, cmd) in commands.iter().copied().enumerate() {
            if cmd.is_empty() {
                continue;
            }

            let result = dispatch(state, cmd);
            if log {
                crate::core::commands::sysmon::log_command(state, cmd, &result, None);
            }
            match result {
                DispatchResult::Output(output) => {
                    // Detect clearterm (ANSI clear screen sequence)
                    if output == "\x1B[2J\x1B[1;1H" {
                        self.output.clear();
                        self.add_output(&crate::core::text::render_success(
                            "Terminal buffer cleared",
                        ));
                    } else if !output.is_empty() {
                        if let Some(log) = self.session_log.as_mut() {
                            log.log_output(&output);
                        }
                        self.add_output(&output);
                    }
                }
                DispatchResult::Exit => {
                    self.add_output(&farewell_string());
                    self.running = false;
                    self.exit_code = 0;
                    return;
                }
                DispatchResult::Logout => {
                    self.detach_requested = true;
                    return;
                }
                DispatchResult::ContextSwitch(ctx) => {
                    state.context = ctx;
                }
                DispatchResult::Ask(ask) => {
                    // The dialog shows it (a window here, the kit's dialog in a UI); the rest
                    // of the line runs with the answer (`Ask::then_run`, as dispatch does).
                    let rest = commands[i + 1..].iter().filter(|c| !c.is_empty()).copied().collect::<Vec<_>>().join("; ");
                    crate::core::dialogs::open(state, if rest.is_empty() { *ask } else { ask.then_run(&rest) });
                    return;
                }
            }
        }

        // Track navigation path changes for Alt-Left/Alt-Right
        if state.navigation_path != nav_before {
            self.push_nav_history(state.navigation_path.clone());
        }

        // Auto-render NAV listing when entering Navigator mode
        if state.shortcut_mode == ShortcutMode::Navigator && mode_before != ShortcutMode::Navigator
        {
            self.nav_cursor = 0;
            self.nav_expanded.clear();
            self.nav_flat_entries.clear();
            self.render_nav_listing(state);
        }
    }

    pub(super) fn add_output(&mut self, text: &str) {
        for line in text.lines() {
            self.output.push(line.to_string());
        }
        // If text ends with newline and doesn't end with double newline,
        // lines() already handles it correctly
        self.view.scroll = 0;
    }

    // ── Suggestions (lumacore_suggest) ─────────────────────────────────

    /// Suggestions for `line` at `cursor` (chars) — by default the session's
    /// own command line where it is now (decisions §30 "12b2").
    pub(super) fn suggestions(
        &self,
        state: &AppState,
        line: Option<String>,
        cursor: Option<usize>,
        surface: lumacore_wire::Surface,
    ) -> lumacore_wire::Suggestions {
        let (line, cursor) = match line {
            Some(line) => {
                let end = line.chars().count();
                (line, cursor.unwrap_or(end).min(end))
            }
            None => (self.get_input(), cursor.unwrap_or(self.textarea.cursor().1)),
        };
        lumacore_suggest::suggest(
            state,
            &lumacore_suggest::Query {
                line: &line,
                cursor,
                surface,
                history: &self.history,
                now_ms: lumacore_suggest::now_ms(),
            },
        )
    }

    /// The ghost text after the cursor (none with Suggestions Off).
    pub(super) fn ghost(&self, state: &AppState) -> Option<String> {
        if lumacore_suggest::mode(state) == lumacore_tree::SuggestionsMode::Off {
            return None;
        }
        lumacore_suggest::ghost(state, &self.get_input(), self.textarea.cursor().1, &self.history)
    }

    /// Write the command line: chars `start..end` become `text`, the cursor
    /// goes after it. Nothing when the line is no longer `expect`.
    pub(super) fn edit_line(&mut self, start: usize, end: usize, text: &str, expect: Option<&str>) {
        let input = self.get_input();
        if expect.is_some_and(|e| e != input) {
            return;
        }
        let chars: Vec<char> = input.chars().collect();
        let end = end.min(chars.len());
        let start = start.min(end);
        let before: String = chars[..start].iter().collect();
        let after: String = chars[end..].iter().collect();
        self.snapshot_input();
        self.set_input(&format!("{before}{text}{after}"));
        let col = (start + text.chars().count()) as u16;
        self.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(0, col));
        self.completion = None;
        self.history_search_prefix = None;
    }

    /// Tab: take the selected suggestion while the list is open; else one
    /// candidate is taken at once, several open the list (a UI client's own
    /// list is drawn by the page: there Tab completes the common part).
    pub(super) fn tab_complete(&mut self, state: &AppState) {
        if self.completion.as_ref().is_some_and(|c| !c.items.is_empty()) {
            self.accept_completion();
            return;
        }
        let s = self.suggestions(state, None, None, lumacore_wire::Surface::Bar);
        let items: Vec<lumacore_wire::SuggestItem> =
            s.items.into_iter().filter(|i| i.disabled.is_none()).collect();
        let replace = (s.replace[0] as usize, s.replace[1] as usize);
        match items.len() {
            0 => {}
            1 => self.edit_line(replace.0, replace.1, &items[0].insert, None),
            _ if state.ui_client => {
                let common = common_prefix(items.iter().map(|i| i.insert.as_str()));
                let typed = replace.1 - replace.0;
                if common.chars().count() > typed {
                    self.edit_line(replace.0, replace.1, &common, None);
                }
            }
            _ => {
                self.completion = Some(CompletionState {
                    items,
                    index: 0,
                    replace,
                    original_line: s.line,
                });
            }
        }
    }

    /// The live list while typing (Suggestions Auto, terminals only — a UI
    /// client draws its own above the command line bar).
    pub(super) fn refresh_live_completion(&mut self, state: &AppState) {
        let input = self.get_input();
        if input.is_empty()
            || state.ui_client
            || lumacore_suggest::mode(state) != lumacore_tree::SuggestionsMode::Auto
        {
            self.completion = None;
            return;
        }
        let s = self.suggestions(state, None, None, lumacore_wire::Surface::Bar);
        let replace = (s.replace[0] as usize, s.replace[1] as usize);
        let fragment: String = input.chars().skip(replace.0).take(replace.1 - replace.0).collect();
        if s.items.is_empty()
            || (s.items.len() == 1 && s.items[0].insert.trim_end().eq_ignore_ascii_case(&fragment))
        {
            self.completion = None;
            return;
        }
        let previous = self
            .completion
            .as_ref()
            .and_then(|c| c.items.get(c.index))
            .map(|i| i.insert.clone());
        let index = previous
            .and_then(|p| s.items.iter().position(|i| i.insert == p))
            .unwrap_or(0);
        self.completion = Some(CompletionState {
            items: s.items,
            index,
            replace,
            original_line: s.line,
        });
    }

    pub(super) fn completion_menu_visible(&self) -> bool {
        self.completion
            .as_ref()
            .map(|comp| !comp.items.is_empty())
            .unwrap_or(false)
    }

    pub(super) fn completion_popup_area(
        &self,
        full_area: Rect,
        input_area: Rect,
        state: &AppState,
    ) -> Rect {
        let prompt_width = self.prompt_width(input_area, state);
        let col = self.textarea.cursor().1;
        let shown = input_area.width.saturating_sub(prompt_width) as usize;
        let cursor_col =
            (col - super::input_offset(self.get_input().chars().count(), col, shown)) as u16;
        let anchor_x = input_area
            .x
            .saturating_add(prompt_width)
            .saturating_add(cursor_col);
        let width = full_area.width.clamp(28, 72);
        let height = self.completion_popup_height(full_area.height);
        let x = anchor_x
            .saturating_sub(2)
            .min(full_area.right().saturating_sub(width));
        let mut y = input_area.y.saturating_sub(height);
        if y < full_area.y {
            y = full_area.y;
        }
        Rect::new(x, y, width, height)
    }

    fn completion_popup_height(&self, terminal_height: u16) -> u16 {
        let Some(comp) = &self.completion else {
            return 0;
        };
        let max_rows = terminal_height.saturating_sub(6).clamp(4, 10);
        let visible = (comp.items.len() as u16).min(max_rows.saturating_sub(2));
        visible.saturating_add(2)
    }

    pub(super) fn prompt_width(&self, input_area: Rect, state: &AppState) -> u16 {
        super::prompt_for(state, input_area.width).chars().count() as u16
    }

    /// Take the selected item: its insert replaces the word it was made for.
    pub(super) fn accept_completion(&mut self) {
        let Some(comp) = self.completion.take() else {
            return;
        };
        let Some(item) = comp.items.get(comp.index) else {
            return;
        };
        if item.disabled.is_some() {
            return;
        }
        let (start, end) = comp.replace;
        let insert = item.insert.clone();
        self.edit_line(start, end, &insert, Some(&comp.original_line));
    }

    // ── Watch system ─────────────────────────────────────────────────

    pub(super) fn refresh_watches(&mut self, state: &mut AppState) {
        self.watch_output.clear();
        let watch_cmds: Vec<String> = state.watches().iter().map(|w| w.command.clone()).collect();

        // Only read-only lines run again (R1-65: a watch never changes the
        // show, files an Oops step or replicates, whatever got stored).
        for cmd in watch_cmds.iter().filter(|c| lumacore_cmd_station::watchable(c)) {
            if let DispatchResult::Output(output) = dispatch(state, cmd)
                && !output.is_empty()
            {
                for line in output.lines() {
                    self.watch_output.push(line.to_string());
                }
            }
        }
    }

    pub(super) fn handle_remote_commands(&mut self, state: &mut AppState) {
        let packets = match self.remote_listener.as_ref() {
            Some(listener) => listener.drain(),
            None => return,
        };

        for (packet, addr) in packets {
            let reply = self.execute_remote_command(state, &packet.user, &packet.cmd);
            if let Some(listener) = self.remote_listener.as_ref() {
                listener.send_reply(addr, reply.ok, reply.message);
            }
            if !self.running {
                break;
            }
        }
    }
}

/// The longest common prefix (any case) of `items`, in the first's spelling.
fn common_prefix<'a>(mut items: impl Iterator<Item = &'a str>) -> String {
    let Some(first) = items.next() else {
        return String::new();
    };
    let mut len = first.chars().count();
    for other in items {
        len = first
            .chars()
            .zip(other.chars())
            .take(len)
            .take_while(|(a, b)| a.eq_ignore_ascii_case(b))
            .count();
    }
    first.chars().take(len).collect()
}
