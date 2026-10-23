// ██████╗ ██╗ █████╗ ██╗      ██████╗  ██████╗
// ██╔══██╗██║██╔══██╗██║     ██╔═══██╗██╔════╝
// ██║  ██║██║███████║██║     ██║   ██║██║  ███╗
// ██║  ██║██║██╔══██║██║     ██║   ██║██║   ██║
// ██████╔╝██║██║  ██║███████╗╚██████╔╝╚██████╔╝
// ╚═════╝ ╚═╝╚═╝  ╚═╝╚══════╝ ╚═════╝  ╚═════╝
// FILE: dialog

//! The station's dialog in a terminal (decisions §30 "12b"): a window in the
//! middle of the dimmed shell, drawn here for the CLI and the station's own
//! console (a UI client draws the kit's dialog from its Status instead).
//! ←/→ or Tab choose, Enter presses, 1–9 press a button, Esc = Cancel, any
//! key stops a countdown. The answer goes the same way as any client's. A
//! dialog's tree (the Update ask, decisions 0293) is an indented list whose
//! tickable rows are numbered fields (Tab to one, Space ticks it); the lines it
//! will run are listed under it as they stand.
//!
//! Owns:          the station's dialog drawn in a terminal
//! Doesn't own:   the dialog's content (core::dialogs)
//! Entry points:  draw_dialog, dialog_key
//! Decisions:     §30 "12b"

use std::collections::BTreeMap;

