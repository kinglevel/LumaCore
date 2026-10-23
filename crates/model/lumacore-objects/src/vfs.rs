//! Virtual Filesystem (VFS) resolver for TreeShell.
//!
//! Bridges the static navigation tree (`nav_children_at`) with live data in `Root`.
//! When a navigation leaf like `Patch.Fixtures` is reached, the VFS looks into the
//! corresponding `BTreeMap` and exposes actual objects as navigable children.

use crate::tree_path::{Kind, Node, TreePath};
use lumacore_tree::{Effect, ExecutorObject, Root, SequenceObject, TreeNode};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ── Public types ─────────────────────────────────────────────────────────────

pub use crate::defs::view::{DynChild, NodeView};

// ── Path helpers ─────────────────────────────────────────────────────────────

/// Normalize a target path relative to cwd, producing a full dot-path segments list.
///
/// Handles `..`, `/`, absolute `root.` prefixes, and relative dot-paths.
/// Returns the segments (without "Root" prefix).
pub fn resolve_path(cwd: &[String], target: &str) -> Vec<String> {
    let target_lower = target.to_lowercase();

    if target_lower == "/" || target_lower == "root" {
        return Vec::new();
    }

    if let Some(rest) = crate::tree_path::strip_root(target) {
        return rest.split('.').map(|s| s.to_string()).collect();
    }

    // Relative path — handle ".." and normal segments; a leading `/` starts at the root.
    let mut base = if target.starts_with('/') { Vec::new() } else { cwd.to_vec() };

    // Tokenize: split on both "." and "/" but treat ".." as a single token.
    let tokens = tokenize_path(target);
    for token in &tokens {
        if token == ".." {
            base.pop();
        } else if !token.is_empty() && token != "." {
            base.push(token.to_string());
        }
    }

    base
}

/// Tokenize a path string, splitting on "." and "/" while keeping ".." intact.
fn tokenize_path(path: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    let chars: Vec<char> = path.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '.' && i + 1 < chars.len() && chars[i + 1] == '.' {
            // ".." — flush current, push ".."
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            tokens.push("..".to_string());
            i += 2;
            // Skip trailing separator
            if i < chars.len() && (chars[i] == '.' || chars[i] == '/') {
                i += 1;
            }
        } else if chars[i] == '.' || chars[i] == '/' {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            i += 1;
        } else {
            current.push(chars[i]);
            i += 1;
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Build a dot-path string from segments, prepending "Root.".
pub fn dot_path_from_segments(segments: &[String]) -> String {
    crate::tree_path::dot(segments)
}

// ── Drives (real filesystem) helpers ─────────────────────────────────────────

/// The text file extensions that `cat` will read and display.
const TEXT_EXTENSIONS: &[&str] = &[
    "txt",
    "json",
    "xml",
    "toml",
    "yaml",
    "yml",
    "csv",
    "gdtf",
    "luma",
    "md",
    "log",
    "cfg",
    "ini",
    "sh",
    "bash",
    "zsh",
    "fish",
    "rs",
    "py",
    "js",
    "ts",
    "html",
    "css",
    "scss",
    "sql",
    "conf",
    "env",
    "gitignore",
    "dockerfile",
    "makefile",
    "lock",
    "svelte",
    "jsx",
    "tsx",
    "vue",
    "go",
    "c",
    "h",
    "cpp",
];

/// Maximum entries returned by `fs_children_at`.
const FS_MAX_ENTRIES: usize = 1000;

/// The drives as folders under `Root.Drives` (decisions §38): every present drive by its id.
fn drive_children() -> Vec<DynChild> {
    crate::drives::mounted_ids()
        .into_iter()
        .map(|id| DynChild { display_name: id.clone(), id: 0, guid: None, node_type: "Drive", is_leaf: false, child_count: 0, name: Some(id) })
        .collect()
}

/// The volume root a drive id names (`Root.Drives.usb1`); the local library is `library/`.
fn drive_root(id: &str) -> Option<PathBuf> {
    if id.eq_ignore_ascii_case(crate::drives::LOCAL) {
        return Some(PathBuf::from("library"));
    }
    crate::drives::mount_of(id)
}

/// List children of a real directory path, returning them as `DynChild` entries.
///
/// Directories come first (alphabetically), then files (alphabetically).
/// Hidden/dotfiles are included. Capped at `FS_MAX_ENTRIES`.
pub fn fs_children_at(real_path: &Path) -> Vec<DynChild> {
    let entries = match lumacore_luma::drive::read_dir(real_path) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    let mut dirs: Vec<DynChild> = Vec::new();
    let mut files: Vec<DynChild> = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let meta = entry.metadata();
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);

        let child = DynChild {
            display_name: if is_dir {
                name
            } else {
                let size_str = format_file_size(size);
                format!("{}  ({})", entry.file_name().to_string_lossy(), size_str)
            },
            id: 0,
            node_type: if is_dir { "Directory" } else { "File" },
            is_leaf: !is_dir,
            child_count: 0,
            name: None,
            guid: None,
        };

        if is_dir {
            dirs.push(child);
        } else {
            files.push(child);
        }

        if dirs.len() + files.len() >= FS_MAX_ENTRIES {
            break;
        }
    }

    dirs.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
    });
    files.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
    });

    dirs.append(&mut files);
    dirs
}

/// Resolve a dot-path under `Root.Drives` to a real filesystem path.
///
/// `Root.Drives` maps to `drives_root()`. Each subsequent segment is a
/// filesystem path component. Segments are matched against actual directory
/// entry names (including dots in filenames) before falling back to splitting.
pub fn resolve_drive_path(dot_path: &str) -> Option<PathBuf> {
    if !crate::tree_path::names_drive(dot_path) {
        return None;
    }
    // What follows `Root.Drives.`: the drive's id, then a path on it.
    let suffix = dot_path.splitn(3, '.').nth(2).unwrap_or("");
    if suffix.is_empty() {
        return None;
    }
    // A hyphenated id (`local-library`) comes quoted in a dot path.
    let (id, rest) = suffix.split_once('.').unwrap_or((suffix, ""));
    let mut current = drive_root(id.trim_matches('"'))?;
    let mut remaining = rest;

    while !remaining.is_empty() {
        // Try to match remaining against actual directory entries (longest match first)
        let matched = if current.is_dir() {
            match_fs_segment(&current, remaining)
        } else {
            None
        };

        if let Some((entry_name, rest)) = matched {
            current = current.join(&entry_name);
            remaining = rest;
        } else {
            // No match — try simple dot-split for the next segment
            let (seg, rest) = if let Some(dot_pos) = remaining.find('.') {
                (&remaining[..dot_pos], &remaining[dot_pos + 1..])
            } else {
                (remaining, "")
            };
            current = current.join(seg);
            remaining = rest;
        }
    }

    if current.exists() {
        Some(current)
    } else {
        None
    }
}

