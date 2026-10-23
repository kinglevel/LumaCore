// ███╗   ██╗ █████╗ ██╗   ██╗██╗ ██████╗  █████╗ ████████╗███████╗
// ████╗  ██║██╔══██╗██║   ██║██║██╔════╝ ██╔══██╗╚══██╔══╝██╔════╝
// ██╔██╗ ██║███████║██║   ██║██║██║  ███╗███████║   ██║   █████╗
// ██║╚██╗██║██╔══██║╚██╗ ██╔╝██║██║   ██║██╔══██║   ██║   ██╔══╝
// ██║ ╚████║██║  ██║ ╚████╔╝ ██║╚██████╔╝██║  ██║   ██║   ███████╗
// ╚═╝  ╚═══╝╚═╝  ╚═╝  ╚═══╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚══════╝
// FILE: navigate

//! The tree's moves (decisions §31, 0291): `enter` (go and claim), `cd` (go) and `ls` (the rows
//! below, from `tree_rows`, the builder NAV reads too).
//!
//! Owns:          enter, cd, ls and how ls draws its rows.
//! Doesn't own:   the rows (tree_rows), the walk (vfs, navigation), the words' rows (lumacore-cmd-treeshell).
//! Entry points:  cmd_enter, cmd_cd, cmd_ls, LS_RECURSIVE.
//! Decisions:     docs/decisions/0106-tree-paths-typed-node, docs/decisions/0291-nav-mode-ls-rows-and-an-inspector.

use crate::{AppState, ReplContext};
use crate::claims::Claims;
use crate::navigation::Navigation;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// `ls -r`: what is below too (the tree shell's `ls` row declares it).
pub const LS_RECURSIVE: crate::commands::flags::FlagDef =
    crate::commands::flags::FlagDef::new("r", "Recursive: what is below too");

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// Handle: Enter [<path> | .. | /]
///
/// Linux-style navigation that changes tree position, sets context, and auto-claims.
/// - `enter` alone → shows current path and scope
/// - `enter patch` → navigates into Patch, claims it, sets Patch context
/// - `enter showdata.devices.dmx.patch` → multi-segment navigation
/// - `enter ..` → go up one level, unclaim if leaving a claimable node
/// - `enter /` or `enter root` → go to root, unclaim all auto-claims
pub fn cmd_enter(state: &mut AppState, args: &[&str]) -> String {
    use nu_ansi_term::{Color, Style};

    if args.is_empty() {
        // Show current path + scope
        let path = state.path_string();
        let scope = state.scope_label();
        return format!(
            "{} {}  {} {}",
            Style::new().dimmed().paint("Path:"),
            Color::Cyan.paint(&path),
            Style::new().dimmed().paint("Scope:"),
            Color::Cyan.paint(scope)
        );
    }

    let target = args[0];
    let target_lower = target.to_lowercase();

    match target_lower.as_str() {
        ".." => {
            // Go up one level — unclaim if we're leaving a claimable node
            if state.navigation_path.is_empty() {
                return crate::errors::render_code("Already at root /", "E-CMD-010");
            }

            if leaves_changed_patch(state) {
                return crate::errors::render_code(PATCH_DRAFT_OPEN, "E-PATCH-008");
            }
            // Unclaim the node we're leaving (if claimable and claimed by us)
            if let Some(obj) = state.claimable_at_path() {
                // Only unclaim if we're at the exact claimable depth
                // (e.g. leaving Patch, not leaving Patch.Fixtures)
                let _ = state.unclaim(&obj);
            }

            state.navigation_path.pop();
            state.context = state.context_from_path();
            let path = state.path_string();
            crate::text::render_success(&path.to_string())
        }

        "/" | "root" => {
            if leaves_changed_patch(state) {
                return crate::errors::render_code(PATCH_DRAFT_OPEN, "E-PATCH-008");
            }
            // Go to root — unclaim any auto-claimed objects
            if let Some(obj) = state.claimable_at_path() {
                let _ = state.unclaim(&obj);
            }
            state.navigation_path.clear();
            state.context = ReplContext::Fixture;
            crate::text::render_success("/")
        }

        _ => {
            let path = match state.resolve_target(target) {
                Ok(path) => path,
                Err(e) => return crate::errors::render_code(&e, "E-SET-004"),
            };
            let original_path = state.navigation_path.clone();
            let original_claims = state.root.runtime.claims.clone();
            let original_claim_order = state.root.runtime.claim_order.clone();
            // An absolute target leaves the node claimed here first.
            let absolute = crate::tree_path::alias(target).is_some()
                || crate::tree_path::strip_root(target).is_some()
                || target.starts_with('/');
            if absolute && let Some(obj) = state.claimable_at_path() {
                let _ = state.unclaim(&obj);
            }
            state.navigation_path = path;

            // Auto-claim the target node if it's claimable
            if let Some(obj) = state.claimable_at_path()
                && !state.is_claimed_by_me(&obj)
                && let Err(e) = state.claim(&obj)
            {
                // Someone else has it claimed — rollback
                state.navigation_path = original_path;
                state.root.runtime.claims = original_claims;
                state.root.runtime.claim_order = original_claim_order;
                state.context = state.context_from_path();
                // A claim conflict arrives already coded (E-CLAIM-001).
                return crate::errors::reply(&e, "E-CMD-007");
            }

            // Derive context from new path
            state.context = state.context_from_path();
            let path = state.path_string();
            let mut msg = path.to_string();

            // Show claim info if we claimed something
            if let Some(obj) = state.claimable_at_path()
                && state.is_claimed_by_me(&obj)
            {
                msg = format!("{path}  [{obj} claimed]");
            }

            crate::text::render_success(&msg)
        }
    }
}

