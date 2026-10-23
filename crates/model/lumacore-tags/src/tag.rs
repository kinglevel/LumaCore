// ████████╗ █████╗  ██████╗
// ╚══██╔══╝██╔══██╗██╔════╝
//    ██║   ███████║██║  ███╗
//    ██║   ██╔══██║██║   ██║
//    ██║   ██║  ██║╚██████╔╝
//    ╚═╝   ╚═╝  ╚═╝ ╚═════╝
// FILE: tag

//! A tag and the tree of tags (decisions §12 "Tags hardened" 2–5): one parent per tag, numbered
//! within the parent, so a tag's number is its path (`1.2.1`). A move renumbers the tag at its new
//! place; the links to it are GUIDs, so tagged objects never notice. What carries a tag is read
//! from the show's identity walk (`NodeMeta.tags` are links of kind `Tag`).
//!
//! Owns:          Tag, TagAction, TagPath, TagTree, Member, the tree operations, members.
//! Doesn't own:   the store (ShowData.tags), the words and the asks (lumacore-cmd-tags).
//! Entry points:  find, find_mut, path_of, all, insert, remove, move_under, lift_children,
//!                siblings, named, subtree, members.
//! Decisions:     §12 "Tags hardened", the lum.md id reading (Path).

use crate::TagError;
use lumacore_base::identity::{Identified, Walk, Walker};
use lumacore_base::node::NodeMeta;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The tags at one level, by their number under the parent (`ShowData.tags` is the top level).
pub type TagTree = BTreeMap<u32, Tag>;

/// A tag: its NodeMeta (name, colour, icon, note, and its own tags), its action and its child tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub meta: NodeMeta,
    /// What a tap on the tag runs in the Tags pool window (decisions §12 "Tags hardened" 9).
    #[serde(default, skip_serializing_if = "TagAction::is_default")]
    pub action: TagAction,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub children: TagTree,
}

/// A tag's tap action: the window's default, or its own. Go+ / Off / Toggle come with playback
/// (decisions §12 "Tags hardened" 9).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TagAction {
    /// The Tags pool window's own default action.
    #[default]
    WindowDefault,
    /// Select the tag's fixtures.
    SelectFixtures,
}

/// A tag's number: one part per level (`1.2.1`). Never a decimal: `1.10` ≠ `1.1`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TagPath(pub Vec<u32>);

/// An object that carries a tag: its GUID and kind, as the identity walk found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub guid: Uuid,
    pub kind: &'static str,
    /// The tag it carries (one of the tags asked for).
    pub via: Uuid,
}

// ██╗███╗   ██╗██╗████████╗
// ██║████╗  ██║██║╚══██╔══╝
// ██║██╔██╗ ██║██║   ██║
// ██║██║╚██╗██║██║   ██║
// ██║██║ ╚████║██║   ██║
// ╚═╝╚═╝  ╚═══╝╚═╝   ╚═╝
// PART: INIT

impl Tag {
    /// A new tag with a fresh GUID and no children.
    pub fn new(name: Option<String>) -> Self {
        Self { meta: NodeMeta::new(name), action: TagAction::default(), children: TagTree::new() }
    }

    /// Its name, or "" when it has none.
    pub fn name(&self) -> &str {
        self.meta.identity.name.as_deref().unwrap_or("")
    }
}

impl TagAction {
    pub fn is_default(&self) -> bool {
        *self == TagAction::WindowDefault
    }
}

impl TagPath {
    /// A path from its dotted text (`"1.2.1"`). Every part is a whole number above 0;
    /// leading zeros are refused, so `1.01` never quietly means `1.1`.
    pub fn parse(text: &str) -> Result<Self, TagError> {
        let bad = || TagError::BadPath(text.to_string());
        let mut parts = Vec::new();
        for part in text.split('.') {
            if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) || (part.len() > 1 && part.starts_with('0')) {
                return Err(bad());
            }
            let n: u32 = part.parse().map_err(|_| bad())?;
            if n == 0 {
                return Err(bad());
            }
            parts.push(n);
        }
        Ok(Self(parts))
    }

    /// The tag's parent (None for a top tag).
    pub fn parent(&self) -> Option<TagPath> {
        (self.0.len() > 1).then(|| TagPath(self.0[..self.0.len() - 1].to_vec()))
    }

    /// Its number under the parent.
    pub fn last(&self) -> u32 {
        *self.0.last().unwrap_or(&0)
    }

    /// The path of its child number `n`.
    pub fn child(&self, n: u32) -> TagPath {
        let mut p = self.0.clone();
        p.push(n);
        TagPath(p)
    }

    /// Whether `other` is this tag or under it.
    pub fn contains(&self, other: &TagPath) -> bool {
        other.0.len() >= self.0.len() && other.0[..self.0.len()] == self.0[..]
    }
}

