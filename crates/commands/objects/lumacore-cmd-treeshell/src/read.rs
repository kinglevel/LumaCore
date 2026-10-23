// ██████╗ ███████╗ █████╗ ██████╗
// ██╔══██╗██╔════╝██╔══██╗██╔══██╗
// ██████╔╝█████╗  ███████║██║  ██║
// ██╔══██╗██╔══╝  ██╔══██║██║  ██║
// ██║  ██║███████╗██║  ██║██████╔╝
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝
// FILE: read

//! Reading one node of the tree (or a file under Root.Drives).
//!
//! Owns:          cat, head, tail and stat: one node's contents, its first or last entries, its facts.
//! Doesn't own:   the tree's walk (vfs), the printing helpers (print.rs).
//! Entry points:  cmd_cat, cmd_head, cmd_tail, cmd_stat.
//! Decisions:     §31, §33 18.

use lumacore_syntax::FenceLine;

use lumacore_objects::AppState;
use lumacore_objects::vfs;

use crate::print::{chrono_lite, colorize_json, format_json_short, vfs_resolve_dot_path};
use crate::words::LINES;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// cat [<path>] — Display node contents as pretty JSON.
pub(super) fn cmd_cat(state: &AppState, line: &FenceLine) -> String {
    use nu_ansi_term::{Color, Style};

    let dot_path = vfs_resolve_dot_path(state, line.arg(0));

    match vfs::resolve_node(&dot_path, &state.root) {
        Some(vfs::NodeView::Object {
            json,
            node_type,
            id,
            guid,
            name,
        }) => {
            let header = format!(
                "{} {} {}  {}",
                Color::Cyan.bold().paint(node_type),
                Color::White.bold().paint(id.to_string()),
                name.as_deref()
                    .map(|n| format!("\"{}\"", Color::Green.paint(n)))
                    .unwrap_or_default(),
                // Runtime nodes have no GUID.
                guid.get(..8)
                    .map(|g| Style::new().dimmed().paint(format!("guid:{g}")).to_string())
                    .unwrap_or_default()
            );
            let body = colorize_json(&serde_json::to_string_pretty(&json).unwrap_or_default());
            format!("{header}\n{body}")
        }
        Some(vfs::NodeView::Structural { name, children }) => {
            let header = format!(
                "{}/  {}",
                Color::Cyan.bold().paint(&name),
                Style::new()
                    .dimmed()
                    .paint(format!("{} children", children.len()))
            );
            let listing: Vec<String> = children.iter().map(|c| format!("  {c}")).collect();
            format!("{header}\n{}", listing.join("\n"))
        }
        Some(vfs::NodeView::FileInfo {
            path,
            name,
            size,
            is_dir,
            modified,
            extension,
        }) => {
            if is_dir {
                // Directory — list children like ls
                let header = format!(
                    "{}/  {}",
                    Color::Cyan.bold().paint(&name),
                    Style::new().dimmed().paint("(directory)")
                );
                return format!("{header}\n  Use 'ls' to list contents.");
            }
            // File — check if text
            let is_text = extension
                .as_deref()
                .map(vfs::is_text_extension)
                .unwrap_or(false)
                || vfs::looks_like_text(&path);

            if is_text {
                let header = format!(
                    "{}  {}",
                    Color::Cyan.bold().paint(&name),
                    Style::new()
                        .dimmed()
                        .paint(format!("({})", vfs::format_file_size_pub(size)))
                );
                match lumacore_luma::drive::read_to_string(&path) {
                    Ok(contents) => {
                        // Cap at 500 lines for sanity
                        let lines: Vec<&str> = contents.lines().take(500).collect();
                        let truncated = contents.lines().count() > 500;
                        let body = lines.join("\n");
                        let suffix = if truncated {
                            format!(
                                "\n{}",
                                Style::new().dimmed().paint("... (truncated at 500 lines)")
                            )
                        } else {
                            String::new()
                        };
                        format!("{header}\n{body}{suffix}")
                    }
                    Err(e) => format!(
                        "{header}\n{}",
                        Color::Red.paint(format!("Error reading file: {e}"))
                    ),
                }
            } else {
                // Binary file — show metadata only
                let dim = Style::new().dimmed();
                let mod_str = modified
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| {
                        let secs = d.as_secs();
                        // Simple date formatting
                        chrono_lite(secs).to_string()
                    })
                    .unwrap_or_else(|| "unknown".to_string());
                let lines = [
                    format!("  {}  {}", dim.paint("File:"), Color::Cyan.paint(&name)),
                    format!(
                        "  {}  {}",
                        dim.paint("Size:"),
                        vfs::format_file_size_pub(size)
                    ),
                    format!("  {}  {}", dim.paint("Modified:"), mod_str),
                    format!("  {}  {}", dim.paint("Path:"), path.display()),
                    format!(
                        "  {}  {}",
                        dim.paint("Type:"),
                        extension.as_deref().unwrap_or("(binary)")
                    ),
                ];
                lines.join("\n")
            }
        }
        None => lumacore_objects::errors::render_code(
            format!("cat: '{}': No such node", line.arg(0).unwrap_or(".")),
            "E-SET-004",
        ),
    }
}