/// Try to match the beginning of `remaining` against actual entries in `dir`.
/// Prefers entries with dots in their name (e.g., "file.gdtf") over plain segments.
/// Returns `(matched_entry_name, rest_of_remaining)`.
fn match_fs_segment<'a>(dir: &Path, remaining: &'a str) -> Option<(String, &'a str)> {
    let entries = lumacore_luma::drive::read_dir(dir).ok()?;
    let remaining_lower = remaining.to_lowercase();

    // Collect entries with dots in their names — these need priority matching
    let mut dotted_entries: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains('.') {
            dotted_entries.push(name);
        }
    }

    // Sort by length descending — match longest first
    dotted_entries.sort_by_key(|entry| std::cmp::Reverse(entry.len()));

    for entry_name in &dotted_entries {
        // The entry name's dots would appear as dots in the dot-path.
        // Check if remaining starts with entry_name (case-insensitive)
        let entry_lower = entry_name.to_lowercase();
        if remaining_lower == entry_lower {
            return Some((entry_name.clone(), ""));
        }
        if remaining_lower.starts_with(&entry_lower)
            && remaining.as_bytes().get(entry_name.len()) == Some(&b'.')
        {
            return Some((entry_name.clone(), &remaining[entry_name.len() + 1..]));
        }
    }

    None
}

/// Format a file size in human-readable form.
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Format a file size for public display.
pub fn format_file_size_pub(bytes: u64) -> String {
    format_file_size(bytes)
}

/// Check whether a file extension indicates a text file.
pub fn is_text_extension(ext: &str) -> bool {
    TEXT_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

/// Attempt to detect if file content is text by checking for null bytes.
pub fn looks_like_text(path: &Path) -> bool {
    use std::io::Read;
    let mut f = match lumacore_luma::drive::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buf = [0u8; 512];
    let n = match f.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return false,
    };
    !buf[..n].contains(&0)
}

// ── Collection mapping ───────────────────────────────────────────────────────

/// `snake_case` → `PascalCase` (a JSON field as the tree names it).
fn pascal(field: &str) -> String {
    field.split('_').map(|w| w[..1].to_uppercase() + &w[1..]).collect()
}

/// The JSON of a profile's Settings or Content (decisions §30 "Users & Profiles follows
/// storage": `Root.ShowData.UserProfiles.<n>.Settings|Content.<Part>`).
fn profile_half(root: &Root, no: u32, half: Node) -> Option<JsonValue> {
    let p = root.show_data.user_profiles.get(&no)?;
    if half == Node::ProfileSettings { serde_json::to_value(&p.settings).ok() } else { serde_json::to_value(&p.content).ok() }
}

