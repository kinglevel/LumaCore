// ██████╗ ██████╗  █████╗ ██╗    ██╗
// ██╔══██╗██╔══██╗██╔══██╗██║    ██║
// ██║  ██║██████╔╝███████║██║ █╗ ██║
// ██║  ██║██╔══██╗██╔══██║██║███╗██║
// ██████╔╝██║  ██║██║  ██║╚███╔███╔╝
// ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝ ╚══╝╚══╝
// FILE: draw

//! How NAV draws (decisions 0291, the approved mockups `2026-10-23-nav-mode`): a header with the
//! place, the filter and the marks; the tree on the left (only the rows that fit, around the
//! cursor); the inspector on the right with its sections; the gesture log's last three lines;
//! the `?` overlay built from `NAV_KEYS`. In the user's theme roles only (`tui_look::role`).
//!
//! Owns:          TuiApp::draw_nav and its parts.
//! Doesn't own:   what is drawn (tui_nav.rs, lumacore-views' inspector).
//! Entry points:  TuiApp::draw_nav.
//! Decisions:     docs/decisions/0291-nav-mode-ls-rows-and-an-inspector.

use ratatui::prelude::*;
use ratatui::widgets::*;

use lumacore_objects::tree_rows::RowKind;
use lumacore_views::inspector::Section;

use super::super::TuiApp;
use super::super::tui_look::role;
use super::{Entry, FLASH, Focus, NAV_KEYS};
use crate::core::AppState;
use crate::core::navigation::Navigation;

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// Lines of the gesture log.
const LOG_LINES: u16 = 3;
/// The tree's share of the width when the two sit side by side, in percent.
const TREE_SHARE: u16 = 45;
/// Narrower than this, the inspector goes below the tree.
const SIDE_BY_SIDE: u16 = 70;
/// The inspector's name column.
const NAME_WIDTH: usize = 16;

