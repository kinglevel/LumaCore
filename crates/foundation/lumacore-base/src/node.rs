// ███╗   ██╗ ██████╗ ██████╗ ███████╗
// ████╗  ██║██╔═══██╗██╔══██╗██╔════╝
// ██╔██╗ ██║██║   ██║██║  ██║█████╗
// ██║╚██╗██║██║   ██║██║  ██║██╔══╝
// ██║ ╚████║╚██████╔╝██████╔╝███████╗
// ╚═╝  ╚═══╝ ╚═════╝ ╚═════╝ ╚══════╝
// FILE: node

//! The meta every node carries, in levels (decisions §12 "Tags part 1b hardened"): an
//! [`Identity`] (guid, display name) on every node under Root, and on every ShowData object an
//! [`Annotation`] too (note, paint, icon, locked, hidden, tags) — together a [`NodeMeta`], saved as
//! one flat `meta`. A node's number is its pool's key, never a field here.
//!
//! Owns:          Identity, Annotation, NodeMeta, the Meta trait, IconRef, TreeNode.
//! Doesn't own:   the walk (`identity`), the properties each level declares (lumacore-state).
//! Entry points:  NodeMeta::new, Identity::new, Identity::named, Meta.
//! Decisions:     §12 "Tags part 1b hardened", §21 "6d".

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a tree listing shows of a node: its type word, GUID and display name. Its number is the
/// key it sits under.
pub trait TreeNode {
    fn guid(&self) -> Uuid;
    fn name(&self) -> Option<&str>;
    fn node_type(&self) -> &'static str;
}

/// A shared show part (decisions §31 "Arc parts") is the node it holds.
impl<T: TreeNode> TreeNode for std::sync::Arc<T> {
    fn guid(&self) -> Uuid {
        (**self).guid()
    }
    fn name(&self) -> Option<&str> {
        (**self).name()
    }
    fn node_type(&self) -> &'static str {
        (**self).node_type()
    }
}

/// Any level of meta: its identity, so its GUID and display name.
pub trait Meta {
    fn identity(&self) -> &Identity;
    fn guid(&self) -> Uuid {
        self.identity().guid
    }
    fn name(&self) -> Option<&str> {
        self.identity().name.as_deref()
    }
}

/// An object's icon (decisions §30 "12a2"): a name in the standard icon set
/// that ships with LumaCore (names are never removed), or a picture of the
/// show's own `Images` pool. Icons are never stored in the show.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IconRef {
    /// Name in the standard icon set (Lucide + cat glyphs).
    Builtin(String),
    /// GUID of a picture in `ShowData.Images`.
    Image(Uuid),
}

/// Who a node is — on every node under Root. Its `Default` is a fresh GUID, so a derived
/// `Default` never hands out a shared one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    /// Stable identifier: links, replication, undo and merge name a node by it.
    pub guid: Uuid,
    /// Display name, optional and not unique. A name that is an identity (a login, a GDTF name,
    /// a key id) is a field of its own type, never this.
    #[serde(default)]
    pub name: Option<String>,
}

/// What an operator adds to a ShowData object. Every field is always written (a declared
/// property writes into the object's JSON by field), the tags only when there are some.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    /// Free-text note.
    #[serde(default)]
    pub note: Option<String>,
    /// Its colour on screen: a colour or a theme slot link (decisions
    /// object-paint-palette-free-theme-links); None = its tags', then its pool's.
    #[serde(default)]
    pub paint: Option<crate::paint::Paint>,
    /// A standard icon or a picture of the show's Images pool.
    #[serde(default)]
    pub icon: Option<IconRef>,
    /// Prevent accidental edits when true.
    #[serde(default)]
    pub locked: bool,
    /// Hide from operator views when true.
    #[serde(default)]
    pub hidden: bool,
    /// The tags it carries, as GUID links to `ShowData.Tags` (decisions §12 "Tags hardened" 1).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Uuid>,
}

/// The meta of a ShowData object: its identity and its annotation, saved as one flat `meta`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeMeta {
    #[serde(flatten)]
    pub identity: Identity,
    #[serde(flatten)]
    pub annotation: Annotation,
}