/// Return dynamic children at a given dot-path by looking into Root's BTreeMaps.
///
/// Returns `None` if this path has no dynamic mapping (i.e. it's a structural-only node).
/// Returns `Some(vec![])` if mapped but the collection is empty.
pub fn dynamic_children_at(dot_path: &str, root: &Root) -> Option<Vec<DynChild>> {
    let path = TreePath::parse_dot(dot_path)?;
    let node = path.node();
    // ── Drives (real filesystem) ────────────────────────────────────
    if node == Node::Drives {
        if path.steps().len() == 1 {
            return Some(drive_children());
        }
        let real_path = resolve_drive_path(dot_path)?;
        return real_path.is_dir().then(|| fs_children_at(&real_path));
    }
    if node.is_under(Node::LoggedIn) {
        return super::vfs_runtime::children(super::vfs_runtime::from_path(&path)?, root);
    }
    if path.entry().is_some() {
        return None;
    }
    // A named node's entries (the protocols of a group): folders by name.
    if matches!(node.kind(), Kind::Named(_)) {
        return Some(
            crate::objects::named_in(node)
                .into_iter()
                .map(|name| DynChild {
                    display_name: name.to_string(),
                    id: 0,
                    node_type: "Folder",
                    is_leaf: false,
                    child_count: 0,
                    name: Some(name.to_string()),
                    guid: None,
                })
                .collect(),
        );
    }
    // Collections objects publish (terminal/objects); a numbered datapool's
    // collections are that datapool's.
    if node.kind() == Kind::Collection {
        return Some(crate::objects::tree_at(&path)?.children_in(root, path.datapool()));
    }
    match node {
        // Every datapool by number (`Root.ShowData.DataPools.2`).
        Node::DataPools => Some(datapool_children(root)),
        // One datapool: its collections are declared children.
        Node::DataPool => root.show_data.datapools.contains_key(&path.datapool()?).then(Vec::new),
        // Running playbacks (ShowData.Playbacks, decisions §13).
        Node::RunningSequences => Some(
            root.show_data
                .playbacks
                .live.sequences
                .iter()
                .filter_map(|(guid, st)| {
                    let (_, id, _) = root.show_data.sequence_by_guid(*guid)?;
                    Some((id, st))
                })
                .map(|(id, st)| DynChild {
                    display_name: format!(
                        "Sequence {id} → Cue {} ({})",
                        lumacore_tree::cue_label(st.current_cue),
                        root.show_data.user_name(st.started_by).unwrap_or("?")
                    ),
                    id,
                    node_type: "Playback",
                    is_leaf: true,
                    child_count: 0,
                    name: None,
                    guid: None,
                })
                .collect(),
        ),
        // The masters (decisions §30 "12f"): the grand group, then each group master.
        Node::LiveMasters => {
            let m = &root.show_data.playbacks.live.masters;
            let now = lumacore_base::clock::session_ms();
            let pct = |v: f64| format!("{:.0}%", v * 100.0);
            let mut rows = vec![
                format!("2.1 Grand {}{}", pct(m.grand.master), if m.grand.on { "" } else { " OFF (blackout)" }),
                format!("2.2 Highlight (everybody) {}", pct(m.grand.highlight)),
                format!("2.4 Solo (everybody) {}", pct(m.grand.solo)),
            ];
            for (guid, level) in &m.groups {
                let name = root
                    .show_data
                    .datapools
                    .values()
                    .flat_map(|dp| dp.groups.iter())
                    .find(|(_, g)| g.meta.identity.guid == *guid)
                    .map_or("a deleted group".to_string(), |(n, g)| match g.name() {
                        Some(name) => format!("Group {n} \"{name}\""),
                        None => format!("Group {n}"),
                    });
                rows.push(format!("{name} master {}", pct(level.at(now))));
            }
            Some(
                rows.into_iter()
                    .enumerate()
                    .map(|(i, display_name)| DynChild {
                        display_name,
                        id: i as u32 + 1,
                        node_type: "Master",
                        is_leaf: true,
                        child_count: 0,
                        name: None,
                        guid: None,
                    })
                    .collect(),
            )
        }
        // Parked fixture attributes and raw channels.
        Node::Park => {
            let park = &root.show_data.playback.park;
            let fixtures = park.fixtures.iter().map(|p| {
                let fixture = root
                    .show_data
                    .fixture_id_for_guid(p.fixture_id)
                    .map_or("?".to_string(), |id| id.to_string());
                format!("Fixture {fixture} {} {}", p.attribute, p.value.readout())
            });
            let channels = park
                .channels
                .iter()
                .map(|c| format!("DMX {}.{:03} {}", c.universe, c.address, c.value));
            Some(
                fixtures
                    .chain(channels)
                    .enumerate()
                    .map(|(i, display_name)| DynChild {
                        display_name,
                        id: i as u32 + 1,
                        node_type: "Park",
                        is_leaf: true,
                        child_count: 0,
                        name: None,
                        guid: None,
                    })
                    .collect(),
            )
        }
        // Every protocol line's live state (decisions §11).
        Node::Lines => Some(
            root.runtime
                .network
                .lines
                .iter()
                .enumerate()
                .map(|(i, l)| DynChild {
                    display_name: format!(
                        "{} {} {} \"{}\" ({}): {}",
                        l.direction, l.protocol, l.line, l.name, l.network, l.state
                    ),
                    id: i as u32 + 1,
                    node_type: "Line",
                    is_leaf: true,
                    child_count: 0,
                    name: None,
                    guid: None,
                })
                .collect(),
        ),
        Node::Nodes => Some(
            root.runtime
                .network
                .nodes
                .iter()
                .enumerate()
                .map(|(i, n)| DynChild {
                    display_name: format!(
                        "{} \"{}\" on {} outputs {}",
                        n.ip,
                        n.name,
                        n.network,
                        n.outputs
                            .iter()
                            .map(|w| lumacore_protocols::wire_label(&n.protocol, *w))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    id: i as u32 + 1,
                    node_type: "Node",
                    is_leaf: true,
                    child_count: 0,
                    name: None,
                    guid: None,
                })
                .collect(),
        ),
        // This station's interfaces and what each serves (§11).
        Node::Interfaces => Some(
            root.runtime
                .network
                .interfaces
                .values()
                .enumerate()
                .map(|(i, n)| DynChild {
                    display_name: format!(
                        "{} {} LumaSession {} Shell {} HTTP {}",
                        n.name,
                        n.address,
                        yes_no(n.lumasession),
                        yes_no(n.shell),
                        yes_no(n.http)
                    ),
                    id: i as u32 + 1,
                    node_type: "Interface",
                    is_leaf: true,
                    child_count: 0,
                    name: None,
                    guid: None,
                })
                .collect(),
        ),
        // Claims (decisions §21 "6d"): `cd Claims` then `List`.
        Node::Claims => Some({
            let rt = &root.runtime;
            rt.claim_order
                .iter()
                .filter_map(|g| rt.claims.get(g))
                .enumerate()
                .map(|(i, c)| DynChild {
                    display_name: format!(
                        "{} — {}{}",
                        c.label,
                        c.user,
                        if c.editor { " (editor)" } else { "" }
                    ),
                    id: i as u32 + 1,
                    node_type: "Claim",
                    is_leaf: true,
                    child_count: 0,
                    name: None,
                    guid: None,
                })
                .collect()
        }),
        // LumaSession: stations seen, members, listeners, connections.
        Node::SessionStations => Some(
            root.runtime
                .luma_sessions
                .discovery
                .values()
                .filter(|p| p.cmd_port != 0)
                .enumerate()
                .map(|(i, p)| DynChild {
                    display_name: format!(
                        "{} {} \"{}\" {} {}",
                        p.name, p.ip, p.lumasession, p.status, p.priority
                    ),
                    id: i as u32 + 1,
                    node_type: "Station",
                    is_leaf: true,
                    child_count: 0,
                    name: None,
                    guid: None,
                })
                .collect(),
        ),
        Node::SessionMembers => Some(
            root.runtime
                .luma_sessions
                .members
                .values()
                .enumerate()
                .map(|(i, m)| DynChild {
                    display_name: format!("{} {}:{}", m.station, m.ip, m.cmd_port),
                    id: i as u32 + 1,
                    node_type: "Member",
                    is_leaf: true,
                    child_count: 0,
                    name: None,
                    guid: None,
                })
                .collect(),
        ),
        Node::SessionListeners => Some(
            root.runtime
                .luma_sessions
                .listeners
                .values()
                .enumerate()
                .map(|(i, l)| DynChild {
                    display_name: format!("{} {} {}:{}", l.name, l.transport, l.bind, l.port),
                    id: i as u32 + 1,
                    node_type: "Listener",
                    is_leaf: true,
                    child_count: 0,
                    name: None,
                    guid: None,
                })
                .collect(),
        ),
        Node::SessionConnections => Some(
            root.runtime
                .luma_sessions
                .connections
                .values()
                .enumerate()
                .map(|(i, c)| DynChild {
                    display_name: format!("{} {} {} {}", c.id, c.transport, c.peer, c.state),
                    id: i as u32 + 1,
                    node_type: "Connection",
                    is_leaf: true,
                    child_count: 0,
                    name: None,
                    guid: None,
                })
                .collect(),
        ),
        // DMX input sources heard now (decisions §11 "Step 4c").
        Node::Sources => Some(
            root.runtime
                .network
                .sources
                .iter()
                .enumerate()
                .map(|(i, s)| DynChild {
                    display_name: format!(
                        "{} \"{}\" {} prio {} on {} universes {}",
                        s.protocol,
                        s.name,
                        s.ip,
                        s.priority,
                        s.network,
                        s.universes
                            .iter()
                            .map(u16::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    id: i as u32 + 1,
                    node_type: "Source",
                    is_leaf: true,
                    child_count: 0,
                    name: None,
                    guid: None,
                })
                .collect(),
        ),
        // Reserved: what devices report back (decisions §11).
        Node::StationDevices => Some(Vec::new()),
        _ => None,
    }
}

/// Whether every numbered step of `path` exists (an object, a datapool, a logged-in user);
/// declared nodes always do. Disk names are the disk's to say.
pub fn path_exists(root: &Root, path: &TreePath) -> bool {
    path.steps().iter().enumerate().all(|(i, step)| match step {
        crate::tree_path::Step::Entry(id) => dynamic_children_at(&path.prefix(i).to_string(), root)
            .is_some_and(|children| children.iter().any(|c| c.id == *id)),
        crate::tree_path::Step::Name(name) => crate::objects::named_in(path.prefix(i).node())
            .iter()
            .any(|n| n.eq_ignore_ascii_case(name)),
        _ => true,
    })
}

/// Resolve a single node at a dot-path, returning its full view.
///
/// For structural nodes: returns name + children list.
/// For object nodes: serializes the object to JSON.
pub fn resolve_node(dot_path: &str, root: &Root) -> Option<NodeView> {
    let path = TreePath::parse_dot(dot_path)?;

    // ── Drives (real filesystem) ────────────────────────────────────
    if path.node() == Node::Drives {
        if let Some(real_path) = resolve_drive_path(dot_path) {
            if real_path.is_dir() {
                // Directory — return as structural with fs children
                let children = fs_children_at(&real_path);
                let name = real_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Drives".to_string());
                return Some(NodeView::Structural {
                    name,
                    children: children
                        .iter()
                        .map(|c| {
                            // Strip size annotation for structural listing
                            c.display_name
                                .split("  (")
                                .next()
                                .unwrap_or(&c.display_name)
                                .to_string()
                        })
                        .collect(),
                });
            } else {
                // File — return FileInfo
                let meta = lumacore_luma::drive::metadata(&real_path).ok();
                return Some(NodeView::FileInfo {
                    name: real_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                    is_dir: false,
                    modified: meta.as_ref().and_then(|m| m.modified().ok()),
                    extension: real_path
                        .extension()
                        .map(|e| e.to_string_lossy().to_string()),
                    path: real_path,
                });
            }
        }
        return None;
    }

    // Live screens / windows under Root.Runtime.Users
    if let Some(rt) = super::vfs_runtime::from_path(&path)
        && let Some(obj) = super::vfs_runtime::object(rt, root)
    {
        return Some(obj);
    }

    // A numbered datapool is a folder of its collections.
    if path.node() == Node::DataPool
        && let Some(dp) = path.datapool()
    {
        if !root.show_data.datapools.contains_key(&dp) {
            return None;
        }
        return Some(NodeView::Structural {
            name: dp.to_string(),
            children: super::nav_children_at(dot_path),
        });
    }

    // A part of a profile's Settings / Content: that field's JSON.
    if let Some(no) = path.numbered(Node::UserProfiles)
        && let Some(half) = path.node().parent().filter(|h| matches!(h, Node::ProfileSettings | Node::ProfileContent))
    {
        let json = profile_half(root, no, half)?;
        let want = path.node().name().to_lowercase();
        let (key, value) = json.as_object()?.iter().find(|(k, _)| k.replace('_', "") == want)?;
        return Some(NodeView::Object { json: value.clone(), node_type: "Setting", id: 0, guid: String::new(), name: Some(pascal(key)) });
    }
    // An object (`Root.ShowData.Devices.Dmx.Patch.Fixtures.1`).
    if let Some(id) = path.entry() {
        return crate::objects::tree_at(&path)?.object_in(root, path.datapool(), id);
    }

    // Check if it's a structural node with static or dynamic children
    let static_children = super::nav_children_at(dot_path);
    let dyn_children = dynamic_children_at(dot_path, root);

    if !static_children.is_empty() || dyn_children.is_some() {
        let name = dot_path.rsplit('.').next().unwrap_or(dot_path).to_string();
        let mut children = static_children;
        if let Some(dc) = dyn_children {
            for c in dc {
                // Named sub-folders keep their name; objects are addressed by number.
                children.push(if c.node_type == "Folder" {
                    c.display_name
                } else {
                    c.id.to_string()
                });
            }
        }
        Some(NodeView::Structural { name, children })
    } else {
        None
    }
}

/// `Root.ShowData.DataPools.<n>.…` → its schema path
/// (`Root.ShowData.DataPools.DataPool.…`) and `n` (decisions §22, §29).
pub fn datapool_path(path: &str) -> (String, Option<u32>) {
    let segments: Vec<&str> = path.split('.').collect();
    if segments.len() > 3
        && segments[0].eq_ignore_ascii_case("root")
        && segments[1].eq_ignore_ascii_case("showdata")
        && segments[2].eq_ignore_ascii_case("datapools")
        && let Ok(n) = segments[3].parse::<u32>()
    {
        let mut out: Vec<String> = segments.iter().map(|s| s.to_string()).collect();
        out[3] = "DataPool".to_string();
        return (out.join("."), Some(n));
    }
    (path.to_string(), None)
}

// ── Tree walker ──────────────────────────────────────────────────────────────

/// Recursively walk all nodes (static + dynamic) from a starting dot-path.
///
/// Calls `visitor(full_dot_path, child)` for each dynamic object found.
pub fn walk_tree(root: &Root, from: &str, visitor: &mut dyn FnMut(&str, &DynChild)) {
    // First, check if `from` itself has dynamic children
    let numbered = TreePath::parse_dot(from).is_some_and(|p| matches!(p.node().kind(), Kind::Numbered(_)));
    if let Some(children) = dynamic_children_at(from, root) {
        for child in &children {
            let child_path = format!("{from}.{}", child.id);
            visitor(&child_path, child);
            // A numbered entry (a datapool) has collections of its own.
            if numbered {
                walk_tree(root, &child_path, visitor);
            }
        }
    }

    // Then recurse into static children (skip Drives — don't walk real filesystem)
    let static_children = super::nav_children_at(from);
    for child_name in &static_children {
        if child_name == "Drives" {
            continue;
        }
        let child_path = format!("{from}.{child_name}");
        walk_tree(root, &child_path, visitor);
    }
}

/// Walk tree and also return structural node names (for `tree` command).
pub fn walk_tree_full(
    root: &Root,
    from: &str,
    visitor: &mut dyn FnMut(&str, Option<&DynChild>, bool),
) {
    let static_children = super::nav_children_at(from);
    let dyn_children = dynamic_children_at(from, root);

    // Report static children (skip Drives — don't walk real filesystem)
    for child_name in &static_children {
        if child_name == "Drives" {
            continue;
        }
        let child_path = format!("{from}.{child_name}");
        visitor(&child_path, None, true);
        walk_tree_full(root, &child_path, visitor);
    }

    // Report dynamic children
    if let Some(children) = dyn_children {
        for child in &children {
            let child_path = format!("{from}.{}", child.id);
            visitor(&child_path, Some(child), false);
        }
    }
}

// ── Collection → DynChild helpers ────────────────────────────────────────────

/// The object view of tree node `id` (`cat`, `info` on a tree path).
pub fn object_view<T: TreeNode + serde::Serialize>(id: u32, t: &T) -> NodeView {
    NodeView::Object {
        json: serde_json::to_value(t).unwrap_or(JsonValue::Null),
        node_type: t.node_type(),
        id,
        guid: t.guid().to_string(),
        name: t.name().map(|s| s.to_string()),
    }
}

/// Every datapool by number — the tree's `DataPools` and the DataPool object's members.
pub fn datapool_children(root: &Root) -> Vec<DynChild> {
    root.show_data
        .datapools
        .iter()
        .map(|(&id, dp)| DynChild {
            display_name: dp.name().map_or(format!("DataPool {id}"), |n| format!("DataPool {id} \"{n}\"")),
            id,
            node_type: "DataPool",
            is_leaf: false,
            child_count: dp.groups.len(),
            name: dp.name().map(str::to_string),
            guid: Some(dp.meta.identity.guid),
        })
        .collect()
}

pub fn btree_to_children<T: TreeNode>(map: &BTreeMap<u32, T>) -> Vec<DynChild> {
    map.iter()
        .map(|(&id, obj)| DynChild {
            display_name: obj
                .name()
                .map(|n| format!("{} \"{}\"", id, n))
                .unwrap_or_else(|| format!("{}", id)),
            id,
            node_type: obj.node_type(),
            is_leaf: true,
            child_count: 0,
            name: obj.name().map(str::to_string),
            guid: Some(obj.guid()),
        })
        .collect()
}

// Playback types don't implement TreeNode — they use bare name fields.

pub fn playback_seq_children(map: &BTreeMap<u32, std::sync::Arc<SequenceObject>>) -> Vec<DynChild> {
    map.iter()
        .map(|(&id, seq)| DynChild {
            display_name: match seq.meta.identity.name.as_deref() {
                None | Some("") => format!("{}", id),
                Some(name) => format!("{} \"{}\"", id, name),
            },
            id,
            node_type: "Sequence",
            is_leaf: true,
            child_count: seq.cues.len(),
            name: seq.meta.identity.name.clone().filter(|n| !n.is_empty()),
            guid: Some(seq.meta.identity.guid),
        })
        .collect()
}

pub fn playback_exec_children(map: &BTreeMap<u32, ExecutorObject>) -> Vec<DynChild> {
    map.iter()
        .map(|(&id, exec)| DynChild {
            display_name: if exec.meta.name().unwrap_or_default().is_empty() {
                format!("{}", id)
            } else {
                format!("{} \"{}\"", id, exec.meta.name().unwrap_or_default())
            },
            id,
            node_type: "Executor",
            is_leaf: true,
            child_count: 0,
            name: None,
            guid: None,
        })
        .collect()
}

pub fn playback_effect_children(map: &BTreeMap<u32, Effect>) -> Vec<DynChild> {
    map.iter()
        .map(|(&id, e)| DynChild {
            display_name: if e.meta.name().unwrap_or_default().is_empty() {
                format!("{}", id)
            } else {
                format!("{} \"{}\"", id, e.meta.name().unwrap_or_default())
            },
            id,
            node_type: "Effect",
            is_leaf: true,
            child_count: 0,
            name: None,
            guid: None,
        })
        .collect()
}

pub fn first_datapool_groups(root: &Root) -> Vec<DynChild> {
    if let Some((_, dp)) = root.show_data.datapools.iter().next() {
        btree_to_children(&dp.groups)
    } else {
        Vec::new()
    }
}

// ── Object search helpers ────────────────────────────────────────────────────

/// Search all dynamic objects whose display name matches a pattern (case-insensitive substring).
pub fn find_by_name(
    root: &Root,
    from: &str,
    pattern: &str,
    type_filter: Option<&str>,
) -> Vec<(String, DynChild)> {
    let pattern_lower = pattern.to_lowercase();
    let mut results = Vec::new();

    walk_tree(root, from, &mut |path, child| {
        // Check type filter
        if let Some(tf) = type_filter
            && !child.node_type.eq_ignore_ascii_case(tf)
        {
            return;
        }
        // Check name pattern (glob-style with * or substring)
        let name_lower = child.display_name.to_lowercase();
        let matches = if pattern_lower.contains('*') {
            glob_match(&pattern_lower, &name_lower)
        } else {
            name_lower.contains(&pattern_lower)
        };
        if matches {
            results.push((path.to_string(), child.clone()));
        }
    });

    results
}

/// Simple glob matching (only supports * wildcards).
fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return text == pattern;
    }

    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if let Some(found) = text[pos..].find(part) {
            if i == 0 && found != 0 {
                return false; // Must match from start if pattern doesn't start with *
            }
            pos += found + part.len();
        } else {
            return false;
        }
    }
    // If pattern doesn't end with *, text must end at pos
    if !pattern.ends_with('*') && pos != text.len() {
        return false;
    }
    true
}