use crossterm::event::{KeyCode, KeyEvent as CtKeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

use lumacore_wire::{ButtonKind, Dialog, DialogAnswer, InputKind};

use super::TuiApp;
use super::tui_look::TuiLook;
use super::tui_types::DialogUi;
use crate::core::AppState;
use crate::core::dialogs::{self, Who};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

use lumacore_base::clock::now_ms;

impl DialogUi {
    fn fresh(d: &Dialog) -> Self {
        Self {
            id: d.id.clone(),
            field: if d.inputs.is_empty() { fields(d) } else { 0 },
            button: d.primary,
            values: d.inputs.iter().map(|i| i.value.clone()).collect(),
            toggles: d.toggles.iter().map(|t| t.on).collect(),
            selectors: d
                .selectors
                .iter()
                .map(|s| s.choices.iter().position(|c| *c == s.value).unwrap_or(0))
                .collect(),
            ticks: tickable(d).map(|n| n.on).collect(),
            paused: false,
        }
    }
}

/// Inputs, toggles, selectors and the tree's ticks before the buttons row.
fn fields(d: &Dialog) -> usize {
    d.inputs.len() + d.toggles.len() + d.selectors.len() + tickable(d).count()
}

/// The tree's nodes that can be ticked, in order.
fn tickable(d: &Dialog) -> impl Iterator<Item = &lumacore_wire::DialogTreeNode> {
    d.tree.iter().flat_map(|t| t.nodes.iter()).filter(|n| n.tickable)
}

/// What the answer's line would run with these ticks (the station builds the real line).
fn tree_preview(d: &Dialog, ui: &DialogUi) -> Vec<String> {
    let Some(tree) = &d.tree else { return Vec::new() };
    let ids: Vec<&str> = tickable(d).map(|n| n.id.as_str()).collect();
    let on = |id: &str| {
        if let Some(i) = ids.iter().position(|x| *x == id) {
            return ui.ticks.get(i).copied();
        }
        d.toggles.iter().position(|t| t.name == id).and_then(|i| ui.toggles.get(i).copied())
    };
    let text = |name: &str| {
        let (i, s) = d.selectors.iter().enumerate().find(|(_, s)| s.name == name)?;
        s.texts.get(ui.selectors.get(i).copied().unwrap_or(0)).cloned()
    };
    tree.statements(on, text)
        .into_iter()
        .map(|mut line| {
            for (i, s) in d.selectors.iter().enumerate() {
                let t = s.texts.get(ui.selectors.get(i).copied().unwrap_or(0)).cloned().unwrap_or_default();
                line = line.replace(&format!("{{{}}}", s.name), &t);
            }
            for (i, t) in d.toggles.iter().enumerate() {
                let flag = if ui.toggles.get(i).copied().unwrap_or(t.on) { t.flag.as_str() } else { "" };
                line = line.replace(&format!("{{{}}}", t.name), flag);
            }
            line.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .collect()
}

impl TuiApp {
    /// The dialog this terminal shows: the oldest one for this client. A UI
    /// client draws its own (from its Status), so none here.
    pub(super) fn dialog_here(state: &AppState) -> Option<Dialog> {
        if state.ui_client {
            return None;
        }
        dialogs::for_client(state, &Who::current(state))
            .0
            .into_iter()
            .next()
    }

    /// A key while a dialog is open; true when it took the key.
    pub(super) fn dialog_key(&mut self, key: CtKeyEvent, state: &mut AppState) -> bool {
        let Some(d) = Self::dialog_here(state) else {
            self.dialog_ui = None;
            return false;
        };
        if self.dialog_ui.as_ref().is_none_or(|u| u.id != d.id) {
            self.dialog_ui = Some(DialogUi::fresh(&d));
        }
        // Any key stops a running countdown (like touching the pop-up).
        let running = d.countdown.as_ref().is_some_and(|c| !c.paused);
        if running && !self.dialog_ui.as_ref().is_some_and(|u| u.paused) {
            if let Some(ui) = self.dialog_ui.as_mut() {
                ui.paused = true;
            }
            if let Some(line) = dialogs::pause(state, &Who::current(state), &d.id) {
                self.run_ui_command(state, &line);
            }
        }
        let n = fields(&d);
        let Some(ui) = self.dialog_ui.as_mut() else {
            return true;
        };
        let on_buttons = ui.field >= n;
        let input = (ui.field < d.inputs.len()).then_some(ui.field);
        let toggle = ui
            .field
            .checked_sub(d.inputs.len())
            .filter(|t| *t < d.toggles.len());
        let selector = ui
            .field
            .checked_sub(d.inputs.len() + d.toggles.len())
            .filter(|s| *s < d.selectors.len());
        let tick = ui
            .field
            .checked_sub(d.inputs.len() + d.toggles.len() + d.selectors.len())
            .filter(|t| *t < ui.ticks.len());
        let step = |from: usize, forward: bool| {
            let len = d.buttons.len().max(1);
            let mut i = from;
            for _ in 0..len {
                i = if forward { (i + 1) % len } else { (i + len - 1) % len };
                if d.buttons.get(i).is_some_and(|b| b.disabled.is_none()) {
                    return i;
                }
            }
            from
        };
        let mut press = None;
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => press = d.cancel,
            (KeyCode::Enter, _) => press = Some(if on_buttons { ui.button } else { d.primary }),
            (KeyCode::Tab, _) | (KeyCode::Down, _) => ui.field = (ui.field + 1) % (n + 1),
            (KeyCode::BackTab, _) | (KeyCode::Up, _) => ui.field = (ui.field + n) % (n + 1),
            (KeyCode::Left, _) if on_buttons => ui.button = step(ui.button, false),
            (KeyCode::Right, _) if on_buttons => ui.button = step(ui.button, true),
            (KeyCode::Left | KeyCode::Right, _) if selector.is_some() => {
                let s = selector.unwrap_or_default();
                let len = d.selectors[s].choices.len().max(1);
                let forward = key.code == KeyCode::Right;
                ui.selectors[s] = if forward {
                    (ui.selectors[s] + 1) % len
                } else {
                    (ui.selectors[s] + len - 1) % len
                };
            }
            (KeyCode::Char(' '), _) if toggle.is_some() => {
                let t = toggle.unwrap_or_default();
                ui.toggles[t] = !ui.toggles[t];
            }
            (KeyCode::Char(' '), _) if tick.is_some() => {
                let t = tick.unwrap_or_default();
                ui.ticks[t] = !ui.ticks[t];
            }
            (KeyCode::Backspace, _) if input.is_some() => {
                ui.values[input.unwrap_or_default()].pop();
            }
            (KeyCode::Char(c), m) if input.is_some() && !m.contains(KeyModifiers::CONTROL) => {
                ui.values[input.unwrap_or_default()].push(c);
            }
            (KeyCode::Char(c @ '1'..='9'), KeyModifiers::NONE) if !input.is_some() => {
                let i = c as usize - '1' as usize;
                if i < d.buttons.len() {
                    press = Some(i);
                }
            }
            _ => {}
        }
        if let Some(button) = press {
            self.answer_dialog(state, &d, button);
        }
        true
    }

    /// Send this terminal's answer: the station closes the dialog
    /// everywhere, then the line runs here as this user.
    fn answer_dialog(&mut self, state: &mut AppState, d: &Dialog, button: usize) {
        let Some(ui) = self.dialog_ui.take() else {
            return;
        };
        let answer = DialogAnswer {
            id: d.id.clone(),
            button,
            inputs: d
                .inputs
                .iter()
                .zip(ui.values.iter())
                .map(|(i, v)| (i.name.clone(), v.clone()))
                .collect(),
            toggles: d
                .toggles
                .iter()
                .zip(ui.toggles.iter())
                .map(|(t, on)| (t.name.clone(), *on))
                .chain(tickable(d).zip(ui.ticks.iter()).map(|(n, on)| (n.id.clone(), *on)))
                .collect(),
            selectors: d
                .selectors
                .iter()
                .zip(ui.selectors.iter())
                .map(|(s, i)| (s.name.clone(), s.choices.get(*i).cloned().unwrap_or_default()))
                .collect::<BTreeMap<_, _>>(),
        };
        match dialogs::answer(state, &Who::current(state), &answer) {
            Ok(Some(line)) => self.run_ui_command(state, &line),
            Ok(None) => {}
            Err(e) => {
                // A bad value keeps the dialog open with what was typed.
                if dialogs::for_client(state, &Who::current(state))
                    .0
                    .iter()
                    .any(|o| o.id == d.id)
                {
                    self.dialog_ui = Some(ui);
                }
                self.add_output(&e);
            }
        }
    }

    /// The window over the dimmed shell (called last, after the theme pass).
    pub(super) fn draw_dialog(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let Some(d) = Self::dialog_here(state) else {
            return;
        };
        let look = TuiLook::new(state);
        // Dim what is behind.
        for cell in frame.buffer_mut().content.iter_mut() {
            cell.fg = TuiLook::blend(cell.fg, cell.bg, 0.35);
        }
        let ui = self
            .dialog_ui
            .clone()
            .filter(|u| u.id == d.id)
            .unwrap_or_else(|| DialogUi::fresh(&d));

        let text = Style::default().fg(look.text).bg(look.surface);
        let dim = Style::default().fg(look.dim).bg(look.surface);
        let mut lines: Vec<Line> = Vec::new();
        for para in d.text.lines() {
            lines.push(Line::styled(para.to_string(), text));
        }
        for r in &d.rows {
            lines.push(Line::from(vec![
                Span::styled(format!("{}  ", r.k), dim),
                Span::styled(r.v.clone(), text.fg(look.tone(r.tone))),
            ]));
        }
        if !d.details.is_empty() && d.tree.is_none() {
            lines.push(Line::raw(""));
            for r in &d.details {
                lines.push(Line::styled(format!("  {}", r.k), text.fg(look.tone(r.tone))));
            }
        }
        // The tree: an indented list, its tickable rows numbered fields.
        if let Some(tree) = &d.tree {
            lines.push(Line::raw(""));
            let first = d.inputs.len() + d.toggles.len() + d.selectors.len();
            let mut t = 0;
            for n in &tree.nodes {
                let indent = "  ".repeat(n.depth as usize);
                let tone = text.fg(look.tone(n.tone));
                let (mark, note, style) = if n.tickable {
                    let on = ui.ticks.get(t).copied().unwrap_or(n.on);
                    let focused = ui.field == first + t;
                    t += 1;
                    let note = if !on && !n.note_off.is_empty() { &n.note_off } else { &n.note };
                    (format!("{t:>2} [{}]", if on { "x" } else { " " }), note.clone(), if focused { tone.fg(look.accent) } else { tone })
                } else {
                    ("     ".to_string(), n.note.clone(), tone)
                };
                let badge = if n.badge.is_empty() { String::new() } else { format!("  {}", n.badge) };
                lines.push(Line::from(vec![
                    Span::styled(format!("{indent}{mark} {} {}{badge}", n.no, n.name), style),
                    Span::styled(format!("  {note}"), dim),
                ]));
            }
        }
        let field_style = |i: usize| {
            if ui.field == i {
                Style::default().fg(look.text).bg(look.raised).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(look.text).bg(look.raised)
            }
        };
        for (i, input) in d.inputs.iter().enumerate() {
            let value = ui.values.get(i).cloned().unwrap_or_default();
            let shown = match input.kind {
                InputKind::Password => "•".repeat(value.chars().count()),
                _ => value,
            };
            let cursor = if ui.field == i { "▏" } else { " " };
            let unit = match &input.kind {
                InputKind::Number { unit: Some(u), .. } => format!(" {u}"),
                _ => String::new(),
            };
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![
                Span::styled(format!("{}  ", input.label), dim),
                Span::styled(format!(" {shown}{cursor}{unit} "), field_style(i)),
            ]));
        }
        for (t, toggle) in d.toggles.iter().enumerate() {
            let i = d.inputs.len() + t;
            let mark = if ui.toggles.get(t).copied().unwrap_or(toggle.on) { "[x]" } else { "[ ]" };
            let style = if ui.field == i { text.fg(look.accent) } else { text };
            lines.push(Line::styled(format!("{mark} {}", toggle.label), style));
        }
        for (s, sel) in d.selectors.iter().enumerate() {
            let i = d.inputs.len() + d.toggles.len() + s;
            let choice = sel
                .choices
                .get(ui.selectors.get(s).copied().unwrap_or(0))
                .cloned()
                .unwrap_or_default();
            lines.push(Line::from(vec![
                Span::styled(format!("{}  ", sel.label), dim),
                Span::styled(format!("‹ {choice} ›"), field_style(i)),
            ]));
        }
        // The lines it will run, as the ticks stand.
        if d.tree.is_some() {
            lines.push(Line::raw(""));
            let preview = tree_preview(&d, &ui);
            if preview.is_empty() {
                lines.push(Line::styled("  Nothing ticked", dim));
            }
            for l in preview {
                lines.push(Line::styled(format!("  {l}"), text.fg(look.tone(Some(lumacore_wire::Tone::Ok)))));
            }
        }
        // The countdown bar.
        let mut meta = d.meta.clone().map(|m| (m, look.tone(d.meta_tone)));
        if let Some(c) = &d.countdown {
            let left_ms = c.deadline_ms.saturating_sub(now_ms());
            let left = left_ms.div_ceil(1000);
            meta = Some(if c.paused || ui.paused {
                ("paused — waits for you".to_string(), look.dim)
            } else {
                let what = d.buttons.get(c.default).map_or("", |b| b.label.as_str());
                (format!("{what} in {left} s"), look.warn)
            });
            if c.total_s > 0 && !c.paused {
                let width = 40usize;
                let filled = ((left_ms as f64 / (f64::from(c.total_s) * 1000.0)) * width as f64)
                    .round()
                    .clamp(0.0, width as f64) as usize;
                lines.push(Line::raw(""));
                lines.push(Line::from(vec![
                    Span::styled("█".repeat(filled), text.fg(look.warn)),
                    Span::styled("░".repeat(width - filled), text.fg(look.raised)),
                ]));
            }
        } else if meta.is_none() && d.cancel.is_some() {
            meta = Some(("waits for you".to_string(), look.dim));
        }
        // The buttons.
        lines.push(Line::raw(""));
        let on_buttons = ui.field >= fields(&d);
        let mut spans = Vec::new();
        for (i, b) in d.buttons.iter().enumerate() {
            let chosen = on_buttons && i == ui.button;
            let fill = match b.kind {
                ButtonKind::Danger => look.alert,
                ButtonKind::Please => look.ok,
                _ => look.accent,
            };
            let style = if b.disabled.is_some() {
                Style::default().fg(look.dim).bg(look.surface).add_modifier(Modifier::CROSSED_OUT)
            } else if chosen {
                Style::default().fg(look.on_accent).bg(fill).add_modifier(Modifier::BOLD)
            } else if b.kind == ButtonKind::Danger {
                Style::default().fg(look.alert).bg(look.raised)
            } else {
                Style::default().fg(look.text).bg(look.raised)
            };
            let number = if d.buttons.len() > 1 { format!("{} ", i + 1) } else { String::new() };
            if i > 0 {
                spans.push(Span::styled("  ", text));
            }
            spans.push(Span::styled(format!(" {number}{} ", b.label), style));
        }
        lines.push(Line::from(spans).alignment(Alignment::Center));
        if let Some(reason) = d.buttons.iter().find_map(|b| b.disabled.clone()) {
            lines.push(Line::styled(reason, dim).alignment(Alignment::Center));
        }
        if let Some(note) = &d.note {
            lines.push(Line::styled(note.clone(), dim));
        }

        // Size: the text's width, inside the terminal.
        let longest = lines.iter().map(Line::width).max().unwrap_or(20) as u16;
        let width = (longest + 6).clamp(44, 96).min(area.width.saturating_sub(2));
        let inner_w = width.saturating_sub(4).max(1);
        let rows: u16 = lines
            .iter()
            .map(|l| (l.width() as u16).div_ceil(inner_w).max(1))
            .sum();
        let height = (rows + 4).min(area.height);
        let rect = Rect::new(
            area.x + area.width.saturating_sub(width) / 2,
            area.y + area.height.saturating_sub(height) / 2,
            width,
            height,
        );
        frame.render_widget(Clear, rect);
        let keys = if d.inputs.is_empty() {
            " ←/→ choose · Enter · 1–9 · Esc = Cancel "
        } else {
            " type · Tab to the buttons · Enter · Esc = Cancel "
        };
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(look.accent).bg(look.surface))
            .style(Style::default().bg(look.surface))
            .title(Line::styled(
                format!(" {} ", d.title),
                Style::default().fg(look.accent).add_modifier(Modifier::BOLD),
            ))
            .title_bottom(Line::styled(keys, Style::default().fg(look.dim)));
        if let Some((m, colour)) = meta {
            block = block.title(Line::styled(format!(" {m} "), Style::default().fg(colour)).right_aligned());
        }
        let inner = block.inner(rect);
        frame.render_widget(block, rect);
        let body = Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), inner.height.saturating_sub(1));
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).style(text), body);
    }
}
