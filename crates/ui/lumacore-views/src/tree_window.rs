// ████████╗██████╗ ███████╗███████╗    ██╗    ██╗██╗███╗   ██╗██████╗  ██████╗ ██╗    ██╗
// ╚══██╔══╝██╔══██╗██╔════╝██╔════╝    ██║    ██║██║████╗  ██║██╔══██╗██╔═══██╗██║    ██║
//    ██║   ██████╔╝█████╗  █████╗      ██║ █╗ ██║██║██╔██╗ ██║██║  ██║██║   ██║██║ █╗ ██║
//    ██║   ██╔══██╗██╔══╝  ██╔══╝      ██║███╗██║██║██║╚██╗██║██║  ██║██║   ██║██║███╗██║
//    ██║   ██║  ██║███████╗███████╗    ╚███╔███╔╝██║██║ ╚████║██████╔╝╚██████╔╝╚███╔███╔╝
//    ╚═╝   ╚═╝  ╚═╝╚══════╝╚══════╝     ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝╚═════╝  ╚═════╝  ╚══╝╚══╝
// FILE: tree_window

//! The Tree window (decisions 0292): NAV's tree and inspector for the web and the app. Two views,
//! each the data of one feed: `tree` — the rows of the window's open branches from the one row
//! builder `ls` and NAV read (`lumacore_objects::tree_rows`), with NAV's filter, the user's
//! bookmarks as badges and, when the window follows the selection, the object it should jump to —
//! and `inspector` — `inspector::view` of one place as JSON, its properties as settings cells.
//! A place is a typed tree path or a GUID (whole or an unambiguous prefix of at least
//! [`GUID_PREFIX`] hex digits). Both read the station's state as the user (their datapool, their
//! bookmarks) and change nothing.
//!
//! Owns:          TreeParams, tree, inspector, place, path_of_guid, matches, follow_target.
//! Doesn't own:   the rows (tree_rows), the inspector's rows (inspector), the feeds (lumacore-http),
//!                drawing (the kit's <luma-tree-window>), NAV's keys (lumacore-terminal).
//! Entry points:  tree, inspector, place, path_of_guid, matches.
//! Decisions:     docs/decisions/0292-tree-window-nav-in-the-web-and-app, docs/decisions/0291-nav-mode-ls-rows-and-an-inspector, docs/decisions/0106-tree-paths-typed-node.

use std::collections::{BTreeMap, BTreeSet};

use lumacore_objects::AppState;
use lumacore_objects::navigation::Navigation;
use lumacore_objects::tree_rows::{self, RowKind, TreeRow};
use serde_json::{Value, json};

use crate::inspector::{Inspector, Section};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a Tree window asks the `tree` feed for (its settings).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeParams {
    /// The selected place (`Self.Path`): a tree path or a GUID; empty = [`HOME`].
    pub path: String,
    /// The branches the user opened (`Self.Open`); the selected place's parents open by themselves.
    pub open: BTreeSet<String>,
    /// NAV's filter over the open branches' objects (`Self.Search`): a number is a slot.
    pub search: String,
    /// Follow = Selected: the data names the object the window should jump to.
    pub follow: bool,
}

/// The shortest GUID prefix a place takes (the first block's 8 hex digits).
pub const GUID_PREFIX: usize = 8;
/// Where a window with no place stands.
pub const HOME: &str = "Root.ShowData";
/// Rows a tree sends at most; past it the data says how many were cut (a search narrows).
pub const MAX_ROWS: usize = 5000;
/// Object kinds Follow never jumps to: the user's own desk, not what they work on.
const NOT_FOLLOWED: &[&str] = &["Window", "View", "ViewButton", "UserProfile", "User", "Screen"];

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Where `text` leads: a whole GUID or an unambiguous prefix of one, else a tree path from the
/// root (cd's one parse, never from the shell's place). Err says why, ready for a toast.
pub fn place(state: &mut AppState, text: &str) -> Result<String, String> {
    let text = text.trim().trim_matches('"').trim();
    if text.is_empty() {
        return Ok(HOME.to_string());
    }
    if let Some(guid) = guid_of(state, text)? {
        return path_of_guid(state, guid).ok_or_else(|| format!("No object with GUID {text} in this show"));
    }
    // From the root: a path bar is absolute (`ShowData.Groups` = `Root.ShowData.Groups`).
    let here = std::mem::take(&mut state.navigation_path);
    let absolute = if text.starts_with("Root") || text.starts_with('/') || text.starts_with('~') { text.to_string() } else { format!("Root.{text}") };
    let found = state.resolve_target(&absolute);
    state.navigation_path = here;
    let segments = found.map_err(|e| lumacore_objects::text::strip_ansi(&e).trim().to_string())?;
    let dot = lumacore_objects::tree_path::dot(&segments);
    match lumacore_objects::tree_path::TreePath::parse_dot(&dot) {
        Some(p) if !lumacore_objects::vfs::path_exists(&state.root, &p) => Err(format!("No node at {text} in this show")),
        _ => Ok(dot),
    }
}

