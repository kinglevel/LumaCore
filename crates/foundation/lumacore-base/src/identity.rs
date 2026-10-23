//! Object identity (decisions §21 "6d hardening"): every object in the show
//! has a GUID and every stored link is a GUID.
//!
//! - [`Guid`] is a GUID value (a derived one: [`Guid::named`]); its `Default` is a fresh random
//!   GUID, so a derived `Default` never hands out a shared one.
//! - [`Identified`] is what every object in the show implements, only through its
//!   [`NodeMeta`](crate::NodeMeta) (decisions §12 "Tags part 1b hardened" 7): a show type without
//!   one doesn't compile.
//! - [`Walk`] is implemented by every type in the show with an exhaustive
//!   destructure, so a new field doesn't compile until it is classified as an
//!   object pool, a link or plain data. The [`Walker`] collects every object
//!   and link: the GUID index, the reference check and claims read it.
//!
//! Identifier names from a standard (GDTF attribute names, UI key ids) are
//! their own identity: their GUID is derived from the name ([`Guid::named`]),
//! so a link by that name is a link by GUID.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use uuid::Uuid;

/// An object's GUID.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Guid(pub Uuid);

/// Namespace for GUIDs derived from identifier names.
const NAMED: Uuid = Uuid::from_u128(0x6c75_6d61_636f_7265_0000_0000_0000_0d01);

impl Guid {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// The GUID of something whose identity is a standard name (`kind` keeps
    /// equal names of different kinds apart).
    pub fn named(kind: &str, name: &str) -> Self {
        Self(Uuid::new_v5(
            &NAMED,
            format!("{kind}:{}", name.to_lowercase()).as_bytes(),
        ))
    }

    pub fn uuid(self) -> Uuid {
        self.0
    }
}

impl Default for Guid {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for Guid {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl From<Guid> for Uuid {
    fn from(g: Guid) -> Self {
        g.0
    }
}

impl PartialEq<Uuid> for Guid {
    fn eq(&self, other: &Uuid) -> bool {
        self.0 == *other
    }
}

impl fmt::Debug for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An object in the show: its NodeMeta (so its GUID and name) and a kind (`"Sequence"`, `"Cue"`, …).
pub trait Identified {
    fn meta(&self) -> &crate::NodeMeta;
    fn kind(&self) -> &'static str;
    fn guid(&self) -> Uuid {
        self.meta().identity.guid
    }
}

/// Implemented by every type in the show (see the module docs).
pub trait Walk {
    fn walk(&self, w: &mut Walker);
}

/// `Identified` for show types (their `meta: NodeMeta`, decisions §12 "Tags part 1b hardened");
/// one copy for every crate that defines show types (decisions §34 25.14).
#[macro_export]
macro_rules! meta_identified {
    ($($t:ty => $kind:literal),* $(,)?) => {$(
        impl $crate::Identified for $t {
            fn meta(&self) -> &$crate::NodeMeta {
                &self.meta
            }
            fn kind(&self) -> &'static str {
                $kind
            }
        }
    )*};
}

/// `Walk` for show types that are only their `NodeMeta` (no other fields).
#[macro_export]
macro_rules! meta_only_walk {
    ($($t:ident),* $(,)?) => {$(
        impl $crate::Walk for $t {
            fn walk(&self, w: &mut $crate::Walker) {
                let Self { meta } = self;
                meta.walk_links(w);
            }
        }
    )*};
}

/// A shared show part (decisions §31 "Arc parts") is what it holds.
impl<T: Identified> Identified for std::sync::Arc<T> {
    fn meta(&self) -> &crate::NodeMeta {
        (**self).meta()
    }
    fn kind(&self) -> &'static str {
        (**self).kind()
    }
}

/// A shared show part (decisions §31 "Arc parts") walks as what it holds.
impl<T: Walk> Walk for std::sync::Arc<T> {
    fn walk(&self, w: &mut Walker) {
        (**self).walk(w);
    }
}

/// One object the walker found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectEntry {
    pub guid: Uuid,
    pub kind: &'static str,
    /// Tree path, e.g. `Playback.Sequences.3.Cues.1500`.
    pub path: String,
    /// GUID of the object it lives in (None at the top).
    pub parent: Option<Uuid>,
    /// Its display name, when it has one.
    pub name: Option<String>,
}

/// One stored link the walker found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkEntry {
    /// Tree path of the field holding the link.
    pub path: String,
    /// The kind of object it points at (`A|B` = either).
    pub kind: &'static str,
    pub target: Uuid,
    /// The holder contains the target (sub-fixture parent, embedded preset):
    /// these links must never loop.
    pub contains: bool,
    /// GUID of the object holding the link.
    pub from: Option<Uuid>,
}