/// One `grep_values` hit: the object's path, the object, and its matching
/// `(field, value)` pairs.
pub type ValueMatch = (String, DynChild, Vec<(String, String)>);

/// Search object values (serialized JSON) for a pattern.
pub fn grep_values(root: &Root, from: &str, pattern: &str) -> Vec<ValueMatch> {
    let pattern_lower = pattern.to_lowercase();
    let mut results = Vec::new();

    walk_tree(root, from, &mut |path, child| {
        // Resolve the object to JSON and search within it
        if let Some(NodeView::Object { json, .. }) = resolve_node(path, root) {
            let mut matching_fields = Vec::new();
            search_json_fields(&json, "", &pattern_lower, &mut matching_fields);
            if !matching_fields.is_empty() {
                results.push((path.to_string(), child.clone(), matching_fields));
            }
        }
    });

    results
}

/// Recursively search JSON fields for a pattern.
fn search_json_fields(
    value: &JsonValue,
    prefix: &str,
    pattern: &str,
    results: &mut Vec<(String, String)>,
) {
    match value {
        JsonValue::Object(map) => {
            for (key, val) in map {
                let field_path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                search_json_fields(val, &field_path, pattern, results);
            }
        }
        JsonValue::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let field_path = format!("{prefix}[{i}]");
                search_json_fields(val, &field_path, pattern, results);
            }
        }
        JsonValue::String(s) => {
            if s.to_lowercase().contains(pattern) {
                results.push((prefix.to_string(), s.clone()));
            }
        }
        JsonValue::Number(n) => {
            let s = n.to_string();
            if s.contains(pattern) {
                results.push((prefix.to_string(), s));
            }
        }
        _ => {}
    }
}

