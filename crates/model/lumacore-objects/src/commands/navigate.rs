//! Tree navigation commands: enter, cd, ls.

use crate::tree_path::{Node, TreePath};
use crate::{AppState, ReplContext};
use crate::claims::Claims;
use crate::navigation::Navigation;

/// `ls -r`: what is below too (the tree shell's `ls` row declares it).
pub const LS_RECURSIVE: crate::commands::flags::FlagDef =
    crate::commands::flags::FlagDef::new("r", "Recursive: what is below too");

/// Handle: Enter [<path> | .. | /]
///
const PATCH_DRAFT_OPEN: &str =
    "The Devices draft has changes: Unclaim Root.ShowData.Devices to keep or discard them before leaving.";

/// Whether leaving the current node would end a changed Patch draft.
fn leaves_changed_patch(state: &AppState) -> bool {
    state.claimable_at_path().as_deref() == Some("Devices")
        && state.is_claimed_by_me("Devices")
        && state.patch_draft_changed()
}

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
/// Linux-style tree browser — lists the children of the current `cd` position,
/// or the children of the specified dot-path if an argument is given.
///
/// Unlike `list`, this is purely about tree navigation structure, not data objects.
/// Output is a simple listing with `/` suffix on directory nodes.
pub fn cmd_ls(state: &AppState, line: &lumacore_syntax::FenceLine) -> String {
    use crate::nav_children_at;
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

    let children = nav_children_at(&dot_path);
    let dyn_children = crate::vfs::dynamic_children_at(&dot_path, &state.root);

    if children.is_empty() && dyn_children.as_ref().map(|d| d.is_empty()).unwrap_or(true) {
        // A node with nothing below: describe it when it is an object or a file.
        {
            if let Some(node_view) = crate::vfs::resolve_node(&dot_path, &state.root) {
                match &node_view {
                    crate::vfs::NodeView::Object {
                        node_type,
                        id,
                        name,
                        ..
                    } => {
                        let name_str = name.as_deref().unwrap_or("");
                        return format!(
                            "{} {} {}  {}",
                            Color::Cyan.paint(node_type.to_string()),
                            Color::White.bold().paint(id.to_string()),
                            if name_str.is_empty() {
                                String::new()
                            } else {
                                format!("\"{}\"", Color::Green.paint(name_str))
                            },
                            Style::new().dimmed().paint("(use 'cat' to view contents)")
                        );
                    }
                    crate::vfs::NodeView::FileInfo {
                        name, size, is_dir, ..
                    } => {
                        if *is_dir {
                            return Style::new().dimmed().paint("(empty directory)").to_string();
                        }
                        return format!(
                            "{}  {}  {}",
                            Color::White.paint(name.as_str()),
                            Style::new()
                                .dimmed()
                                .paint(crate::vfs::format_file_size_pub(*size)),
                            Style::new().dimmed().paint("(use 'cat' to view contents)")
                        );
                    }
                    _ => {}
                }
            }
            return Style::new().dimmed().paint("(empty)").to_string();
        }
    }

    let mut lines: Vec<String> = Vec::new();
    let display_root = dot_path.rsplit('.').next().unwrap_or(&dot_path);
    lines.push(format!("{}/", Color::Cyan.bold().paint(display_root)));

    let max_depth = if recursive { 10 } else { 1 };
    ls_tree_recursive(state, &dot_path, &children, "", &mut lines, 1, max_depth);

    // Append dynamic children (objects or filesystem entries) after structural children
    let is_drives = crate::tree_path::names_drive(&dot_path);
    if let Some(dyn_children) = dyn_children
        && !dyn_children.is_empty()
    {
        let total = dyn_children.len();
        let display_limit = if is_drives { 100 } else { total };
        let truncated = total > display_limit;

        for (i, dc) in dyn_children.iter().take(display_limit).enumerate() {
            let is_last = i == display_limit.min(total) - 1 && children.is_empty();
            let connector = if is_last && (children.is_empty() || i == display_limit.min(total) - 1)
            {
                "└── "
            } else {
                "├── "
            };
            let dim = Style::new().dimmed();

            if is_drives {
                // Filesystem entry rendering
                let name = dc
                    .display_name
                    .split("  (")
                    .next()
                    .unwrap_or(&dc.display_name);
                let size_part = dc
                    .display_name
                    .find("  (")
                    .map(|pos| &dc.display_name[pos..])
                    .unwrap_or("");
                if dc.node_type == "Directory" {
                    lines.push(format!(
                        "{}{}{}/",
                        dim.paint(connector),
                        Color::Cyan.bold().paint(name),
                        "",
                    ));
                } else {
                    lines.push(format!(
                        "{}{}{}",
                        dim.paint(connector),
                        Color::White.paint(name),
                        dim.paint(size_part),
                    ));
                }
            } else if dc.node_type == "Folder" {
                // A named entry (a protocol under its group): by its name, a folder to cd into.
                lines.push(format!("{}{}/", dim.paint(connector), Color::Cyan.bold().paint(&dc.display_name)));
            } else {
                // Standard VFS object rendering
                let id_str = Color::White.bold().paint(dc.id.to_string()).to_string();
                let type_str = dim.paint(format!("({})", dc.node_type)).to_string();
                let name_part = if dc.display_name.contains('"') {
                    let name = dc.display_name.split('"').nth(1).unwrap_or("");
                    if name.is_empty() {
                        String::new()
                    } else {
                        format!("  \"{}\"", Color::Green.paint(name))
                    }
                } else {
                    String::new()
                };
                lines.push(format!(
                    "{}{}{}{}  {}",
                    dim.paint(connector),
                    id_str,
                    name_part,
                    if dc.child_count > 0 {
                        format!("  {}", dim.paint(format!("[{} children]", dc.child_count)))
                    } else {
                        String::new()
                    },
                    type_str
                ));
            }
        }
        if truncated {
            let dim = Style::new().dimmed();
            lines.push(format!(
                "    {}",
                dim.paint(format!("({} more...)", total - display_limit))
            ));
        }
    }

    lines.join("\n")
}

