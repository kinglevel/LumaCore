// ███╗   ██╗ █████╗ ██╗   ██╗
// ████╗  ██║██╔══██╗██║   ██║
// ██╔██╗ ██║███████║██║   ██║
// ██║╚██╗██║██╔══██║╚██╗ ██╔╝
// ██║ ╚████║██║  ██║ ╚████╔╝
// ╚═╝  ╚═══╝╚═╝  ╚═╝  ╚═══╝
// FILE: nav

//! NAV mode (decisions 0291): the tree on the left — exactly `ls`'s rows from the one builder
//! (`lumacore_objects::tree_rows`), opened by their typed paths — and the inspector on the right
//! (`lumacore_views::inspector`), following the cursor. Every gesture that changes something is a
//! command line: NAV puts `Set` / `Reset` / range / `Delete` lines on the command line and runs
//! `cd`, `Shortcuts` and `Set UserProfile "NavLog"`; copying goes through the one clipboard door;
//! a bookmark is filed through the change path (`commands::record_change`). The gesture log
//! shows what NAV did, three lines above the command line.
//!
//! Owns:          Nav (the tree's entries, the inspector, marks, filters, bookmarks, the log), NAV_KEYS, the keys.
//! Doesn't own:   the rows (tree_rows), the inspector's rows (lumacore-views), drawing (tui_nav/draw.rs), the clipboard (clipboard.rs).
//! Entry points:  TuiApp::nav_key, TuiApp::nav_refresh, TuiApp::nav_reset, NAV_KEYS, range_line.
//! Decisions:     docs/decisions/0291-nav-mode-ls-rows-and-an-inspector, docs/decisions/0106-tree-paths-typed-node.

mod draw;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent as CtKeyEvent, KeyModifiers};
use lumacore_objects::tree_rows::{self, RowKind, TreeRow};
use lumacore_views::inspector::{Inspector, Section};
use lumacore_wire::UiFocus;

use super::TuiApp;
use super::clipboard;
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

/// NAV's keys, as the `?` overlay lists them (decisions 0291).
pub(super) const NAV_KEYS: &[(&str, &str)] = &[
    ("↑ ↓  PgUp PgDn  Home End", "move"),
    ("→  ←", "open · close, or go to the parent"),
    ("Enter", "tree: open, or to the inspector · property: its Set line · link: go there"),
    ("Tab", "tree ⇄ inspector"),
    ("0–9", "jump to that slot"),
    ("/  n  N", "filter (the pool by name or number, the inspector by property) · next / previous hit"),
    ("Space", "mark a row: the marked rows become one range on the command line"),
    ("y", "inspector: copy the focused row"),
    ("yg yp ya yn ys", "tree: copy the GUID · path · target · name · the property's Set line"),
    ("r", "on a changed ● property: its Reset line"),
    ("v", "only the changed properties"),
    ("m + letter", "bookmark this object (by GUID)"),
    ("' + letter", "jump to a bookmark"),
    ("p", "pin the inspector to this object"),
    ("c", "make this the prompt's place (cd)"),
    ("Delete", "its Delete line"),
    ("L", "the gesture log on / off"),
    ("?", "this help"),
    ("Esc", "close · clear the marks · leave NAV"),
];

/// Digits typed this close together are one slot number.
const DIGIT_GAP: Duration = Duration::from_millis(1000);
/// How often the tree and the inspector are rebuilt while nothing else moves them (live rows).
const REFRESH: Duration = Duration::from_millis(1000);
/// How long a value that changed under the inspector stays lit.
pub(super) const FLASH: Duration = Duration::from_millis(1500);
/// Lines the gesture log keeps.
const LOG_KEEP: usize = 50;

/// Where NAV's keys go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Focus {
    #[default]
    Tree,
    Inspector,
}

/// One visible row of the tree.
#[derive(Debug, Clone)]
pub(super) struct Entry {
    pub row: TreeRow,
    pub depth: usize,
    pub expanded: bool,
    /// It matches the filter.
    pub hit: bool,
}