impl fmt::Display for TagPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.0.iter().map(|n| n.to_string()).collect();
        write!(f, "{}", parts.join("."))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

// *************************************************************************************************
// Find
// A tag by its path, by its GUID, or by name among one parent's children.
// *************************************************************************************************

/// The tag at `path`.
pub fn find<'a>(tree: &'a TagTree, path: &TagPath) -> Option<&'a Tag> {
    let (first, rest) = path.0.split_first()?;
    let mut tag = tree.get(first)?;
    for n in rest {
        tag = tag.children.get(n)?;
    }
    Some(tag)
}

/// The tag at `path`, to change it.
pub fn find_mut<'a>(tree: &'a mut TagTree, path: &TagPath) -> Option<&'a mut Tag> {
    let (first, rest) = path.0.split_first()?;
    let mut tag = tree.get_mut(first)?;
    for n in rest {
        tag = tag.children.get_mut(n)?;
    }
    Some(tag)
}

/// The children at `parent` (the top level when None).
pub fn level<'a>(tree: &'a TagTree, parent: Option<&TagPath>) -> Option<&'a TagTree> {
    match parent {
        None => Some(tree),
        Some(p) => find(tree, p).map(|t| &t.children),
    }
}

fn level_mut<'a>(tree: &'a mut TagTree, parent: Option<&TagPath>) -> Option<&'a mut TagTree> {
    match parent {
        None => Some(tree),
        Some(p) => find_mut(tree, p).map(|t| &mut t.children),
    }
}

/// Every tag, depth first, with its path (a parent before its children).
pub fn all(tree: &TagTree) -> Vec<(TagPath, &Tag)> {
    fn walk<'a>(level: &'a TagTree, at: &TagPath, out: &mut Vec<(TagPath, &'a Tag)>) {
        for (n, tag) in level {
            let p = at.child(*n);
            out.push((p.clone(), tag));
            walk(&tag.children, &p, out);
        }
    }
    let mut out = Vec::new();
    walk(tree, &TagPath(Vec::new()), &mut out);
    out
}

/// Where the tag with this GUID is.
pub fn path_of(tree: &TagTree, guid: Uuid) -> Option<TagPath> {
    all(tree).into_iter().find(|(_, t)| t.meta.identity.guid == guid).map(|(p, _)| p)
}

/// The tags named `name` (any case): among `parent`'s children when given, else anywhere.
pub fn named(tree: &TagTree, parent: Option<&TagPath>, name: &str) -> Vec<TagPath> {
    match parent {
        Some(p) => find(tree, p)
            .map(|t| t.children.iter().filter(|(_, c)| c.name().eq_ignore_ascii_case(name)).map(|(n, _)| p.child(*n)).collect())
            .unwrap_or_default(),
        None => all(tree).into_iter().filter(|(_, t)| t.name().eq_ignore_ascii_case(name)).map(|(p, _)| p).collect(),
    }
}

/// The one tag `name` names, or why not (E-TAG-001 none, E-TAG-006 several).
pub fn named_one(tree: &TagTree, parent: Option<&TagPath>, name: &str) -> Result<TagPath, TagError> {
    let found = named(tree, parent, name);
    match found.len() {
        0 => Err(TagError::NoSuchTag(format!("\"{name}\""))),
        1 => Ok(found.into_iter().next().unwrap_or_else(|| TagPath(Vec::new()))),
        count => Err(TagError::Ambiguous {
            name: name.to_string(),
            count,
            paths: found.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "),
        }),
    }
}