impl TuiApp {
    /// NAV in place of the output: header, tree, inspector, the gesture log, the help.
    pub(in super::super) fn draw_nav(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let log_on = state
            .root
            .show_data
            .profile_for_user(state.effective_user_name())
            .is_none_or(|p| p.settings.preferences.nav_log);
        let log_height = if log_on && area.height > LOG_LINES + 6 { LOG_LINES + 1 } else { 0 };
        let [header, body, log] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(log_height)]).areas(area);
        self.draw_nav_header(frame, header, state);
        let [tree, inspector] = if body.width >= SIDE_BY_SIDE {
            Layout::horizontal([Constraint::Percentage(TREE_SHARE), Constraint::Fill(1)]).areas(body)
        } else {
            Layout::vertical([Constraint::Percentage(50), Constraint::Fill(1)]).areas(body)
        };
        self.draw_nav_tree(frame, tree);
        self.draw_nav_inspector(frame, inspector);
        if log_height > 0 {
            self.draw_nav_log(frame, log);
        }
        if self.nav.help {
            draw_help(frame, area);
        }
    }

    fn draw_nav_header(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let dim = Style::default().fg(role::DIM);
        let mut spans = vec![
            Span::styled(" NAV ", Style::default().fg(role::INFO).add_modifier(Modifier::BOLD)),
            Span::styled(state.path_string(), Style::default().fg(role::ACCENT)),
        ];
        let filter = |label: &str, text: &str, typing: bool| {
            Span::styled(format!("   {label}/{text}{}", if typing { "▏" } else { "" }), Style::default().fg(role::ACCENT_BRIGHT))
        };
        if !self.nav.filter.is_empty() || self.nav.typing == Some(Focus::Tree) {
            spans.push(filter("", &self.nav.filter, self.nav.typing == Some(Focus::Tree)));
            spans.push(Span::styled(format!("  {} hidden", self.nav.hidden), dim));
        }
        if !self.nav.ifilter.is_empty() || self.nav.typing == Some(Focus::Inspector) {
            spans.push(filter("properties ", &self.nav.ifilter, self.nav.typing == Some(Focus::Inspector)));
        }
        if !self.nav.marks.is_empty() {
            spans.push(Span::styled(format!("   • {} marked", self.nav.marks.len()), Style::default().fg(role::PROGRAMMER)));
        }
        if self.nav.changed_only {
            spans.push(Span::styled("   ● changed only", Style::default().fg(role::ACCENT)));
        }
        if let Some(p) = self.nav.pending {
            spans.push(Span::styled(format!("   {p}…"), Style::default().fg(role::ACCENT_BRIGHT)));
        }
        spans.push(Span::styled("   ? keys", dim));
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    /// The tree: only the rows that fit, the cursor kept in view.
    fn draw_nav_tree(&self, frame: &mut Frame, area: Rect) {
        let focused = self.nav.focus == Focus::Tree;
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(role::DIM));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let height = inner.height as usize;
        if self.nav.entries.is_empty() {
            frame.render_widget(Paragraph::new(Span::styled("  (empty)", Style::default().fg(role::DIM))), inner);
            return;
        }
        let start = window_start(self.nav.cursor, self.nav.entries.len(), height);
        let lines: Vec<Line> = self.nav.entries[start..(start + height).min(self.nav.entries.len())]
            .iter()
            .enumerate()
            .map(|(i, e)| self.tree_line(e, start + i == self.nav.cursor, focused))
            .collect();
        frame.render_widget(Paragraph::new(lines), inner);
    }

    fn tree_line(&self, e: &Entry, cursor: bool, focused: bool) -> Line<'static> {
        let dim = Style::default().fg(role::DIM);
        let row = &e.row;
        let marked = self.nav.marks.iter().any(|m| *m == row.path);
        let mut spans = vec![
            Span::styled(if cursor { "►" } else { " " }, Style::default().fg(role::ACCENT)),
            Span::styled(if marked { "•" } else { " " }, Style::default().fg(role::PROGRAMMER)),
            Span::raw("  ".repeat(e.depth)),
            Span::styled(
                match (row.branch, e.expanded) {
                    (true, true) => "▾ ",
                    (true, false) => "▸ ",
                    _ => "  ",
                },
                dim,
            ),
        ];
        let hit = |s: Style| if e.hit { s.fg(role::ACCENT_BRIGHT) } else { s };
        match row.kind {
            RowKind::Node | RowKind::Folder | RowKind::Dir => {
                let name = if row.branch { format!("{}/", row.name) } else { row.name.clone() };
                let style = Style::default().fg(role::ACCENT);
                spans.push(Span::styled(name, hit(if row.branch { style.add_modifier(Modifier::BOLD) } else { style })));
                if !row.meta.is_empty() {
                    spans.push(Span::styled(format!("  {}", row.meta), dim));
                }
            }
            RowKind::File => {
                spans.push(Span::styled(row.name.clone(), hit(Style::default().fg(role::STRONG))));
                if !row.meta.is_empty() {
                    spans.push(Span::styled(format!("  ({})", row.meta), dim));
                }
            }
            RowKind::Object => {
                // A show object's colour as a mark before its slot (decisions 0294).
                if let Some(c) = super::super::tui_look::object_paint(row.paint.as_deref()) {
                    spans.push(Span::styled("■ ", Style::default().fg(c)));
                }
                spans.push(Span::styled(row.slot.map(|s| s.to_string()).unwrap_or_default(), Style::default().fg(role::STRONG).add_modifier(Modifier::BOLD)));
                if !row.name.is_empty() {
                    let name = if row.quoted { format!("  \"{}\"", row.name) } else { format!("  {}", row.name) };
                    spans.push(Span::styled(name, hit(Style::default().fg(role::OK))));
                }
                if !row.meta.is_empty() {
                    spans.push(Span::styled(format!("  {}", row.meta), dim));
                }
                spans.push(Span::styled(format!("  ({})", row.type_name), dim));
            }
        }
        let line = Line::from(spans);
        if cursor && focused { line.style(Style::default().add_modifier(Modifier::REVERSED)) } else { line }
    }

    /// The inspector: its title, then each section's rows; the cursor kept in view.
    fn draw_nav_inspector(&self, frame: &mut Frame, area: Rect) {
        let focused = self.nav.focus == Focus::Inspector;
        let inspector = &self.nav.inspector;
        let pin = if self.nav.pinned.is_some() { "  [pinned]" } else { "" };
        let title = Line::from(vec![
            Span::styled(format!(" {}", inspector.title), Style::default().fg(role::STRONG).add_modifier(Modifier::BOLD)),
            Span::styled(pin, Style::default().fg(role::ACCENT)),
        ]);
        let inner = Rect { y: area.y + 1, height: area.height.saturating_sub(1), x: area.x + 1, width: area.width.saturating_sub(1) };
        frame.render_widget(Paragraph::new(title), Rect { height: 1, ..area });
        // Section headers between the rows; `at` is the line of the cursor's row.
        let mut lines: Vec<Line> = Vec::new();
        let mut at = 0;
        let mut section = None;
        for (n, &i) in self.nav.visible().iter().enumerate() {
            let row = &inspector.rows[i];
            if section != Some(row.section) {
                section = Some(row.section);
                lines.push(Line::from(Span::styled(
                    match row.section {
                        Section::Identity => "─ Identity",
                        Section::Info => "─ Facts",
                        Section::Property => "─ Properties",
                        Section::Link => "─ Links",
                    },
                    Style::default().fg(role::DIM),
                )));
            }
            if n == self.nav.icursor {
                at = lines.len();
            }
            lines.push(self.inspector_line(row, n == self.nav.icursor, focused));
        }
        if lines.is_empty() {
            lines.push(Line::from(Span::styled("(nothing to show)", Style::default().fg(role::DIM))));
        }
        let height = inner.height as usize;
        let start = window_start(at, lines.len(), height);
        let shown: Vec<Line> = lines.into_iter().skip(start).take(height).collect();
        frame.render_widget(Paragraph::new(shown), inner);
    }

    fn inspector_line(&self, row: &lumacore_views::inspector::Row, cursor: bool, focused: bool) -> Line<'static> {
        let dim = Style::default().fg(role::DIM);
        let lit = self.nav.flashes.get(&row.name).is_some_and(|at| at.elapsed() < FLASH);
        let mut value_style = match () {
            _ if row.missing => Style::default().fg(role::ALERT),
            _ if row.link.is_some() => Style::default().fg(role::INFO),
            _ => Style::default().fg(role::STRONG),
        };
        if lit {
            value_style = value_style.fg(role::ACCENT_BRIGHT).add_modifier(Modifier::BOLD | Modifier::REVERSED);
        }
        let value = match (&row.link, row.missing) {
            (Some(_), _) => format!("→ {}", row.value),
            (None, true) => format!("✕ {}", row.value),
            _ if row.value.is_empty() => "—".to_string(),
            _ => row.value.clone(),
        };
        let mut spans = vec![
            Span::styled(if row.changed { "●" } else { " " }, Style::default().fg(role::ACCENT)),
            Span::styled(format!("{:<NAME_WIDTH$} ", row.name), Style::default().fg(if row.set.is_some() { role::ACCENT } else { role::DIM })),
            Span::styled(value, value_style),
        ];
        if !row.kind.is_empty() {
            spans.push(Span::styled(format!("  {}", row.kind), dim));
        }
        let line = Line::from(spans);
        match (cursor, focused) {
            (true, true) => line.style(Style::default().add_modifier(Modifier::REVERSED)),
            (true, false) => line.style(Style::default().add_modifier(Modifier::UNDERLINED)),
            _ => line,
        }
    }

    /// The gesture log's last lines: what NAV put on the command line, ran and copied.
    fn draw_nav_log(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::TOP).border_style(Style::default().fg(role::DIM));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let skip = self.nav.log.len().saturating_sub(inner.height as usize);
        let lines: Vec<Line> = self.nav.log[skip..]
            .iter()
            .map(|l| Line::from(Span::styled(format!(" {l}"), Style::default().fg(if l.starts_with("copied") { role::OK } else { role::DIM }))))
            .collect();
        frame.render_widget(Paragraph::new(lines), inner);
    }
}

/// The `?` overlay: every NAV key from the one table.
fn draw_help(frame: &mut Frame, area: Rect) {
    let width = (area.width * 4 / 5).max(40).min(area.width);
    let height = (NAV_KEYS.len() as u16 + 2).min(area.height);
    let rect = Rect { x: area.x + (area.width - width) / 2, y: area.y + (area.height - height) / 2, width, height };
    frame.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" NAV keys ")
        .title_bottom(" any key closes ")
        .border_style(Style::default().fg(role::ACCENT));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    let keys = NAV_KEYS.iter().map(|(k, _)| k.chars().count()).max().unwrap_or(0);
    let lines: Vec<Line> = NAV_KEYS
        .iter()
        .map(|(k, what)| {
            Line::from(vec![
                Span::styled(format!(" {k:<keys$}  "), Style::default().fg(role::ACCENT).add_modifier(Modifier::BOLD)),
                Span::raw(*what),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
}

/// The first of `len` lines to show in `height` so that `cursor` stays in view, centred.
fn window_start(cursor: usize, len: usize, height: usize) -> usize {
    if len <= height {
        return 0;
    }
    cursor.saturating_sub(height / 2).min(len - height)
}
