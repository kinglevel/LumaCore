// ████████╗██████╗ ███████╗███████╗
// ╚══██╔══╝██╔══██╗██╔════╝██╔════╝
//    ██║   ██████╔╝█████╗  █████╗
//    ██║   ██╔══██╗██╔══╝  ██╔══╝
//    ██║   ██║  ██║███████╗███████╗
//    ╚═╝   ╚═╝  ╚═╝╚══════╝╚══════╝
// FILE: tree

//! Drawing the tree below a path; under Root.Drives the walk stops after 300 ms or 2,000 lines
//! and says so (it runs on the state thread).
//!
//! Owns:          tree: the tree below a path, drawn, and the disk budget of a walk under Root.Drives.
//! Doesn't own:   listing one level (the kernel's ls), the drive door (lumacore-luma drive).
//! Entry points:  cmd_tree.
//! Decisions:     §31, §33 18.

use lumacore_syntax::FenceLine;

use lumacore_objects::AppState;
use lumacore_objects::vfs;

use crate::print::vfs_resolve_dot_path;
use crate::words::TREE_DEPTH;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// tree [<path>] [-d N] — Recursive tree with object counts at leaves.
pub(super) fn cmd_tree(state: &AppState, line: &FenceLine) -> String {
    use nu_ansi_term::Color;

    let mut max_depth: usize = 20;
    let depth_given = match line.value(TREE_DEPTH.name).map(str::parse::<usize>) {
        None => None,
        Some(Ok(d)) => Some(d),
        Some(Err(_)) => {
            return lumacore_objects::errors::render_code("tree -d takes a number", "E-SYN-014");
        }
    };
    if let Some(d) = depth_given {
        max_depth = d;
    }
    let path_arg = line.arg(0);

    let dot_path = vfs_resolve_dot_path(state, path_arg);
    let is_drives = lumacore_objects::tree_path::names_drive(&dot_path);
    // Default depth limit is 5 for Drives, 20 otherwise (unless -d was given)
    if is_drives && depth_given.is_none() {
        max_depth = 5;
    }
    let display_root = dot_path.rsplit('.').next().unwrap_or(&dot_path);

    let mut lines = vec![format!("{}/", Color::Cyan.bold().paint(display_root))];
    let mut disk = DiskBudget::new();
    tree_recursive(state, &dot_path, "", &mut lines, 0, max_depth, &mut disk);
    if disk.cut {
        lines.push(
            nu_ansi_term::Style::new()
                .dimmed()
                .paint(format!(
                    "… stopped after {} lines / {} ms: the disk is big — name a folder or use tree -d 2",
                    DISK_LINES,
                    DISK_TIME.as_millis()
                ))
                .to_string(),
        );
    }
    lines.join("\n")
}

/// How much of the disk one `tree` reads (§31): it runs on the state thread, so a walk of a
/// big drive stops after this long or this many lines and says so.
const DISK_TIME: std::time::Duration = std::time::Duration::from_millis(300);
const DISK_LINES: usize = 2_000;

struct DiskBudget {
    started: std::time::Instant,
    cut: bool,
}

impl DiskBudget {
    fn new() -> Self {
        Self {
            started: std::time::Instant::now(),
            cut: false,
        }
    }

    /// May the walk read one more folder? Once out, it stays out.
    fn more(&mut self, lines: usize) -> bool {
        if !self.cut && (lines >= DISK_LINES || self.started.elapsed() >= DISK_TIME) {
            self.cut = true;
        }
        !self.cut
    }
}

fn tree_recursive(
    state: &AppState,
    parent_path: &str,
    prefix: &str,
    lines: &mut Vec<String>,
    depth: usize,
    max_depth: usize,
    disk: &mut DiskBudget,
) {
    use lumacore_objects::nav_children_at;
    use nu_ansi_term::{Color, Style};

    if depth >= max_depth {
        return;
    }
    if lumacore_objects::tree_path::names_drive(parent_path) && !disk.more(lines.len()) {
        return;
    }

    let static_children = nav_children_at(parent_path);
    let dyn_children = vfs::dynamic_children_at(parent_path, &state.root).unwrap_or_default();
    let dim = Style::new().dimmed();

    for (i, child) in static_children.iter().enumerate() {
        let is_last = i == static_children.len() - 1 && dyn_children.is_empty();
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };
        let child_path = format!("{parent_path}.{child}");
        // `Root.Drives` is the real disk: shown as a folder, read only when the line names it
        // (decisions §25 reach; walking into it from above read the whole disk).
        let on_disk = lumacore_objects::tree_path::names_drive(&child_path)
            && !lumacore_objects::tree_path::names_drive(parent_path);
        let grandchildren = nav_children_at(&child_path);
        let dyn_gc = if on_disk {
            None
        } else {
            vfs::dynamic_children_at(&child_path, &state.root)
        };
        let dyn_gc_count = dyn_gc.as_ref().map(|c| c.len()).unwrap_or(0);

        let has_children = !grandchildren.is_empty() || dyn_gc_count > 0 || on_disk;
        let name_styled = if has_children {
            format!("{}/", Color::Cyan.bold().paint(child.as_str()))
        } else {
            Color::Cyan.paint(child.as_str()).to_string()
        };

        let meta = if dyn_gc_count > 0 {
            format!("  {}", Color::Green.paint(format!("({dyn_gc_count})")))
        } else {
            String::new()
        };

        lines.push(format!(
            "{}{}{}{}",
            dim.paint(prefix),
            dim.paint(connector),
            name_styled,
            meta
        ));

        if has_children && !on_disk {
            let new_prefix = format!("{prefix}{child_prefix}");
            tree_recursive(
                state,
                &child_path,
                &new_prefix,
                lines,
                depth + 1,
                max_depth,
                disk,
            );
        }
    }

    // Dynamic children (objects or filesystem entries) — show as leaf items
    let is_drives = lumacore_objects::tree_path::names_drive(parent_path);
    for (i, dc) in dyn_children.iter().enumerate() {
        let is_last = i == dyn_children.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix_str = if is_last { "    " } else { "│   " };

        if is_drives {
            let name = dc
                .display_name
                .split("  (")
                .next()
                .unwrap_or(&dc.display_name);
            if dc.node_type == "Directory" {
                lines.push(format!(
                    "{}{}{}/",
                    dim.paint(prefix),
                    dim.paint(connector),
                    Color::Cyan.bold().paint(name),
                ));
                // Recurse into directory
                if depth + 1 < max_depth {
                    let child_path = format!("{parent_path}.{name}");
                    let new_prefix = format!("{prefix}{child_prefix_str}");
                    tree_recursive(
                        state,
                        &child_path,
                        &new_prefix,
                        lines,
                        depth + 1,
                        max_depth,
                        disk,
                    );
                }
            } else {
                let size_part = dc
                    .display_name
                    .find("  (")
                    .map(|pos| &dc.display_name[pos..])
                    .unwrap_or("");
                lines.push(format!(
                    "{}{}{}{}",
                    dim.paint(prefix),
                    dim.paint(connector),
                    Color::White.paint(name),
                    dim.paint(size_part),
                ));
            }
        } else {
            lines.push(format!(
                "{}{}{}  {}",
                dim.paint(prefix),
                dim.paint(connector),
                Color::White.paint(dc.display_name.clone()),
                dim.paint(format!("({})", dc.node_type))
            ));
        }
    }
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
