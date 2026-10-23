// ██████╗ ██████╗  █████╗ ██╗    ██╗
// ██╔══██╗██╔══██╗██╔══██╗██║    ██║
// ██║  ██║██████╔╝███████║██║ █╗ ██║
// ██║  ██║██╔══██╗██╔══██║██║███╗██║
// ██████╔╝██║  ██║██║  ██║╚███╔███╔╝
// ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝ ╚══╝╚══╝
// FILE: draw

//! TUI drawing/rendering methods for `TuiApp`.
//!
//! Owns:          TuiApp's frame: path bar, output, watches, input line
//! Doesn't own:   colours (tui_look)
//! Entry points:  TuiApp::draw
//! Decisions:     §23 "Step 8", §30 "12b"

use ratatui::prelude::*;

use super::tui_look::role;
use ratatui::widgets::*;

use super::tui_types::ViewState;
use super::tui_types::classify_verb;
use super::TuiApp;
use crate::core::AppState;
use crate::core::claims::Claims;
use crate::core::navigation::Navigation;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl TuiApp {
    // ── Drawing ──────────────────────────────────────────────────────

    /// Draw the full terminal with the session's own view (local terminal,
    /// tests, a remote client without viewports).
    pub(super) fn draw(&mut self, frame: &mut Frame, state: &AppState) {
        let mut view = std::mem::take(&mut self.view);
        self.compose(frame, state, &mut view);
        self.view = view;
    }

    /// Composite the session's shared buffers into `frame` for one view.
    ///
    /// Layers stack top to bottom: watch pane, output, status bar, path bar,
    /// command line — each only if the view shows it. Overlays (suggestion
    /// menu, NAV popup) go on top, only in a view with `overlays`. Updates the
    /// view's scroll anchor and output height.
    /// Then the frame is put into the client's theme and an open dialog of
    /// the station is drawn over it (decisions §30 "12b").
    pub(super) fn compose(&self, frame: &mut Frame, state: &AppState, view: &mut ViewState) {
        self.compose_layers(frame, state, view);
        let l = view.layers;
        let bar = l.input && !(l.output || l.watch || l.status || l.path);
        let ground = if bar { super::tui_look::Ground::Bar } else { super::tui_look::Ground::Terminal };
        super::tui_look::paint(frame.buffer_mut(), state, ground);
        let area = frame.area();
        if view.overlays && area.height >= 6 && area.width >= 20 {
            self.draw_dialog(frame, area, state);
        }
    }

    fn compose_layers(&self, frame: &mut Frame, state: &AppState, view: &mut ViewState) {
        #[derive(Clone, Copy, PartialEq)]
        enum Layer {
            Watch,
            Output,
            Status,
            Path,
            Input,
        }

        let area = frame.area();
        let layers = view.layers;
        // A view of only the command line (the bar) is never blanked: it
        // shrinks its prompt and scrolls instead (§30 "12e hardening").
        let input_only = !(layers.watch || layers.output || layers.status || layers.path);
        if area.height < 1 || (area.width < 20 && !input_only) {
            frame.render_widget(Paragraph::new("Terminal too small"), area);
            return;
        }

        let mut stack: Vec<(Layer, Constraint)> = Vec::new();
        if layers.watch && !self.watch_output.is_empty() {
            // Watch lines + separator, capped at half the height so the
            // output always has room.
            let lines: usize = self
                .watch_output
                .iter()
                .map(|s| s.lines().count().max(1))
                .sum();
            let max_watch = (area.height as usize / 2).max(3);
            stack.push((
                Layer::Watch,
                Constraint::Length(((lines + 1) as u16).min(max_watch as u16)),
            ));
        }
        if layers.output {
            stack.push((Layer::Output, Constraint::Fill(1)));
        }
        for (on, layer) in [
            (layers.status, Layer::Status),
            (layers.path, Layer::Path),
            (layers.input, Layer::Input),
        ] {
            if on {
                stack.push((layer, Constraint::Length(1)));
            }
        }

        let areas = Layout::vertical(stack.iter().map(|(_, c)| *c)).split(area);
        let mut input_area = None;
        for ((layer, _), rect) in stack.iter().zip(areas.iter().copied()) {
            match layer {
                Layer::Watch => self.draw_watch_pane(frame, rect),
                // NAV draws its tree and inspector where the output goes (decisions 0291).
                Layer::Output if state.shortcut_mode == crate::core::app_state::ShortcutMode::Navigator => {
                    view.output_height = rect.height;
                    self.draw_nav(frame, rect, state);
                }
                Layer::Output => self.draw_history(frame, rect, view),
                Layer::Status => self.draw_status_bar(frame, rect, state),
                Layer::Path => self.draw_path_bar(frame, rect, state),
                Layer::Input => {
                    // A command-line-only view (the screen's bar) always shows
                    // the ghost hint: it is where you look while typing.
                    let input_only = !layers.output && !layers.status && !layers.path;
                    self.draw_input(frame, rect, state, view.overlays || input_only);
                    input_area = Some(rect);
                }
            }
        }

        // Popups need room; a one-line view (the command line bar) has none.
        if !view.overlays || area.height < 6 {
            return;
        }
        if let Some(input_area) = input_area
            && self.completion_menu_visible()
        {
            self.draw_completion_menu(frame, area, input_area, state);
        }
    }

    /// Lines PageUp / PageDown scroll in the current view.
    pub(super) fn page_size(&self) -> usize {
        (self.view.output_height as usize).saturating_sub(1).max(1)
    }

    fn draw_path_bar(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let path = state.path_string();
        let dim = Style::default().add_modifier(Modifier::DIM);
        let cyan = Style::default().fg(role::ACCENT);
        let orange = Style::default().fg(role::ACCENT_BRIGHT);
        let green = Style::default().fg(role::OK);
        let red = Style::default().fg(role::ALERT);
        let magenta = Style::default().fg(role::PROGRAMMER);

        let mut spans: Vec<Span> = vec![Span::styled("───", dim), Span::styled(path, cyan)];
        let effective_user = state.effective_user_name();

        // Programmer is the user's permanent workspace and should always be visible
        // in the claim strip even though it is not stored in the normal claim map.
        spans.push(Span::raw(" "));
        spans.push(Span::styled("[Programmer]", orange));

        if let Some(programmer) = state.current_programmer() {
            let selected = programmer.selected_fixtures.len();
            if selected > 0 {
                let label = if selected == 1 {
                    "[1 Fixture]".to_string()
                } else {
                    format!("[{selected} Fixtures]")
                };
                spans.push(Span::raw(" "));
                spans.push(Span::styled(label, orange));
            }
        }

        for (obj, owner, _) in state.claims_in_order() {
            spans.push(Span::raw(" "));
            if owner == effective_user {
                spans.push(Span::styled(format!("[{obj}]"), orange));
            } else {
                spans.push(Span::styled(format!("[{obj}:{owner}]"), dim));
            }
        }

        for (protocol, lines) in &state.protocols_running {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(format!("{protocol}:{lines}"), green));
        }

        if !state.watches().is_empty() {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("WATCH:{}", state.watches().len()),
                magenta,
            ));
        }

        if state.debug_mode {
            spans.push(Span::raw(" "));
            spans.push(Span::styled("DEBUG", red));
        }

        spans.push(Span::styled("───", dim));

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    /// Draw the sticky watch pane (top of screen, never scrolls away).
    fn draw_watch_pane(&self, frame: &mut Frame, area: Rect) {
        use ansi_to_tui::IntoText;

        let mut combined = String::new();
        for line_str in &self.watch_output {
            combined.push_str(line_str);
            combined.push('\n');
        }
        combined.push_str("\x1b[2m─── watch ───\x1b[0m");

        let text = match combined.as_str().into_text() {
            Ok(t) => t,
            Err(_) => Text::raw(&combined),
        };

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }

    /// Draw the scrollable command history output.
    fn draw_history(&self, frame: &mut Frame, area: Rect, view: &mut ViewState) {
        use ansi_to_tui::IntoText;

        let mut combined = String::new();
        for line_str in &self.output {
            combined.push_str(line_str);
            combined.push('\n');
        }

        let text = match combined.as_str().into_text() {
            Ok(t) => t,
            Err(_) => Text::raw(&combined),
        };

        // Snap to bottom unless the view is scrolled up. A scrolled-up view
        // keeps showing the same lines while new output arrives below.
        let total_lines = text.lines.len();
        if view.scroll > 0 && total_lines > view.seen_lines {
            view.scroll += total_lines - view.seen_lines;
        }
        view.seen_lines = total_lines;
        let visible_height = area.height as usize;
        view.output_height = area.height;
        let max_scroll = total_lines.saturating_sub(visible_height);
        view.scroll = view.scroll.min(max_scroll);
        let scroll_from_top = max_scroll - view.scroll;

        let paragraph = Paragraph::new(text).scroll((scroll_from_top as u16, 0));
        frame.render_widget(paragraph, area);
    }

    /// The command line. `overlays` = draw the ghost hint (focused view only).
    fn draw_input(&self, frame: &mut Frame, area: Rect, state: &AppState, overlays: bool) {
        // The prompt shrinks on a narrow line (scope from the tree path).
        let prompt = super::prompt_for(state, area.width);
        let prompt_width = prompt.chars().count() as u16;
        let layout =
            Layout::horizontal([Constraint::Length(prompt_width), Constraint::Fill(1)])
                .split(area);

        // Render prompt (transient: dim for old prompts is handled in output, current is normal)
        frame.render_widget(Paragraph::new(prompt.as_str()), layout[0]);

        // The line scrolls sideways to keep the cursor in view.
        let input = self.get_input();
        let (_, col) = self.textarea.cursor();
        let shown = layout[1].width as usize;
        let len = input.chars().count();
        let off = super::input_offset(len, col, shown);
        // In KBD mode the textarea widget draws its native cursor while the
        // line fits; otherwise syntax-highlighted spans, windowed.
        if input.is_empty()
            || (off == 0
                && len < shown
                && state.shortcut_mode == crate::core::app_state::ShortcutMode::Keyboard)
        {
            frame.render_widget(&self.textarea, layout[1]);
        } else {
            let spans = self.syntax_highlight(&input);
            let line = window_spans(&spans, off, shown, col, len);
            frame.render_widget(Paragraph::new(Line::from(line)), layout[1]);
        }

        // Ghost hint
        let cursor_col = col - off;
        if overlays
            && self.completion.is_none()
            && let Some(hint) = self.ghost(state)
        {
            let hint_x = layout[1].x + cursor_col as u16;
            let hint_width = (layout[1].right().saturating_sub(hint_x)) as usize;
            if hint_width > 0 {
                let truncated: String = hint.chars().take(hint_width).collect();
                let hint_area = Rect::new(
                    layout[1].x + cursor_col as u16,
                    layout[1].y,
                    truncated.len() as u16,
                    1,
                );
                let dim = Style::default().add_modifier(Modifier::DIM);
                frame.render_widget(Paragraph::new(Span::styled(truncated, dim)), hint_area);
            }
        }

        frame.set_cursor_position((layout[1].x + cursor_col as u16, layout[1].y));
    }

    /// The suggestion list (decisions §30 "12b2"): the same items as the
    /// page's list above the command line bar, in the user's theme (roles).
    fn draw_completion_menu(
        &self,
        frame: &mut Frame,
        full_area: Rect,
        input_area: Rect,
        state: &AppState,
    ) {
        let Some(comp) = &self.completion else {
            return;
        };
        if comp.items.is_empty() {
            return;
        }

        let popup = self.completion_popup_area(full_area, input_area, state);
        if popup.height < 3 || popup.width < 8 {
            return;
        }

        let visible_rows = popup.height.saturating_sub(2) as usize;
        let start = comp.index.saturating_sub(visible_rows.saturating_sub(1));
        let end = (start + visible_rows).min(comp.items.len());
        let inner = popup.width.saturating_sub(2) as usize;
        let rows: Vec<ListItem> = comp.items[start..end]
            .iter()
            .enumerate()
            .map(|(offset, item)| {
                let is_selected = start + offset == comp.index;
                let off = item.disabled.is_some()
                    || item.status == lumacore_wire::ItemStatus::Planned;
                let (fg, bg) = match (is_selected, off) {
                    (true, _) => (role::GROUND, Some(role::ACCENT)),
                    (false, true) => (role::DIM, None),
                    (false, false) => (role::STRONG, None),
                };
                let base = |s: Style| match bg {
                    Some(bg) => s.bg(bg),
                    None => s,
                };
                let glyph_fg = if is_selected { role::GROUND } else { kind_role(item.kind) };
                let title: String = item.title.chars().take(24).collect();
                let room = inner.saturating_sub(4 + 24 + 2);
                let subtitle: String = item.subtitle.chars().take(room).collect();
                let mut title_style = base(Style::default().fg(fg));
                if is_selected {
                    title_style = title_style.add_modifier(Modifier::BOLD);
                }
                let sub_fg = if is_selected { role::GROUND } else { role::DIM };
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", kind_glyph(item.kind)), base(Style::default().fg(glyph_fg))),
                    Span::styled(format!("{title:<24}"), title_style),
                    Span::styled(format!("  {subtitle:<room$}"), base(Style::default().fg(sub_fg))),
                ]))
            })
            .collect();

        let title = format!(
            " Suggestions {} of {} · Tab takes · Enter runs the line ",
            comp.index + 1,
            comp.items.len()
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(role::ACCENT));
        frame.render_widget(Clear, popup);
        frame.render_widget(List::new(rows).block(block), popup);
    }

    /// Syntax-highlight the input line (verb coloring).
    pub(super) fn syntax_highlight<'a>(&self, input: &'a str) -> Vec<Span<'a>> {
        if input.is_empty() {
            return vec![Span::raw("")];
        }

        // Split into verb and rest
        let trimmed = input.trim_start();
        let leading_spaces = input.len() - trimmed.len();

        let first_space = trimmed.find(' ');
        let (verb, rest) = match first_space {
            Some(pos) => (&trimmed[..pos], &trimmed[pos..]),
            None => (trimmed, ""),
        };

        let mut spans = Vec::new();

        // Leading whitespace
        if leading_spaces > 0 {
            spans.push(Span::raw(&input[..leading_spaces]));
        }

        // Verb with color
        let verb_style = classify_verb(verb);
        spans.push(Span::styled(verb, verb_style));

        // Rest of line -- default color, but highlight numbers
        if !rest.is_empty() {
            spans.push(Span::styled(rest, Style::default().fg(role::STRONG)));
        }

        spans
    }

    fn draw_status_bar(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let dim = Style::default().add_modifier(Modifier::DIM);
        let mut spans: Vec<Span> = Vec::new();

        // Shortcut mode indicator
        use crate::core::app_state::ShortcutMode;
        let (mode_label, mode_style) = match state.shortcut_mode {
            ShortcutMode::Keyboard => ("KBD", Style::default().add_modifier(Modifier::DIM)),
            ShortcutMode::Programmer => (
                "PRG",
                Style::default()
                    .fg(role::OK)
                    .add_modifier(Modifier::BOLD),
            ),
            ShortcutMode::Navigator => (
                "NAV",
                Style::default()
                    .fg(role::INFO)
                    .add_modifier(Modifier::BOLD),
            ),
        };
        spans.push(Span::styled(mode_label, mode_style));

        // This user's modes, and blackout (grand master off)
        let modes = state
            .root
            .runtime
            .user_ref(state.effective_user_name())
            .map(|u| u.modes)
            .unwrap_or_default();
        for (on, label) in [
            (modes.highlight, "HI"),
            (modes.solo, "SOLO"),
            (modes.blind, "BLIND"),
            (modes.freeze, "FRZ"),
        ] {
            if on {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    label,
                    Style::default()
                        .fg(role::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }
        if !state.root.show_data.playbacks.grand().1 {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                "BO",
                Style::default().fg(role::ALERT).add_modifier(Modifier::BOLD),
            ));
        }

        // Walk indicator
        if let Some(prog) = state.current_programmer()
            && prog.is_walking()
        {
            let idx = prog.walk_index + 1;
            let total = prog.selection_pool.as_ref().map(|p| p.len()).unwrap_or(0);
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("WALK:{idx}/{total}"),
                Style::default().fg(role::ACCENT),
            ));
        }

        // Fill remaining width with dim separator
        let used: usize = spans.iter().map(|s| s.content.len()).sum();
        let remaining = (area.width as usize).saturating_sub(used);
        if remaining > 0 {
            spans.push(Span::styled(" ".repeat(remaining), dim));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }
}