/// NAV's state in one shell session.
#[derive(Default)]
pub(super) struct Nav {
    pub entries: Vec<Entry>,
    pub cursor: usize,
    pub expanded: BTreeSet<String>,
    pub focus: Focus,
    pub inspector: Inspector,
    pub icursor: usize,
    pub pinned: Option<String>,
    /// The tree's filter and the inspector's.
    pub filter: String,
    pub ifilter: String,
    /// A filter being typed, and where.
    pub typing: Option<Focus>,
    /// Object rows the filter hides.
    pub hidden: usize,
    pub changed_only: bool,
    /// Marked rows' paths, in marking order.
    pub marks: Vec<String>,
    /// The range line NAV put on the command line (NAV keys go on while it stands there).
    pub owned_line: Option<String>,
    /// `y`, `m` or `'` waiting for its second key.
    pub pending: Option<char>,
    digits: Option<(String, Instant)>,
    pub log: Vec<String>,
    pub help: bool,
    /// Inspector rows (by name) whose value changed, and when.
    pub flashes: HashMap<String, Instant>,
    /// A member station's bookmarks (its master keeps the show's): this session only.
    local_marks: BTreeMap<char, uuid::Uuid>,
    /// What the last build saw: the base path and the show's generation, and when.
    built: Option<(String, u64, Instant)>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Nav {
    /// The entry under the cursor.
    pub(super) fn current(&self) -> Option<&Entry> {
        self.entries.get(self.cursor)
    }

    /// The inspector rows shown (the property filter, changed only), by index.
    pub(super) fn visible(&self) -> Vec<usize> {
        let filter = self.ifilter.to_lowercase();
        self.inspector
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| filter.is_empty() || r.name.to_lowercase().contains(&filter))
            .filter(|(_, r)| !self.changed_only || r.changed)
            .map(|(i, _)| i)
            .collect()
    }

    /// The inspector row under its cursor.
    pub(super) fn focused_row(&self) -> Option<&lumacore_views::inspector::Row> {
        self.visible().get(self.icursor).map(|&i| &self.inspector.rows[i])
    }

    pub(super) fn say(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > LOG_KEEP {
            self.log.remove(0);
        }
    }
}

impl TuiApp {
    /// Back to the top of the tree (entering NAV, `cd`).
    pub(super) fn nav_reset(&mut self, state: &mut AppState) {
        self.nav.cursor = 0;
        self.nav.icursor = 0;
        self.nav.expanded.clear();
        self.nav.filter.clear();
        self.nav.marks.clear();
        self.nav.focus = Focus::Tree;
        self.nav_rebuild(state);
    }

    /// Rebuild when the show moved, the shell went elsewhere, or a while passed (live rows); a
    /// frame calls it before it draws.
    pub(super) fn nav_refresh(&mut self, state: &mut AppState) {
        if state.shortcut_mode != ShortcutMode::Navigator {
            return;
        }
        let base = crate::core::tree_path::dot(&state.navigation_path);
        let fresh = self
            .nav
            .built
            .as_ref()
            .is_some_and(|(b, g, at)| *b == base && *g == state.show_generation && at.elapsed() < REFRESH);
        if !fresh {
            self.nav_rebuild(state);
        }
    }

    /// The tree's entries and the inspector, now.
    pub(super) fn nav_rebuild(&mut self, state: &mut AppState) {
        let base = crate::core::tree_path::dot(&state.navigation_path);
        let at = self.nav.current().map(|e| e.row.path.clone());
        let mut entries = Vec::new();
        let mut hidden = 0;
        collect(state, &base, 0, &self.nav.expanded, &self.nav.filter, &mut entries, &mut hidden);
        self.nav.entries = entries;
        self.nav.hidden = hidden;
        self.nav.cursor = at
            .and_then(|p| self.nav.entries.iter().position(|e| e.row.path == p))
            .unwrap_or(self.nav.cursor)
            .min(self.nav.entries.len().saturating_sub(1));
        self.nav_inspect(state);
        self.nav.built = Some((base, state.show_generation, Instant::now()));
    }