/// The GUID `text` names: Ok(None) when it isn't one; an Err when a prefix fits no object or
/// several.
fn guid_of(state: &AppState, text: &str) -> Result<Option<uuid::Uuid>, String> {
    if let Ok(g) = text.parse::<uuid::Uuid>() {
        return Ok(Some(g));
    }
    let hex = text.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    if !hex || text.chars().filter(char::is_ascii_hexdigit).count() < GUID_PREFIX {
        return Ok(None);
    }
    let prefix = text.to_lowercase();
    let walk = state.root.show_data.walk_identity();
    let mut hits = walk.objects.iter().filter(|o| o.guid.to_string().starts_with(&prefix)).map(|o| o.guid);
    match (hits.next(), hits.next()) {
        (Some(g), None) => Ok(Some(g)),
        (Some(_), Some(_)) => Err(format!("More than one object's GUID starts with {text}: type more of it")),
        (None, _) => Err(format!("No object with GUID {text} in this show")),
    }
}

/// Where an object is in the tree now, by its GUID (NAV's bookmarks and links use it too).
pub fn path_of_guid(state: &AppState, guid: uuid::Uuid) -> Option<String> {
    let walk = state.root.show_data.walk_identity();
    let o = walk.objects.iter().find(|o| o.guid == guid)?;
    let path = format!("{}.{}", lumacore_objects::tree_path::Node::ShowData.path(), o.path);
    state.resolve_target(&path).ok().map(|segments| lumacore_objects::tree_path::dot(&segments))
}

/// NAV's filter: a number matches the slot, words the name or the row's text (any case).
pub fn matches(row: &TreeRow, filter: &str) -> bool {
    if let Ok(n) = filter.trim().parse::<u32>() {
        return row.slot == Some(n);
    }
    let f = filter.trim().to_lowercase();
    row.name.to_lowercase().contains(&f) || row.text().to_lowercase().contains(&f)
}

