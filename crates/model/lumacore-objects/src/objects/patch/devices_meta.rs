// ██████╗ ███████╗██╗   ██╗██╗ ██████╗███████╗███████╗    ███╗   ███╗███████╗████████╗ █████╗
// ██╔══██╗██╔════╝██║   ██║██║██╔════╝██╔════╝██╔════╝    ████╗ ████║██╔════╝╚══██╔══╝██╔══██╗
// ██║  ██║█████╗  ██║   ██║██║██║     █████╗  ███████╗    ██╔████╔██║█████╗     ██║   ███████║
// ██║  ██║██╔══╝  ╚██╗ ██╔╝██║██║     ██╔══╝  ╚════██║    ██║╚██╔╝██║██╔══╝     ██║   ██╔══██║
// ██████╔╝███████╗ ╚████╔╝ ██║╚██████╗███████╗███████║    ██║ ╚═╝ ██║███████╗   ██║   ██║  ██║
// ╚═════╝ ╚══════╝  ╚═══╝  ╚═╝ ╚═════╝╚══════╝╚══════╝    ╚═╝     ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝
// FILE: devices_meta

//! The MVR tree's words (decisions §34 "34g4 part 3c hardened"): `MvrLayer` and `MvrGroup`, the
//! numbered pools of `Devices.Metadata.MVR` the Patch dialog's sidebar draws (a group's place is
//! its parent, a layer or a group), and `MvrClass` / `MvrPosition` (read only: they come by
//! import). A bare one selects the fixtures in it — a layer or a group with everything under it.
//! Every write needs the Devices claim and lands in the draft.
//!
//! Owns:          MVR_LAYER, MVR_GROUP, MVR_CLASS, MVR_POSITION, their doors and tree views,
//!                Place and the tree's lookups (what is under a node, a node's label).
//! Doesn't own:   the data (`lumacore_fixtures::devices_meta`), the door (`commands::door`).
//! Entry points:  MVR_LAYER, MVR_GROUP, MVR_CLASS, MVR_POSITION, place_of, label, fixtures_under.
//! Decisions:     §34 "34g4 patch hardened" 12, "34g4 part 3c hardened".

use std::collections::BTreeMap;

use lumacore_fixtures::{MvrClass, MvrGroup, MvrLayer, MvrMetadata, MvrPosition};
use lumacore_tree::{DevicesData, Root};
use uuid::Uuid;

use crate::commands::typed::typed;
use crate::objects::{Doc, ObjectDef, TreeDef, Verb, json_view};
use crate::vfs::{DynChild, NodeView};
use crate::AppState;

mod assign;
mod delete;
mod nest;
pub mod nodes;
mod select;
mod store;

pub use assign::{AssignFixture, UnassignFixture};
pub use nest::MoveFixture;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A node of the MVR tree a fixture or a group can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Layer(u32),
    Group(u32),
}

/// The four words, for the verbs they share.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Layer,
    Group,
    Class,
    Position,
}

impl Kind {
    pub fn object(self) -> &'static ObjectDef {
        match self {
            Kind::Layer => &MVR_LAYER,
            Kind::Group => &MVR_GROUP,
            Kind::Class => &MVR_CLASS,
            Kind::Position => &MVR_POSITION,
        }
    }

    /// Its pool's numbers, names and GUIDs.
    pub fn rows(self, md: &MvrMetadata) -> Vec<(u32, Option<String>, Uuid)> {
        fn of<T>(m: &BTreeMap<u32, T>, meta: fn(&T) -> &lumacore_base::NodeMeta) -> Vec<(u32, Option<String>, Uuid)> {
            m.iter().map(|(n, t)| (*n, meta(t).identity.name.clone(), meta(t).identity.guid)).collect()
        }
        match self {
            Kind::Layer => of(&md.layers, |t| &t.meta),
            Kind::Group => of(&md.groups, |t| &t.meta),
            Kind::Class => of(&md.classes, |t| &t.meta),
            Kind::Position => of(&md.positions, |t| &t.meta),
        }
    }
}

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The words' flags: `/NoConfirmation` answers Move's nesting question (a global flag).
const FLAGS: &[crate::commands::flags::FlagDef] = &[
    crate::commands::flags::OVERWRITE.on(&[Verb::Copy, Verb::Paste]),
    crate::commands::flags::BY.on(&[Verb::Compact]),
    crate::commands::flags::KEEP_CHILDREN.on(&[Verb::Delete]),
    crate::commands::flags::WITH_CHILDREN.on(&[Verb::Delete]),
];

crate::door_verbs!(layer_door, "MvrLayer", crate::objects::patch::devices_meta::MVR_LAYER);
crate::door_verbs!(group_door, "MvrGroup", crate::objects::patch::devices_meta::MVR_GROUP);

