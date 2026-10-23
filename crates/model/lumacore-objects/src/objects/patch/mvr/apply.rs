//  █████╗ ██████╗ ██████╗ ██╗     ██╗   ██╗
// ██╔══██╗██╔══██╗██╔══██╗██║     ╚██╗ ██╔╝
// ███████║██████╔╝██████╔╝██║      ╚████╔╝
// ██╔══██║██╔═══╝ ██╔═══╝ ██║       ╚██╔╝
// ██║  ██║██║     ██║     ███████╗   ██║
// ╚═╝  ╚═╝╚═╝     ╚═╝     ╚══════╝   ╚═╝
// FILE: apply

//! `Import Mvr {rows} /Apply` (decisions §34 "Import MVR hardened" 3–4): the ticked rows of the
//! review written into the draft — names, addresses, positions, ids and types as their cells say,
//! new fixtures made (their GDTF from the zip through the one door), lost ones deleted when the
//! preset says so —, the review dropped. A new fixture whose MVR group is gone asks first, one ask
//! per group (`/Gone Recreate | Parent | None`, `/Group "name"` for one), the answers kept on
//! the review until every group has one; then the Apply writes once. `/Take` / `/Offer` tick one
//! cell of a row.
//!
//! Owns:          apply, take_cell, the placement ask, the writes into the draft.
//! Doesn't own:   the review's rule (review.rs), the file (read again only for an embedded GDTF).
//! Entry points:  apply, take_cell.
//! Decisions:     §34 "Import MVR hardened" 3–4.

use std::collections::BTreeSet;

use lumacore_fixtures::fixture_mvr::Matrix;
use lumacore_fixtures::{Fixture, FixtureBreak};
use lumacore_syntax::ast::{Id, Item, Range, SetOp, Target};
use lumacore_tree::node::NodeMeta;
use lumacore_tree::MvrTake;
use uuid::Uuid;

use super::import::ImportMvr;
use super::word;
use crate::errors::render_code;
use crate::mvr_review::{ImportReview, ReviewRow, RowStatus};
use crate::{AppState, DispatchResult};
// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC


