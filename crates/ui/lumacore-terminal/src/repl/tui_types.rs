// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// FILE: types

//! Public types, enums, and small utility functions used across the TUI.
//!
//! Owns:          the TUI's shared types and small helpers
//! Doesn't own:   behaviour (the TuiApp files)
//! Entry points:  the types
//! Decisions:     §23 "Step 8"

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use super::tui_look::role;
use lumacore_wire::ViewLayers;
use serde::{Deserialize, Serialize};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl From<ViewLayers> for Layers {
    fn from(l: ViewLayers) -> Self {
        let none = Layers {
            watch: false,
            output: false,
            status: false,
            path: false,
            input: false,
        };
        match l {
            ViewLayers::Full => Layers::default(),
            ViewLayers::Input => Layers {
                input: true,
                ..none
            },
            ViewLayers::Output => Layers {
                output: true,
                ..none
            },
        }
    }
}

/// Which parts of the session a view shows, top to bottom. A full terminal
/// shows everything; a command-line-only or history-only window picks some.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layers {
    pub watch: bool,
    pub output: bool,
    pub status: bool,
    pub path: bool,
    pub input: bool,
}

impl Default for Layers {
    fn default() -> Self {
        Self {
            watch: true,
            output: true,
            status: true,
            path: true,
            input: true,
        }
    }
}

/// Per-view state for compositing a shell session. The session's buffers
/// (input, output, history, suggestions, popups) are shared; each view — a
/// terminal window, the local terminal — has its own size, scroll and layers.
#[derive(Debug, Clone, Default)]
pub struct ViewState {
    /// Output lines scrolled up from the bottom (0 = follow new output).
    pub scroll: usize,
    /// Height of the output area at the last compose (page size for PageUp).
    pub output_height: u16,
    /// Output length at the last compose: a scrolled-up view stays anchored
    /// on the same lines while output grows.
    pub seen_lines: usize,
    /// Draw overlays (suggestion menu, ghost hint, NAV popup). Only the
    /// focused view does, so other screens aren't cluttered.
    pub overlays: bool,
    pub layers: Layers,
}