/// A word's row: the shared parts (rights as a fixture's: selecting is Playback, the tree Setup);
/// the rest (`.delete_ask`, `.props`) follows as builders.
macro_rules! mvr_object {
    ($static:ident, $word:literal, $aliases:expr, $node:ident, $children:ident, $one:ident, $door:expr, $syntax:literal, $desc:literal, $examples:expr, [$($verb:expr),* $(,)?] $($more:tt)*) => {
        pub static $static: ObjectDef = ObjectDef::new(crate::objects::ObjectDecl {
            keyword: $word,
            lum: crate::objects::LumShape::ids(1),
            default_verb: Verb::SelectFixtures,
            flags: FLAGS,
            aliases: $aliases,
            doc: Doc {
                display: $word,
                category: crate::commands::help::HelpCategory::Fixture,
                also_applies_to: &[],
                syntax: $syntax,
                description: $desc,
                examples: $examples,
                related: &["fixture", "assign", "move", "tag"],
                alias_depth: crate::commands::registry::AliasDepth::Base,
                hidden: false,
            },
            family: None,
            call_undo: crate::commands::registry::SELECTION,
            call_class: crate::commands::registry::CommandClass::Commit,
            scope: crate::objects::on_master,
            reach: crate::objects::show_reach,
            rights: crate::rights::fixture,
            planned: &[],
        })
        .verbs(&[$($verb),*])
        .tree(TreeDef { node: crate::tree_path::Node::$node, name: None, children: $children, object: $one, datapool: None, door: $door })
        $($more)*;
    };
}

mvr_object!(MVR_LAYER, "MvrLayer", &["mvrlayers"], MvrLayers, layers, layer_view, Some(&LAYER_DOOR),
    "MvrLayer <n> | Store MvrLayer [<n>] [\"name\"] | Label MvrLayer <n> \"name\" | Move MvrLayer <n> At <n> | Delete MvrLayer <n> | List MvrLayer",
    "A layer of the MVR tree (Devices.Metadata.MVR), the top of it: groups and fixtures sit in it. Alone it selects every fixture under it. Changing it needs the Devices claim.",
    &["Store MvrLayer \"Rig\"            -- a new layer", "MvrLayer 1                       -- select every fixture in it", "Delete MvrLayer 2                -- with its groups and fixtures (asked first)"],
    [
        (Verb::SelectFixtures, typed::<select::SelectMvrLayer>),
        (Verb::List, typed::<select::ListMvrLayer>),
        (Verb::Info, typed::<select::InfoMvrLayer>),
        (Verb::Store, typed::<store::StoreMvrLayer>),
        (Verb::Delete, typed::<delete::DeleteMvrLayer>),
        (Verb::Move, typed::<nest::MoveMvrLayer>),
        (Verb::Copy, typed::<layer_door::Copy>),
        (Verb::Cut, typed::<layer_door::Cut>),
        (Verb::Paste, typed::<layer_door::Paste>),
        (Verb::Insert, typed::<layer_door::Insert>),
        (Verb::Compact, typed::<layer_door::Compact>),
    ]
    .delete_ask(|s, a| delete::ask(s, Kind::Layer, a))
    .props(&LAYER_PROPS));

mvr_object!(MVR_GROUP, "MvrGroup", &["mvrgroups"], MvrGroups, groups, group_view, Some(&GROUP_DOOR),
    "MvrGroup <n> | Store MvrGroup [<n>] [\"name\"] | Move MvrGroup <n> At MvrLayer <n> | MvrGroup <n> | <n> [/NoConfirmation] | Assign Fixture <ids> At MvrGroup <n> | Delete MvrGroup <n> | List MvrGroup",
    "A group of the MVR tree (MVR's GroupObject): in a layer or in another group. Alone it selects every fixture in it and under it. Assign Fixture … At MvrGroup puts fixtures in (a fixture has one place); Move … At a layer or a group nests it, asked first. Changing it needs the Devices claim.",
    &[
        "Store MvrGroup \"Truss 1\"                 -- a new group in the first layer",
        "Move MvrGroup 3 At MvrGroup 2             -- nest it (asked first)",
        "Assign Fixture 101 Thru 104 At MvrGroup 4 -- put the fixtures in",
        "MvrGroup 3                                -- select its fixtures",
    ],
    [
        (Verb::SelectFixtures, typed::<select::SelectMvrGroup>),
        (Verb::List, typed::<select::ListMvrGroup>),
        (Verb::Info, typed::<select::InfoMvrGroup>),
        (Verb::Store, typed::<store::StoreMvrGroup>),
        (Verb::Delete, typed::<delete::DeleteMvrGroup>),
        (Verb::Move, typed::<nest::MoveMvrGroup>),
        (Verb::Copy, typed::<group_door::Copy>),
        (Verb::Cut, typed::<group_door::Cut>),
        (Verb::Paste, typed::<group_door::Paste>),
        (Verb::Insert, typed::<group_door::Insert>),
        (Verb::Compact, typed::<group_door::Compact>),
    ]
    .delete_ask(|s, a| delete::ask(s, Kind::Group, a))
    .props(&GROUP_PROPS));