/// `Import Mvr [rows] /Apply [/Gone …] [/Group "…"]`.
pub fn apply(state: &mut AppState, cmd: &ImportMvr) -> DispatchResult {
    let Some(review) = state.patch_draft.as_ref().and_then(|d| d.review.clone()) else {
        return crate::objects::out(render_code("No MVR review to apply (Import Mvr \"file\" opens one).", "E-CMD-009"));
    };
    let rows = match rows_of(&review, cmd.target.as_ref()) {
        Ok(r) => r,
        Err(e) => return crate::objects::out(e),
    };
    if rows.is_empty() {
        return crate::objects::out(render_code("Nothing ticked: tick rows in the review (Import Mvr 101 + 102 /Apply names them).", "E-CMD-009"));
    }
    if let Some(c) = rows.iter().find(|r| r.status == RowStatus::Clash) {
        return crate::objects::out(render_code(format!("Row {} clashes ({}): untick it or change the address first.", if c.no == 0 { c.id.clone() } else { c.no.to_string() }, c.clash.as_deref().unwrap_or("")), "E-CMD-010"));
    }
    // The placement answers (decision 4): /Gone on the line, the preset, or an ask per group.
    let mut review = review;
    let answer = cmd.gone.as_ref().map(|v| word(v)).map(|w| w.to_ascii_lowercase());
    if let Some(a) = &answer
        && !["recreate", "parent", "none"].contains(&a.as_str())
    {
        return crate::objects::out(render_code(format!("/Gone takes Recreate, Parent or None, not {a}."), "E-CMD-008"));
    }
    let missing: Vec<(Uuid, String, usize)> = {
        let mut seen: Vec<(Uuid, String, usize)> = Vec::new();
        for r in rows.iter().filter(|r| r.status == RowStatus::New && r.group_missing) {
            if let Some(g) = r.group {
                match seen.iter_mut().find(|(u, _, _)| *u == g) {
                    Some(s) => s.2 += 1,
                    None => seen.push((g, r.group_name.clone(), 1)),
                }
            }
        }
        seen
    };
    let preset = match review.settings.missing.to_ascii_lowercase().as_str() {
        "recreate" => Some("recreate".to_string()),
        "parent" => Some("parent".to_string()),
        "none" => Some("none".to_string()),
        _ => None,
    };
    let only = cmd.group.as_ref().map(|v| word(v));
    for (g, name, _) in &missing {
        if review.answers.contains_key(g) {
            continue;
        }
        if let Some(a) = &answer
            && only.as_ref().is_none_or(|o| o.eq_ignore_ascii_case(name))
        {
            review.answers.insert(*g, a.clone());
        } else if let Some(p) = &preset {
            review.answers.insert(*g, p.clone());
        }
    }
    if let Some((g, name, count)) = missing.iter().find(|(g, _, _)| !review.answers.contains_key(g)) {
        // Ask for this group; the answers so far ride on the review.
        if let Some(d) = state.patch_draft.as_mut() {
            d.review = Some(review.clone());
        }
        let left = missing.iter().filter(|(x, _, _)| !review.answers.contains_key(x)).count();
        let layer = rows.iter().find(|r| r.group == Some(*g)).map(|r| r.layer_name.clone()).unwrap_or_default();
        let base = format!("Import Mvr {}/Apply", if cmd.target.is_some() { format!("{} ", rows_text(&rows)) } else { String::new() });
        let ask = crate::dialogs::Ask::new(
            "Import MVR · a group is gone",
            format!("The file puts {count} new fixture{} in \"{name}\"{}, a group this show no longer has. Where do they go?", if *count == 1 { "" } else { "s" }, if layer.is_empty() { String::new() } else { format!(" (layer {layer})") }),
        )
        .detail(format!("Suggested: recreate \"{name}\" under its parent, its uuid kept so the next import finds it"), None)
        .toggle("only", "Only this group (the others ask on)", false, &format!("/Group \"{}\"", name.replace(['"', ';'], "")))
        .button("Recreate the group", lumacore_wire::ButtonKind::Primary, format!("{base} /Gone Recreate {{only}}"))
        .button("Into the parent", lumacore_wire::ButtonKind::Plain, format!("{base} /Gone Parent {{only}}"))
        .button("No group", lumacore_wire::ButtonKind::Plain, format!("{base} /Gone None {{only}}"))
        .cancel("Cancel")
        .note(format!("{left} missing group{} to answer; nothing is written until the last answer. Cancel keeps the review.", if left == 1 { "" } else { "s" }))
        .flags("/Gone Recreate | Parent | None [/Group \"name\"]");
        return DispatchResult::Ask(Box::new(ask));
    }
    crate::objects::out(write(state, review, &rows).unwrap_or_else(|e| e))
}