/// head <path> [-n N] — First N entries of a collection (default 10).
pub(super) fn cmd_head(state: &AppState, line: &FenceLine) -> String {
    cmd_head_tail(state, line, true)
}

/// tail <path> [-n N] — Last N entries of a collection (default 10).
pub(super) fn cmd_tail(state: &AppState, line: &FenceLine) -> String {
    cmd_head_tail(state, line, false)
}

fn cmd_head_tail(state: &AppState, line: &FenceLine, is_head: bool) -> String {
    use nu_ansi_term::{Color, Style};

    let cmd_name = if is_head { "head" } else { "tail" };

    let count = match line.value(LINES.name).map(str::parse::<usize>) {
        None => 10,
        Some(Ok(n)) => n,
        Some(Err(_)) => {
            return lumacore_objects::errors::render_code(
                format!("{cmd_name} -n takes a number"),
                "E-SYN-014",
            );
        }
    };
    let path_arg = line.arg(0);

    let dot_path = vfs_resolve_dot_path(state, path_arg);
    let dyn_children = vfs::dynamic_children_at(&dot_path, &state.root);

    match dyn_children {
        Some(children) if !children.is_empty() => {
            let selected: Vec<&vfs::DynChild> = if is_head {
                children.iter().take(count).collect()
            } else {
                let skip = children.len().saturating_sub(count);
                children.iter().skip(skip).collect()
            };

            let dim = Style::new().dimmed();
            let header = format!(
                "{} {}/{} objects",
                dim.paint(format!("{cmd_name}:")),
                selected.len(),
                children.len()
            );
            let mut lines = vec![header];
            for dc in selected {
                let type_str = dim.paint(format!("({})", dc.node_type));
                lines.push(format!(
                    "  {}  {}",
                    Color::White.bold().paint(dc.display_name.clone()),
                    type_str
                ));
            }
            lines.join("\n")
        }
        _ => {
            // Try as an object — show first/last N fields of JSON
            if let Some(vfs::NodeView::Object { json, .. }) =
                vfs::resolve_node(&dot_path, &state.root)
            {
                if let Some(obj) = json.as_object() {
                    let keys: Vec<&String> = obj.keys().collect();
                    let selected: Vec<&&String> = if is_head {
                        keys.iter().take(count).collect()
                    } else {
                        let skip = keys.len().saturating_sub(count);
                        keys.iter().skip(skip).collect()
                    };
                    let mut lines = Vec::new();
                    for key in selected {
                        let val = &obj[key.as_str()];
                        lines.push(format!(
                            "  {}: {}",
                            Color::Cyan.paint(key.as_str()),
                            format_json_short(val)
                        ));
                    }
                    lines.join("\n")
                } else {
                    lumacore_objects::errors::render_code(
                        format!("{cmd_name}: node is not a collection"),
                        "E-CMD-010",
                    )
                }
            } else {
                lumacore_objects::errors::render_code(
                    format!(
                        "{cmd_name}: '{}': No such node or empty collection",
                        path_arg.unwrap_or(".")
                    ),
                    "E-SET-004",
                )
            }
        }
    }
}