/// Collects objects and links (see [`Walk`]).
#[derive(Debug, Default)]
pub struct Walker {
    path: Vec<String>,
    owners: Vec<Uuid>,
    pub objects: Vec<ObjectEntry>,
    pub links: Vec<LinkEntry>,
}

impl Walker {
    pub fn new() -> Self {
        Self::default()
    }

    fn here(&self, last: &str) -> String {
        let mut p = self.path.join(".");
        if !last.is_empty() {
            if !p.is_empty() {
                p.push('.');
            }
            p.push_str(last);
        }
        p
    }

    fn enter_object<T: Identified + Walk>(&mut self, segment: String, obj: &T) {
        let guid = obj.guid();
        let path = self.here(&segment);
        self.objects.push(ObjectEntry {
            guid,
            kind: obj.kind(),
            path,
            parent: self.owners.last().copied(),
            name: obj.meta().identity.name.clone(),
        });
        self.path.push(segment);
        self.owners.push(guid);
        obj.walk(self);
        self.owners.pop();
        self.path.pop();
    }

    /// A numbered (or named) pool of objects.
    pub fn pool<K: fmt::Display, T: Identified + Walk>(&mut self, name: &str, pool: &BTreeMap<K, T>) {
        for (k, obj) in pool {
            self.enter_object(format!("{name}.{k}"), obj);
        }
    }

    /// One object of a pool whose step the caller spells (a fixture by its number, not its slot —
    /// decisions §40 5).
    pub fn pool_member<T: Identified + Walk>(&mut self, name: &str, step: impl fmt::Display, obj: &T) {
        self.enter_object(format!("{name}.{step}"), obj);
    }

    /// An ordered list of objects (macro lines, modes): numbered from 1.
    pub fn list<T: Identified + Walk>(&mut self, name: &str, list: &[T]) {
        for (i, obj) in list.iter().enumerate() {
            self.enter_object(format!("{name}.{}", i + 1), obj);
        }
    }

    /// A single object held in a field.
    pub fn object<T: Identified + Walk>(&mut self, name: &str, obj: &T) {
        self.enter_object(name.to_string(), obj);
    }

    /// A plain struct that groups fields (not an object of its own).
    pub fn child<T: Walk + ?Sized>(&mut self, name: &str, x: &T) {
        self.path.push(name.to_string());
        x.walk(self);
        self.path.pop();
    }

    /// Plain data: no objects and no links inside.
    pub fn data<T: ?Sized>(&mut self, _x: &T) {}

    /// A link to an object of `kind`.
    pub fn link(&mut self, field: &str, kind: &'static str, target: Uuid) {
        self.push_link(field, kind, target, false);
    }

    pub fn link_opt(&mut self, field: &str, kind: &'static str, target: Option<Uuid>) {
        if let Some(t) = target {
            self.link(field, kind, t);
        }
    }

    pub fn links<'a>(&mut self, field: &str, kind: &'static str, targets: impl IntoIterator<Item = &'a Uuid>) {
        for t in targets {
            self.link(field, kind, *t);
        }
    }

    /// A link to an object the holder contains (must never loop).
    pub fn contains(&mut self, field: &str, kind: &'static str, target: Option<Uuid>) {
        if let Some(t) = target {
            self.push_link(field, kind, t, true);
        }
    }

    fn push_link(&mut self, field: &str, kind: &'static str, target: Uuid, contains: bool) {
        let path = self.here(field);
        let from = self.owners.last().copied();
        self.links.push(LinkEntry {
            path,
            kind,
            target,
            contains,
            from,
        });
    }
}

impl<T: Walk> Walk for Vec<T> {
    fn walk(&self, w: &mut Walker) {
        for x in self {
            x.walk(w);
        }
    }
}

impl<T: Walk> Walk for Option<T> {
    fn walk(&self, w: &mut Walker) {
        if let Some(x) = self {
            x.walk(w);
        }
    }
}

impl<K, T: Walk> Walk for BTreeMap<K, T> {
    fn walk(&self, w: &mut Walker) {
        for x in self.values() {
            x.walk(w);
        }
    }
}