/// `Import Mvr 102 /Take "Address"` / `/Offer "Address"`: one cell ticked or unticked.
pub fn take_cell(state: &mut AppState, cmd: &ImportMvr) -> DispatchResult {
    let (field, take) = match (&cmd.take, &cmd.offer) {
        (Some(v), _) => (word(v), true),
        (_, Some(v)) => (word(v), false),
        _ => unreachable!("called for /Take or /Offer"),
    };
    let field = match field.to_ascii_lowercase().as_str() {
        "name" | "names" => "name",
        "address" | "addresses" => "address",
        "position" | "pos" => "pos",
        "rotation" | "rot" => "rot",
        "type" | "revision" | "revisions" | "fixturetype" => "type", // tidy: not-keywords (the review's cell names)
        "unit" | "unitnumber" => "unit",
        "customid" | "custom_id" | "cid" => "custom_id",
        other => return crate::objects::out(render_code(format!("No cell \"{other}\": Name, Address, Position, Rotation, Type, Unit or CustomId."), "E-CMD-008")),
    };
    let Some(review) = state.patch_draft.as_mut().and_then(|d| d.review.as_mut()) else {
        return crate::objects::out(render_code("No MVR review open.", "E-CMD-009"));
    };
    let numbers: Vec<u32> = match rows_of(review, cmd.target.as_ref()) {
        Ok(rows) if cmd.target.is_some() && !rows.is_empty() => rows.iter().map(|r| r.no).collect(),
        Ok(_) => return crate::objects::out(render_code("Which row? Import Mvr 102 /Take \"Address\".", "E-CMD-006")),
        Err(e) => return crate::objects::out(e),
    };
    let mut done = Vec::new();
    for no in numbers {
        let Some(row) = review.rows.iter_mut().find(|r| r.no == no) else {
            return crate::objects::out(render_code(format!("No review row {no}."), "E-CMD-007"));
        };
        let Some(cell) = row.cells.iter_mut().find(|c| c.field == field) else {
            return crate::objects::out(render_code(format!("Row {no} has no {field} cell to take."), "E-CMD-009"));
        };
        if take && cell.clash.is_some() {
            return crate::objects::out(render_code(format!("Row {no}'s {field} clashes: {}.", cell.clash.as_deref().unwrap_or("")), "E-CMD-010"));
        }
        cell.taken = take && cell.differs;
        cell.mode = if take { MvrTake::Take } else { MvrTake::Offer };
        if matches!(row.status, RowStatus::Changed | RowStatus::Same) {
            row.taken = row.cells.iter().any(|c| c.taken);
        }
        done.push(no);
    }
    state.show_generation += 1;
    crate::objects::out(crate::text::render_success(&format!("{} {} of row{} {}.", if take { "Took" } else { "Offered" }, field, if done.len() == 1 { "" } else { "s" }, done.iter().map(u32::to_string).collect::<Vec<_>>().join(" + "))))
}

/// The rows the line names (numbers), or every ticked row. The one range rule (R1-34): a range
/// visits the rows that exist; a single number must.
fn rows_of(review: &ImportReview, target: Option<&Target>) -> Result<Vec<ReviewRow>, String> {
    let picks = numbers_of(target)?;
    if picks.is_empty() {
        return Ok(review.rows.iter().filter(|r| r.taken).cloned().collect());
    }
    let mut numbers: Vec<u32> = Vec::new();
    for (op, pick) in picks {
        let these: Vec<u32> = match pick {
            Pick::One(no) => {
                review.rows.iter().find(|r| r.no == no).ok_or_else(|| render_code(format!("No review row {no} (List Mvr)."), "E-CMD-007"))?;
                vec![no]
            }
            Pick::Range(a, b) => review.rows.iter().map(|r| r.no).filter(|n| *n != 0 && (a.min(b)..=a.max(b)).contains(n)).collect(),
        };
        match op {
            SetOp::Minus => numbers.retain(|n| !these.contains(n)),
            _ => {
                for n in these {
                    if !numbers.contains(&n) {
                        numbers.push(n);
                    }
                }
            }
        }
    }
    Ok(numbers.iter().filter_map(|no| review.rows.iter().find(|r| r.no == *no).cloned()).collect())
}

/// One item of a rows target: a number, or a range of them.
enum Pick {
    One(u32),
    Range(u32, u32),
}

fn rows_text(rows: &[ReviewRow]) -> String {
    rows.iter().map(|r| r.no.to_string()).collect::<Vec<_>>().join(" + ")
}

/// The picks of a target (`101 + 102`, `201 Thru 224`), with how each joins; none when there is
/// no target.
fn numbers_of(target: Option<&Target>) -> Result<Vec<(SetOp, Pick)>, String> {
    let Some(target) = target else { return Ok(Vec::new()) };
    let bad = || render_code("Import Mvr takes review rows by number: Import Mvr 101 + 102 /Apply.", "E-CMD-008");
    let whole = |id: &Option<Id>| match id {
        Some(Id::Number(n)) => n.node.parse::<u32>().map_err(|_| bad()),
        _ => Err(bad()),
    };
    let mut out: Vec<(SetOp, Pick)> = Vec::new();
    for (_, item) in &target.items {
        // The bare object (`Import Mvr /Apply`) names no rows.
        let Some(ids) = (match item {
            Item::Path(segments) => segments.last().and_then(|s| s.ids.as_ref()),
            Item::Ids(ids) => Some(ids),
            _ => return Err(bad()),
        }) else {
            continue;
        };
        for (op, range) in &ids.ranges {
            let pick = match range {
                Range::One(id) => Pick::One(whole(&Some(id.clone()))?),
                Range::Thru { from, to, .. } => Pick::Range(whole(from)?, whole(to)?),
            };
            out.push((op.as_ref().map_or(SetOp::Plus, |o| o.node), pick));
        }
    }
    Ok(out)
}

