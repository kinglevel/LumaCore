//! Tree navigation helpers: path display, context derivation, `cd` logic, and static tree.

use super::ReplContext;
use super::app_state::AppState;
use super::tree_path::TreePath;
use super::vfs;

/// Tree navigation on AppState (decisions §31 "Tree paths are typed"): an extension of the state, since it reads the VFS above it (decisions §34 15).
pub trait Navigation {
    fn path_string(&self) -> String;
    fn scope_label(&self) -> String;
    fn tree_path(&self) -> TreePath;
    fn claimable_at_path(&self) -> Option<String>;
    fn context_from_path(&self) -> ReplContext;
    fn resolve_target(&self, target: &str) -> Result<Vec<String>, String>;
    fn cd_into(&mut self, target: &str) -> Result<String, String>;
    fn step_into(&self, at: &[String], target: &str) -> Result<String, String>;
}

impl Navigation for AppState {
    /// Build the path string for the current tree navigation position.
    fn path_string(&self) -> String {
        if self.navigation_path.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", self.navigation_path.join("/"))
        }
    }

    /// Derive the scope label from the current navigation path.
    ///
    /// Returns the last segment of the path (e.g. "Patch", "Library", "Groups"),
    /// or "Root" when at the top level.  Used for the prompt display.
    fn scope_label(&self) -> String {
        match self.navigation_path.last() {
            Some(seg) => seg.clone(),
            None => "Root".to_string(),
        }
    }

    /// Where the shell stands, as a tree path (`Root` when the path doesn't parse).
    fn tree_path(&self) -> TreePath {
        TreePath::parse_dot(&super::tree_path::dot(&self.navigation_path)).unwrap_or_default()
    }

    /// The object `enter` claims at the current position (`Node::claim`); none at the root.
    fn claimable_at_path(&self) -> Option<String> {
        if self.navigation_path.is_empty() {
            return None;
        }
        self.tree_path().node().claim().map(str::to_string)
    }

    /// The REPL context the current position sets (`Node::context`).
    fn context_from_path(&self) -> ReplContext {
        self.tree_path().node().context()
    }

    /// Where `target` leads from the current position, as navigation segments — the one parse
    /// of cd, enter and ls (decisions §31): an alias (`patch`, `claims`), `/` or `Root`, an
    /// absolute path (`Root.ShowData.Devices`, `/ShowData/Devices`) or a relative one
    /// (`Devices.Dmx.Patch`, `Devices/Dmx/Patch`, `..`); each name matches like cd always did
    /// (exact, then a unique prefix, then a unique substring; objects by number).
    fn resolve_target(&self, target: &str) -> Result<Vec<String>, String> {
        if let Some(node) = super::tree_path::alias(target) {
            return Ok(node.path().split('.').skip(1).map(str::to_string).collect());
        }
        let lower = target.to_lowercase();
        let (start, rest): (Vec<String>, &str) = if lower == "/" || lower == "root" {
            return Ok(Vec::new());
        } else if let Some(rest) = super::tree_path::strip_root(target) {
            (Vec::new(), rest)
        } else if let Some(rest) = target.strip_prefix('/') {
            (Vec::new(), rest)
        } else if let Some(rest) = target.strip_prefix('~') {
            // Unix's home (decisions §38): `~` the local library, `~usb1` that drive's library.
            let (id, rest) = rest.split_once(['.', '/']).unwrap_or((rest, ""));
            let mut start = vec![super::tree_path::Node::Drives.name().to_string()];
            if id.is_empty() || id.eq_ignore_ascii_case(super::drives::LOCAL) {
                start.push(super::drives::LOCAL.to_string());
            } else {
                let Some(drive) = self.drive(id) else {
                    return Err(format!("No drive \"{id}\" (List Drive)"));
                };
                start.push(drive.id.clone());
                for part in super::drives::LIBRARY_DIR.split('/') {
                    start.push(part.to_string());
                }
            }
            (start, rest)
        } else {
            (self.navigation_path.clone(), target)
        };
        // A path with `/` keeps dots in its names (files on a drive).
        let segments: Vec<&str> = if rest.contains('/') || rest == ".." {
            rest.split('/').collect()
        } else {
            rest.split('.').collect()
        };
        let mut path = start;
        for segment in segments {
            match segment {
                "" | "." => {}
                ".." => {
                    path.pop();
                }
                _ => {
                    let name = self.step_into(&path, segment)?;
                    path.push(name);
                }
            }
        }
        Ok(path)
    }

    /// Try to navigate into a named child node from the current navigation path.
    ///
    /// Returns Ok(display_name) if the target is a valid child of the current node,
    /// or Err(message) if the target is not a known child at this level.
    fn cd_into(&mut self, target: &str) -> Result<String, String> {
        let display = self.step_into(&self.navigation_path, target)?;
        self.navigation_path.push(display.clone());
        Ok(display)
    }

    /// The child of `at` that `target` names (exact, then a unique prefix, then a unique
    /// substring; objects by number or name; files on a drive by name).
    fn step_into(&self, at: &[String], target: &str) -> Result<String, String> {
        let target_lower = target.to_lowercase();
        let here = if at.is_empty() { "/".to_string() } else { format!("/{}", at.join("/")) };
        let current_dot_path = super::tree_path::dot(at);

        // Use the same static tree the completer uses to validate children.
        let valid_children = nav_children_at(&current_dot_path);

        // Try exact case-insensitive match first, then prefix match, then substring match.
        let exact = valid_children
            .iter()
            .find(|n| n.to_lowercase() == target_lower);
        let canonical = if let Some(c) = exact {
            Some(c.clone())
        } else {
            // Prefix match — require unique prefix
            let prefix_matches: Vec<String> = valid_children
                .iter()
                .filter(|n| n.to_lowercase().starts_with(&target_lower))
                .cloned()
                .collect();
            if prefix_matches.len() == 1 {
                Some(prefix_matches[0].clone())
            } else if prefix_matches.len() > 1 {
                return Err(format!(
                    "Ambiguous: '{}' matches {}",
                    target,
                    prefix_matches.join(", ")
                ));
            } else {
                // Substring match — require unique substring (lenient fallback)
                let substr_matches: Vec<String> = valid_children
                    .iter()
                    .filter(|n| n.to_lowercase().contains(&target_lower))
                    .cloned()
                    .collect();
                if substr_matches.len() == 1 {
                    Some(substr_matches[0].clone())
                } else if substr_matches.len() > 1 {
                    return Err(format!(
                        "Ambiguous: '{}' matches {}",
                        target,
                        substr_matches.join(", ")
                    ));
                } else {
                    None
                }
            }
        };

        // A number under a declared node is its child's No (decisions §33 54); entries keep
        // their member numbers below.
        let numbered = target
            .parse::<u32>()
            .ok()
            .and_then(|_| super::tree_path::TreePath::parse_dot(&current_dot_path)?.join(target))
            .and_then(|p| match p.steps().last() {
                Some(super::tree_path::Step::Node(n)) => Some(n.name().to_string()),
                _ => None,
            });
        if let Some(display) = canonical.or(numbered) {
            Ok(display)
        } else if valid_children.is_empty() || target.parse::<u32>().is_ok() {
            // Static tree says leaf (or a number next to named children, as
            // the numbered datapools) — try VFS dynamic children (objects).
            // Check if target matches a dynamic child by ID or name.
            let is_under_drives = super::tree_path::names_drive(&current_dot_path);
            if let Some(dyn_children) = vfs::dynamic_children_at(&current_dot_path, &self.root) {
                if is_under_drives {
                    // Drives mode: match by filesystem entry name (strip size annotation)
                    let fs_name_of = |c: &vfs::DynChild| -> String {
                        c.display_name
                            .split("  (")
                            .next()
                            .unwrap_or(&c.display_name)
                            .to_string()
                    };
                    // Exact match first (case-insensitive)
                    let exact = dyn_children
                        .iter()
                        .find(|c| fs_name_of(c).to_lowercase() == target_lower);
                    if let Some(child) = exact {
                        let display = fs_name_of(child);
                        return Ok(display);
                    }
                    // Prefix match
                    let prefix_matches: Vec<&vfs::DynChild> = dyn_children
                        .iter()
                        .filter(|c| fs_name_of(c).to_lowercase().starts_with(&target_lower))
                        .collect();
                    if prefix_matches.len() == 1 {
                        let display = fs_name_of(prefix_matches[0]);
                        return Ok(display);
                    } else if prefix_matches.len() > 1 {
                        let candidates: Vec<String> = prefix_matches
                            .iter()
                            .take(10)
                            .map(|c| fs_name_of(c))
                            .collect();
                        return Err(format!(
                            "Ambiguous: '{}' matches {}",
                            target,
                            candidates.join(", ")
                        ));
                    }
                } else {
                    // Standard VFS mode: match by ID or name
                    if let Ok(id) = target.parse::<u32>()
                        && dyn_children.iter().any(|c| c.id == id)
                    {
                        let display = id.to_string();
                        return Ok(display);
                    }
                    let name_matches: Vec<&vfs::DynChild> = dyn_children
                        .iter()
                        .filter(|c| {
                            c.display_name.to_lowercase().contains(&target_lower)
                                || c.id.to_string() == target
                        })
                        .collect();
                    if name_matches.len() == 1 {
                        // Named sub-folders (e.g. "Screens") keep their name in the path.
                        let display = if name_matches[0].node_type == "Folder" {
                            name_matches[0].display_name.clone()
                        } else {
                            name_matches[0].id.to_string()
                        };
                        return Ok(display);
                    } else if name_matches.len() > 1 {
                        let candidates: Vec<String> = name_matches
                            .iter()
                            .map(|c| c.display_name.clone())
                            .collect();
                        return Err(format!(
                            "Ambiguous: '{}' matches {}",
                            target,
                            candidates.join(", ")
                        ));
                    }
                }
                if dyn_children.is_empty() {
                    Err(format!(
                        "'{}' has no sub-nodes. You are at a leaf.",
                        here
                    ))
                } else {
                    let available: Vec<String> = dyn_children
                        .iter()
                        .take(10)
                        .map(|c| c.display_name.clone())
                        .collect();
                    let suffix = if dyn_children.len() > 10 {
                        format!(" ... ({} total)", dyn_children.len())
                    } else {
                        String::new()
                    };
                    Err(format!(
                        "Unknown node '{}'. Available here: {}{}",
                        target,
                        available.join(", "),
                        suffix
                    ))
                }
            } else {
                Err(format!(
                    "'{}' has no sub-nodes. You are at a leaf.",
                    here
                ))
            }
        } else {
            let available = valid_children.join(", ");
            Err(format!(
                "Unknown node '{}'. Available here: {}",
                target, available
            ))
        }
    }
}

// ── Navigation tree helpers ───────────────────────────────────────────────────

/// The child node names at a dot path (`Root.ShowData.DataPools.2`): the declared children of
/// its node in the one table (`crate::tree_path`); nothing below an object, nothing for a path
/// the table doesn't know. Numbered entries and objects are the VFS's (`vfs::dynamic_children_at`).
pub fn nav_children_at(path: &str) -> Vec<String> {
    match crate::tree_path::TreePath::parse_dot(path) {
        Some(p) if p.entry().is_none() => p.node().children().map(|n| n.name().to_string()).collect(),
        _ => Vec::new(),
    }
}