/// The tag and every tag under it, by GUID (the tag first).
pub fn subtree(tag: &Tag) -> Vec<Uuid> {
    let mut out = vec![tag.meta.identity.guid];
    for child in tag.children.values() {
        out.extend(subtree(child));
    }
    out
}

// *************************************************************************************************
// Ranges
// Thru walks the children of one parent; only existing tags are visited (the one range rule).
// *************************************************************************************************

/// The tags from `from` to `to` (to the last child when None) under one parent, in typed order
/// (`5 Thru 2` walks down). Ends under different parents are E-TAG-005.
pub fn siblings(tree: &TagTree, from: &TagPath, to: Option<&TagPath>) -> Result<Vec<TagPath>, TagError> {
    let parent = from.parent();
    if let Some(to) = to
        && to.parent() != parent
    {
        return Err(TagError::AcrossParents { from: from.to_string(), to: to.to_string() });
    }
    let Some(level) = level(tree, parent.as_ref()) else {
        return Err(TagError::NoSuchTag(from.to_string()));
    };
    let (a, b) = (from.last(), to.map(|t| t.last()).unwrap_or(u32::MAX));
    let at = |n: u32| match &parent {
        Some(p) => p.child(n),
        None => TagPath(vec![n]),
    };
    let out: Vec<TagPath> = if a <= b {
        level.range(a..=b).map(|(n, _)| at(*n)).collect()
    } else {
        level.range(b..=a).rev().map(|(n, _)| at(*n)).collect()
    };
    Ok(out)
}

// *************************************************************************************************
// Change
// Insert, remove and move keep GUIDs; a moved tag takes the next free number at its new place.
// *************************************************************************************************

/// The next free number at a level (one above the highest).
pub fn next_free(level: &TagTree) -> u32 {
    level.keys().next_back().map(|n| n + 1).unwrap_or(1)
}

/// Put `tag` at `path`: its parent must exist, the number be free and the name free among its
/// siblings.
pub fn insert(tree: &mut TagTree, path: &TagPath, tag: Tag) -> Result<(), TagError> {
    let parent = path.parent();
    let parent_text = parent.as_ref().map(|p| format!("tag {p}")).unwrap_or_else(|| "the top".into());
    let level = level_mut(tree, parent.as_ref()).ok_or_else(|| TagError::NoSuchTag(parent_text.clone()))?;
    if level.contains_key(&path.last()) {
        return Err(TagError::SlotTaken(path.to_string()));
    }
    if !tag.name().is_empty() && level.values().any(|t| t.name().eq_ignore_ascii_case(tag.name())) {
        return Err(TagError::NameTaken { name: tag.name().to_string(), parent: parent_text });
    }
    level.insert(path.last(), tag);
    Ok(())
}

/// Take the tag at `path` (with its children) out of the tree.
pub fn remove(tree: &mut TagTree, path: &TagPath) -> Option<Tag> {
    let level = level_mut(tree, path.parent().as_ref())?;
    level.remove(&path.last())
}

/// Move the tag at `from` (with its subtree) under `to` (the top when None), at the next free
/// number there; returns its new path. Under itself is E-TAG-002; a sibling with its name is
/// E-TAG-003.
pub fn move_under(tree: &mut TagTree, from: &TagPath, to: Option<&TagPath>) -> Result<TagPath, TagError> {
    if find(tree, from).is_none() {
        return Err(TagError::NoSuchTag(from.to_string()));
    }
    if let Some(to) = to {
        if from.contains(to) {
            return Err(TagError::IntoItself(from.to_string()));
        }
        if find(tree, to).is_none() {
            return Err(TagError::NoSuchTag(to.to_string()));
        }
    }
    if from.parent().as_ref() == to {
        return Ok(from.clone());
    }
    let name = find(tree, from).map(|t| t.name().to_string()).unwrap_or_default();
    let target = level(tree, to).map(|l| (next_free(l), l.values().any(|t| !name.is_empty() && t.name().eq_ignore_ascii_case(&name))));
    let Some((n, clash)) = target else {
        return Err(TagError::NoSuchTag(to.map(|t| t.to_string()).unwrap_or_default()));
    };
    if clash {
        let parent = to.map(|p| format!("tag {p}")).unwrap_or_else(|| "the top".into());
        return Err(TagError::NameTaken { name, parent });
    }
    // The paths below `from` shift when it leaves a level above `to`'s: take it out first,
    // then find `to` again by its GUID.
    let to_guid = to.and_then(|t| find(tree, t)).map(|t| t.meta.identity.guid);
    let Some(tag) = remove(tree, from) else {
        return Err(TagError::NoSuchTag(from.to_string()));
    };
    let new_parent = match to_guid {
        Some(g) => path_of(tree, g),
        None => None,
    };
    let dest = match &new_parent {
        Some(p) => p.child(n),
        None => TagPath(vec![n]),
    };
    if let Some(level) = level_mut(tree, new_parent.as_ref()) {
        level.insert(n, tag);
    }
    Ok(dest)
}

