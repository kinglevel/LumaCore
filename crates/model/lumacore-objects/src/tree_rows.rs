// ████████╗██████╗ ███████╗███████╗    ██████╗  ██████╗ ██╗    ██╗███████╗
// ╚══██╔══╝██╔══██╗██╔════╝██╔════╝    ██╔══██╗██╔═══██╗██║    ██║██╔════╝
//    ██║   ██████╔╝█████╗  █████╗      ██████╔╝██║   ██║██║ █╗ ██║███████╗
//    ██║   ██╔══██╗██╔══╝  ██╔══╝      ██╔══██╗██║   ██║██║███╗██║╚════██║
//    ██║   ██║  ██║███████╗███████╗    ██║  ██║╚██████╔╝╚███╔███╔╝███████║
//    ╚═╝   ╚═╝  ╚═╝╚══════╝╚══════╝    ╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝ ╚══════╝
// FILE: tree_rows

//! The rows below one tree node (decisions 0291): the one builder `ls`, the NAV tree and the
//! NAV inspector read, so they can't drift apart. A row's path step is a declared name, an
//! object's number or a disk name — never a label — so every row opens with the typed path
//! (decisions 0106).
//!
//! Owns:          TreeRow, RowKind, rows / node_rows, what a row prints (plain and styled), a node's meta.
//! Doesn't own:   the walk (vfs, navigation), drawing the tree (ls, the TUI's NAV).
//! Entry points:  rows, node_rows, TreeRow::text, TreeRow::styled, meta_of.
//! Decisions:     docs/decisions/0291-nav-mode-ls-rows-and-an-inspector, docs/decisions/0106-tree-paths-typed-node.

use crate::AppState;
use crate::tree_path::{Node, TreePath};
use crate::vfs::DynChild;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a row stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    /// A declared node of the tree (`Devices`, `Groups`).
    Node,
    /// A numbered object (`Group 3`, `DataPool 1`) or a live entry (a line, a claim).
    Object,
    /// A named entry of a named node (a protocol under its group).
    Folder,
    /// A drive or a directory on one.
    Dir,
    /// A file on a drive.
    File,
}

/// One row below a node, as `ls` prints it and NAV draws it.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeRow {
    pub kind: RowKind,
    /// The path step that opens it: a declared name, an object's number, a disk name.
    pub segment: String,
    /// Its full dot path (`Root.ShowData.DataPools.1`).
    pub path: String,
    /// An object's number.
    pub slot: Option<u32>,
    /// A node's or file's name; an object's stored name (or its live text when it has none).
    pub name: String,
    /// The name is the object's own (shown quoted).
    pub quoted: bool,
    /// An object's kind (`Group`); empty for the rest.
    pub type_name: &'static str,
    /// What `ls` says beside it: a node's counts, an object's children, a file's size.
    pub meta: String,
    /// Something is below it.
    pub branch: bool,
    pub guid: Option<uuid::Uuid>,
    /// A show object's colour as the acting user sees it (decisions 0294), `#rrggbb`; None = its
    /// pool's or none.
    pub paint: Option<String>,
}

/// Names shorter than this are padded so the metas line up (`ls`'s column).
const NAME_COLUMN: usize = 20;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Every row below `dot_path`: its declared nodes first, then its objects or files, in `ls`'s
/// order. Empty for a leaf or a path that doesn't parse.
pub fn rows(state: &AppState, dot_path: &str) -> Vec<TreeRow> {
    let mut out = node_rows(state, dot_path);
    if let Some(children) = crate::vfs::dynamic_children_at(dot_path, &state.root) {
        let start = out.len();
        out.extend(children.iter().map(|c| object_row(dot_path, c)));
        // Show objects wear their colour (decisions 0294): one walk for the rows that have one.
        if out[start..].iter().any(|r| r.guid.is_some()) && dot_path.starts_with(&Node::ShowData.path()) {
            let index = crate::paint::Index::of(&state.root.show_data);
            let painter = crate::paint::Painter::new(&state.root.show_data, state.effective_user_name());
            for r in &mut out[start..] {
                r.paint = r.guid.and_then(|g| painter.of_guid(&index, g)).and_then(|s| s.hex);
            }
        }
    }
    out
}