/// The writes, in one pass; the review dropped after.
fn write(state: &mut AppState, review: ImportReview, rows: &[ReviewRow]) -> Result<String, String> {
    let path = {
        let folder = state.library_folder(review.drive.as_deref(), "mvr", false)?;
        let path = lumacore_luma::drive::library_file(&folder, &review.file).map_err(|e| render_code(e, "E-CMD-008"))?;
        if path.extension().is_some() { path } else { path.with_extension("mvr") }
    };
    let mut made = 0;
    let mut changed = 0;
    let mut deleted = 0;
    // Why a row was skipped, once per reason with a count (a type that won't read is one line, not
    // one per fixture): reason → (the first name, how many).
    let mut skipped: std::collections::BTreeMap<String, (String, usize)> = std::collections::BTreeMap::new();
    let mut skip = |name: String, why: String| {
        let e = skipped.entry(why).or_insert((name, 0));
        e.1 += 1;
    };
    let ignore_rotation = review.settings.rotation.eq_ignore_ascii_case("ignore");
    let delete_lost = review.settings.lost.eq_ignore_ascii_case("delete");
    let mut seen_types: std::collections::BTreeMap<String, (u32, usize)> = std::collections::BTreeMap::new();
    for row in rows {
        match row.status {
            RowStatus::Lost => {
                if delete_lost {
                    state.root.show_data.devices.dmx.patch.remove_fid(row.no);
                    deleted += 1;
                }
            }
            RowStatus::New => {
                match make(state, &review, row, &path, &mut seen_types, ignore_rotation) {
                    Ok(()) => made += 1,
                    Err(why) => skip(row.name.clone(), why),
                }
            }
            RowStatus::Changed | RowStatus::Same | RowStatus::Outside => {
                match change(state, &review, row, &path, &mut seen_types, ignore_rotation) {
                    Ok(true) => changed += 1,
                    Ok(false) => {}
                    Err(why) => skip(if row.no == 0 { row.id.clone() } else { row.no.to_string() }, why),
                }
            }
            RowStatus::Clash => {}
        }
    }
    let show = &mut state.root.show_data;
    show.rebuild_id_pools();
    show.rebuild_occupancy();
    if let Some(d) = state.patch_draft.as_mut() {
        d.review = None;
    }
    state.patch_generation += 1;
    state.show_generation += 1;
    let mut parts = Vec::new();
    if made > 0 {
        parts.push(format!("{made} new"));
    }
    if changed > 0 {
        parts.push(format!("{changed} changed"));
    }
    if deleted > 0 {
        parts.push(format!("{deleted} deleted"));
    }
    Ok(crate::text::render_success(&format!(
        "Applied \"{}\": {} in the draft{}. The review is closed; Unclaim Root.ShowData.Devices /Keep keeps the draft.",
        review.file,
        if parts.is_empty() { "nothing".to_string() } else { parts.join(", ") },
        if skipped.is_empty() {
            String::new()
        } else {
            let lines: Vec<String> = skipped.iter().map(|(why, (name, n))| if *n == 1 { format!("{name} ({why})") } else { format!("{name} and {} more ({why})", n - 1) }).collect();
            format!(" — skipped {}", lines.join("; "))
        }
    )))
}

