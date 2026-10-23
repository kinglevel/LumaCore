//! Tree-navigation list functions: dot-paths and the context list.
//!
//! Overviews are drawn from the tree itself — the structural nodes
//! (`nav_children_at`) plus the live collections (`vfs::dynamic_children_at`)
//! — so they can't drift from it. A few nodes have a richer listing of their
//! own (fixtures, universes, users, property sheets).

use lumacore_tree::TreeNode;
use nu_ansi_term::{Color, Style};

use crate::navigation::nav_children_at;
use crate::errors::render_code;
use crate::text::PropertySheet;
use crate::vfs::dynamic_children_at;
use crate::tree_path::{Node, TreePath};
use crate::{AppState, ReplContext};

use super::list_fixtures::{list_fixture_types, list_fixtures};
use super::list_misc::list_users;
use super::list_runtime::{
    list_global_variables, list_network, list_network_engines, list_rendering_engines,
    list_lumasession, list_session_engines, list_shells,
};
use super::list_universes::list_universes;
use crate::navigation::Navigation;

/// List objects in the current context.
///
/// Bare `list` shows the root summary overview (data objects, not tree navigation).
/// Use `ls` for tree navigation listing.
pub fn list_context(state: &AppState, recursive: bool) -> String {
    if recursive {
        // /r = full recursive tree (old behavior)
        return match &state.context {
            ReplContext::Patch => tree_overview(state, &Node::Patch.path(), 3),
            _ => tree_overview(state, "Root", 3),
        };
    }

    // Inside the Claims place (`cd Claims`): every claimed object.
    if state.tree_path() == Node::Claims.tree_path() {
        return crate::commands::list_claims(state);
    }
    match &state.context {
        // In Fixture context, bare `list` = list fixtures (context-scoped)
        ReplContext::Fixture => list_fixtures(state),
        ReplContext::Patch => tree_overview(state, &Node::Patch.path(), 1),
        ReplContext::Universe => list_universes(state),
        ReplContext::UserProfile => list_users(state),
        ReplContext::Group => crate::commands::groups::list_groups(state),
        ReplContext::Sequence
        | ReplContext::Executor
        | ReplContext::Effect
        | ReplContext::Macro
        | ReplContext::Preset
        | ReplContext::Timecode
        | ReplContext::EffectEngine
        | ReplContext::DataPool => {
            // Stub: list the context type when fully wired
            format!("{} list — not yet populated.", state.context)
        }
        ReplContext::Custom(s) if s == "ShowData" => {
            tree_overview(state, &Node::ShowData.path(), if recursive { 3 } else { 1 })
        }
        ReplContext::Custom(s) if s.starts_with("Fixture ") => {
            // Scoped to a specific fixture instance: show its details
            let id_str = s.trim_start_matches("Fixture ");
            if let Ok(id) = id_str.parse::<u32>() {
                super::list_fixtures::list_fixture_detail(state, id)
            } else {
                list_fixtures(state)
            }
        }
        ReplContext::Custom(_) => list_fixtures(state),
    }
}

/// The patch's children (`list patch`).
pub fn list_patch_children(state: &AppState) -> String {
    tree_overview(state, &Node::Patch.path(), 1)
}

/// The tree path for a dot-path typed in any case (`root.showdata.patch` →
/// `Root.ShowData.Devices.Dmx.Patch` is not guessed: each name must be declared, each number exist).
fn canonical_path(state: &AppState, path: &str) -> Option<TreePath> {
    if !path.split('.').next().is_some_and(|s| s.eq_ignore_ascii_case("root")) {
        return None;
    }
    let path = TreePath::parse_dot(path)?;
    crate::vfs::path_exists(&state.root, &path).then_some(path)
}

/// How many children a node has (live collection, else structural).
fn child_count(state: &AppState, path: &str) -> usize {
    dynamic_children_at(path, &state.root)
        .map(|c| c.len())
        .unwrap_or_else(|| {
            nav_children_at(path)
                .iter()
                .filter(|c| *c != "Drives")
                .count()
        })
}