/// Only the declared nodes below `dot_path` (`ls -r` walks these).
pub fn node_rows(state: &AppState, dot_path: &str) -> Vec<TreeRow> {
    crate::nav_children_at(dot_path)
        .into_iter()
        .map(|name| {
            let path = format!("{dot_path}.{name}");
            // `Root.Drives` is the real disk: a folder, read only when the path names it.
            let drives = TreePath::parse_dot(&path).is_some_and(|p| p == Node::Drives.tree_path());
            let branch = drives
                || !crate::nav_children_at(&path).is_empty()
                || crate::vfs::dynamic_children_at(&path, &state.root).is_some_and(|c| !c.is_empty());
            TreeRow {
                kind: RowKind::Node,
                meta: meta_of(state, &path),
                segment: name.clone(),
                path,
                slot: None,
                name,
                quoted: false,
                type_name: "",
                branch,
                guid: None,
                paint: None,
            }
        })
        .collect()
}

/// A collection's entry as a row.
fn object_row(parent: &str, c: &DynChild) -> TreeRow {
    let (kind, segment, name, meta) = match c.node_type {
        "Folder" => (RowKind::Folder, c.display_name.clone(), c.display_name.clone(), String::new()),
        "Directory" | "Drive" => (RowKind::Dir, c.display_name.clone(), c.display_name.clone(), String::new()),
        "File" => {
            let (name, size) = c.display_name.split_once("  (").unwrap_or((&c.display_name, ""));
            (RowKind::File, name.to_string(), name.to_string(), size.trim_end_matches(')').to_string())
        }
        _ => {
            // A live entry without a stored name keeps its text (`Sequence 3 → Cue 2 (Bob)`).
            let name = match &c.name {
                Some(n) => n.clone(),
                None if c.display_name != c.id.to_string() && !c.display_name.contains('"') => c.display_name.clone(),
                None => c.display_name.split('"').nth(1).unwrap_or("").to_string(),
            };
            let meta = match c.child_count {
                0 => String::new(),
                n => format!("[{n} children]"),
            };
            (RowKind::Object, c.id.to_string(), name, meta)
        }
    };
    TreeRow {
        quoted: kind == RowKind::Object && (c.name.is_some() || c.display_name.contains('"')),
        branch: !c.is_leaf,
        path: format!("{parent}.{segment}"),
        slot: (kind == RowKind::Object).then_some(c.id),
        type_name: if kind == RowKind::Object { c.node_type } else { "" },
        guid: c.guid,
        paint: None,
        kind,
        segment,
        name,
        meta,
    }
}

impl TreeRow {
    /// The object a line names (`Fixture 101`); None for a node, a folder or a file.
    pub fn target(&self) -> Option<String> {
        Some(format!("{} {}", self.type_name, self.slot?))
    }

    /// The row as `ls` prints it, without colour: `1  "Default"  [77 children]  (DataPool)`,
    /// `Groups/             77 groups`, `show.luma  (12 KB)`.
    pub fn text(&self) -> String {
        match self.kind {
            RowKind::Node => {
                let name = if self.branch { format!("{}/", self.name) } else { self.name.clone() };
                if self.meta.is_empty() {
                    name
                } else {
                    let pad = NAME_COLUMN.saturating_sub(name.chars().count()).max(1);
                    format!("{name}{}{}", " ".repeat(pad), self.meta)
                }
            }
            RowKind::Folder | RowKind::Dir => format!("{}/", self.name),
            RowKind::File if self.meta.is_empty() => self.name.clone(),
            RowKind::File => format!("{}  ({})", self.name, self.meta),
            RowKind::Object => {
                let mut out = self.slot.map(|s| s.to_string()).unwrap_or_default();
                match (self.name.is_empty(), self.quoted) {
                    (true, _) => {}
                    (false, true) => out.push_str(&format!("  \"{}\"", self.name)),
                    (false, false) => out.push_str(&format!("  {}", self.name)),
                }
                if !self.meta.is_empty() {
                    out.push_str(&format!("  {}", self.meta));
                }
                out.push_str(&format!("  ({})", self.type_name));
                out
            }
        }
    }