impl ViewState {
    /// A full, focused view (the local terminal, a lone remote client).
    pub fn focused() -> Self {
        Self {
            output_height: 20,
            overlays: true,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedShellFrame {
    pub ansi: String,
}

/// The TUI's suggestion list (decisions §30 "12b2"): items from
/// `lumacore_suggest`. Enter always runs the line as typed; Tab takes the
/// selected item, ↑ ↓ move while it is open.
pub(super) struct CompletionState {
    pub(super) items: Vec<lumacore_wire::SuggestItem>,
    pub(super) index: usize,
    /// The chars of `original_line` an item's insert replaces.
    pub(super) replace: (usize, usize),
    pub(super) original_line: String,
}

/// A terminal's place in the station's open dialog (decisions §30 "12b"):
/// which field has the cursor, the chosen button, what was typed.
#[derive(Debug, Clone, Default)]
pub(super) struct DialogUi {
    pub(super) id: String,
    /// 0..inputs = an input, then the toggles, then the buttons row.
    pub(super) field: usize,
    pub(super) button: usize,
    pub(super) values: Vec<String>,
    pub(super) toggles: Vec<bool>,
    pub(super) selectors: Vec<usize>,
    /// The tree's ticks (decisions 0293), one per tickable node in order.
    pub(super) ticks: Vec<bool>,
    /// The countdown was paused by a key here.
    pub(super) paused: bool,
}

// *************************************************************************************************
// Key name parser
// *************************************************************************************************
/// A key name ("Ctrl-R", "F5") as crossterm types (the grammar is
/// `lumacore_cmd_keys::parse_key_name`).
pub fn parse_key_name(name: &str) -> Option<(KeyCode, KeyModifiers)> {
    let (code, modifiers) = lumacore_cmd_keys::parse_key_name(name)?;
    let event = to_crossterm(&lumacore_wire::RemoteShellKey {
        code,
        modifiers,
        repeat: None,
    });
    Some((event.code, event.modifiers))
}

/// A client key as a crossterm event (the TUI's input type).
pub fn to_crossterm(key: &lumacore_wire::RemoteShellKey) -> crossterm::event::KeyEvent {
    {
        // Wire bits (Shift 1, Alt 2, Ctrl 4) → crossterm's (Shift, Ctrl, Alt).
        use lumacore_wire::shell::modifiers as wire;
        let mut modifiers = KeyModifiers::NONE;
        if key.modifiers & wire::SHIFT != 0 {
            modifiers |= KeyModifiers::SHIFT;
        }
        if key.modifiers & wire::ALT != 0 {
            modifiers |= KeyModifiers::ALT;
        }
        if key.modifiers & wire::CTRL != 0 {
            modifiers |= KeyModifiers::CONTROL;
        }
        let code = match key.code {
            lumacore_wire::RemoteShellCode::Char(ch) => KeyCode::Char(ch),
            lumacore_wire::RemoteShellCode::Enter => KeyCode::Enter,
            lumacore_wire::RemoteShellCode::Tab => KeyCode::Tab,
            lumacore_wire::RemoteShellCode::Backspace => KeyCode::Backspace,
            lumacore_wire::RemoteShellCode::Delete => KeyCode::Delete,
            lumacore_wire::RemoteShellCode::Home => KeyCode::Home,
            lumacore_wire::RemoteShellCode::End => KeyCode::End,
            lumacore_wire::RemoteShellCode::PageUp => KeyCode::PageUp,
            lumacore_wire::RemoteShellCode::PageDown => KeyCode::PageDown,
            lumacore_wire::RemoteShellCode::Up => KeyCode::Up,
            lumacore_wire::RemoteShellCode::Down => KeyCode::Down,
            lumacore_wire::RemoteShellCode::Left => KeyCode::Left,
            lumacore_wire::RemoteShellCode::Right => KeyCode::Right,
            lumacore_wire::RemoteShellCode::Esc => KeyCode::Esc,
            lumacore_wire::RemoteShellCode::Insert => KeyCode::Insert,
            lumacore_wire::RemoteShellCode::F(n) => KeyCode::F(n),
            lumacore_wire::RemoteShellCode::PrintScreen => KeyCode::PrintScreen,
            lumacore_wire::RemoteShellCode::Pause => KeyCode::Pause,
        };
        crossterm::event::KeyEvent::new(code, modifiers)
    }
}

// *************************************************************************************************
// Abbreviations
// *************************************************************************************************
/// Expand a short form on Space/Enter: when the last word is a short alias of
/// a command or object (up to three letters: `hi`, `g`, `ft`), it becomes the
/// full name (`Highlight`, `Group`, `FixtureType`). The short forms are the
/// registry's aliases — there is no separate abbreviation table.
/// Returns the expanded input if the last word was one, None otherwise.
pub(super) fn try_expand_abbreviation(input: &str) -> Option<String> {
    use crate::core::commands::registry;
    let trimmed = input.trim_end();
    let (prefix, word) = match trimmed.rfind(' ') {
        Some(pos) => (&trimmed[..=pos], &trimmed[pos + 1..]),
        None => ("", trimmed),
    };
    if word.is_empty() || word.len() > 3 || !word.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let full = registry::lookup_word(word)?;
    let label = full.label();
    if label.eq_ignore_ascii_case(word) || full.names()[0].eq_ignore_ascii_case(word) {
        return None; // already the full name
    }
    Some(format!("{prefix}{label}"))
}

/// Classify the first token (verb) of the input for syntax highlighting.
/// Returns a Style to apply to the verb portion.
pub(super) fn classify_verb(verb: &str) -> Style {
    use crate::core::commands::resolve_verb;
    if verb.is_empty() {
        return Style::default();
    }
    match resolve_verb(verb) {
        Ok(_) => Style::default().fg(role::OK), // valid command
        Err(_) => Style::default().fg(role::ALERT),  // invalid command
    }
}