mvr_object!(MVR_CLASS, "MvrClass", &["mvrclasses"], MvrClasses, classes, class_view, None,
    "MvrClass <n> | List MvrClass | Info MvrClass <n>",
    "An MVR class (AUXData), read only: classes come with an MVR import. Alone it selects the fixtures of that class.",
    &["MvrClass 2                       -- select the fixtures of class 2"],
    [
        (Verb::SelectFixtures, typed::<select::SelectMvrClass>),
        (Verb::List, typed::<select::ListMvrClass>),
        (Verb::Info, typed::<select::InfoMvrClass>),
    ]);

mvr_object!(MVR_POSITION, "MvrPosition", &["mvrpositions"], MvrPositions, positions, position_view, None,
    "MvrPosition <n> | List MvrPosition | Info MvrPosition <n>",
    "An MVR position (AUXData), read only: positions come with an MVR import. Alone it selects the fixtures at that position.",
    &["MvrPosition 1                    -- select the fixtures at position 1"],
    [
        (Verb::SelectFixtures, typed::<select::SelectMvrPosition>),
        (Verb::List, typed::<select::ListMvrPosition>),
        (Verb::Info, typed::<select::InfoMvrPosition>),
    ]);

/// Name, Note, Paint, … from the meta; written into the draft's JSON.
static LAYER_PROPS: crate::props::Props = crate::props::Props { meta: crate::props::MetaLevel::Node, apply: Some(apply_layer), ..crate::props::Props::BASE };
static GROUP_PROPS: crate::props::Props = crate::props::Props { meta: crate::props::MetaLevel::Node, apply: Some(apply_group), ..crate::props::Props::BASE };

/// The pools on the object door (decisions §35): Copy / Cut / Paste / Insert / Compact keep the
/// rules of every pool. A copied node keeps its parent link and comes without its contents.
static LAYER_DOOR: crate::pools::door::BTreePool<MvrLayer> = crate::pools::door::BTreePool {
    object: "MvrLayer",
    in_datapool: false,
    max: crate::pools::MAX_NO,
    map: |s, _| Some(&mut s.root.show_data.devices.metadata.mvr.layers),
    map_ref: |s, _| Some(&s.root.show_data.devices.metadata.mvr.layers),
    name: |l| l.meta.identity.name.clone(),
    identify: |l, guid| l.meta.identity.guid = guid,
    rules: crate::pools::door::PoolRules { can_delete: |_, _| Ok(()), deleted: |_, _| {}, changed: |_| {}, merge: None, orders: &[] },
};

static GROUP_DOOR: crate::pools::door::BTreePool<MvrGroup> = crate::pools::door::BTreePool {
    object: "MvrGroup",
    in_datapool: false,
    max: crate::pools::MAX_NO,
    map: |s, _| Some(&mut s.root.show_data.devices.metadata.mvr.groups),
    map_ref: |s, _| Some(&s.root.show_data.devices.metadata.mvr.groups),
    name: |g| g.meta.identity.name.clone(),
    identify: |g, guid| g.meta.identity.guid = guid,
    rules: crate::pools::door::PoolRules { can_delete: |_, _| Ok(()), deleted: |_, _| {}, changed: |_| {}, merge: None, orders: &[] },
};

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

fn child(word: &'static str, n: u32, name: Option<String>, guid: Uuid, inside: usize) -> DynChild {
    DynChild {
        display_name: match &name {
            Some(s) => format!("{word} {n} \"{s}\""),
            None => format!("{word} {n}"),
        },
        id: n,
        node_type: word,
        is_leaf: true,
        child_count: inside,
        name,
        guid: Some(guid),
    }
}

fn layers(root: &Root) -> Vec<DynChild> {
    let d = &root.show_data.devices;
    Kind::Layer.rows(&d.metadata.mvr).into_iter().map(|(n, name, g)| child("MvrLayer", n, name, g, children_of(d, g))).collect()
}

fn groups(root: &Root) -> Vec<DynChild> {
    let d = &root.show_data.devices;
    Kind::Group.rows(&d.metadata.mvr).into_iter().map(|(n, name, g)| child("MvrGroup", n, name, g, children_of(d, g))).collect()
}

fn classes(root: &Root) -> Vec<DynChild> {
    Kind::Class.rows(&root.show_data.devices.metadata.mvr).into_iter().map(|(n, name, g)| child("MvrClass", n, name, g, 0)).collect()
}