    /// The row as `ls` prints it in colour.
    pub fn styled(&self) -> String {
        use nu_ansi_term::{Color, Style};
        let dim = Style::new().dimmed();
        match self.kind {
            RowKind::Node => {
                let name = if self.branch {
                    format!("{}/", Color::Cyan.bold().paint(self.name.as_str()))
                } else {
                    Color::Cyan.paint(self.name.as_str()).to_string()
                };
                if self.meta.is_empty() {
                    return name;
                }
                let plain = self.name.chars().count() + usize::from(self.branch);
                let pad = NAME_COLUMN.saturating_sub(plain).max(1);
                // A leaf's nonzero count is green; the rest are dim.
                let counted = !self.branch && self.meta.chars().any(|c| c.is_ascii_digit()) && !self.meta.starts_with("no ");
                let meta = if counted { Color::Green.paint(self.meta.as_str()).to_string() } else { dim.paint(self.meta.as_str()).to_string() };
                format!("{name}{}{meta}", " ".repeat(pad))
            }
            RowKind::Folder | RowKind::Dir => format!("{}/", Color::Cyan.bold().paint(self.name.as_str())),
            RowKind::File if self.meta.is_empty() => Color::White.paint(self.name.as_str()).to_string(),
            RowKind::File => format!("{}{}", Color::White.paint(self.name.as_str()), dim.paint(format!("  ({})", self.meta))),
            RowKind::Object => {
                let mut out = Color::White.bold().paint(self.slot.map(|s| s.to_string()).unwrap_or_default()).to_string();
                match (self.name.is_empty(), self.quoted) {
                    (true, _) => {}
                    (false, true) => out.push_str(&format!("  \"{}\"", Color::Green.paint(self.name.as_str()))),
                    (false, false) => out.push_str(&format!("  {}", self.name)),
                }
                if !self.meta.is_empty() {
                    out.push_str(&format!("  {}", dim.paint(self.meta.as_str())));
                }
                out.push_str(&format!("  {}", dim.paint(format!("({})", self.type_name))));
                out
            }
        }
    }
}