/// The type and revision a row's GDTF stands for in the draft: known by hash, else the embedded
/// file through the one door, else an error naming what is missing.
fn type_of(state: &mut AppState, row: &ReviewRow, path: &std::path::Path, seen: &mut std::collections::BTreeMap<String, (u32, usize)>) -> Result<(u32, usize), String> {
    if let Some(t) = seen.get(&row.gdtf_spec) {
        return Ok(*t);
    }
    let found = if row.gdtf_embedded {
        let open = lumacore_luma::drive::open(path).map(std::io::BufReader::new).map_err(|e| format!("the file can't be read: {e}"))?;
        let bytes = lumacore_mvr::gdtf_entry(open, &row.gdtf_spec).map_err(|e| e.to_string())?.ok_or_else(|| format!("no {} in the zip", row.gdtf_spec))?;
        crate::commands::import::insert_gdtf_bytes(state, &bytes, &row.gdtf_spec)?.at()
    } else {
        // Not in the zip: a type of the draft named like the file's stem, else nothing.
        let stem = row.gdtf_spec.trim_end_matches(".gdtf").to_lowercase();
        let patch = &state.root.show_data.devices.dmx.patch;
        patch
            .fixture_types
            .iter()
            .find(|(_, t)| t.meta.name().is_some_and(|n| n.to_lowercase() == stem) || state.gdtf_sources.values().any(|s| s.file_name.to_lowercase() == format!("{stem}.gdtf")))
            .map(|(no, t)| (*no, t.revisions.len().saturating_sub(1)))
            .ok_or_else(|| format!("no GDTF {} in the zip or the show", row.gdtf_spec))?
    };
    seen.insert(row.gdtf_spec.clone(), found);
    Ok(found)
}

/// The mode of a type's revision by the file's mode name (the first when it names none known).
fn mode_of(state: &AppState, no: u32, revision: usize, name: &str) -> Result<(Uuid, Uuid, Uuid, u16), String> {
    let t = state.root.show_data.devices.dmx.patch.fixture_types.get(&no).ok_or("no such type")?;
    let rev = t.revisions.get(revision).ok_or("no such revision")?;
    let m = rev.modes.iter().find(|m| m.meta.name().is_some_and(|n| n.eq_ignore_ascii_case(name))).or_else(|| rev.modes.first()).ok_or("the type has no modes")?;
    Ok((t.meta.identity.guid, rev.meta.identity.guid, m.meta.identity.guid, m.dmx_footprint))
}

/// The address cell's universe and channel, the universe made when the draft lacks it.
fn address_of(state: &mut AppState, text: &str) -> Option<(Uuid, u16)> {
    let (u, c) = text.split_once('.')?;
    let (u, c): (u32, u16) = (u.parse().ok()?, c.parse().ok()?);
    if u == 0 || !(1..=512).contains(&c) {
        return None;
    }
    let guid = match state.root.show_data.universe_guid(u) {
        Some(g) => g,
        None => {
            let patch = &mut state.root.show_data.devices.dmx.patch;
            patch.universes.insert(u, lumacore_fixtures::Universe::new(None));
            patch.universe_id_pool.reserve(u);
            state.root.show_data.universe_guid(u)?
        }
    };
    Some((guid, c))
}

/// The file's matrix with the rig offset, from the review's cells (the position in metres, the
/// rotation in degrees as the cells show them).
fn matrix_of(row: &ReviewRow, live: Option<&Matrix>, ignore_rotation: bool) -> Option<Matrix> {
    let pos = row.cells.iter().find(|c| c.field == "pos")?;
    let nums = |s: &str| s.split_whitespace().filter_map(|x| x.parse::<f64>().ok()).collect::<Vec<_>>();
    let p = nums(&pos.file);
    if p.len() != 3 {
        return None;
    }
    let mut m = live.cloned().unwrap_or_default();
    m.o = [p[0] * 1000.0, p[1] * 1000.0, p[2] * 1000.0];
    if !ignore_rotation
        && let Some(rot) = row.cells.iter().find(|c| c.field == "rot")
    {
        let r = nums(&rot.file);
        if r.len() == 3 {
            for (axis, deg) in r.into_iter().enumerate() {
                m.set_rotation_deg(axis, deg);
            }
        }
    }
    Some(m)
}

