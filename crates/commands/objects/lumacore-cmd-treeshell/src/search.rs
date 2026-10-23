// ███████╗███████╗ █████╗ ██████╗  ██████╗██╗  ██╗
// ██╔════╝██╔════╝██╔══██╗██╔══██╗██╔════╝██║  ██║
// ███████╗█████╗  ███████║██████╔╝██║     ███████║
// ╚════██║██╔══╝  ██╔══██║██╔══██╗██║     ██╔══██║
// ███████║███████╗██║  ██║██║  ██║╚██████╗██║  ██║
// ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝
// FILE: search

//! Searching the tree: `find` by a node's name (and `--type`), `grep` by what it holds.
//!
//! Owns:          find and grep: nodes by name or type, values by text.
//! Doesn't own:   the tree's walk (vfs).
//! Entry points:  cmd_find, cmd_grep.
//! Decisions:     §33 18.

use lumacore_syntax::FenceLine;

use lumacore_objects::AppState;
use lumacore_objects::vfs;

use crate::print::vfs_resolve_dot_path;
use crate::words::FIND_TYPE;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// find <pattern> [--type TYPE] — Search node names.
pub(super) fn cmd_find(state: &AppState, line: &FenceLine) -> String {
    use nu_ansi_term::{Color, Style};

    let pattern = line.arg(0).unwrap_or_default();
    if pattern.is_empty() {
        return lumacore_objects::errors::render_code(
            "find needs what to look for: find \"Front\" [--type Group]",
            "E-SYN-002",
        );
    }
    let type_filter = line.value(FIND_TYPE.name);

    let cwd = lumacore_objects::tree_path::dot(&state.navigation_path);

    let results = vfs::find_by_name(&state.root, &cwd, pattern, type_filter);

    if results.is_empty() {
        return Style::new()
            .dimmed()
            .paint(format!("No matches for '{pattern}'"))
            .to_string();
    }

    let dim = Style::new().dimmed();
    let mut lines: Vec<String> = Vec::new();
    for (path, child) in &results {
        let rel_path = lumacore_objects::tree_path::slashed(path);
        lines.push(format!(
            "  {:<41}  {}  {}",
            rel_path,
            Color::White.bold().paint(child.display_name.clone()),
            dim.paint(format!("({})", child.node_type))
        ));
    }
    let header = format!("{} match(es):", results.len());
    format!("{}\n{}", dim.paint(header), lines.join("\n"))
}

/// grep <pattern> [<path>] — Search object values.
pub(super) fn cmd_grep(state: &AppState, line: &FenceLine) -> String {
    use nu_ansi_term::{Color, Style};

    let Some(pattern) = line.arg(0).filter(|p| !p.is_empty()) else {
        return lumacore_objects::errors::render_code(
            "grep needs what to look for: grep \"Back Light\" [<path>]",
            "E-SYN-002",
        );
    };
    let path_arg = line.arg(1);
    let from = match vfs_resolve_dot_path(state, path_arg) {
        Ok(path) => path,
        Err(e) => return e,
    };

    let results = vfs::grep_values(&state.root, &from, pattern);

    if results.is_empty() {
        return Style::new()
            .dimmed()
            .paint(format!("No matches for '{pattern}'"))
            .to_string();
    }

    let dim = Style::new().dimmed();
    let mut lines: Vec<String> = Vec::new();
    for (path, child, fields) in &results {
        let rel_path = lumacore_objects::tree_path::slashed(path);
        lines.push(format!(
            "  {} {}",
            Color::Cyan.paint(&rel_path),
            dim.paint(format!("({})", child.node_type))
        ));
        for (field, value) in fields {
            lines.push(format!(
                "    {}: {}",
                Color::White.paint(field.as_str()),
                Color::Green.paint(value.as_str())
            ));
        }
    }
    let header = format!("{} object(s) matched:", results.len());
    format!("{}\n{}", dim.paint(header), lines.join("\n"))
}
