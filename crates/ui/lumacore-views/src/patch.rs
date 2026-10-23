// ██████╗  █████╗ ████████╗ ██████╗██╗  ██╗
// ██╔══██╗██╔══██╗╚══██╔══╝██╔════╝██║  ██║
// ██████╔╝███████║   ██║   ██║     ███████║
// ██╔═══╝ ██╔══██║   ██║   ██║     ██╔══██║
// ██║     ██║  ██║   ██║   ╚██████╗██║  ██║
// ╚═╝     ╚═╝  ╚═╝   ╚═╝    ╚═════╝╚═╝  ╚═╝
// FILE: patch
//! The Patch setup dialog's rows (decisions §34 "34g4 patch hardened" 9, 12; the locked mockup
//! `2026-10-18-34g4`): the fixtures with the columns the Fixture declares, the universes, the
//! fixture types and the MVR tree (layers › nested groups) with counts. The holder of the
//! Devices draft sees the draft; everyone else the live devices, with the live columns named.
//! Built for the `patch` feed from the published show and the published draft.
//!
//! Owns:          `view`, `tree_order` (the one tree order, the device sheet reads it too).
//! Doesn't own:   the draft (`lumacore_state::patch_draft`), the menu row (`objects::patch`),
//!                the words the dialog's lines use.
//! Entry points:  view, tree_order.
//! Decisions:     §34 "34g4 patch hardened" 9, 12; §34 20.7.

use std::collections::BTreeMap;