/// What the walk of a whole show says about identity.
#[derive(Debug, Default, Clone)]
pub struct IdentityReport {
    pub objects: usize,
    pub links: usize,
    /// GUIDs held by more than one object: (guid, paths).
    pub duplicates: Vec<(Uuid, Vec<String>)>,
    /// Objects whose GUID is nil.
    pub nil: Vec<String>,
    /// Links to a GUID no object has.
    pub broken: Vec<LinkEntry>,
    /// Links to an object of another kind than the field expects.
    pub wrong_kind: Vec<(LinkEntry, &'static str)>,
    /// Containment loops (each as the chain of paths).
    pub loops: Vec<Vec<String>>,
}

impl IdentityReport {
    /// Nothing wrong.
    pub fn is_clean(&self) -> bool {
        self.duplicates.is_empty()
            && self.nil.is_empty()
            && self.broken.is_empty()
            && self.wrong_kind.is_empty()
            && self.loops.is_empty()
    }
}

impl Walker {
    /// Check what was walked: duplicates, nil GUIDs, broken or mistyped links
    /// and containment loops.
    pub fn report(&self) -> IdentityReport {
        let mut by_guid: BTreeMap<Uuid, Vec<&ObjectEntry>> = BTreeMap::new();
        let mut nil = Vec::new();
        for o in &self.objects {
            if o.guid.is_nil() {
                nil.push(o.path.clone());
                continue;
            }
            by_guid.entry(o.guid).or_default().push(o);
        }
        let duplicates = by_guid
            .iter()
            .filter(|(_, v)| v.len() > 1)
            .map(|(g, v)| (*g, v.iter().map(|o| o.path.clone()).collect()))
            .collect();
        let mut broken = Vec::new();
        let mut wrong_kind = Vec::new();
        for l in &self.links {
            match by_guid.get(&l.target) {
                None => broken.push(l.clone()),
                Some(v) if !v.iter().any(|o| l.kind.split('|').any(|k| k == o.kind)) => {
                    wrong_kind.push((l.clone(), v[0].kind))
                }
                _ => {}
            }
        }
        // Containment loops: follow `contains` links from each holder.
        let mut next: BTreeMap<Uuid, Vec<Uuid>> = BTreeMap::new();
        for l in self.links.iter().filter(|l| l.contains) {
            if let Some(from) = l.from {
                next.entry(from).or_default().push(l.target);
            }
        }
        let path_of = |g: &Uuid| {
            by_guid
                .get(g)
                .map_or_else(|| g.to_string(), |v| v[0].path.clone())
        };
        let mut loops: Vec<Vec<String>> = Vec::new();
        let mut seen_loop: std::collections::BTreeSet<Uuid> = Default::default();
        for start in next.keys() {
            let mut chain = vec![*start];
            let mut at = *start;
            while let Some(&n) = next.get(&at).and_then(|v| v.first()) {
                if n == *start {
                    if chain.iter().all(|g| seen_loop.insert(*g)) {
                        loops.push(chain.iter().map(path_of).collect());
                    }
                    break;
                }
                if chain.contains(&n) || chain.len() > 64 {
                    break;
                }
                chain.push(n);
                at = n;
            }
        }
        IdentityReport {
            objects: self.objects.len(),
            links: self.links.len(),
            duplicates,
            nil,
            broken,
            wrong_kind,
            loops,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Leaf {
        meta: crate::NodeMeta,
        to: Option<Uuid>,
    }
    crate::meta_identified! { Leaf => "Leaf" }
    impl Walk for Leaf {
        fn walk(&self, w: &mut Walker) {
            let Self { meta: _, to } = self;
            w.contains("To", "Leaf", *to);
        }
    }
    fn leaf(guid: Uuid, to: Uuid) -> Leaf {
        Leaf { meta: crate::NodeMeta::with_guid(guid, None), to: Some(to) }
    }

    #[test]
    fn walker_finds_duplicates_broken_links_and_loops() {
        let a = Guid::new();
        let b = Guid::new();
        let mut pool = BTreeMap::new();
        pool.insert(1u32, leaf(a.0, b.0));
        pool.insert(2u32, leaf(b.0, a.0));
        pool.insert(3u32, leaf(a.0, Uuid::new_v4()));
        let mut w = Walker::new();
        w.pool("Leaves", &pool);
        let r = w.report();
        assert_eq!(r.duplicates.len(), 1);
        assert_eq!(r.broken.len(), 1);
        assert_eq!(r.broken[0].path, "Leaves.3.To");
        assert_eq!(r.loops.len(), 1);
        assert!(!r.is_clean());
    }

    #[test]
    fn named_guids_are_stable_and_kind_scoped() {
        assert_eq!(Guid::named("Attribute", "Dimmer"), Guid::named("Attribute", "dimmer"));
        assert_ne!(Guid::named("Attribute", "Dimmer"), Guid::named("UiKey", "Dimmer"));
        assert_ne!(Guid::default(), Guid::default());
    }
}
