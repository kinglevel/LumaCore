// ███╗   ██╗ ██████╗ ██████╗ ███████╗███████╗
// ████╗  ██║██╔═══██╗██╔══██╗██╔════╝██╔════╝
// ██╔██╗ ██║██║   ██║██║  ██║█████╗  ███████╗
// ██║╚██╗██║██║   ██║██║  ██║██╔══╝  ╚════██║
// ██║ ╚████║╚██████╔╝██████╔╝███████╗███████║
// ╚═╝  ╚═══╝ ╚═════╝ ╚═════╝ ╚══════╝╚══════╝
// FILE: nodes

//! The tree's nodes as one kind (decisions §40 "40d hardened"): a layer, an MVR group or a
//! fixture that holds fixtures (a folder — no number, a second id — or any fixture, §40 3). What
//! `At` names as a destination, what sits under a node, how a line names it.
//!
//! Owns:          node_target, node_word, node_label, parent_of, is_under, fixtures_under,
//!                folders_under, nodes_under.
//! Doesn't own:   the words (`assign`, `nest`, the fixture's Delete), the data.
//! Entry points:  node_target, node_word, node_label, fixtures_under, folders_under, is_under.
//! Decisions:     §40 "40d hardened" 3–4.

use std::collections::BTreeMap;

use lumacore_syntax::ast::Target;
use lumacore_tree::DevicesData;
use uuid::Uuid;

use crate::AppState;
use crate::errors::render_code;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The one node `At` names: `MvrLayer n`, `MvrGroup n`, `Fixture n` or `Fixture Media n`. A
/// bare kind (`At MvrGroup`) is `None`; anything else is the usage error.
pub fn node_target(state: &AppState, to: &Target) -> Result<Option<Uuid>, String> {
    let usage = || render_code("At MvrGroup <n>, At MvrLayer <n> or At Fixture <id>: a node of the tree (a layer, a group, a fixture that holds fixtures).", "E-PATCH-011");
    let items = crate::resolve::resolve(state, to, &super::MVR_GROUP)?;
    let [(None, r)] = items.as_slice() else { return Err(usage()) };
    let is = |o: &crate::objects::ObjectDef| std::ptr::eq(r.object, o);
    if !is(&super::MVR_GROUP) && !is(&super::MVR_LAYER) && !is(&crate::objects::FIXTURE) {
        return Err(usage());
    }
    if !r.named {
        return Ok(None);
    }
    let [m] = r.members.as_slice() else { return Err(render_code("At names one node.", "E-CMD-008")) };
    Ok(Some(m.guid.ok_or_else(|| render_code(format!("No such {}.", r.object.keyword), "E-CMD-007"))?))
}

/// How a line names the node: `MvrLayer 1`, `MvrGroup 2`, `Fixture 849`, `Fixture Media 1`.
pub fn node_word(devices: &DevicesData, node: Uuid) -> String {
    match super::place_of(&devices.metadata.mvr, node) {
        Some(super::Place::Layer(n)) => format!("MvrLayer {n}"),
        Some(super::Place::Group(n)) => format!("MvrGroup {n}"),
        None => devices.dmx.patch.fixture_by_guid(node).map(|(_, f)| f.handle()).unwrap_or_default(),
    }
}

/// The node with its name, for a reply: `Fixture Media 1 "Stage 1 Mainstage"`.
pub fn node_label(devices: &DevicesData, node: Uuid) -> String {
    match super::place_of(&devices.metadata.mvr, node) {
        Some(_) => super::label(&devices.metadata.mvr, node),
        None => match devices.dmx.patch.fixture_by_guid(node) {
            Some((_, f)) => match &f.meta.identity.name {
                Some(name) => format!("{} \"{name}\"", f.handle()),
                None => f.handle(),
            },
            None => "No MVR node".into(),
        },
    }
}

/// Every node's parent (a group's, a fixture's): the walk up.
fn parents(devices: &DevicesData) -> BTreeMap<Uuid, Uuid> {
    let mut out: BTreeMap<Uuid, Uuid> = devices.metadata.mvr.groups.values().map(|g| (g.meta.identity.guid, g.parent)).collect();
    out.extend(devices.dmx.patch.fixtures.values().filter_map(|f| f.mvr.parent.map(|p| (f.meta.identity.guid, p))));
    out
}

/// The node `x` sits in, when it is in one.
pub fn parent_of(devices: &DevicesData, x: Uuid) -> Option<Uuid> {
    parents(devices).get(&x).copied()
}

/// Whether `x` sits under `node` at any depth (a broken loop is walked once).
pub fn is_under(devices: &DevicesData, x: Uuid, node: Uuid) -> bool {
    under(&parents(devices), x, node)
}

fn under(parents: &BTreeMap<Uuid, Uuid>, x: Uuid, node: Uuid) -> bool {
    let mut at = parents.get(&x).copied();
    let mut hops = 0;
    while let Some(p) = at.filter(|_| hops < 64) {
        if p == node {
            return true;
        }
        at = parents.get(&p).copied();
        hops += 1;
    }
    false
}

/// The numbered fixtures under `node`, at any depth, by number.
pub fn fixtures_under(devices: &DevicesData, node: Uuid) -> Vec<u32> {
    let parents = parents(devices);
    let mut out: Vec<u32> = devices.dmx.patch.fixtures.values().filter(|f| under(&parents, f.meta.identity.guid, node)).filter_map(|f| f.mvr.fid).collect();
    out.sort_unstable();
    out
}

/// The folders (fixtures without a number) under `node`, at any depth.
pub fn folders_under(devices: &DevicesData, node: Uuid) -> usize {
    let parents = parents(devices);
    devices.dmx.patch.fixtures.values().filter(|f| f.mvr.fid.is_none() && under(&parents, f.meta.identity.guid, node)).count()
}

/// Every fixture (numbered or not) under `node`, by GUID, parents before their children.
pub fn nodes_under(devices: &DevicesData, node: Uuid) -> Vec<Uuid> {
    let parents = parents(devices);
    let mut out: Vec<(usize, Uuid)> = devices
        .dmx
        .patch
        .fixtures
        .values()
        .map(|f| f.meta.identity.guid)
        .filter(|g| under(&parents, *g, node))
        .map(|g| {
            let mut depth = 0;
            let mut at = parents.get(&g).copied();
            while let Some(p) = at.filter(|_| depth < 64) {
                depth += 1;
                at = parents.get(&p).copied();
            }
            (depth, g)
        })
        .collect();
    out.sort();
    out.into_iter().map(|(_, g)| g).collect()
}