/// Live children listed per node before "… n more".
const MAX_LISTED: usize = 20;

/// An overview of the tree below `path`, `depth` levels deep, drawn from
/// the tree itself: structural nodes with their child counts, then the live
/// objects of a collection.
pub fn tree_overview(state: &AppState, path: &str, depth: usize) -> String {
    let name = path.rsplit('.').next().unwrap_or(path);
    let mut lines = vec![format!(
        "{}  {}",
        Style::new().bold().paint(name),
        count_label(child_count(state, path))
    )];
    overview_level(state, path, depth, "", &mut lines);
    if path == "Root" {
        let show = &state.root.show_data.show_settings.show_name;
        lines.push(String::new());
        lines.push(
            Style::new()
                .dimmed()
                .paint(format!(
                    "Show: \"{}\"  —  'list /r' for the deeper tree, 'list root.showdata' to drill down",
                    if show.is_empty() { "untitled" } else { show }
                ))
                .to_string(),
        );
    }
    lines.join("\n")
}

fn count_label(n: usize) -> String {
    if n == 0 {
        Style::new().dimmed().paint("empty").to_string()
    } else {
        Color::Green.paint(n.to_string()).to_string()
    }
}

fn overview_level(
    state: &AppState,
    path: &str,
    depth: usize,
    prefix: &str,
    lines: &mut Vec<String>,
) {
    if depth == 0 {
        return;
    }
    // (label, child path to descend into)
    let mut rows: Vec<(String, Option<String>)> = nav_children_at(path)
        .into_iter()
        .filter(|c| c != "Drives")
        .map(|c| {
            let child = format!("{path}.{c}");
            let label = format!(
                "{}  {}",
                Color::Cyan.paint(&c),
                count_label(child_count(state, &child))
            );
            (label, Some(child))
        })
        .collect();
    if let Some(dynamic) = dynamic_children_at(path, &state.root) {
        let more = dynamic.len().saturating_sub(MAX_LISTED);
        for child in dynamic.iter().take(MAX_LISTED) {
            // A named sub-folder (a profile's Settings / Content) by its name.
            if child.node_type == "Folder" {
                let sub = format!("{path}.{}", child.display_name);
                let n = if child.is_leaf { 0 } else { child_count(state, &sub) };
                rows.push((format!("{}  {}", Color::Cyan.paint(&child.display_name), count_label(n)), (!child.is_leaf).then_some(sub)));
                continue;
            }
            let mut label = format!(
                "{}  {}",
                Color::Cyan.paint(child.id.to_string()),
                child.display_name
            );
            if child.child_count > 0 {
                label.push_str(&format!("  {}", count_label(child.child_count)));
            }
            rows.push((label, None));
        }
        if more > 0 {
            rows.push((
                Style::new()
                    .dimmed()
                    .paint(format!("… {more} more"))
                    .to_string(),
                None,
            ));
        }
    }
    let last = rows.len().saturating_sub(1);
    for (i, (label, child)) in rows.into_iter().enumerate() {
        let (branch, indent) = if i == last {
            ("└── ", "    ")
        } else {
            ("├── ", "│   ")
        };
        lines.push(format!("{prefix}{branch}{label}"));
        if let Some(child) = child {
            overview_level(
                state,
                &child,
                depth - 1,
                &format!("{prefix}{indent}"),
                lines,
            );
        }
    }
}