use lumacore_tree::dmx_slots::{Run, SlotMap};
use lumacore_tree::tree::PatchData;
use lumacore_tree::{DevicesData, ShowData};
use serde_json::{Value, json};
use uuid::Uuid;

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The dialog's rows for `user`: the draft's when `draft` is theirs (holder, devices), else the
/// live show's.
pub fn view(show: &ShowData, draft: Option<(&str, &DevicesData, &Marks, Option<&Value>)>, user: &str) -> Value {
    let mine = draft.is_some_and(|(holder, _, _, _)| holder.eq_ignore_ascii_case(user));
    let devices = match draft {
        Some((_, d, _, _)) if mine => d,
        _ => &show.devices,
    };
    let patch = &devices.dmx.patch;
    let mvr = &devices.metadata.mvr;
    let tag_names: BTreeMap<Uuid, String> =
        lumacore_tree::tags::tag::all(&show.tags).into_iter().map(|(_, t)| (t.meta.identity.guid, t.name().to_string())).collect();
    // Each row wears its fixture's colour as this viewer sees it (decisions 0294).
    let paint = lumacore_objects::paint::Painter::new(show, user);
    let mut fixtures = fixture_rows(devices, &tag_names, &paint);
    // The holder's draft: a gold dot on every cell that differs from the live rig (§34 round 3,
    // 10), by fixture GUID.
    if mine {
        let live: BTreeMap<String, Value> =
            fixture_rows(&show.devices, &tag_names, &paint).into_iter().map(|r| (r["guid"].to_string(), r)).collect();
        for r in &mut fixtures {
            let was = live.get(&r["guid"].to_string());
            let changed: Vec<&str> = SHOWN.iter().copied().filter(|k| was.is_none_or(|w| w[*k] != r[*k])).collect();
            r["changed"] = json!(changed);
            // The violet dot: cells someone else's live edit brought in (field → who).
            let guid = r["guid"].as_str().and_then(|g| g.parse::<Uuid>().ok());
            if let Some(cells) = guid.and_then(|g| draft.and_then(|(_, _, m, _)| m.get(&g))) {
                r["live_changed"] = json!(cells);
            }
        }
    }
    // How many numbered fixtures sit in each node, the nodes under it included — a folder
    // fixture is a node too, never counted (decisions §40 "40d hardened").
    let mut parent_of: BTreeMap<Uuid, Uuid> = mvr.groups.values().map(|g| (g.meta.identity.guid, g.parent)).collect();
    parent_of.extend(patch.fixtures.values().filter_map(|f| f.mvr.parent.map(|p| (f.meta.identity.guid, p))));
    let mut counts: BTreeMap<Uuid, usize> = BTreeMap::new();
    for f in patch.fixtures.values().filter(|f| f.mvr.fid.is_some()) {
        let mut at = f.mvr.parent;
        let mut hops = 0;
        while let Some(node) = at.filter(|_| hops < 64) {
            *counts.entry(node).or_default() += 1;
            at = parent_of.get(&node).copied();
            hops += 1;
        }
    }
    let count = |g: Uuid| counts.get(&g).copied().unwrap_or(0);
    let mut tree: Vec<Value> = Vec::new();
    let next_layer = lumacore_objects::pools::door::next_free(&mvr.layers.keys().copied().collect::<Vec<_>>());
    let next_group = lumacore_objects::pools::door::next_free(&mvr.groups.keys().copied().collect::<Vec<_>>());
    for (no, l) in &mvr.layers {
        let lines = node_lines("MvrLayer", *no, next_layer, next_group);
        tree.push(json!({ "kind": "layer", "no": no, "guid": l.meta.identity.guid, "name": l.meta.identity.name, "parent": null, "count": count(l.meta.identity.guid), "lines": lines }));
    }
    for (no, g) in &mvr.groups {
        let lines = node_lines("MvrGroup", *no, next_group, next_group);
        tree.push(json!({ "kind": "group", "no": no, "guid": g.meta.identity.guid, "name": g.meta.identity.name, "parent": g.parent, "count": count(g.meta.identity.guid), "lines": lines }));
    }
    // A fixture that holds fixtures, or has no number (a folder), is a node of the tree: under
    // its parent in its order, named by its handle (`Fixture Media 1`).
    let holds: std::collections::BTreeSet<Uuid> = patch.fixtures.values().filter_map(|f| f.mvr.parent).collect();
    let mut folders: Vec<&lumacore_tree::fixtures::Fixture> = patch.fixtures.values().filter(|f| f.mvr.fid.is_none() || holds.contains(&f.meta.identity.guid)).collect();
    folders.sort_by_key(|f| f.mvr.order);
    for f in folders {
        let me = f.handle();
        tree.push(json!({
            "kind": "folder", "no": 0, "guid": f.meta.identity.guid, "name": f.meta.identity.name, "parent": f.mvr.parent,
            "id": me.trim_start_matches("Fixture ").to_string(), "count": count(f.meta.identity.guid),
            "lines": { "select": me, "at": me, "label": format!("Label {me} {{name}}"), "child": null, "copy": null, "move": format!("Move {me} At {{to}}"), "delete": format!("Delete {me}") },
        }));
    }
    let classes: Vec<Value> = mvr
        .classes
        .iter()
        .map(|(no, c)| json!({ "kind": "class", "no": no, "guid": c.meta.identity.guid, "name": c.meta.identity.name, "parent": null,
            // How a line names it as a destination (`Assign Fixture … At MvrClass 2`).
            "at": format!("{} {no}", lumacore_objects::objects::MVR_CLASS.keyword), "count": patch.fixtures.values().filter(|f| f.mvr.class == Some(c.meta.identity.guid)).count() }))
        .collect();
    // The tag tree, each tag counting the fixtures that carry it (the sidebar's Tag grouping).
    let tag_nodes: Vec<Value> = lumacore_tree::tags::tag::all(&show.tags)
        .into_iter()
        .map(|(path, t)| {
            let parent = path.parent().and_then(|p| lumacore_tree::tags::tag::find(&show.tags, &p)).map(|p| p.meta.identity.guid);
            let g = t.meta.identity.guid;
            json!({ "kind": "tag", "guid": g, "name": t.name(), "parent": parent, "count": patch.fixtures.values().filter(|f| f.meta.annotation.tags.contains(&g)).count() })
        })
        .collect();
    let slots = SlotMap::of(patch);
    let universes: Vec<Value> = patch
        .universes
        .iter()
        .map(|(no, u)| {
            json!({
                "no": no, "guid": u.meta.identity.guid, "name": u.meta.identity.name.clone().unwrap_or_default(),
                "merge_mode": u.merge_mode.label(),
                "fixtures": patch.fixtures.values().filter(|f| f.mvr.breaks.iter().any(|b| b.universe_id == u.meta.identity.guid)).count(),
                // Slots in use of 512 (the Map view's usage bar; a shared slot once).
                "used": slots.used.get(&u.meta.identity.guid).copied().unwrap_or(0),
            })
        })
        .collect();
    let fixture_types: Vec<Value> = patch
        .fixture_types
        .iter()
        .map(|(no, t)| {
            json!({
                "no": no, "guid": t.meta.identity.guid, "name": t.meta.identity.name.clone().unwrap_or_default(),
                "manufacturer": t.manufacturer, "revisions": t.revisions.len(),
                "modes": t.revisions.last().map_or(0, |r| r.modes.len()),
                "fixtures": patch.fixtures.values().filter(|f| f.mvr.fixture_type == t.meta.identity.guid).count(),
            })
        })
        .collect();
    // The holder's open MVR import review (decisions §34 "Import MVR hardened" 3): the grid's
    // third mode.
    let review = draft.and_then(|(_, _, _, r)| r).filter(|_| mine);
    json!({
        "mode": if review.is_some() { "review" } else if mine { "draft" } else { "live" },
        // Where the dialog is, for its title (§34 round 3: the path, not "in the show").
        "path": lumacore_objects::tree_path::Node::Patch.path().to_string(),
        "holder": draft.map(|(h, _, _, _)| h),
        "review": review,
        "fixtures": fixtures,
        "unsorted": patch.fixtures.values().filter(|f| f.mvr.parent.is_none()).count(),
        "tree": tree,
        "classes": classes,
        "tag_nodes": tag_nodes,
        "universes": universes,
        "fixture_types": fixture_types,
        "runs": runs(patch, &slots),
        "conflicts": conflicts(patch, &slots),
    })
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// Cells a live edit brought into a draft: fixture GUID → row field → who.
pub type Marks = BTreeMap<Uuid, BTreeMap<String, String>>;

/// The fields a fixture's row shows, compared for the draft's dots.
const SHOWN: &[&str] =
    &["no", "name", "note", "fixture_type", "mode", "address", "pos_x", "pos_y", "pos_z", "rot_x", "rot_y", "rot_z", "master_react", "group"];

/// Every fixture's row of `devices`: its cells, its slots (`footprint`) and how its address
/// stands — `address_tone` `alert` where a slot is shared with another fixture or the run passes
/// 512 (both are parked until resolved, §34 round 3, 18), "no universe" when its universe is gone.
/// Every fixture, folders too, in tree order (decisions §40 "40d hardened" 2): a node's fixtures
/// together, each node's children by their order, the number last. The one order the Patch
/// dialog and the device sheet show.
pub fn tree_order(devices: &DevicesData) -> Vec<&lumacore_tree::fixtures::Fixture> {
    let patch = &devices.dmx.patch;
    let mvr = &devices.metadata.mvr;
    let mut parent_of: BTreeMap<Uuid, Uuid> = mvr.groups.values().map(|g| (g.meta.identity.guid, g.parent)).collect();
    parent_of.extend(patch.fixtures.values().filter_map(|f| f.mvr.parent.map(|p| (f.meta.identity.guid, p))));
    // A step of the path: its order, its number, its GUID — an order is only unique under one
    // parent, so siblings of different parents that share one never interleave.
    let step_of: BTreeMap<Uuid, (u32, u32, Uuid)> = mvr
        .groups
        .values()
        .map(|g| (g.meta.identity.guid, (g.order, 0, g.meta.identity.guid)))
        .chain(mvr.layers.values().map(|l| (l.meta.identity.guid, (l.order, 0, l.meta.identity.guid))))
        .chain(patch.fixtures.values().map(|f| (f.meta.identity.guid, (f.mvr.order, f.mvr.fid.unwrap_or(0), f.meta.identity.guid))))
        .collect();
    let step = |g: Uuid| step_of.get(&g).copied().unwrap_or((0, 0, g));
    let tree_key = |g: Uuid| -> Vec<(u32, u32, Uuid)> {
        let mut path = vec![step(g)];
        let mut at = parent_of.get(&g).copied();
        let mut hops = 0;
        while let Some(p) = at.filter(|_| hops < 64) {
            path.push(step(p));
            at = parent_of.get(&p).copied();
            hops += 1;
        }
        path.reverse();
        path
    };
    let mut all: Vec<&lumacore_tree::fixtures::Fixture> = patch.fixtures.values().collect();
    all.sort_by_cached_key(|f| tree_key(f.meta.identity.guid));
    all
}

fn fixture_rows(devices: &DevicesData, tag_names: &BTreeMap<Uuid, String>, paint: &lumacore_objects::paint::Painter) -> Vec<Value> {
    let patch = &devices.dmx.patch;
    let mvr = &devices.metadata.mvr;
    let universe_no: BTreeMap<Uuid, u32> = patch.universes.iter().map(|(n, u)| (u.meta.identity.guid, *n)).collect();
    // The MVR tree's nodes by GUID: their names, for a fixture's group cell.
    let mut node_name: BTreeMap<Uuid, String> = BTreeMap::new();
    for l in mvr.layers.values() {
        node_name.insert(l.meta.identity.guid, l.meta.identity.name.clone().unwrap_or_default());
    }
    for g in mvr.groups.values() {
        node_name.insert(g.meta.identity.guid, g.meta.identity.name.clone().unwrap_or_default());
    }
    // Who sits on which slot (one door with the render plan): an overlap or a run past 512 parks
    // both sides.
    let slots = SlotMap::of(patch);
    // The numbered fixtures only — a folder is a node of the sidebar, never a row (decisions
    // §40 "40d hardened" 2) — in tree order.
    let folder_name = |g: Uuid| patch.fixture_by_guid(g).and_then(|(_, f)| f.meta.identity.name.clone()).unwrap_or_default();
    tree_order(devices)
        .into_iter()
        .filter(|f| f.mvr.fid.is_some())
        .map(|f| {
            // 0 = no number; `id` is how a line names it (`Fixture Media 7003`).
            let no = f.mvr.fid.unwrap_or(0);
            let ft = patch.fixture_types.values().find(|t| t.meta.identity.guid == f.mvr.fixture_type);
            let in_mode = ft.and_then(|t| t.mode_of(f.mvr.revision, f.mvr.mode)).map(|(_, m)| m);
            let mode = in_mode.map(|m| m.meta.identity.name.clone().unwrap_or_default());
            // Its slots (the Address dialog's free runs and overlaps, §34 round 3, 18).
            let footprint = in_mode.map(|m| m.dmx_footprint).or_else(|| f.mvr.breaks.first().map(|b| b.channel_count)).unwrap_or(0);
            let address = f.primary_break().map_or(String::new(), |b| {
                universe_no.get(&b.universe_id).map_or_else(|| "no universe".to_string(), |u| format!("{u}.{:03}", b.start_address))
            });
            let gone = f.primary_break().is_some_and(|b| !universe_no.contains_key(&b.universe_id));
            let parked = slots.is_parked(f.meta.identity.guid);
            let tone = if parked { "alert" } else if gone { "warn" } else { "" };
            let [px, py, pz] = f.mvr.matrix.position_m();
            let [rx, ry, rz] = f.mvr.matrix.rotation_deg();
            let shown = paint.of(&f.meta);
            json!({
                "no": no, "guid": f.meta.identity.guid,
                "name": f.meta.identity.name.clone().unwrap_or_default(),
                "paint": shown.hex,
                "paint_from": shown.from_text(),
                "paint_word": f.meta.annotation.paint.as_ref().map_or_else(|| "none".to_string(), |p| p.word()),
                "note": f.meta.annotation.note,
                "fixture_type": ft.map(|t| t.meta.identity.name.clone().unwrap_or_default()).unwrap_or_default(),
                "mode": mode.unwrap_or_default(),
                "address": address,
                "footprint": footprint,
                "address_tone": tone,
                "parked": parked,
                "pos_x": lumacore_tree::fixtures::fixture_mvr::decimal(px), "pos_y": lumacore_tree::fixtures::fixture_mvr::decimal(py), "pos_z": lumacore_tree::fixtures::fixture_mvr::decimal(pz),
                "rot_x": lumacore_tree::fixtures::fixture_mvr::decimal(rx), "rot_y": lumacore_tree::fixtures::fixture_mvr::decimal(ry), "rot_z": lumacore_tree::fixtures::fixture_mvr::decimal(rz),
                "master_react": format!("{:?}", f.luma.master_react),
                "tags": f.meta.annotation.tags.iter().filter_map(|g| tag_names.get(g)).cloned().collect::<Vec<_>>().join(" · "),
                "tag_guids": f.meta.annotation.tags,
                "group": f.mvr.parent.map(|p| node_name.get(&p).cloned().unwrap_or_else(|| folder_name(p))).unwrap_or_default(),
                "parent": f.mvr.parent,
                "class": f.mvr.class,
                "id": f.handle(),
                "id_type": f.mvr.id_type.name(),
                "cid": f.mvr.cid,
            })
        })
        .collect()
}

/// Every patched run for the DMX map (decisions §34 round 3, 12, 18; `dmx-map.html` v2): a
/// fixture's break or a multipatch copy's, with its universe number, slots, type (the block's
/// colour) and whether it is parked.
fn runs(patch: &PatchData, slots: &SlotMap) -> Vec<Value> {
    let universe_no: BTreeMap<Uuid, u32> = patch.universes.iter().map(|(n, u)| (u.meta.identity.guid, *n)).collect();
    slots
        .runs
        .iter()
        .map(|r| {
            let f = patch.fixture_by_guid(r.guid).map(|(_, f)| f).expect("a run names a fixture of the patch");
            json!({
                "no": r.fixture, "guid": r.guid, "copy": r.copy, "break": r.brk,
                "name": f.meta.identity.name.clone().unwrap_or_default(),
                "universe": universe_no.get(&r.universe), "start": r.start, "count": r.count,
                "fixture_type": patch.fixture_types.iter().find(|(_, t)| t.meta.identity.guid == f.mvr.fixture_type).map(|(n, _)| *n),
                "parked": slots.is_parked(r.guid),
            })
        })
        .collect()
}

/// The conflict list (decisions §34 round 3, 18): each overlap or run past 512 with its one-tap
/// fixes, each one line — the later run to the next free address, Swap the two fixtures,
/// Unpatch the later one (Delete when it is a multipatch copy).
fn conflicts(patch: &PatchData, slots: &SlotMap) -> Vec<Value> {
    let universe_no: BTreeMap<Uuid, u32> = patch.universes.iter().map(|(n, u)| (u.meta.identity.guid, *n)).collect();
    let word = |r: &Run| match r.copy {
        Some(c) => format!("Fixture {} Multipatch {c}", r.fixture),
        None => format!("Fixture {}", r.fixture),
    };
    let range = |r: &Run| {
        let u = universe_no.get(&r.universe).copied().unwrap_or(0);
        format!("{u}.{:03}–{u}.{:03}", r.start, r.end())
    };
    slots
        .conflicts
        .iter()
        .map(|c| {
            let a = &slots.runs[c.run];
            let b = c.other.map(|o| &slots.runs[o]);
            // The run that landed on top moves or goes; past 512 it is the run itself.
            let later = b.unwrap_or(a);
            let mut fixes = vec![json!({ "label": format!("{} to next free", later.fixture), "line": format!("Assign {} At DMX", word(later)), "primary": true })];
            if let Some(b) = b.filter(|b| a.copy.is_none() && b.copy.is_none() && a.fixture != b.fixture) {
                fixes.push(json!({ "label": format!("Swap {} ⇄ {}", a.fixture, b.fixture), "line": format!("Assign Fixture {} At Fixture {} DMX /Swap", a.fixture, b.fixture) }));
            }
            // A copy has no address of its own to take away: it goes (lum.md, Multipatch).
            fixes.push(match later.copy {
                Some(c) => json!({ "label": format!("Delete {} Multipatch {c}", later.fixture), "line": format!("Delete {}", word(later)) }),
                None => json!({ "label": format!("Unpatch {}", later.fixture), "line": format!("Unassign {} At DMX", word(later)) }),
            });
            json!({
                "kind": if b.is_some() { "overlap" } else { "past" },
                "a": { "no": a.fixture, "guid": a.guid, "word": word(a), "range": range(a) },
                "b": b.map(|b| json!({ "no": b.fixture, "guid": b.guid, "word": word(b), "range": range(b) })),
                "fixes": fixes,
            })
        })
        .collect()
}

/// A tree node's ≡ menu lines (decisions §34 "34g4 patch hardened" 12, "34g4 part 3c
/// hardened"): `at` is how a Move names it as the destination; `{name}` / `{to}` the menu fills.
/// A new group lands in the first layer and is nested here at once; Move asks on the station.
fn node_lines(word: &str, no: u32, next_same: u32, next_group: u32) -> Value {
    let me = format!("{word} {no}");
    json!({
        "select": me,
        "at": me,
        "label": format!("Label {me} {{name}}"),
        "child": format!("Store MvrGroup {next_group} {{name}}; Move MvrGroup {next_group} At {me} /NoConfirmation"),
        "copy": format!("Copy {me} At {next_same}"),
        "move": (word == "MvrGroup").then(|| format!("Move {me} At {{to}}")),
        "delete": format!("Delete {me}"),
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