/// The draft group a new fixture goes into: its file group when the draft has it, else as
/// answered (Recreate makes the chain up to the layer, uuids kept; Parent the nearest ancestor
/// the draft has; None straight under the patch).
fn place_of(state: &mut AppState, review: &ImportReview, row: &ReviewRow) -> Option<Uuid> {
    let md = &state.root.show_data.devices.metadata.mvr;
    let has = |u: Uuid| md.layers.values().any(|l| l.meta.identity.guid == u) || md.groups.values().any(|g| g.meta.identity.guid == u);
    let Some(group) = row.group else {
        // Straight under its layer: the layer made if the draft lacks it (never asked: a layer is
        // the file's own frame).
        return Some(ensure_layer(state, row.layer, &row.layer_name));
    };
    if has(group) {
        return Some(group);
    }
    match review.answers.get(&group).map(String::as_str) {
        Some("recreate") => {
            let layer = ensure_layer(state, row.layer, &row.layer_name);
            let md = &mut state.root.show_data.devices.metadata.mvr;
            let n = md.groups.keys().max().copied().unwrap_or(0) + 1;
            let mut meta = NodeMeta::new(Some(row.group_name.clone()).filter(|s| !s.is_empty()));
            meta.identity.guid = group;
            md.groups.insert(n, lumacore_fixtures::devices_meta::MvrGroup { meta, parent: layer, matrix: Default::default(), order: 0 });
            Some(group)
        }
        Some("parent") => Some(ensure_layer(state, row.layer, &row.layer_name)),
        _ => None,
    }
}

/// The draft's layer with that uuid, made when it lacks it.
fn ensure_layer(state: &mut AppState, uuid: Uuid, name: &str) -> Uuid {
    let md = &mut state.root.show_data.devices.metadata.mvr;
    if md.layers.values().any(|l| l.meta.identity.guid == uuid) {
        return uuid;
    }
    if uuid.is_nil() {
        return md.layers.values().next().map(|l| l.meta.identity.guid).unwrap_or(uuid);
    }
    let n = md.layers.keys().max().copied().unwrap_or(0) + 1;
    let mut meta = NodeMeta::new(Some(name.to_string()).filter(|s| !s.is_empty()));
    meta.identity.guid = uuid;
    md.layers.insert(n, lumacore_fixtures::devices_meta::MvrLayer { meta, matrix: Default::default(), order: 0 });
    uuid
}

/// A new fixture from a New row.
fn make(state: &mut AppState, review: &ImportReview, row: &ReviewRow, path: &std::path::Path, seen: &mut std::collections::BTreeMap<String, (u32, usize)>, ignore_rotation: bool) -> Result<(), String> {
    if row.no == 0 && row.cid.is_none() {
        return Err("no fixture number".to_string());
    }
    if row.no != 0 && state.root.show_data.devices.dmx.patch.has_fid(row.no) {
        return Err(format!("Fixture {} is taken", row.no));
    }
    let id_type = lumacore_fixtures::IdType::from_name(&row.id_type).unwrap_or_default();
    if let (0, Some(cid)) = (row.no, row.cid)
        && state.root.show_data.devices.dmx.patch.fixture_by_id(id_type, cid).is_some()
    {
        return Err(format!("{} is taken", row.id));
    }
    let (no, rev) = type_of(state, row, path, seen)?;
    let (ft, rev, mode, footprint) = mode_of(state, no, rev, &row.gdtf_mode)?;
    let addr = row.cells.iter().find(|c| c.field == "address").map(|c| c.file.clone()).unwrap_or_default();
    let (universe, channel) = address_of(state, &addr).unwrap_or((Uuid::nil(), 1));
    // Nested in the file (decisions §40 3): under its parent fixture when the draft has it (its
    // row was written first — the file is in tree order), else where its group says.
    let parent = row
        .parent_fixture
        .filter(|p| state.root.show_data.devices.dmx.patch.fixture_by_guid(*p).is_some())
        .or_else(|| place_of(state, review, row));
    let mut f = Fixture::new(Some(row.name.clone()).filter(|s| !s.is_empty()), ft, rev, mode, universe, channel, footprint);
    f.meta.identity.guid = row.uuid;
    if universe.is_nil() {
        f.mvr.breaks.clear();
    }
    f.mvr.parent = parent;
    if let Some(m) = matrix_of(row, None, ignore_rotation) {
        f.mvr.matrix = m;
    }
    f.mvr.fixture_id = if row.no == 0 { String::new() } else { row.no.to_string() };
    f.mvr.fid = (row.no != 0).then_some(row.no);
    f.mvr.order = row.order;
    if let Some(u) = row.cells.iter().find(|c| c.field == "unit").and_then(|c| c.file.parse::<u32>().ok()) {
        f.mvr.unit_number = u;
    }
    // Its second id as the review showed it (the file's type, else the preset's; decisions §40 9).
    if let Some(c) = row.cid {
        f.mvr.cid = Some(c);
        f.mvr.id_type = id_type;
    }
    state.root.show_data.devices.dmx.patch.insert_fixture(f);
    Ok(())
}