fn positions(root: &Root) -> Vec<DynChild> {
    Kind::Position.rows(&root.show_data.devices.metadata.mvr).into_iter().map(|(n, name, g)| child("MvrPosition", n, name, g, 0)).collect()
}

fn layer_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data.devices.metadata.mvr.layers.get(&id).map(|t| json_view("MvrLayer", id, t))
}

fn group_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data.devices.metadata.mvr.groups.get(&id).map(|t| json_view("MvrGroup", id, t))
}

fn class_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data.devices.metadata.mvr.classes.get(&id).map(|t: &MvrClass| json_view("MvrClass", id, t))
}

fn position_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data.devices.metadata.mvr.positions.get(&id).map(|t: &MvrPosition| json_view("MvrPosition", id, t))
}

/// `MvrGroup 3 "Truss 2"` / `MvrLayer 1` — a node by GUID, for replies and questions.
pub fn label(md: &MvrMetadata, guid: Uuid) -> String {
    let named = |word: &str, n: u32, name: &Option<String>| match name {
        Some(s) => format!("{word} {n} \"{s}\""),
        None => format!("{word} {n}"),
    };
    match place_of(md, guid) {
        Some(Place::Layer(n)) => named("MvrLayer", n, &md.layers[&n].meta.identity.name),
        Some(Place::Group(n)) => named("MvrGroup", n, &md.groups[&n].meta.identity.name),
        None => "No MVR group".into(),
    }
}

/// "2 groups, 88 fixtures".
pub(crate) fn counts_text(groups: usize, fixtures: usize) -> String {
    let s = |n: usize| if n == 1 { "" } else { "s" };
    format!("{groups} group{}, {fixtures} fixture{}", s(groups), s(fixtures))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// Which layer or group a GUID is.
pub fn place_of(md: &MvrMetadata, guid: Uuid) -> Option<Place> {
    if let Some((n, _)) = md.layers.iter().find(|(_, l)| l.meta.identity.guid == guid) {
        return Some(Place::Layer(*n));
    }
    md.groups.iter().find(|(_, g)| g.meta.identity.guid == guid).map(|(n, _)| Place::Group(*n))
}

/// Every group under `node`, at any depth, nearest first (a broken loop is walked once).
pub fn groups_under(md: &MvrMetadata, node: Uuid) -> Vec<Uuid> {
    let mut out: Vec<Uuid> = Vec::new();
    let mut at = 0;
    let mut next: Vec<Uuid> = vec![node];
    while at < next.len() {
        let parent = next[at];
        at += 1;
        for g in md.groups.values().filter(|g| g.parent == parent) {
            let id = g.meta.identity.guid;
            if id != node && !out.contains(&id) {
                out.push(id);
                next.push(id);
            }
        }
    }
    out
}

/// The fixtures in `node` and every group under it.
pub fn fixtures_under(devices: &DevicesData, node: Uuid) -> Vec<u32> {
    let mut places = groups_under(&devices.metadata.mvr, node);
    places.push(node);
    devices.dmx.patch.fixtures.values().filter(|f| f.mvr.parent.is_some_and(|p| places.contains(&p))).filter_map(|f| f.mvr.fid).collect()
}

/// What sits right in `node`: its groups and its fixtures.
fn children_of(devices: &DevicesData, node: Uuid) -> usize {
    devices.metadata.mvr.groups.values().filter(|g| g.parent == node).count()
        + devices.dmx.patch.fixtures.values().filter(|f| f.mvr.parent == Some(node)).count()
}

/// The nodes a target names, by number and GUID (the one resolver).
pub(crate) fn targeted(state: &AppState, target: &lumacore_syntax::ast::Target, kind: Kind) -> Result<Vec<(u32, Uuid)>, String> {
    let found = crate::resolve::resolve_one(state, target, kind.object())?;
    let out: Vec<(u32, Uuid)> = found.members.iter().filter_map(|m| m.guid.map(|g| (m.no, g))).collect();
    if out.is_empty() {
        return Err(crate::errors::render_code(format!("No such {}.", kind.object().keyword), "E-CMD-007"));
    }
    Ok(out)
}

fn apply_layer(root: &mut Root, _: Option<u32>, id: u32, edit: &mut dyn FnMut(&mut serde_json::Value) -> Result<(), String>) -> Result<(), String> {
    let l = root.show_data.devices.metadata.mvr.layers.get_mut(&id).ok_or_else(|| format!("no MvrLayer {id}"))?;
    crate::props::edit_json(l, edit)
}

fn apply_group(root: &mut Root, _: Option<u32>, id: u32, edit: &mut dyn FnMut(&mut serde_json::Value) -> Result<(), String>) -> Result<(), String> {
    let g = root.show_data.devices.metadata.mvr.groups.get_mut(&id).ok_or_else(|| format!("no MvrGroup {id}"))?;
    crate::props::edit_json(g, edit)
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