/// A one-cell sign of an item's kind (the page uses the kit's icons).
fn kind_glyph(kind: lumacore_wire::ItemKind) -> char {
    use lumacore_wire::ItemKind as K;
    match kind {
        K::History => '↺',
        K::Verb => '▶',
        K::Keyword => '⌘',
        K::Object => '◆',
        K::Value => '#',
        K::TreeNode => '▸',
        K::Flag => '/',
    }
}

/// The theme role an item kind's sign is drawn in (as the mockup's colours).
fn kind_role(kind: lumacore_wire::ItemKind) -> Color {
    use lumacore_wire::ItemKind as K;
    match kind {
        K::History => role::DIM,
        K::Verb => role::ACCENT,
        K::Keyword => role::INFO,
        K::Object => role::OK,
        K::Value | K::TreeNode | K::Flag => role::STRONG,
    }
}

/// The part of a highlighted line that shows in `w` columns from character
/// `off`, with `‹` / `›` (dim) where text is cut off — never over the cursor.
fn window_spans(spans: &[Span<'_>], off: usize, w: usize, cursor: usize, len: usize) -> Vec<Span<'static>> {
    let mut shown: Vec<(char, Style)> = spans
        .iter()
        .flat_map(|s| s.content.chars().map(move |c| (c, s.style)))
        .skip(off)
        .take(w)
        .collect();
    let mark = Style::default().add_modifier(Modifier::DIM);
    if off > 0 && cursor != off
        && let Some(first) = shown.first_mut()
    {
        *first = ('‹', mark);
    }
    if off + w < len + 1 && cursor + 1 != off + w
        && let Some(last) = shown.last_mut()
    {
        *last = ('›', mark);
    }
    let mut out: Vec<Span<'static>> = Vec::new();
    for (c, style) in shown {
        match out.last_mut() {
            Some(s) if s.style == style => s.content.to_mut().push(c),
            _ => out.push(Span::styled(c.to_string(), style)),
        }
    }
    out
}