/// Handle: cd [<path> | .. | /]
///
/// Navigate the tree only — no claiming, no context change.
/// Use `enter` to navigate + claim + set context.
pub fn cmd_cd(state: &mut AppState, line: &lumacore_syntax::FenceLine) -> String {
    let Some(first) = line.arg(0) else {
        // Bare cd — go to root
        state.navigation_path.clear();
        return crate::text::render_success("/");
    };
    // `cd Root 1.5` = `cd Root.1.5`: the Root object's numbers (decisions §33 54).
    let target = match line.arg(1) {
        Some(numbers) if first.eq_ignore_ascii_case(crate::tree_path::Node::Root.name()) => format!("{}.{numbers}", crate::tree_path::Node::Root.name()),
        _ => first.to_string(),
    };
    let target = target.as_str();
    if target == ".." && state.navigation_path.is_empty() {
        return crate::errors::render_code("Already at root /", "E-CMD-010");
    }
    match state.resolve_target(target) {
        Ok(path) => {
            state.navigation_path = path;
            crate::text::render_success(&state.path_string())
        }
        Err(e) => crate::errors::render_code(&e, "E-SET-004"),
    }
}

/// Handle: ls [<path>]
///
/// Linux-style tree browser: the rows below the current `cd` position, or below the path given
/// (`tree_rows::rows`, the builder NAV reads too — decisions 0291). One list, one set of
/// connectors: the last row drawn gets `└──`, whatever kind it is.
pub fn cmd_ls(state: &AppState, line: &lumacore_syntax::FenceLine) -> String {
    use nu_ansi_term::{Color, Style};

    let recursive = line.has(LS_RECURSIVE.name);

    // The same parse as cd (decisions §31); nothing named = where the shell stands.
    let target = line.arg(0).unwrap_or(".");
    let dot_path = match state.resolve_target(target) {
        Ok(path) => crate::tree_path::dot(&path),
        Err(_) => {
            return crate::errors::render_code(
                format!("ls: cannot access '{target}': No such node"),
                "E-SET-004",
            );
        }
    };

    let rows = crate::tree_rows::rows(state, &dot_path);
    if rows.is_empty() {
        return describe_leaf(state, &dot_path);
    }

    let mut lines: Vec<String> = Vec::new();
    let display_root = dot_path.rsplit('.').next().unwrap_or(&dot_path);
    lines.push(format!("{}/", Color::Cyan.bold().paint(display_root)));

    // A drive lists its first entries only.
    let limit = if crate::tree_path::names_drive(&dot_path) { DRIVE_ROWS } else { rows.len() };
    let shown = &rows[..limit.min(rows.len())];
    let max_depth = if recursive { RECURSIVE_DEPTH } else { 1 };
    draw_rows(state, shown, "", &mut lines, 1, max_depth);
    if rows.len() > shown.len() {
        lines.push(format!("    {}", Style::new().dimmed().paint(format!("({} more...)", rows.len() - shown.len()))));
    }
    lines.join("\n")
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `ls` on a node with nothing below: what it is, when it is an object or a file.
fn describe_leaf(state: &AppState, dot_path: &str) -> String {
    use nu_ansi_term::{Color, Style};
    let hint = Style::new().dimmed().paint("(use 'cat' to view contents)");
    match crate::vfs::resolve_node(dot_path, &state.root) {
        Some(crate::vfs::NodeView::Object { node_type, id, name, .. }) => format!(
            "{} {} {}  {hint}",
            Color::Cyan.paint(node_type.to_string()),
            Color::White.bold().paint(id.to_string()),
            name.as_deref()
                .filter(|n| !n.is_empty())
                .map(|n| format!("\"{}\"", Color::Green.paint(n)))
                .unwrap_or_default(),
        ),
        Some(crate::vfs::NodeView::FileInfo { is_dir: true, .. }) => {
            Style::new().dimmed().paint("(empty directory)").to_string()
        }
        Some(crate::vfs::NodeView::FileInfo { name, size, .. }) => format!(
            "{}  {}  {hint}",
            Color::White.paint(name.as_str()),
            Style::new().dimmed().paint(crate::vfs::format_file_size_pub(size)),
        ),
        _ => Style::new().dimmed().paint("(empty)").to_string(),
    }
}

/// Draw `rows` with box-drawing connectors under `prefix` (`│   ` per level above); `ls -r`
/// goes on into the declared nodes below, up to `max_depth`.
fn draw_rows(state: &AppState, rows: &[crate::tree_rows::TreeRow], prefix: &str, lines: &mut Vec<String>, depth: usize, max_depth: usize) {
    use nu_ansi_term::Style;
    let dim = Style::new().dimmed();
    for (i, row) in rows.iter().enumerate() {
        let last = i + 1 == rows.len();
        let connector = if last { "└── " } else { "├── " };
        lines.push(format!("{}{}{}", dim.paint(prefix), dim.paint(connector), row.styled()));
        if depth < max_depth && row.kind == crate::tree_rows::RowKind::Node {
            let below = crate::tree_rows::node_rows(state, &row.path);
            if !below.is_empty() {
                let prefix = format!("{prefix}{}", if last { "    " } else { "│   " });
                draw_rows(state, &below, &prefix, lines, depth + 1, max_depth);
            }
        }
    }
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// How many entries `ls` shows on a drive.
const DRIVE_ROWS: usize = 100;
/// How deep `ls -r` goes.
const RECURSIVE_DEPTH: usize = 10;

const PATCH_DRAFT_OPEN: &str =
    "The Devices draft has changes: Unclaim Root.ShowData.Devices to keep or discard them before leaving.";

/// Whether leaving the current node would end a changed Patch draft.
fn leaves_changed_patch(state: &AppState) -> bool {
    state.claimable_at_path().as_deref() == Some("Devices")
        && state.is_claimed_by_me("Devices")
        && state.patch_draft_changed()
}