/// Dot-path listing: `list root.showdata`, `list root.runtime.health`, …
pub fn list_dot_path(state: &AppState, path: &str, recursive: bool) -> String {
    let Some(canonical) = canonical_path(state, path) else {
        return render_code(format!(
            "Unknown path '{path}'. 'list root' shows the tree; 'cd' moves through it."
        ), "E-SET-004");
    };
    match canonical.node() {
        Node::Fixtures if canonical.entry().is_none() => list_fixtures(state),
        Node::FixtureTypes if canonical.entry().is_none() => list_fixture_types(state),
        Node::Universes if canonical.entry().is_none() => list_universes(state),
        Node::UserProfiles if canonical.entry().is_none() => list_users(state),
        Node::ShowSettings => {
            let ss = &state.root.show_data.show_settings;
            let or_dash = |s: &str| {
                if s.is_empty() {
                    "\u{2014}".to_string()
                } else {
                    s.to_string()
                }
            };
            PropertySheet::new("ShowSettings")
                .field("Name", &or_dash(&ss.show_name))
                .field("Description", &or_dash(&ss.description))
                .field("Created with", &or_dash(&ss.created_with_version))
                .field("Format", &ss.format_version.to_string())
                .field("Saved by", &or_dash(&ss.saved_with_commit))
                .build()
        }
        Node::Network => list_network(state),
        Node::NetworkEngines => list_network_engines(state),
        Node::LoggedIn => list_shells(state),
        Node::LumaSessions => list_lumasession(state),
        Node::SessionEngines => list_session_engines(state),
        Node::RenderingEngines => list_rendering_engines(state),
        Node::GlobalVariables => list_global_variables(state),
        Node::Journal => super::journal(state, &[]),
        Node::Health => super::health(state, &[]),
        Node::StationSettings => {
            let st = &state.root.station_settings;
            PropertySheet::new("StationSettings")
                .field("Station", &st.station_name)
                .field("CID", &st.sacn_cid.to_string())
                .field("DataDir", &st.data_dir.display().to_string())
                .build()
        }
        Node::Library => super::list_misc::list_library_all(state),
        _ => tree_overview(state, &canonical.to_string(), if recursive { 3 } else { 1 }),
    }
}

