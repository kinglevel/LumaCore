// ██████╗ ██╗███████╗███████╗
// ██╔══██╗██║██╔════╝██╔════╝
// ██║  ██║██║█████╗  █████╗
// ██║  ██║██║██╔══╝  ██╔══╝
// ██████╔╝██║██║     ██║
// ╚═════╝ ╚═╝╚═╝     ╚═╝
// FILE: diff

//! Comparing two nodes of the tree field by field (`--all`: the same ones too).
//!
//! Owns:          diff: two nodes, field by field.
//! Doesn't own:   the tree's walk (vfs), the printing helpers (print.rs).
//! Entry points:  cmd_diff.
//! Decisions:     §33 18.

use lumacore_syntax::FenceLine;

use lumacore_objects::AppState;
use lumacore_objects::vfs;

use crate::print::vfs_resolve_dot_path;
use crate::words::DIFF_ALL;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// diff <path1> <path2> — Compare two objects.
pub(super) fn cmd_diff(state: &AppState, line: &FenceLine) -> String {
    use nu_ansi_term::{Color, Style};

    let _show_all = line.has(DIFF_ALL.name);
    let path_args: Vec<&str> = line.positional.iter().map(String::as_str).collect();
    if path_args.len() < 2 {
        return lumacore_objects::errors::render_code(
            "diff needs two paths: diff <path1> <path2> [--all]",
            "E-SYN-002",
        );
    }

    let dot1 = match vfs_resolve_dot_path(state, Some(path_args[0])) {
        Ok(path) => path,
        Err(e) => return e,
    };
    let dot2 = match vfs_resolve_dot_path(state, Some(path_args[1])) {
        Ok(path) => path,
        Err(e) => return e,
    };

    let node1 = vfs::resolve_node(&dot1, &state.root);
    let node2 = vfs::resolve_node(&dot2, &state.root);

    let (json1, json2) = match (node1, node2) {
        (
            Some(vfs::NodeView::Object { json: j1, .. }),
            Some(vfs::NodeView::Object { json: j2, .. }),
        ) => (j1, j2),
        (None, _) => {
            return lumacore_objects::errors::render_code(
                format!("diff: '{}': No such node", path_args[0]),
                "E-SET-004",
            );
        }
        (_, None) => {
            return lumacore_objects::errors::render_code(
                format!("diff: '{}': No such node", path_args[1]),
                "E-SET-004",
            );
        }
        _ => {
            return lumacore_objects::errors::render_code(
                "diff: both paths must be objects, not directories",
                "E-CMD-006",
            );
        }
    };

    let diffs = vfs::diff_json(&json1, &json2);

    if diffs.is_empty() {
        return Style::new()
            .dimmed()
            .paint("Objects are identical.")
            .to_string();
    }

    let dim = Style::new().dimmed();
    let mut lines = vec![format!(
        "{} {} vs {}",
        dim.paint("diff:"),
        Color::Cyan.paint(path_args[0]),
        Color::Cyan.paint(path_args[1])
    )];
    for (field, old, new) in &diffs {
        lines.push(format!("  {}", Color::White.paint(field.as_str())));
        lines.push(format!(
            "    {} {}",
            Color::Red.paint("-"),
            Color::Red.paint(old.as_str())
        ));
        lines.push(format!(
            "    {} {}",
            Color::Green.paint("+"),
            Color::Green.paint(new.as_str())
        ));
    }
    format!("{} field(s) differ:\n{}", diffs.len(), lines.join("\n"))
}