/// Recursively render tree children with box-drawing connectors.
///
/// `prefix` is the indentation string inherited from the parent (e.g. "│   ").
/// `depth` is the current depth, `max_depth` limits recursion.
fn ls_tree_recursive(
    state: &AppState,
    parent_path: &str,
    children: &[String],
    prefix: &str,
    lines: &mut Vec<String>,
    depth: usize,
    max_depth: usize,
) {
    use crate::nav_children_at;
    use nu_ansi_term::{Color, Style};

    for (i, child) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        let child_path = format!("{parent_path}.{child}");
        let grandchildren = nav_children_at(&child_path);
        let meta = ls_node_meta(state, &child_path);

        // `Root.Drives` is the real disk: shown as a folder, read only when
        // the path names it (decisions §29, finding 5).
        let drives = TreePath::parse_dot(&child_path).is_some_and(|p| p == Node::Drives.tree_path());
        let has_dyn_children = drives
            || crate::vfs::dynamic_children_at(&child_path, &state.root)
                .map(|c| !c.is_empty())
                .unwrap_or(false);
        let is_directory = !grandchildren.is_empty() || has_dyn_children;

        let name_styled = if is_directory {
            format!("{}/", Color::Cyan.bold().paint(child.as_str()))
        } else {
            Color::Cyan.paint(child.as_str()).to_string()
        };

        let name_plain = if is_directory {
            format!("{child}/")
        } else {
            child.clone()
        };
        let pad = if name_plain.len() < 20 {
            20 - name_plain.len()
        } else {
            1
        };
        let spaces = " ".repeat(pad);

        let dim = Style::new().dimmed();
        if meta.is_empty() {
            lines.push(format!(
                "{}{}{}",
                dim.paint(prefix),
                dim.paint(connector),
                name_styled
            ));
        } else {
            // Leaf nodes get green meta only when they contain a nonzero count
            let has_count = !is_directory
                && meta.chars().any(|c| c.is_ascii_digit())
                && !meta.starts_with("no ");
            let meta_styled = if has_count {
                Color::Green.paint(meta.as_str()).to_string()
            } else {
                dim.paint(meta.as_str()).to_string()
            };
            lines.push(format!(
                "{}{}{}{}{}",
                dim.paint(prefix),
                dim.paint(connector),
                name_styled,
                spaces,
                meta_styled
            ));
        }

        // Recurse into children if within depth limit
        if !grandchildren.is_empty() && depth < max_depth {
            let new_prefix = format!("{prefix}{child_prefix}");
            ls_tree_recursive(
                state,
                &child_path,
                &grandchildren,
                &new_prefix,
                lines,
                depth + 1,
                max_depth,
            );
        }
    }
}

