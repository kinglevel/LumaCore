// ████████╗ █████╗  ██████╗ ███████╗
// ╚══██╔══╝██╔══██╗██╔════╝ ██╔════╝
//    ██║   ███████║██║  ███╗███████╗
//    ██║   ██╔══██║██║   ██║╚════██║
//    ██║   ██║  ██║╚██████╔╝███████║
//    ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
// FILE: tags

//! The Tags pool window (decisions §12 "Tags hardened", the locked mockup round 2): the tag tree
//! with each tag's member count and Action badge, and the picked tag's members — each object that
//! carries it (Direct) or it and the tags under it (Deep), "from Right" when through a child. Built
//! for the `tags` feed from the published show; every gesture the page makes is one of the lines
//! given here.
//!
//! Owns:          `view` (the tree and the members pane).
//! Doesn't own:   the tag tree and its member index (`lumacore-tags`), the Tag words
//!                (`lumacore-cmd-tags`), the window's settings (`lumacore-tree` window types).
//! Entry points:  view.
//! Decisions:     §12 "Tags hardened", §34 20.7.

use std::collections::{BTreeMap, BTreeSet};

use lumacore_objects::menu_defs::{Col, frozen, ro};
use lumacore_tree::ShowData;
use lumacore_tree::tags::tag::{self as tree, Tag, TagAction, TagPath};
use serde_json::{Value, json};
use uuid::Uuid;

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The members grid's columns (read-only: a member changes through its own EditSetting).
static COLUMNS: &[Col] = &[frozen(ro("no", "No", 64, 1)), ro("kind", "Kind", 120, 1), ro("name", "Name", 200, 1), ro("via", "Tagged", 140, 2)];

/// The Tags window's view. `which` = the picked tag's GUID (the window's `Tag` setting; empty =
/// none picked), `deep` = members of the tags under it too.
pub fn view(show: &ShowData, which: &Value, deep: bool) -> Value {
    let walk = show.walk_identity();
    // Who carries each tag directly, in the walk's order.
    let mut carried: BTreeMap<Uuid, Vec<Uuid>> = BTreeMap::new();
    for l in walk.links.iter().filter(|l| l.kind == "Tag") {
        if let Some(from) = l.from {
            let list = carried.entry(l.target).or_default();
            if !list.contains(&from) {
                list.push(from);
            }
        }
    }
    let all = tree::all(&show.tags);
    let reach = |t: &Tag| -> Vec<Uuid> { if deep { tree::subtree(t) } else { vec![t.meta.identity.guid] } };
    let count = |t: &Tag| -> usize { reach(t).iter().flat_map(|g| carried.get(g).into_iter().flatten()).collect::<BTreeSet<_>>().len() };
    let nodes: Vec<Value> = all
        .iter()
        .map(|(path, t)| {
            json!({
                "path": path.to_string(), "guid": t.meta.identity.guid, "name": t.name(),
                "colour": crate::attribute_board::hex(t.meta.annotation.paint),
                "count": count(t), "badge": badge(t.action),
            })
        })
        .collect();
    let picked = which.as_str().and_then(|s| Uuid::parse_str(s).ok());
    let found = picked.and_then(|g| all.iter().find(|(_, t)| t.meta.identity.guid == g));
    let Some((path, tag)) = found else {
        return json!({ "nodes": nodes, "deep": deep, "tag": null, "members": [], "missing": picked.is_some() });
    };
    // The members, each once: direct first, then through the tags under it.
    let objects: BTreeMap<Uuid, &lumacore_base::ObjectEntry> = walk.objects.iter().map(|o| (o.guid, o)).collect();
    let mut seen = BTreeSet::new();
    let mut members = Vec::new();
    for via in reach(tag) {
        let through = (via != tag.meta.identity.guid).then(|| all.iter().find(|(_, t)| t.meta.identity.guid == via)).flatten();
        for from in carried.get(&via).into_iter().flatten() {
            if !seen.insert(*from) {
                continue;
            }
            let Some(o) = objects.get(from) else { continue };
            members.push(member(show, o, through.map(|(_, t)| t.name())));
        }
    }
    let mut kinds: Vec<(String, usize)> = Vec::new();
    for m in &members {
        let k = m["kind"].as_str().unwrap_or_default();
        match kinds.iter_mut().find(|(n, _)| n == k) {
            Some((_, n)) => *n += 1,
            None => kinds.push((k.to_string(), 1)),
        }
    }
    json!({
        "nodes": nodes,
        "deep": deep,
        "tag": picked_tag(path, tag, deep),
        "members": members,
        "columns": COLUMNS.iter().filter_map(|c| crate::menus::column_json(None, c).ok()).collect::<Vec<_>>(),
        "kinds": kinds.iter().map(|(k, n)| json!({ "kind": k, "count": n })).collect::<Vec<_>>(),
        "missing": false,
    })
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The Action badge of a tag with its own Action (none for the window's default).
fn badge(action: TagAction) -> Value {
    match action {
        TagAction::WindowDefault => Value::Null,
        TagAction::SelectFixtures => json!("Select fixtures"),
    }
}

/// The picked tag and the lines its pane and ≡ menu run (`{name}` = a name the page asks for).
fn picked_tag(path: &TagPath, tag: &Tag, deep: bool) -> Value {
    let r = if deep { " /R" } else { "" };
    json!({
        "path": path.to_string(), "guid": tag.meta.identity.guid, "name": tag.name(),
        "own_action": !tag.action.is_default(),
        "lines": {
            "select": format!("Tag {path}{r}"),
            "settings": format!("EditSetting Tag {path}"),
            "label": format!("Label Tag {path} {{name}}"),
            "child": format!("Store Tag {path} Tag {{name}}"),
            "top": format!("Move Tag {path} At Root"),
            "delete": format!("Delete Tag {path}"),
        },
    })
}

/// One member row: its kind, number and name, the target a line names it by
/// (`DataPool 2 Sequence 3` for an object in a DataPool), and "direct" or "from <tag>".
fn member(show: &ShowData, o: &lumacore_base::ObjectEntry, through: Option<&str>) -> Value {
    let parts: Vec<&str> = o.path.split('.').collect();
    // A tag carrying a tag is named by its path; anything else by its number in its pool.
    let no = match tree::path_of(&show.tags, o.guid) {
        Some(p) => p.to_string(),
        None => parts.last().copied().unwrap_or_default().to_string(),
    };
    let datapool = match parts.as_slice() {
        ["DataPools", dp, _, ..] if parts.len() > 3 => Some(*dp),
        _ => None,
    };
    let target = match datapool {
        Some(dp) => format!("DataPool {dp} {} {no}", o.kind),
        None => format!("{} {no}", o.kind),
    };
    json!({
        "guid": o.guid, "kind": o.kind, "no": no, "name": o.name.clone().unwrap_or_default(),
        "target": target,
        "via": through.map_or_else(|| "direct".to_string(), |t| format!("from {t}")),
    })
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