/// Give the tag at `from` the number `n` under its own parent (`Move Tag 1.4 At 2`); the number
/// must be free there. Returns its new path.
pub fn renumber(tree: &mut TagTree, from: &TagPath, n: u32) -> Result<TagPath, TagError> {
    if n == 0 {
        return Err(TagError::BadPath("0".into()));
    }
    let parent = from.parent();
    let dest = match &parent {
        Some(p) => p.child(n),
        None => TagPath(vec![n]),
    };
    if dest == *from {
        return Ok(dest);
    }
    if find(tree, &dest).is_some() {
        return Err(TagError::SlotTaken(dest.to_string()));
    }
    let tag = remove(tree, from).ok_or_else(|| TagError::NoSuchTag(from.to_string()))?;
    if let Some(level) = level_mut(tree, parent.as_ref()) {
        level.insert(n, tag);
    }
    Ok(dest)
}

/// Move every child of the tag at `path` up to its parent (each at the next free number there):
/// "Keep children" when the tag is deleted. Returns the children's new paths. A child whose name
/// is taken at the new level keeps it with " 2" (" 3", …) added.
pub fn lift_children(tree: &mut TagTree, path: &TagPath) -> Vec<TagPath> {
    let parent = path.parent();
    let kids: Vec<Tag> = match find_mut(tree, path) {
        Some(t) => std::mem::take(&mut t.children).into_values().collect(),
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    if let Some(level) = level_mut(tree, parent.as_ref()) {
        for mut kid in kids {
            let base = kid.name().to_string();
            let mut k = 2;
            while !base.is_empty() && level.values().any(|t| t.name().eq_ignore_ascii_case(kid.name())) {
                kid.meta.identity.name = Some(format!("{base} {k}"));
                k += 1;
            }
            let n = next_free(level);
            level.insert(n, kid);
            out.push(match &parent {
                Some(p) => p.child(n),
                None => TagPath(vec![n]),
            });
        }
    }
    out
}

// *************************************************************************************************
// Members
// What carries a tag: the identity walk's links of kind Tag, by the object holding them.
// *************************************************************************************************

/// The objects carrying any of `tags` (a link of kind `Tag` in their NodeMeta), each once, in
/// the walk's order. `kind` is the object's kind as the walk names it (`Fixture`, `Sequence`, …).
pub fn members(walk: &Walker, tags: &[Uuid]) -> Vec<Member> {
    let kinds: BTreeMap<Uuid, &'static str> = walk.objects.iter().map(|o| (o.guid, o.kind)).collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for link in &walk.links {
        if link.kind != "Tag" || !tags.contains(&link.target) {
            continue;
        }
        let Some(from) = link.from else { continue };
        if seen.insert(from) {
            out.push(Member { guid: from, kind: kinds.get(&from).copied().unwrap_or("?"), via: link.target });
        }
    }
    out
}

// *************************************************************************************************
// Identity
// A tag is an object of kind Tag; its children are a pool under it, its own tags are links.
// *************************************************************************************************

impl Identified for Tag {
    fn meta(&self) -> &lumacore_base::NodeMeta {
        &self.meta
    }
    fn kind(&self) -> &'static str {
        "Tag"
    }
}

impl Walk for Tag {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, action: _, children } = self;
        meta.walk_links(w);
        w.pool("Children", children);
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