/// What `ls` says beside a declared node: its live counts or what it holds (empty when it says
/// nothing).
pub fn meta_of(state: &AppState, dot_path: &str) -> String {
    let Some(path) = TreePath::parse_dot(dot_path) else {
        return String::new();
    };
    if path.entry().is_some() {
        return String::new();
    }
    let show = &state.root.show_data;
    let fixtures = show.devices.dmx.patch.fixtures.len();
    let universes = show.devices.dmx.patch.universes.len();
    // A pool's count in the path's datapool (every datapool's on the schema path).
    let datapool = path.datapool().and_then(|n| show.datapools.get(&n));
    let in_pool = |f: fn(&lumacore_tree::Datapool) -> usize| match datapool {
        Some(dp) => f(dp),
        None => show.datapools.values().map(f).sum(),
    };
    let pool_or = |f: fn(&lumacore_tree::Datapool) -> usize, many: &str| match datapool {
        Some(dp) => plural_or_none(f(dp), many),
        None => many.to_string(),
    };
    match path.node() {
        Node::ShowData => format!(
            "show data and configuration  ({fixtures} fixtures, {universes} universes, {} users)",
            show.user_profiles.len()
        ),
        Node::Devices if fixtures == 0 && universes == 0 => "DMX devices and their metadata".to_string(),
        Node::Patch if fixtures == 0 && universes == 0 => "fixtures and universes".to_string(),
        Node::Devices | Node::Patch => format!("{fixtures} fixtures, {universes} universes"),
        Node::Dmx => "the DMX patch".to_string(),
        Node::DevicesMetadata => "what the device kinds share".to_string(),
        Node::MetadataMvr => {
            let m = &show.devices.metadata.mvr;
            format!("{} layers, {} groups, {} classes, {} files", m.layers.len(), m.groups.len(), m.classes.len(), m.files.len())
        }
        Node::MvrLayers => count(show.devices.metadata.mvr.layers.len(), "layer", "layers"),
        Node::MvrGroups => count(show.devices.metadata.mvr.groups.len(), "group", "groups"),
        Node::MvrClasses => count(show.devices.metadata.mvr.classes.len(), "class", "classes"),
        Node::MvrPositions => count(show.devices.metadata.mvr.positions.len(), "position", "positions"),
        Node::MvrMappingDefinitions => count(show.devices.metadata.mvr.mapping_definitions.len(), "mapping definition", "mapping definitions"),
        Node::MvrFiles => count(show.devices.metadata.mvr.files.len(), "imported file", "imported files"),
        Node::Fixtures => match fixtures {
            0 => "no fixtures patched".to_string(),
            1 => "1 fixture patched".to_string(),
            n => format!("{n} fixtures patched"),
        },
        Node::FixtureTypes => match show.devices.dmx.patch.fixture_types.len() {
            0 => "no fixture types loaded".to_string(),
            n => format!("{n} fixture types loaded"),
        },
        Node::Universes => count(universes, "universe", "universes"),
        Node::Protocols => match show.protocols.all_lines().count() {
            0 => "networks and protocol lines".to_string(),
            1 => "1 line".to_string(),
            n => format!("{n} lines"),
        },
        Node::DmxProtocols => "protocols whose lines carry universes".to_string(),
        Node::DataProtocols => "protocols whose messages become command lines".to_string(),
        Node::Bidirectional => "bidirectional protocol endpoints".to_string(),
        Node::Playback => "effects, executors, park".to_string(),
        Node::Sequences => plural_or_none(in_pool(|d| d.sequences.len()), "sequences"),
        Node::PlaybacksLive => "what runs, saved: sequences and masters".to_string(),
        Node::LiveMasters => "the grand masters and the group masters".to_string(),
        Node::Effects => plural_or_none(show.playback.effects.len(), "effects"),
        Node::TimingPresets => plural_or_none(in_pool(|d| d.timing_presets.len()), "timing presets"),
        Node::CurvePresets => plural_or_none(in_pool(|d| d.curve_presets.len()), "curve presets"),
        Node::Executors => plural_or_none(show.playback.executors.len(), "executors"),
        Node::DataPools => match show.datapools.values().map(|dp| dp.groups.len()).sum::<usize>() {
            0 => "datapools and pool priority orders".to_string(),
            n => format!("{n} groups"),
        },
        Node::DatapoolPriorityOrders => plural_or_none(show.datapool_priority_orders.len(), "datapool priority orders"),
        // This datapool's groups (the schema path: the first datapool's, as `ls` lists).
        Node::Groups => {
            let dp = match path.datapool() {
                Some(n) => show.datapools.get(&n),
                None => show.datapools.values().next(),
            };
            count(dp.map_or(0, |dp| dp.groups.len()), "group", "groups")
        }
        Node::Presets => plural_or_none(in_pool(|d| d.presets.len()), "presets"),
        // A numbered datapool's pools say their counts; the schema path says what they hold.
        Node::Layouts => pool_or(|d| d.layouts.len(), "layouts"),
        Node::Filters => pool_or(|d| d.filters.len(), "filters"),
        Node::Worlds => pool_or(|d| d.worlds.len(), "worlds"),
        Node::Macros => pool_or(|d| d.macros.len(), "macros"),
        Node::DistributePresets => pool_or(|d| d.distribute_presets.len(), "distribution presets"),
        Node::DynamicStates => pool_or(|d| d.dynamic_states.len(), "dynamic states"),
        Node::Plugins => pool_or(|d| d.plugins.len(), "plugins"),
        Node::Timecodes => pool_or(|d| d.timecodes.len(), "timecodes"),
        Node::Priorities => pool_or(|d| d.priorities.len(), "local priorities"),
        Node::UserProfiles => count(show.user_profiles.len(), "user", "users"),
        Node::ShowSettings => match show.show_settings.show_name.as_str() {
            "" => "show configuration".to_string(),
            name => format!("\"{name}\""),
        },
        Node::EffectEngines => plural_or_none(show.effect_engines.engines.len(), "effect engines"),
        Node::Variables => plural_or_none(show.variables.values.len(), "show variables"),
        Node::Runtime => {
            let rt = &state.root.runtime;
            format!(
                "live execution state  ({} users, {} connections, {} global vars)",
                rt.users.len(),
                rt.luma_sessions.connections.len(),
                rt.global_variables.len()
            )
        }
        Node::Network => {
            let n = &state.root.runtime.network;
            format!("{} interfaces, {} lines, {} sources", n.interfaces.len(), n.lines.len(), n.sources.len())
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
        Node::NetworkEngines => plural_or_none(state.root.runtime.network_engines.len(), "network engines"),
        Node::LoggedIn => match state.root.runtime.users.len() {
            0 => "no users logged in".to_string(),
            n => count(n, "user", "users"),
        },
        Node::SessionEngines => plural_or_none(state.root.runtime.session_engines.len(), "session engines"),
        Node::RenderingEngines => plural_or_none(state.root.runtime.rendering_engines.len(), "rendering engines"),
        Node::StationDevices => "reserved: what devices report back".to_string(),
        Node::Playbacks => match show.playbacks.live.sequences.len() {
            0 => "nothing running".to_string(),
            n => format!("{n} running sequences"),
        },
        Node::GlobalVariables => plural_or_none(state.root.runtime.global_variables.len(), "global variables"),
        Node::Journal => format!("{} recent changes", state.root.runtime.journal.len()),
        Node::Health => format!("{} threads reporting", state.root.runtime.health.snapshot().len()),
        Node::UIEngines => "live presentation engines".to_string(),
        Node::Resources => "shared assets".to_string(),
        Node::Diagnostics => "system-wide diagnostics".to_string(),
        Node::StationSettings => match state.root.station_settings.station_name.as_str() {
            "" => "station configuration".to_string(),
            name => format!("station: {name}"),
        },
        Node::Drives => "browse filesystem".to_string(),
        Node::Library => "gdtf, shows, presets, mvr".to_string(),
        Node::LibraryGdtf => match crate::commands::list::count_library_gdtf() {
            0 => "no GDTF files on disk".to_string(),
            1 => "1 GDTF file".to_string(),
            n => format!("{n} GDTF files"),
        },
        Node::LibraryShows => "saved show files (.luma)".to_string(),
        Node::LibraryPresets => "preset files".to_string(),
        Node::LibraryMvr => "MVR scene files".to_string(),
        // A logged-in user's undo steps and screens (decisions §10).
        Node::UserUndo | Node::UserScreens => {
            let Some(user) = path.numbered(Node::LoggedIn).and_then(|u| show.users.get(&u)).map(|u| u.login.as_str()) else {
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

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// `n` things, in words (`no layers`, `1 layer`, `3 layers`).
fn count(n: usize, one: &str, many: &str) -> String {
    match n {
        0 => format!("no {many}"),
        1 => format!("1 {one}"),
        _ => format!("{n} {many}"),
    }
}

/// `no effects` / `3 effects` (the plural for one too, as `ls` always said).
fn plural_or_none(n: usize, many: &str) -> String {
    if n == 0 { format!("no {many}") } else { format!("{n} {many}") }
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