    /// Build the inspector for the pinned object or the cursor's row; values that changed under
    /// the same object light up.
    fn nav_inspect(&mut self, state: &mut AppState) {
        let path = match (&self.nav.pinned, self.nav.current()) {
            (Some(p), _) => p.clone(),
            (None, Some(e)) => e.row.path.clone(),
            (None, None) => crate::core::tree_path::dot(&state.navigation_path),
        };
        let next = lumacore_views::inspector::view(state, &path);
        if next.path == self.nav.inspector.path {
            let now = Instant::now();
            for row in next.rows.iter().filter(|r| r.section != Section::Link) {
                let before = self.nav.inspector.rows.iter().find(|o| o.name == row.name && o.section == row.section);
                if before.is_some_and(|o| o.value != row.value) {
                    self.nav.flashes.insert(row.name.clone(), now);
                }
            }
        } else {
            self.nav.flashes.clear();
            self.nav.icursor = 0;
        }
        self.nav.flashes.retain(|_, at| at.elapsed() < FLASH);
        self.nav.inspector = next;
        self.nav.icursor = self.nav.icursor.min(self.nav.visible().len().saturating_sub(1));
    }

    /// NAV's keys while the shell is in NAV, its terminal has the focus and the command line is
    /// empty (or holds the range line NAV put there). Returns whether the key was NAV's.
    pub(super) fn nav_key(&mut self, key: CtKeyEvent, state: &mut AppState) -> bool {
        if state.shortcut_mode != ShortcutMode::Navigator || !matches!(self.focus, UiFocus::Terminal) {
            return false;
        }
        let input = self.get_input();
        if !(input.is_empty() || self.nav.owned_line.as_deref() == Some(input.as_str())) {
            return false;
        }
        if key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) {
            return false;
        }
        if self.nav.entries.is_empty() && self.nav.built.is_none() {
            self.nav_rebuild(state);
        }
        if self.nav.help {
            self.nav.help = false;
            return true;
        }
        if let Some(target) = self.nav.typing {
            return self.nav_type_filter(key, target, state);
        }
        let ch = match key.code {
            KeyCode::Char(c) => Some(c),
            _ => None,
        };
        if let Some(prefix) = self.nav.pending.take() {
            if let Some(c) = ch {
                match prefix {
                    'y' => self.nav_copy_key(c),
                    'm' => self.nav_set_bookmark(state, c),
                    _ => self.nav_jump_bookmark(state, c),
                }
            }
            return true;
        }
        if let Some(d) = ch.filter(char::is_ascii_digit)
            && self.nav.focus == Focus::Tree
        {
            self.nav_digit(d);
            self.nav_inspect(state);
            return true;
        }
        self.nav.digits = None;
        let tree = self.nav.focus == Focus::Tree;
        match key.code {
            KeyCode::Up => self.nav_move(-1),
            KeyCode::Down => self.nav_move(1),
            KeyCode::PageUp => self.nav_move(-(self.page_size() as isize)),
            KeyCode::PageDown => self.nav_move(self.page_size() as isize),
            KeyCode::Home => self.nav_move(isize::MIN / 2),
            KeyCode::End => self.nav_move(isize::MAX / 2),
            KeyCode::Right if tree => self.nav_open(state),
            KeyCode::Left if tree => self.nav_close(state),
            KeyCode::Left | KeyCode::Tab | KeyCode::BackTab if !tree => self.nav.focus = Focus::Tree,
            KeyCode::Tab | KeyCode::BackTab => self.nav.focus = Focus::Inspector,
            KeyCode::Enter => self.nav_enter(state),
            KeyCode::Esc => self.nav_escape(state),
            KeyCode::Delete if tree => self.nav_delete_line(state),
            KeyCode::Char(' ') if tree => self.nav_mark(state),
            KeyCode::Char('y') if !tree => self.nav_copy_focused(),
            KeyCode::Char(c @ ('y' | 'm' | '\'')) => self.nav.pending = Some(c),
            KeyCode::Char('/') => self.nav.typing = Some(self.nav.focus),
            KeyCode::Char('n') => self.nav_hit(1),
            KeyCode::Char('N') => self.nav_hit(-1),
            KeyCode::Char('r') if !tree => self.nav_reset_line(),
            KeyCode::Char('v') => {
                self.nav.changed_only = !self.nav.changed_only;
                self.nav.icursor = 0;
            }
            KeyCode::Char('p') => self.nav_pin(),
            KeyCode::Char('c') => self.nav_cd_here(state),
            KeyCode::Char('L') => self.nav_line(state, "Set UserProfile \"NavLog\" Toggle"),
            KeyCode::Char('?') => self.nav.help = true,
            _ => return false,
        }
        if matches!(key.code, KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End | KeyCode::Char('n' | 'N' | 'p')) && tree {
            self.nav_inspect(state);
        }
        true
    }

    /// A key while a filter is typed: characters add, Backspace takes back, Enter keeps it, Esc
    /// drops it.
    fn nav_type_filter(&mut self, key: CtKeyEvent, target: Focus, state: &mut AppState) -> bool {
        let filter = if target == Focus::Tree { &mut self.nav.filter } else { &mut self.nav.ifilter };
        match key.code {
            KeyCode::Char(c) => filter.push(c),
            KeyCode::Backspace => {
                filter.pop();
            }
            KeyCode::Enter => self.nav.typing = None,
            KeyCode::Esc => {
                filter.clear();
                self.nav.typing = None;
            }
            _ => return true,
        }
        if target == Focus::Tree {
            self.nav_rebuild(state);
            if let Some(i) = self.nav.entries.iter().position(|e| e.hit) {
                self.nav.cursor = i;
                self.nav_inspect(state);
            }
        } else {
            self.nav.icursor = 0;
        }
        true
    }

    /// Move the cursor of the focused side by `by` rows (clamped).
    fn nav_move(&mut self, by: isize) {
        let (cursor, len) = match self.nav.focus {
            Focus::Tree => (&mut self.nav.cursor, self.nav.entries.len()),
            Focus::Inspector => {
                let len = self.nav.visible().len();
                (&mut self.nav.icursor, len)
            }
        };
        if len == 0 {
            return;
        }
        *cursor = (*cursor as isize).saturating_add(by).clamp(0, len as isize - 1) as usize;
    }

    /// → : open a branch, or step into an open one.
    fn nav_open(&mut self, state: &mut AppState) {
        let Some(e) = self.nav.current() else { return };
        if !e.row.branch {
            return;
        }
        if e.expanded {
            self.nav_move(1);
            self.nav_inspect(state);
        } else {
            self.nav.expanded.insert(e.row.path.clone());
            self.nav_rebuild(state);
        }
    }

    /// ← : close an open branch, else go to the parent row, else `cd ..`.
    fn nav_close(&mut self, state: &mut AppState) {
        let Some(e) = self.nav.current() else {
            self.nav_cd(state, "cd ..");
            return;
        };
        if e.expanded {
            let path = e.row.path.clone();
            let prefix = format!("{path}.");
            self.nav.expanded.retain(|p| *p != path && !p.starts_with(&prefix));
            self.nav_rebuild(state);
        } else if e.depth > 0 {
            let depth = e.depth;
            if let Some(i) = (0..self.nav.cursor).rev().find(|&i| self.nav.entries[i].depth == depth - 1) {
                self.nav.cursor = i;
            }
            self.nav_inspect(state);
        } else {
            self.nav_cd(state, "cd ..");
        }
    }

    /// Enter: run the range line NAV made; on the tree open a branch or go to the inspector;
    /// on a property put its Set line on the command line; on a link go there.
    fn nav_enter(&mut self, state: &mut AppState) {
        if self.nav.owned_line.is_some() {
            self.nav.owned_line = None;
            self.nav.marks.clear();
            self.submit_command(state);
            self.nav_rebuild(state);
            return;
        }
        if self.nav.focus == Focus::Tree {
            match self.nav.current() {
                Some(e) if e.row.branch => {
                    let path = e.row.path.clone();
                    if !self.nav.expanded.remove(&path) {
                        self.nav.expanded.insert(path);
                    }
                    self.nav_rebuild(state);
                }
                Some(_) => self.nav.focus = Focus::Inspector,
                None => {}
            }
            return;
        }
        let Some(row) = self.nav.focused_row().cloned() else { return };
        if let Some(set) = row.set {
            self.nav_put(&set);
        } else if let Some(path) = row.link {
            self.nav_follow(state, &path);
        } else if row.missing {
            self.nav.say(format!("{}: missing — the object it pointed at is gone", row.name));
        }
    }

    /// Esc: clear the marks; else leave NAV.
    fn nav_escape(&mut self, state: &mut AppState) {
        if !self.nav.marks.is_empty() {
            self.nav.marks.clear();
            if self.nav.owned_line.take().is_some() {
                self.set_input("");
            }
            return;
        }
        self.nav_line(state, "Shortcuts Keyboard");
    }

    /// Digits: the row with that slot (beside the cursor first, else anywhere in the tree).
    fn nav_digit(&mut self, d: char) {
        let mut number = match self.nav.digits.take() {
            Some((n, at)) if at.elapsed() < DIGIT_GAP => n,
            _ => String::new(),
        };
        number.push(d);
        let Ok(slot) = number.parse::<u32>() else { return };
        let parent = self.nav.current().map(|e| parent_of(&e.row.path).to_string()).unwrap_or_default();
        let found = self
            .nav
            .entries
            .iter()
            .position(|e| e.row.slot == Some(slot) && parent_of(&e.row.path) == parent)
            .or_else(|| self.nav.entries.iter().position(|e| e.row.slot == Some(slot)));
        if let Some(i) = found {
            self.nav.cursor = i;
        }
        self.nav.digits = Some((number, Instant::now()));
    }

    /// n / N: the next or previous hit of the filter.
    fn nav_hit(&mut self, step: isize) {
        if self.nav.focus == Focus::Inspector {
            self.nav_move(step);
            return;
        }
        let n = self.nav.entries.len() as isize;
        if n == 0 {
            return;
        }
        let mut i = self.nav.cursor as isize;
        for _ in 0..n {
            i = (i + step).rem_euclid(n);
            if self.nav.entries[i as usize].hit {
                self.nav.cursor = i as usize;
                return;
            }
        }
    }

    /// Space: mark or unmark the row; the marked rows become one range line.
    fn nav_mark(&mut self, state: &mut AppState) {
        let Some(e) = self.nav.current() else { return };
        if e.row.slot.is_none() {
            self.nav.say(format!("{}: only objects take a mark", e.row.name));
            return;
        }
        let path = e.row.path.clone();
        match self.nav.marks.iter().position(|m| *m == path) {
            Some(i) => {
                self.nav.marks.remove(i);
            }
            None => self.nav.marks.push(path),
        }
        let targets: Vec<(String, u32)> = self.nav.marks.iter().filter_map(|p| target_of(state, p)).collect();
        let line = range_line(&targets);
        if line.is_empty() {
            if self.nav.owned_line.take().is_some() {
                self.set_input("");
            }
        } else {
            self.snapshot_input();
            self.set_input(&line);
            self.nav.owned_line = Some(line.clone());
            self.nav.say(format!("→ {line}"));
        }
        self.nav_move(1);
        self.nav_inspect(state);
    }

    /// y + g / p / a / n / s on a tree row.
    fn nav_copy_key(&mut self, c: char) {
        let Some(e) = self.nav.current() else { return };
        let row = &e.row;
        let (label, text) = match c {
            'g' => ("GUID", row.guid.map(|g| g.to_string()).unwrap_or_default()),
            'p' => ("path", row.path.clone()),
            'a' => ("target", self.nav.inspector.target.clone().filter(|_| self.nav.inspector.path == row.path).or_else(|| row.target()).unwrap_or_default()),
            'n' => ("name", row.name.clone()),
            's' => ("Set line", self.nav.focused_row().and_then(|r| r.set.clone()).map(|s| s.trim_end().to_string()).unwrap_or_default()),
            _ => return,
        };
        self.nav_copy(label, &text);
    }

    /// y in the inspector: the focused row (a link's path, else its value).
    fn nav_copy_focused(&mut self) {
        let Some(row) = self.nav.focused_row().cloned() else { return };
        let text = row.link.clone().unwrap_or(row.value);
        self.nav_copy(&row.name, &text);
    }

    /// Copy through the one clipboard door; the log says what.
    fn nav_copy(&mut self, label: &str, text: &str) {
        if text.is_empty() {
            self.nav.say(format!("nothing to copy as {label} here"));
            return;
        }
        clipboard::copy(&mut self.ui_outbox, label, text);
        self.nav.say(format!("copied {label}: {text}"));
    }

    /// r: a changed property's Reset line.
    fn nav_reset_line(&mut self) {
        match self.nav.focused_row().and_then(|r| r.reset.clone()) {
            Some(line) => self.nav_put(&line),
            None => self.nav.say("only a changed ● property has a Reset line"),
        }
    }

    /// Delete: the row's Delete line on the command line.
    fn nav_delete_line(&mut self, state: &mut AppState) {
        let Some(path) = self.nav.current().map(|e| e.row.path.clone()) else { return };
        match target_of(state, &path) {
            Some((kind, no)) => self.nav_put(&format!("Delete {kind} {no}")),
            None => self.nav.say("only an object has a Delete line"),
        }
    }

    /// p: pin the inspector to its object, or let it follow the cursor again.
    fn nav_pin(&mut self) {
        self.nav.pinned = match self.nav.pinned.take() {
            Some(_) => None,
            None => Some(self.nav.inspector.path.clone()),
        };
    }

    /// c: the cursor's node becomes the prompt's place.
    fn nav_cd_here(&mut self, state: &mut AppState) {
        let Some(path) = self.nav.current().map(|e| e.row.path.clone()) else { return };
        self.nav_cd(state, &format!("cd {path}"));
    }

    /// m + letter: bookmark the object by its GUID — saved per user in the show through the
    /// change path, never undone (`ShowSubtree::Bookmarks`); a member keeps it for this session.
    fn nav_set_bookmark(&mut self, state: &mut AppState, letter: char) {
        if !letter.is_ascii_alphabetic() {
            return;
        }
        let guid = match self.nav.focus {
            Focus::Inspector => self.nav.inspector.guid,
            Focus::Tree => self.nav.current().and_then(|e| e.row.guid),
        };
        let Some(guid) = guid else {
            self.nav.say("only an object with a GUID takes a bookmark");
            return;
        };
        if state.link.is_some() {
            self.nav.local_marks.insert(letter, guid);
            self.nav.say(format!("bookmark {letter} (this session: the master keeps the show's)"));
            return;
        }
        let user = state.effective_user_name().to_string();
        let undo = crate::core::commands::registry::Undo::Untracked;
        crate::core::commands::record_change(state, &format!("Bookmark {letter}"), undo, |s| {
            if let Some(u) = s.root.show_data.user_mut(&user) {
                u.bookmarks.set(letter, guid);
            }
        });
        self.nav.say(format!("bookmark {letter}: {}", self.nav.inspector.title));
    }

    /// ' + letter: go to the bookmarked object wherever it is now.
    fn nav_jump_bookmark(&mut self, state: &mut AppState, letter: char) {
        let user = state.effective_user_name().to_string();
        let guid = self
            .nav
            .local_marks
            .get(&letter)
            .copied()
            .or_else(|| state.root.show_data.user(&user).and_then(|u| u.bookmarks.get(letter)));
        let Some(guid) = guid else {
            self.nav.say(format!("no bookmark {letter}"));
            return;
        };
        match path_of_guid(state, guid) {
            Some(path) => self.nav_follow(state, &path),
            None => self.nav.say(format!("bookmark {letter}: missing — its object is gone")),
        }
    }

    /// Go to `path` in the tree: from the root when it is outside the shell's place, its
    /// parents opened, the cursor on it (nothing runs, nothing changes).
    pub(super) fn nav_follow(&mut self, state: &mut AppState, path: &str) {
        let mut base = crate::core::tree_path::dot(&state.navigation_path);
        if !path.starts_with(&format!("{base}.")) {
            self.execute_command(state, "cd /");
            base = crate::core::tree_path::dot(&state.navigation_path);
        }
        let mut at = path;
        while let Some((parent, _)) = at.rsplit_once('.') {
            if parent.len() <= base.len() {
                break;
            }
            self.nav.expanded.insert(parent.to_string());
            at = parent;
        }
        self.nav.filter.clear();
        self.nav.pinned = None;
        self.nav_rebuild(state);
        match self.nav.entries.iter().position(|e| e.row.path == path) {
            Some(i) => {
                self.nav.cursor = i;
                self.nav.focus = Focus::Tree;
                self.nav_inspect(state);
            }
            None => self.nav.say(format!("not in the tree: {path}")),
        }
    }

    /// Put a line on the command line for the user to finish; NAV's keys rest until it's gone.
    fn nav_put(&mut self, line: &str) {
        self.snapshot_input();
        self.set_input(line);
        self.nav.owned_line = None;
        self.nav.say(format!("→ {}", line.trim_end()));
    }

    /// Run one of NAV's own lines (`cd`, `Shortcuts`, `Set UserProfile "NavLog"`), logged.
    fn nav_line(&mut self, state: &mut AppState, line: &str) {
        self.nav.say(format!("ran {line}"));
        self.execute_command(state, line);
    }

    /// Change directory from the navigator and start the tree there.
    pub(super) fn nav_cd(&mut self, state: &mut AppState, cmd: &str) {
        self.nav_line(state, cmd);
        self.nav_reset(state);
    }
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The rows below `path`, and below every open one, with the filter applied to the objects.
fn collect(state: &AppState, path: &str, depth: usize, expanded: &BTreeSet<String>, filter: &str, out: &mut Vec<Entry>, hidden: &mut usize) {
    for row in tree_rows::rows(state, path) {
        let hit = !filter.is_empty() && matches_filter(&row, filter);
        if !filter.is_empty() && row.kind != RowKind::Node && !hit {
            *hidden += 1;
            continue;
        }
        let open = row.branch && expanded.contains(&row.path);
        let below = row.path.clone();
        out.push(Entry { row, depth, expanded: open, hit });
        if open {
            collect(state, &below, depth + 1, expanded, filter, out, hidden);
        }
    }
}