/// stat [<path>] — Metadata property sheet.
pub(super) fn cmd_stat(state: &AppState, line: &FenceLine) -> String {
    use nu_ansi_term::{Color, Style};

    let dot_path = vfs_resolve_dot_path(state, line.arg(0));

    match vfs::resolve_node(&dot_path, &state.root) {
        Some(vfs::NodeView::Object {
            json,
            node_type,
            id,
            guid,
            name,
        }) => {
            let dim = Style::new().dimmed();
            let size = serde_json::to_string(&json).map(|s| s.len()).unwrap_or(0);
            let children = json.as_object().map(|o| o.len()).unwrap_or(0);
            let lines = [
                format!("  {}  {}", dim.paint("Type:"), Color::Cyan.paint(node_type)),
                format!(
                    "  {}    {}",
                    dim.paint("ID:"),
                    Color::White.bold().paint(id.to_string())
                ),
                format!("  {}  {}", dim.paint("GUID:"), guid),
                format!(
                    "  {}  {}",
                    dim.paint("Name:"),
                    name.as_deref().unwrap_or("(unnamed)")
                ),
                format!(
                    "  {}  {}",
                    dim.paint("Path:"),
                    lumacore_objects::tree_path::slashed(&dot_path)
                ),
                format!("  {}  {}", dim.paint("Fields:"), children),
                format!("  {}  {} bytes", dim.paint("Size:"), size),
            ];
            lines.join("\n")
        }
        Some(vfs::NodeView::Structural { name, children }) => {
            let dim = Style::new().dimmed();
            let dyn_count = vfs::dynamic_children_at(&dot_path, &state.root)
                .map(|c| c.len())
                .unwrap_or(0);
            let lines = [
                format!(
                    "  {}  {}",
                    dim.paint("Type:"),
                    Color::Cyan.paint("Directory")
                ),
                format!("  {}  {}", dim.paint("Name:"), name),
                format!(
                    "  {}  {}",
                    dim.paint("Path:"),
                    lumacore_objects::tree_path::slashed(&dot_path)
                ),
                format!(
                    "  {}  {} structural, {} objects",
                    dim.paint("Children:"),
                    children.len(),
                    dyn_count
                ),
            ];
            lines.join("\n")
        }
        Some(vfs::NodeView::FileInfo {
            path,
            name,
            size,
            is_dir,
            modified,
            extension,
        }) => {
            let dim = Style::new().dimmed();
            let mod_str = modified
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| chrono_lite(d.as_secs()))
                .unwrap_or_else(|| "unknown".to_string());
            let type_str = if is_dir {
                "Directory".to_string()
            } else {
                extension
                    .as_deref()
                    .map(|e| format!("File (.{e})"))
                    .unwrap_or_else(|| "File".to_string())
            };
            let lines = [
                format!("  {}  {}", dim.paint("Type:"), Color::Cyan.paint(&type_str)),
                format!("  {}  {}", dim.paint("Name:"), name),
                format!(
                    "  {}  {}",
                    dim.paint("Size:"),
                    vfs::format_file_size_pub(size)
                ),
                format!("  {}  {}", dim.paint("Modified:"), mod_str),
                format!("  {}  {}", dim.paint("Path:"), path.display()),
            ];
            lines.join("\n")
        }
        None => lumacore_objects::errors::render_code(
            format!("stat: '{}': No such node", line.arg(0).unwrap_or(".")),
            "E-SET-004",
        ),
    }
}