/// The object Follow = Selected jumps to, and a key that changes with each new pick: the
/// object the user's newest line named (their `Usage`), or — when that was a fixture — the last
/// fixture of their programmer selection (`Fixture 1 Thru 10` → Fixture 10).
pub fn follow_target(state: &AppState) -> Option<(String, String)> {
    let show = &state.root.show_data;
    let user = show.user(state.effective_user_name())?;
    let walk = show.walk_identity();
    let kinds: BTreeMap<uuid::Uuid, &str> = walk.objects.iter().map(|o| (o.guid, o.kind)).collect();
    let (guid, last_ms) = user
        .usage
        .entries
        .iter()
        .filter_map(|(k, e)| k.parse::<uuid::Uuid>().ok().map(|g| (g, e.last_ms)))
        .filter(|(g, _)| kinds.get(g).is_some_and(|k| !NOT_FOLLOWED.contains(k)))
        .max_by_key(|(g, ms)| (*ms, *g))?;
    let guid = match (kinds.get(&guid), user.programmer.selected_fixtures.last()) {
        (Some(&"Fixture"), Some(last)) => *last,
        _ => guid,
    };
    Some((path_of_guid(state, guid)?, format!("{guid}@{last_ms}")))
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The `tree` feed's data: the rows of the open branches below `Root` (each with its depth, slot,
/// name, kind, meta, whether it is open and the user's bookmark letters), the selected place
/// (`path`, or `missing` with why), the objects the filter hid, the rows past [`MAX_ROWS`] and
/// `follow` when asked.
pub fn tree(state: &mut AppState, p: &TreeParams) -> Value {
    let (at, missing) = match place(state, &p.path) {
        Ok(at) => (Some(at), None),
        Err(why) => (None, Some(why)),
    };
    // The selected place's parents open by themselves (a jump shows where it landed).
    let mut open = p.open.clone();
    if let Some(at) = &at {
        let mut up = at.as_str();
        while let Some((parent, _)) = up.rsplit_once('.') {
            open.insert(parent.to_string());
            up = parent;
        }
    }
    let marks = bookmarks(state);
    let mut rows = Vec::new();
    let mut hidden = 0usize;
    collect(state, "Root", 0, &open, p.search.trim(), &marks, &mut rows, &mut hidden);
    let cut = rows.len().saturating_sub(MAX_ROWS);
    rows.truncate(MAX_ROWS);
    let follow = if p.follow { follow_target(state).map(|(path, key)| json!({ "path": path, "key": key })) } else { None };
    json!({
        "path": at,
        "missing": missing,
        "rows": rows,
        "hidden": hidden,
        "cut": cut,
        "follow": follow,
    })
}

/// The `inspector` feed's data: `inspector::view` of the place, or `{missing}` with why.
pub fn inspector(state: &mut AppState, text: &str) -> Value {
    let at = match place(state, text) {
        Ok(at) => at,
        Err(why) => return json!({ "missing": why, "asked": text }),
    };
    let i = crate::inspector::view(state, &at);
    let marks = bookmarks(state);
    to_json(&i, i.guid.and_then(|g| marks.get(&g)).cloned().unwrap_or_default())
}

/// An inspector as the kit reads it.
fn to_json(i: &Inspector, marks: String) -> Value {
    let section = |s: Section| match s {
        Section::Identity => "identity",
        Section::Info => "info",
        Section::Property => "property",
        Section::Link => "link",
    };
    let rows: Vec<Value> = i
        .rows
        .iter()
        .map(|r| {
            json!({
                "section": section(r.section), "name": r.name, "value": r.value, "kind": r.kind,
                "set": r.set, "changed": r.changed, "reset": r.reset, "link": r.link, "missing": r.missing,
                "field": r.field,
            })
        })
        .collect();
    json!({
        "title": i.title, "path": i.path, "target": i.target, "guid": i.guid.map(|g| g.to_string()),
        "name": i.name, "marks": marks, "rows": rows,
    })
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The rows below `path` and below every open one, NAV's filter on the objects.
#[allow(clippy::too_many_arguments, reason = "one recursive walk")]
fn collect(state: &AppState, path: &str, depth: usize, open: &BTreeSet<String>, filter: &str, marks: &BTreeMap<uuid::Uuid, String>, out: &mut Vec<Value>, hidden: &mut usize) {
    for row in tree_rows::rows(state, path) {
        let hit = !filter.is_empty() && matches(&row, filter);
        if !filter.is_empty() && row.kind != RowKind::Node && !hit {
            *hidden += 1;
            continue;
        }
        let opened = row.branch && open.contains(&row.path);
        out.push(row_json(state, &row, depth, opened, hit, marks));
        if opened && out.len() <= MAX_ROWS {
            collect(state, &row.path, depth + 1, open, filter, marks, out, hidden);
        }
    }
}

fn row_json(state: &AppState, r: &TreeRow, depth: usize, open: bool, hit: bool, marks: &BTreeMap<uuid::Uuid, String>) -> Value {
    let kind = match r.kind {
        RowKind::Node => "node",
        RowKind::Object => "object",
        RowKind::Folder => "folder",
        RowKind::Dir => "dir",
        RowKind::File => "file",
    };
    json!({
        "path": r.path, "depth": depth, "kind": kind, "no": r.slot, "name": r.name, "quoted": r.quoted,
        "type": r.type_name, "meta": r.meta, "branch": r.branch, "open": open, "hit": hit,
        "guid": r.guid.map(|g| g.to_string()), "object": object_word(state, &r.path),
        "target": object_word(state, &r.path).zip(r.slot).map(|(o, n)| format!("{o} {n}")),
        "marks": r.guid.and_then(|g| marks.get(&g)).cloned().unwrap_or_default(),
        // Its colour as the viewer sees it (decisions 0294): the swatch.
        "paint": r.paint,
    })
}

/// The words a line names the object at `path` by (`Group`, `DataPool 2 Group`): owner first
/// when it lives in another datapool than the user's (decisions §33); None for a node.
fn object_word(state: &AppState, path: &str) -> Option<String> {
    let p = lumacore_objects::tree_path::TreePath::parse_dot(path)?;
    let (object, datapool, _) = lumacore_objects::objects::at_path(&p)?;
    Some(match datapool {
        Some(dp) if dp != state.selected_datapool() => format!("DataPool {dp} {}", object.keyword),
        _ => object.keyword.to_string(),
    })
}

/// The acting user's bookmarks: each GUID with its letters (`ab`).
fn bookmarks(state: &AppState) -> BTreeMap<uuid::Uuid, String> {
    let mut out: BTreeMap<uuid::Uuid, String> = BTreeMap::new();
    if let Some(u) = state.root.show_data.user(state.effective_user_name()) {
        for (letter, guid) in &u.bookmarks.marks {
            out.entry(*guid).or_default().push(*letter);
        }
    }
    out
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