/// List structure inside a show file on disk: list show "demo"
pub fn list_show_file(name: &str, recursive: bool) -> String {
    let name_clean = name.trim_end_matches(".luma");
    let path = format!("library/shows/{name_clean}.luma");

    let contents = match lumacore_luma::drive::read(&path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| lumacore_luma::show_file::decode(&bytes).map(|c| (bytes.len(), c.json)))
    {
        Ok(c) => c,
        Err(e) => return render_code(format!("Cannot read '{path}': {e}"), "E-CMD-012"),
    };

    let (_, contents) = contents;
    let sd: lumacore_tree::ShowData = match serde_json::from_str(&contents) {
        Ok(sd) => sd,
        Err(e) => return render_code(format!("Invalid show file '{path}': {e}"), "E-CMD-012"),
    };

    let bold = |s: &str| Style::new().bold().paint(s).to_string();
    let cyan = |s: &str| Color::Cyan.paint(s).to_string();
    let dim = |s: &str| Style::new().dimmed().paint(s).to_string();
    let green = |s: &str| Color::Green.paint(s).to_string();

    let count = |n: usize, label: &str| -> String {
        if n == 0 {
            dim(&format!("{n} {label}"))
        } else {
            green(&format!("{n} {label}"))
        }
    };

    // The numbered ones; a folder is not a fixture (decisions §40 "40d hardened").
    let fixture_count = sd.devices.dmx.patch.fids().len();
    let ft_count = sd.devices.dmx.patch.fixture_types.len();
    let universe_count = sd.devices.dmx.patch.universes.len();
    let line_count = sd.protocols.all_lines().count();
    let user_count = sd.user_profiles.len();

    let show_name = if sd.show_settings.show_name.is_empty() {
        "untitled"
    } else {
        &sd.show_settings.show_name
    };

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "{}  {}",
        bold(&format!("ShowData: {name_clean}")),
        dim(&format!("\"{}\"", show_name))
    ));

    if !recursive {
        // Direct children only
        lines.push(format!(
            "├── {}  {}",
            cyan("GDTF"),
            count(ft_count, "imported")
        ));
        lines.push(format!(
            "├── {}  {}  {}  {}",
            cyan("Patch"),
            count(fixture_count, "fixtures"),
            count(ft_count, "types"),
            count(universe_count, "universes"),
        ));
        lines.push(format!(
            "├── {}  {}",
            cyan("Protocols"),
            count(line_count, "lines")
        ));
        lines.push(format!("├── {}  {}", cyan("Datapool"), dim("\u{2014}")));
        lines.push(format!(
            "├── {}  {}",
            cyan("UserProfiles"),
            count(user_count, "users")
        ));
        lines.push(format!(
            "└── {}  {}",
            cyan("ShowSettings"),
            dim(&format!("\"{}\"", show_name))
        ));
    } else {
        // Recursive
        lines.push(format!(
            "├── {}  {}",
            cyan("GDTF"),
            count(ft_count, "imported")
        ));

        lines.push(format!("├── {}", cyan("Patch")));
        lines.push(format!(
            "│   ├── {}  {}",
            cyan("Fixtures"),
            count(fixture_count, "patched")
        ));
        for (i, (_, f)) in sd.devices.dmx.patch.fixtures.iter().enumerate() {
            let is_last = i == fixture_count - 1;
            let c = if is_last { "└" } else { "├" };
            let fname = f.meta.name().unwrap_or("<unnamed>");
            let loc = f
                .primary_break()
                .map(|b| format!("{}.{:03}", b.universe_id, b.start_address))
                .unwrap_or_else(|| "\u{2014}".to_string());
            lines.push(format!(
                "│   │   {}── {}  {}  {}",
                c,
                cyan(&f.handle()),
                fname,
                dim(&loc)
            ));
        }
        lines.push(format!(
            "│   ├── {}  {}",
            cyan("FixtureTypes"),
            count(ft_count, "loaded")
        ));
        for (i, (no, ft)) in sd.devices.dmx.patch.fixture_types.iter().enumerate() {
            let is_last = i == ft_count - 1;
            let c = if is_last { "└" } else { "├" };
            let ftname = ft.meta.name().unwrap_or("<unnamed>");
            lines.push(format!(
                "│   │   {}── {}  {} \u{2014} {}",
                c,
                cyan(&format!("{no}")),
                ftname,
                dim(&ft.manufacturer)
            ));
        }
        lines.push(format!(
            "│   └── {}  {}",
            cyan("Universe"),
            count(universe_count, "created")
        ));
        for (i, (no, u)) in sd.devices.dmx.patch.universes.iter().enumerate() {
            let is_last = i == universe_count - 1;
            let c = if is_last { "└" } else { "├" };
            let uname = u.meta.name().unwrap_or("<unnamed>");
            let occupied = u.occupied_count();
            let pct = if occupied > 0 {
                (occupied * 100) / 512
            } else {
                0
            };
            lines.push(format!(
                "│       {}── {}  {}  {}",
                c,
                cyan(&format!("{no}")),
                uname,
                dim(&format!("{occupied}/512 ({pct}%)"))
            ));
        }

        lines.push(format!("├── {}", cyan("Protocols")));
        lines.push(format!(
            "│   └── {}  {}",
            cyan("DMX"),
            count(line_count, "lines")
        ));

        lines.push(format!(
            "├── {}  {}",
            cyan("UserProfiles"),
            count(user_count, "users")
        ));
        for (i, (uid, profile)) in sd.user_profiles.iter().enumerate() {
            let is_last = i == user_count - 1;
            let c = if is_last { "└" } else { "├" };
            let pname = profile.name().unwrap_or("<unnamed>");
            let role = if profile.locked { "template".to_string() } else { String::new() };
            lines.push(format!(
                "│   {}── {}  {}  {}",
                c,
                cyan(&format!("{uid}")),
                pname,
                dim(&role)
            ));
        }

        lines.push(format!(
            "└── {}  {}",
            cyan("ShowSettings"),
            dim(&format!("\"{}\"", show_name))
        ));
    }

    lines.join("\n")
}