/// Return a short metadata string for a node at the given dot-path.
///
/// Used by `cmd_ls` to annotate tree entries with live data counts or descriptions.
fn ls_node_meta(state: &AppState, dot_path: &str) -> String {
    let Some(path) = TreePath::parse_dot(dot_path) else {
        return String::new();
    };
    if path.entry().is_some() {
        return String::new();
    }
    match path.node() {
        Node::ShowData => {
            let fx = state.root.show_data.devices.dmx.patch.fixtures.len();
            let uni = state.root.show_data.devices.dmx.patch.universes.len();
            let users = state.root.show_data.user_profiles.len();
            format!("show data and configuration  ({fx} fixtures, {uni} universes, {users} users)")
        }
        Node::Devices => {
            let fx = state.root.show_data.devices.dmx.patch.fixtures.len();
            let uni = state.root.show_data.devices.dmx.patch.universes.len();
            if fx == 0 && uni == 0 {
                "DMX devices and their metadata".to_string()
            } else {
                format!("{fx} fixtures, {uni} universes")
            }
        }
        Node::Patch => {
            let fx = state.root.show_data.devices.dmx.patch.fixtures.len();
            let uni = state.root.show_data.devices.dmx.patch.universes.len();
            if fx == 0 && uni == 0 {
                "fixtures and universes".to_string()
            } else {
                format!("{fx} fixtures, {uni} universes")
            }
        }
        Node::Dmx => "the DMX patch".to_string(),
        Node::DevicesMetadata => "what the device kinds share".to_string(),
        Node::MetadataMvr => {
            let m = &state.root.show_data.devices.metadata.mvr;
            format!("{} layers, {} groups, {} classes, {} files", m.layers.len(), m.groups.len(), m.classes.len(), m.files.len())
        }
        Node::MvrLayers => count(state.root.show_data.devices.metadata.mvr.layers.len(), "layer", "layers"),
        Node::MvrGroups => count(state.root.show_data.devices.metadata.mvr.groups.len(), "group", "groups"),
        Node::MvrClasses => count(state.root.show_data.devices.metadata.mvr.classes.len(), "class", "classes"),
        Node::MvrPositions => count(state.root.show_data.devices.metadata.mvr.positions.len(), "position", "positions"),
        Node::MvrMappingDefinitions => count(state.root.show_data.devices.metadata.mvr.mapping_definitions.len(), "mapping definition", "mapping definitions"),
        Node::MvrFiles => count(state.root.show_data.devices.metadata.mvr.files.len(), "imported file", "imported files"),
        Node::Fixtures => {
            let n = state.root.show_data.devices.dmx.patch.fixtures.len();
            if n == 0 {
                "no fixtures patched".to_string()
            } else if n == 1 {
                "1 fixture patched".to_string()
            } else {
                format!("{n} fixtures patched")
            }
        }
        Node::FixtureTypes => {
            let n = state.root.show_data.devices.dmx.patch.fixture_types.len();
            if n == 0 {
                "no fixture types loaded".to_string()
            } else {
                format!("{n} fixture types loaded")
            }
        }
        Node::Universes => {
            let n = state.root.show_data.devices.dmx.patch.universes.len();
            if n == 0 {
                "no universes".to_string()
            } else if n == 1 {
                "1 universe".to_string()
            } else {
                format!("{n} universes")
            }
        }
        Node::Protocols => {
            let n = state.root.show_data.protocols.all_lines().count();
            match n {
                0 => "networks and protocol lines".to_string(),
                1 => "1 line".to_string(),
                _ => format!("{n} lines"),
            }
        }
        Node::DmxProtocols => "protocols whose lines carry universes".to_string(),
        Node::DataProtocols => "protocols whose messages become command lines".to_string(),
        Node::Bidirectional => "bidirectional protocol endpoints".to_string(),
        Node::Playback => "effects, executors, park".to_string(),
        Node::Sequences => {
            let n = state.root.show_data.sequences().count();
            if n == 0 {
                "no sequences".to_string()
            } else {
                format!("{n} sequences")
            }
        }
        Node::PlaybacksLive => "what runs, saved: sequences and masters".to_string(),
        Node::LiveMasters => "the grand masters and the group masters".to_string(),
        Node::Effects => {
            let n = state.root.show_data.playback.effects.len();
            if n == 0 {
                "no effects".to_string()
            } else {
                format!("{n} effects")
            }
        }
        Node::TimingPresets => {
            let n: usize = state.root.show_data.datapools.values().map(|d| d.timing_presets.len()).sum();
            if n == 0 {
                "no timing presets".to_string()
            } else {
                format!("{n} timing presets")
            }
        }
        Node::CurvePresets => {
            let n: usize = state.root.show_data.datapools.values().map(|d| d.curve_presets.len()).sum();
            if n == 0 {
                "no curve presets".to_string()
            } else {
                format!("{n} curve presets")
            }
        }
        Node::Executors => {
            let n = state.root.show_data.playback.executors.len();
            if n == 0 {
                "no executors".to_string()
            } else {
                format!("{n} executors")
            }
        }
        Node::DataPools => {
            let groups: usize = state
                .root
                .show_data
                .datapools
                .values()
                .map(|dp| dp.groups.len())
                .sum();
            if groups == 0 {
                "datapools and pool priority orders".to_string()
            } else {
                format!("{groups} groups")
            }
        }
        Node::DatapoolPriorityOrders => {
            let n = state.root.show_data.datapool_priority_orders.len();
            if n == 0 {
                "no datapool priority orders".to_string()
            } else {
                format!("{n} datapool priority orders")
            }
        }
        // This datapool's groups (the schema path: the first datapool's, as `ls` lists).
        Node::Groups => {
            let datapools = &state.root.show_data.datapools;
            let dp = match path.datapool() {
                Some(n) => datapools.get(&n),
                None => datapools.values().next(),
            };
            let n = dp.map_or(0, |dp| dp.groups.len());
            if n == 0 {
                "no groups".to_string()
            } else if n == 1 {
                "1 group".to_string()
            } else {
                format!("{n} groups")
            }
        }
        Node::Presets => {
            let n: usize = state.root.show_data.datapools.values().map(|d| d.presets.len()).sum();
            if n == 0 { "no presets".to_string() } else { format!("{n} presets") }
        }
        Node::Layouts => "layouts".to_string(),
        Node::Filters => "filters".to_string(),
        Node::Worlds => "worlds".to_string(),
        Node::Macros => "macros".to_string(),
        Node::DistributePresets => "distribution presets".to_string(),
        Node::DynamicStates => "dynamic states".to_string(),
        Node::Plugins => "plugins".to_string(),
        Node::Timecodes => "timecodes".to_string(),
        Node::Priorities => "local priorities".to_string(),
        Node::UserProfiles => {
            let n = state.root.show_data.user_profiles.len();
            if n == 0 {
                "no users".to_string()
            } else if n == 1 {
                "1 user".to_string()
            } else {
                format!("{n} users")
            }
        }
        Node::ShowSettings => {
            let name = &state.root.show_data.show_settings.show_name;
            if name.is_empty() {
                "show configuration".to_string()
            } else {
                format!("\"{}\"", name)
            }
        }
        Node::EffectEngines => {
            let n = state.root.show_data.effect_engines.engines.len();
            if n == 0 {
                "no effect engines".to_string()
            } else {
                format!("{n} effect engines")
            }
        }
        Node::Variables => {
            let n = state.root.show_data.variables.values.len();
            if n == 0 {
                "no show variables".to_string()
            } else {
                format!("{n} show variables")
            }
        }
        Node::Runtime => {
            let users = state.root.runtime.users.len();
            let connections = state.root.runtime.luma_sessions.connections.len();
            let globals = state.root.runtime.global_variables.len();
            format!(
                "live execution state  ({users} users, {connections} connections, {globals} global vars)"
            )
        }
        Node::Network => {
            let n = &state.root.runtime.network;
            format!(
                "{} interfaces, {} lines, {} sources",
                n.interfaces.len(),
                n.lines.len(),
                n.sources.len()
            )
        }
        Node::LumaSessions => {
            let ls = &state.root.runtime.luma_sessions;
            format!(
                "{}, {} stations seen, {} members",
                crate::session_link::status_word(state),
                ls.discovery.len().saturating_sub(1),
                ls.members.len()
            )
        }
        Node::NetworkEngines => {
            let n = state.root.runtime.network_engines.len();
            if n == 0 {
                "no network engines".to_string()
            } else {
                format!("{n} network engines")
            }
        }
        Node::LoggedIn => {
            let n = state.root.runtime.users.len();
            if n == 0 {
                "no users logged in".to_string()
            } else if n == 1 {
                "1 user".to_string()
            } else {
                format!("{n} users")
            }
        }
        Node::SessionEngines => {
            let n = state.root.runtime.session_engines.len();
            if n == 0 {
                "no session engines".to_string()
            } else {
                format!("{n} session engines")
            }
        }
        Node::RenderingEngines => {
            let n = state.root.runtime.rendering_engines.len();
            if n == 0 {
                "no rendering engines".to_string()
            } else {
                format!("{n} rendering engines")
            }
        }
        Node::StationDevices => "reserved: what devices report back".to_string(),
        Node::Playbacks => {
            let n = state.root.show_data.playbacks.live.sequences.len();
            if n == 0 {
                "nothing running".to_string()
            } else {
                format!("{n} running sequences")
            }
        }
        Node::GlobalVariables => {
            let n = state.root.runtime.global_variables.len();
            if n == 0 {
                "no global variables".to_string()
            } else {
                format!("{n} global variables")
            }
        }
        Node::Journal => format!("{} recent changes", state.root.runtime.journal.len()),
        Node::Health => format!(
            "{} threads reporting",
            state.root.runtime.health.snapshot().len()
        ),
        Node::UIEngines => "live presentation engines".to_string(),
        Node::Resources => "shared assets".to_string(),
        Node::Diagnostics => "system-wide diagnostics".to_string(),
        Node::StationSettings => {
            let name = &state.root.station_settings.station_name;
            if name.is_empty() {
                "station configuration".to_string()
            } else {
                format!("station: {name}")
            }
        }
        Node::Drives => "browse filesystem".to_string(),
        Node::Library => "gdtf, shows, presets, mvr".to_string(),
        Node::LibraryGdtf => {
            let n = super::list::count_library_gdtf();
            if n == 0 {
                "no GDTF files on disk".to_string()
            } else if n == 1 {
                "1 GDTF file".to_string()
            } else {
                format!("{n} GDTF files")
            }
        }
        Node::LibraryShows => "saved show files (.luma)".to_string(),
        Node::LibraryPresets => "preset files".to_string(),
        Node::LibraryMvr => "MVR scene files".to_string(),
        // A logged-in user's undo steps and screens (decisions §10).
        Node::UserUndo | Node::UserScreens => {
            let Some(user) = path
                .numbered(Node::LoggedIn)
                .and_then(|u| state.root.show_data.users.get(&u))
                .map(|u| u.login.as_str())
            else {
                return String::new();
            };
            if path.node() == Node::UserUndo {
                match state.root.runtime.user_ref(user).map_or(0, |r| r.undo.len()) {
                    1 => "1 step".to_string(),
                    n => format!("{n} steps"),
                }
            } else {
                let n = state.root.runtime.live_screens.read().map_or(0, |s| s.for_user(user).count());
                format!("{n} screens")
            }
        }
        _ => String::new(),
    }
}

/// `n` things, in words (`no layers`, `1 layer`, `3 layers`).
fn count(n: usize, one: &str, many: &str) -> String {
    match n {
        0 => format!("no {many}"),
        1 => format!("1 {one}"),
        _ => format!("{n} {many}"),
    }
}