/// A row matches a filter by its slot (a number) or its name and text (any case).
fn matches_filter(row: &TreeRow, filter: &str) -> bool {
    if let Ok(n) = filter.parse::<u32>() {
        return row.slot == Some(n);
    }
    let f = filter.to_lowercase();
    row.name.to_lowercase().contains(&f) || row.text().to_lowercase().contains(&f)
}

fn parent_of(path: &str) -> &str {
    path.rsplit_once('.').map_or("", |(p, _)| p)
}

/// The words that name the object at `path` and its number (`DataPool 2 Group`, 3): owner first
/// when it lives in another datapool than the user's.
fn target_of(state: &AppState, path: &str) -> Option<(String, u32)> {
    let p = crate::core::tree_path::TreePath::parse_dot(path)?;
    let (object, datapool, no) = crate::core::objects::at_path(&p)?;
    Some(match datapool {
        Some(dp) if dp != state.selected_datapool() => (format!("DataPool {dp} {}", object.keyword), no),
        _ => (object.keyword.to_string(), no),
    })
}

/// Where an object is in the tree now, by its GUID.
fn path_of_guid(state: &AppState, guid: uuid::Uuid) -> Option<String> {
    let walk = state.root.show_data.walk_identity();
    let o = walk.objects.iter().find(|o| o.guid == guid)?;
    let path = format!("{}.{}", crate::core::tree_path::Node::ShowData.path(), o.path);
    state.resolve_target(&path).ok().map(|segments| crate::core::tree_path::dot(&segments))
}

/// Marked objects as one range line (the one range rule): `Group 1 Thru 3 + 7`, kinds in the
/// order first marked (`Group 1 + Fixture 7`).
pub(super) fn range_line(targets: &[(String, u32)]) -> String {
    let mut kinds: Vec<(&str, Vec<u32>)> = Vec::new();
    for (kind, no) in targets {
        match kinds.iter_mut().find(|(k, _)| k == kind) {
            Some((_, nos)) => nos.push(*no),
            None => kinds.push((kind, vec![*no])),
        }
    }
    let mut parts = Vec::new();
    for (kind, mut nos) in kinds {
        nos.sort_unstable();
        nos.dedup();
        let mut runs: Vec<String> = Vec::new();
        let mut i = 0;
        while i < nos.len() {
            let start = nos[i];
            while i + 1 < nos.len() && nos[i + 1] == nos[i] + 1 {
                i += 1;
            }
            runs.push(if nos[i] > start { format!("{start} Thru {}", nos[i]) } else { start.to_string() });
            i += 1;
        }
        parts.push(format!("{kind} {}", runs.join(" + ")));
    }
    parts.join(" + ")
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