/// The taken cells of a matched row into its fixture; whether anything changed.
fn change(state: &mut AppState, _review: &ImportReview, row: &ReviewRow, path: &std::path::Path, seen: &mut std::collections::BTreeMap<String, (u32, usize)>, ignore_rotation: bool) -> Result<bool, String> {
    let taken: BTreeSet<&str> = row.cells.iter().filter(|c| c.taken && c.differs).map(|c| c.field.as_str()).collect();
    if taken.is_empty() {
        return Ok(false);
    }
    let cell = |f: &str| row.cells.iter().find(|c| c.field == f).map(|c| c.file.clone()).unwrap_or_default();
    let mut type_change = None;
    if taken.contains("type") {
        let (no, rev) = type_of(state, row, path, seen)?;
        type_change = Some(mode_of(state, no, rev, &row.gdtf_mode)?);
    }
    let address = if taken.contains("address") { address_of(state, &cell("address")) } else { None };
    let slot = row.matched.and_then(|g| state.root.show_data.devices.dmx.patch.slot_of(g)).or_else(|| state.root.show_data.devices.dmx.patch.fixture_by_fid(row.no).map(|(s, _)| s)).ok_or_else(|| format!("Fixture {} is gone", row.no))?;
    let live_matrix = state.root.show_data.devices.dmx.patch.fixtures.get(&slot).map(|f| f.mvr.matrix.clone());
    let matrix = if taken.contains("pos") || taken.contains("rot") { matrix_of(row, live_matrix.as_ref(), ignore_rotation || !taken.contains("rot")) } else { None };
    let f = state.root.show_data.devices.dmx.patch.fixtures.get_mut(&slot).ok_or_else(|| format!("Fixture {} is gone", row.no))?;
    if taken.contains("name") {
        f.meta.identity.name = Some(cell("name")).filter(|s| !s.is_empty());
    }
    if let Some((universe, channel)) = address {
        let len = f.primary_break().map(|b| b.channel_count).unwrap_or(1);
        match f.mvr.breaks.first_mut() {
            Some(b) => {
                b.universe_id = universe;
                b.start_address = channel;
            }
            None => f.mvr.breaks.push(FixtureBreak::new(universe, channel, len)),
        }
    }
    if let Some(m) = matrix {
        f.mvr.matrix = m;
    }
    if taken.contains("unit")
        && let Ok(u) = cell("unit").parse::<u32>()
    {
        f.mvr.unit_number = u;
    }
    if taken.contains("custom_id")
        && let Ok(c) = cell("custom_id").parse::<u32>()
    {
        f.mvr.cid = (c != 0).then_some(c);
    }
    if let Some((ft, rev, mode, footprint)) = type_change {
        f.mvr.fixture_type = ft;
        f.mvr.revision = rev;
        f.mvr.mode = mode;
        if let Some(b) = f.mvr.breaks.first_mut() {
            b.channel_count = footprint;
        }
    }
    Ok(true)
}
