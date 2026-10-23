//  ██████╗ ██████╗ ██╗   ██╗███╗   ██╗████████╗
// ██╔════╝██╔═══██╗██║   ██║████╗  ██║╚══██╔══╝
// ██║     ██║   ██║██║   ██║██╔██╗ ██║   ██║
// ██║     ██║   ██║██║   ██║██║╚██╗██║   ██║
// ╚██████╗╚██████╔╝╚██████╔╝██║ ╚████║   ██║
//  ╚═════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═══╝   ╚═╝
// FILE: count

//! Counting what is below a path.
//!
//! Owns:          du and wc: how much is below a path.
//! Doesn't own:   the tree's walk (vfs).
//! Entry points:  cmd_du, cmd_wc.
//! Decisions:     §33 18.

use lumacore_syntax::FenceLine;

use lumacore_objects::AppState;
use lumacore_objects::vfs;

use crate::print::vfs_resolve_dot_path;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// du [<path>] — Recursive object count ("disk usage").
pub(super) fn cmd_du(state: &AppState, line: &FenceLine) -> String {
    use nu_ansi_term::{Color, Style};

    let dot_path = vfs_resolve_dot_path(state, line.arg(0));
    let counts = vfs::count_objects(&state.root, &dot_path);

    if counts.is_empty() {
        return Style::new().dimmed().paint("0 objects").to_string();
    }

    let total: usize = counts.iter().map(|(_, n)| n).sum();
    let parts: Vec<String> = counts
        .iter()
        .map(|(t, n)| format!("{n} {}", t.to_lowercase()))
        .collect();
    format!(
        "{} — {} total",
        parts.join(", "),
        Color::Green.bold().paint(format!("{total} objects"))
    )
}

/// wc [<path>] — Count direct children (non-recursive).
pub(super) fn cmd_wc(state: &AppState, line: &FenceLine) -> String {
    use nu_ansi_term::Color;

    let dot_path = vfs_resolve_dot_path(state, line.arg(0));
    let count = vfs::count_direct_children(&dot_path, &state.root);
    format!("{}", Color::Green.bold().paint(format!("{count} children")))
}