/// Count all objects recursively from a path.
pub fn count_objects(root: &Root, from: &str) -> Vec<(&'static str, usize)> {
    let mut counts: Vec<(&'static str, usize)> = Vec::new();

    walk_tree(root, from, &mut |_path, child| {
        if let Some(entry) = counts.iter_mut().find(|(t, _)| *t == child.node_type) {
            entry.1 += 1;
        } else {
            counts.push((child.node_type, 1));
        }
    });

    counts
}

/// Count direct dynamic children at a path.
pub fn count_direct_children(dot_path: &str, root: &Root) -> usize {
    let static_count = super::nav_children_at(dot_path).len();
    let dyn_count = dynamic_children_at(dot_path, root)
        .map(|c| c.len())
        .unwrap_or(0);
    static_count + dyn_count
}

/// Diff two JSON values, returning fields that differ.
pub fn diff_json(a: &JsonValue, b: &JsonValue) -> Vec<(String, String, String)> {
    let mut diffs = Vec::new();
    diff_json_inner(a, b, "", &mut diffs);
    diffs
}

fn diff_json_inner(
    a: &JsonValue,
    b: &JsonValue,
    prefix: &str,
    diffs: &mut Vec<(String, String, String)>,
) {
    match (a, b) {
        (JsonValue::Object(obj_a), JsonValue::Object(obj_b)) => {
            // Check all keys in a
            for (key, va) in obj_a {
                let field = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                if let Some(vb) = obj_b.get(key) {
                    diff_json_inner(va, vb, &field, diffs);
                } else {
                    diffs.push((field, format_json_short(va), "(absent)".to_string()));
                }
            }
            // Keys only in b
            for (key, vb) in obj_b {
                if !obj_a.contains_key(key) {
                    let field = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}.{key}")
                    };
                    diffs.push((field, "(absent)".to_string(), format_json_short(vb)));
                }
            }
        }
        (JsonValue::Array(aa), JsonValue::Array(ab)) => {
            let max_len = aa.len().max(ab.len());
            for i in 0..max_len {
                let field = format!("{prefix}[{i}]");
                match (aa.get(i), ab.get(i)) {
                    (Some(va), Some(vb)) => diff_json_inner(va, vb, &field, diffs),
                    (Some(va), None) => {
                        diffs.push((field, format_json_short(va), "(absent)".to_string()))
                    }
                    (None, Some(vb)) => {
                        diffs.push((field, "(absent)".to_string(), format_json_short(vb)))
                    }
                    (None, None) => {}
                }
            }
        }
        _ => {
            if a != b {
                diffs.push((
                    prefix.to_string(),
                    format_json_short(a),
                    format_json_short(b),
                ));
            }
        }
    }
}