// ██╗███╗   ██╗██╗████████╗
// ██║████╗  ██║██║╚══██╔══╝
// ██║██╔██╗ ██║██║   ██║
// ██║██║╚██╗██║██║   ██║
// ██║██║ ╚████║██║   ██║
// ╚═╝╚═╝  ╚═══╝╚═╝   ╚═╝
// PART: INIT

impl Default for Identity {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Identity {
    /// A fresh GUID: a node the station makes at run time (a login, a Shell, a claim).
    pub fn new(name: Option<String>) -> Self {
        Self { guid: Uuid::new_v4(), name }
    }

    /// A derived GUID: a node built from code or keyed by something stable (a window type, an
    /// interface, a station) is the same node on every station and every start.
    pub fn named(kind: &str, key: &str, name: Option<String>) -> Self {
        Self { guid: crate::Guid::named(kind, key).0, name }
    }
}

impl NodeMeta {
    /// A new ShowData object's meta with a fresh GUID.
    pub fn new(name: Option<String>) -> Self {
        Self::with_guid(Uuid::new_v4(), name)
    }

    /// With a known GUID (a derived one, or one read back).
    pub fn with_guid(guid: Uuid, name: Option<String>) -> Self {
        Self { identity: Identity { guid, name }, annotation: Annotation::default() }
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Meta for Identity {
    fn identity(&self) -> &Identity {
        self
    }
}

impl Meta for NodeMeta {
    fn identity(&self) -> &Identity {
        &self.identity
    }
}

impl Identity {
    /// Its display name (as [`Meta::name`], without the trait in scope).
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl NodeMeta {
    /// Its GUID (as [`Meta::guid`], without the trait in scope).
    pub fn guid(&self) -> Uuid {
        self.identity.guid
    }

    /// Its display name (as [`Meta::name`]).
    pub fn name(&self) -> Option<&str> {
        self.identity.name.as_deref()
    }
}

impl NodeMeta {
    /// The links a NodeMeta holds (its show icon, its tags), for [`crate::Walk`].
    pub fn walk_links(&self, w: &mut crate::Walker) {
        if let Some(IconRef::Image(g)) = &self.annotation.icon {
            w.link("Icon", "Image", *g);
        }
        w.links("Tags", "Tag", &self.annotation.tags);
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
mod tests {
    use super::*;

    #[test]
    fn a_new_meta_has_a_fresh_guid_and_its_name() {
        let meta = NodeMeta::new(Some("Test".to_string()));
        assert_eq!(meta.name(), Some("Test"));
        assert_eq!(meta.guid().get_version(), Some(uuid::Version::Random));
        assert_eq!(NodeMeta::new(None).name(), None);
    }

    #[test]
    fn a_default_meta_never_shares_a_guid() {
        assert_ne!(NodeMeta::default().guid(), NodeMeta::default().guid());
    }

    #[test]
    fn a_derived_identity_is_the_same_every_time() {
        assert_eq!(Identity::named("WindowType", "tags", None).guid, Identity::named("WindowType", "tags", None).guid);
        assert_ne!(Identity::named("WindowType", "tags", None).guid, Identity::named("WindowType", "sequence", None).guid);
    }

    /// One flat `meta`: the blocks' fields side by side; no tags, no `tags` key.
    #[test]
    fn the_blocks_save_as_one_flat_meta() {
        let mut meta = NodeMeta::with_guid(Uuid::nil(), Some("Front".into()));
        assert_eq!(
            serde_json::to_value(&meta).unwrap(),
            serde_json::json!({ "guid": Uuid::nil(), "name": "Front", "note": null, "paint": null, "icon": null, "locked": false, "hidden": false })
        );
        meta.annotation.note = Some("hung inverted".into());
        meta.annotation.locked = true;
        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!((json["note"].as_str(), json["locked"].as_bool()), (Some("hung inverted"), Some(true)));
        assert_eq!(serde_json::from_value::<NodeMeta>(json).unwrap(), meta);
    }
}