/// Short JSON value formatting for diff output.
fn format_json_short(v: &JsonValue) -> String {
    match v {
        JsonValue::String(s) => format!("\"{}\"", s),
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Array(a) => format!("[{} items]", a.len()),
        JsonValue::Object(m) => format!("{{{} fields}}", m.len()),
    }
}

fn yes_no(b: bool) -> &'static str {
    if b { "Yes" } else { "No" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumacore_tree::{Root};
    use lumacore_fixtures::{Fixture, Universe};
    use uuid::Uuid;

    /// Create a Root with some test fixtures and universes.
    fn test_root() -> Root {
        let mut root = Root::new();

        // Add fixtures
        let ft_guid = Uuid::new_v4();
        root.show_data.devices.dmx.patch.insert_fid(1, Fixture::new(Some("Front Wash L".to_string()),
                ft_guid,
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                Uuid::nil(),
                1,
                6,
            ),
        );
        root.show_data.devices.dmx.patch.insert_fid(2, Fixture::new(Some("Front Wash R".to_string()),
                ft_guid,
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                Uuid::nil(),
                7,
                6,
            ),
        );
        root.show_data.devices.dmx.patch.insert_fid(3, Fixture::new(Some("Back Light".to_string()),
                ft_guid,
                uuid::Uuid::nil(),
                uuid::Uuid::nil(),
                Uuid::nil(),
                13,
                6,
            ),
        );

        // Add a universe
        root.show_data
            .devices
            .dmx
            .patch
            .universes
            .insert(1, Universe::new(Some("Main".to_string())));

        root
    }

    // ── resolve_path tests ───────────────────────────────────────────────

    #[test]
    fn resolve_path_root() {
        let result = resolve_path(&[], "/");
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_path_absolute() {
        let result = resolve_path(&[], "root.ShowData.Devices");
        assert_eq!(result, vec!["ShowData", "Devices"]);
    }

    #[test]
    fn resolve_path_relative() {
        let cwd = vec!["ShowData".to_string()];
        let result = resolve_path(&cwd, "Devices.Dmx.Patch");
        assert_eq!(result, vec!["ShowData", "Devices", "Dmx", "Patch"]);
    }

    #[test]
    fn resolve_path_dotdot() {
        let cwd = vec!["ShowData".to_string(), "Devices".to_string()];
        let result = resolve_path(&cwd, "..");
        assert_eq!(result, vec!["ShowData"]);
    }

    #[test]
    fn resolve_path_dotdot_chain() {
        let cwd = vec![
            "ShowData".to_string(),
            "Devices".to_string(),
            "Dmx".to_string(),
            "Patch".to_string(),
        ];
        let result = resolve_path(&cwd, "../..");
        assert_eq!(result, vec!["ShowData", "Devices"]);
    }

    #[test]
    fn dot_path_from_empty_segments() {
        assert_eq!(dot_path_from_segments(&[]), "Root");
    }

    #[test]
    fn dot_path_from_segments_basic() {
        let segs = vec!["ShowData".to_string(), "Devices".to_string()];
        assert_eq!(dot_path_from_segments(&segs), "Root.ShowData.Devices");
    }

    // ── dynamic_children_at tests ────────────────────────────────────────

    #[test]
    fn dynamic_children_at_fixtures() {
        let root = test_root();
        let children = dynamic_children_at("Root.ShowData.Devices.Dmx.Patch.Fixtures", &root);
        assert!(children.is_some());
        let children = children.unwrap();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].id, 1);
        assert_eq!(children[0].node_type, "Fixture");
        assert!(children[0].display_name.contains("Front Wash L"));
    }

    #[test]
    fn dynamic_children_at_universes() {
        let root = test_root();
        let children = dynamic_children_at("Root.ShowData.Devices.Dmx.Patch.Universes", &root);
        assert!(children.is_some());
        let children = children.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, 1);
        assert_eq!(children[0].node_type, "Universe");
    }

    #[test]
    fn dynamic_children_at_unmapped_path() {
        let root = test_root();
        let children = dynamic_children_at("Root.ShowData", &root);
        assert!(children.is_none());
    }

    #[test]
    fn dynamic_children_at_empty_collection() {
        let root = test_root();
        let children = dynamic_children_at("Root.ShowData.Playbacks.Live.Sequences", &root);
        assert!(children.is_some());
        assert!(children.unwrap().is_empty());
    }

    #[test]
    fn dynamic_children_case_insensitive() {
        let root = test_root();
        let lower = dynamic_children_at("root.showdata.devices.dmx.patch.fixtures", &root);
        let mixed = dynamic_children_at("Root.ShowData.Devices.Dmx.Patch.Fixtures", &root);
        assert_eq!(
            lower.as_ref().map(|c| c.len()),
            mixed.as_ref().map(|c| c.len())
        );
    }

    // ── resolve_node tests ───────────────────────────────────────────────

    #[test]
    fn resolve_node_fixture_object() {
        let root = test_root();
        let node = resolve_node("Root.ShowData.Devices.Dmx.Patch.Fixtures.1", &root);
        assert!(node.is_some());
        match node.unwrap() {
            NodeView::Object {
                node_type,
                id,
                name,
                ..
            } => {
                assert_eq!(node_type, "Fixture");
                assert_eq!(id, 1);
                assert_eq!(name, Some("Front Wash L".to_string()));
            }
            _ => panic!("Expected Object"),
        }
    }

    #[test]
    fn resolve_node_nonexistent_object() {
        let root = test_root();
        let node = resolve_node("Root.ShowData.Devices.Dmx.Patch.Fixtures.999", &root);
        assert!(node.is_none());
    }

    #[test]
    fn resolve_node_structural() {
        let root = test_root();
        let node = resolve_node("Root.ShowData.Devices.Dmx.Patch", &root);
        assert!(node.is_some());
        match node.unwrap() {
            NodeView::Structural { name, children } => {
                assert_eq!(name, "Patch");
                assert!(!children.is_empty());
            }
            _ => panic!("Expected Structural"),
        }
    }

    // ── walk_tree tests ──────────────────────────────────────────────────

    #[test]
    fn walk_tree_finds_all_fixtures() {
        let root = test_root();
        let mut found = Vec::new();
        walk_tree(&root, "Root.ShowData.Devices.Dmx.Patch", &mut |path, child| {
            found.push((path.to_string(), child.clone()));
        });
        // Should find 3 fixtures + 1 universe
        assert_eq!(found.len(), 4);
        assert!(found.iter().any(|(_, c)| c.node_type == "Fixture"));
        assert!(found.iter().any(|(_, c)| c.node_type == "Universe"));
    }

    #[test]
    fn walk_tree_empty_subtree() {
        let root = Root::new();
        let mut found = Vec::new();
        walk_tree(&root, "Root.ShowData", &mut |path, child| {
            found.push((path.to_string(), child.clone()));
        });
        // Empty show — may find user profiles
        // Just verify it doesn't panic
    }

    // ── find_by_name tests ───────────────────────────────────────────────

    #[test]
    fn find_by_name_substring() {
        let root = test_root();
        let results = find_by_name(&root, "Root", "Front", None);
        assert_eq!(results.len(), 2); // Front Wash L, Front Wash R
    }

    #[test]
    fn find_by_name_with_type_filter() {
        let root = test_root();
        let results = find_by_name(&root, "Root", "1", Some("Universe"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.node_type, "Universe");
    }

    #[test]
    fn find_by_name_no_match() {
        let root = test_root();
        let results = find_by_name(&root, "Root", "zzzzzzz", None);
        assert!(results.is_empty());
    }

    #[test]
    fn find_by_name_glob_pattern() {
        let root = test_root();
        // display_name includes ID: `1 "Front Wash L"` — glob must account for this
        let results = find_by_name(&root, "Root", "*Front*", None);
        assert_eq!(results.len(), 2);
    }

    // ── grep_values tests ────────────────────────────────────────────────

    #[test]
    fn grep_values_finds_fixture_name() {
        let root = test_root();
        let results = grep_values(&root, "Root.ShowData.Devices.Dmx.Patch.Fixtures", "Back Light");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.id, 3);
    }

    #[test]
    fn grep_values_no_match() {
        let root = test_root();
        let results = grep_values(&root, "Root", "zzzzzzz");
        assert!(results.is_empty());
    }

    // ── count_objects tests ──────────────────────────────────────────────

    #[test]
    fn count_objects_fixtures() {
        let root = test_root();
        let counts = count_objects(&root, "Root.ShowData.Devices.Dmx.Patch.Fixtures");
        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0], ("Fixture", 3));
    }

    #[test]
    fn count_objects_all_patch() {
        let root = test_root();
        let counts = count_objects(&root, "Root.ShowData.Devices.Dmx.Patch");
        let total: usize = counts.iter().map(|(_, n)| n).sum();
        assert_eq!(total, 4); // 3 fixtures + 1 universe
    }

    #[test]
    fn count_direct_children_patch() {
        let root = test_root();
        let count = count_direct_children("Root.ShowData.Devices.Dmx.Patch", &root);
        // Patch has structural children: Fixtures, FixtureTypes, Universes
        assert!(count >= 3);
    }

    // ── diff_json tests ──────────────────────────────────────────────────

    #[test]
    fn diff_json_identical() {
        let a = serde_json::json!({"name": "test", "value": 42});
        let b = serde_json::json!({"name": "test", "value": 42});
        let diffs = diff_json(&a, &b);
        assert!(diffs.is_empty());
    }

    #[test]
    fn diff_json_different_values() {
        let a = serde_json::json!({"name": "Alpha", "value": 10});
        let b = serde_json::json!({"name": "Beta", "value": 10});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].0, "name");
    }

    #[test]
    fn diff_json_missing_field() {
        let a = serde_json::json!({"name": "test", "extra": true});
        let b = serde_json::json!({"name": "test"});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].0, "extra");
    }

    // ── glob_match tests ─────────────────────────────────────────────────

    #[test]
    fn glob_match_star_prefix() {
        assert!(glob_match("*wash", "front wash"));
    }

    #[test]
    fn glob_match_star_suffix() {
        assert!(glob_match("front*", "front wash l"));
    }

    #[test]
    fn glob_match_middle_star() {
        assert!(glob_match("front*l", "front wash l"));
    }

    #[test]
    fn glob_match_no_star() {
        assert!(glob_match("exact", "exact"));
        assert!(!glob_match("exact", "not exact"));
    }

    // ── Drives / filesystem tests ───────────────────────────────────

    /// A probe folder as a stick (decisions §38): `Root.Drives.usb1` is its root.
    fn stick() -> std::path::PathBuf {
        let tmp = std::env::temp_dir().join(format!("lumacore_vfs_stick_{}", std::process::id()));
        let _ = std::fs::create_dir_all(tmp.join("Vectorworks"));
        let vol = lumacore_drives::probe::Volume { key: "probe:stick".into(), label: "STICK".into(), mount: tmp.clone(), kind: lumacore_drives::probe::VolumeKind::Removable, free: 1, total: 2 };
        // The stick is remembered already, so the table doesn't change and no settings file is written.
        let mut s = crate::AppState::new();
        s.root.station_settings.drives.insert("usb1".into(), lumacore_tree::DriveMemory { key: "probe:stick".into(), kind: lumacore_tree::DriveKind::Usb, label: "STICK".into(), path: None, last_seen: 1 });
        s.drives = vec![crate::drives::Drive { id: "usb1".into(), kind: lumacore_tree::DriveKind::Usb, label: "STICK".into(), mount: tmp.clone(), present: true, free: 1, total: 2, since_ms: 1 }];
        crate::drives::reconcile(&mut s, &[vol], (0, 0));
        tmp
    }

    #[test]
    fn root_drives_is_the_drive_list_not_a_folder() {
        let tmp = stick();
        assert!(resolve_drive_path("Root.Drives").is_none());
        assert_eq!(resolve_drive_path("Root.Drives.usb1").unwrap(), tmp);
        assert_eq!(resolve_drive_path("root.drives.USB1.Vectorworks").unwrap(), tmp.join("Vectorworks"));
        assert!(resolve_drive_path("Root.Drives.usb7").is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn fs_children_at_temp_dir() {
        use std::fs;
        let tmp = std::env::temp_dir().join("lumacore_test_drives");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("hello.txt"), "world").unwrap();
        fs::create_dir_all(tmp.join("subdir")).unwrap();

        let children = fs_children_at(&tmp);
        assert!(children.len() >= 2);

        // Directories come first
        let dir_child = children.iter().find(|c| c.node_type == "Directory");
        assert!(dir_child.is_some());
        assert_eq!(dir_child.unwrap().display_name, "subdir");

        // File entry includes size annotation
        let file_child = children.iter().find(|c| c.node_type == "File");
        assert!(file_child.is_some());
        assert!(file_child.unwrap().display_name.starts_with("hello.txt"));

        // Cleanup
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn dynamic_children_at_drives_root() {
        let tmp = stick();
        let root = test_root();
        let children = dynamic_children_at("Root.Drives", &root).unwrap();
        let names: Vec<&str> = children.iter().map(|c| c.display_name.as_str()).collect();
        assert!(names.contains(&"local-library") && names.contains(&"usb1"), "{names:?}");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn resolve_drive_path_nonexistent() {
        let path = resolve_drive_path("Root.Drives.this_path_should_not_exist_zzz_12345");
        assert!(path.is_none());
    }

    #[test]
    fn format_file_size_units() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert!(format_file_size(2048).contains("KB"));
        assert!(format_file_size(2 * 1024 * 1024).contains("MB"));
    }

    #[test]
    fn is_text_extension_common() {
        assert!(is_text_extension("txt"));
        assert!(is_text_extension("JSON"));
        assert!(is_text_extension("rs"));
        assert!(is_text_extension("gdtf"));
        assert!(!is_text_extension("exe"));
        assert!(!is_text_extension("png"));
    }

    #[test]
    fn walk_tree_skips_drives() {
        let root = test_root();
        let mut found_drives = false;
        walk_tree(&root, "Root", &mut |path, _child| {
            if path.to_lowercase().contains("drives") {
                found_drives = true;
            }
        });
        assert!(
            !found_drives,
            "walk_tree should skip Drives to avoid filesystem walk"
        );
    }
}

