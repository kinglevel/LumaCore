//! Merge on rejoin (decisions §21 "6d hardening"): a 3-way merge of two
//! diverged copies of a show — the base both had at the split, the
//! session's (master) and mine — matched by GUID, never by slot.
//!
//! The unit is the smallest object with a GUID (a cue part, a macro line, a
//! preset, a group, a fixture):
//! - added on one side → added; both added in the **same slot** → the
//!   master's keeps it, mine goes to the next free slot;
//! - deleted on one side and untouched on the other → deleted; deleted on
//!   one side but **edited on the other → the edit wins**;
//! - changed on both sides → **the master's version**;
//! - a move is a slot change of the same GUID: it applies, nothing doubles;
//! - running playbacks (`ShowData.Playbacks`) are always the master's.
//!
//! Everything kept from one side over the other is listed in the
//! [`MergeReport`]. The merge works on the show's JSON: an object is a JSON
//! object with a `guid` (or `meta.guid`), a pool a map or list of them.

use crate::ShowData;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// What a merge did, path by path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MergeReport {
    /// Objects only the master's side added.
    pub added_master: Vec<String>,
    /// Objects only my side added.
    pub added_mine: Vec<String>,
    /// Objects deleted (by either side) and gone.
    pub deleted: Vec<String>,
    /// Changed on both sides: the master's version kept.
    pub conflicts: Vec<String>,
    /// Deleted on one side, edited on the other: kept.
    pub edit_beats_delete: Vec<String>,
    /// Mine took another slot (`path → new slot`).
    pub moved_slots: Vec<String>,
    /// Moves applied (`path → new slot`).
    pub moves: Vec<String>,
}

impl MergeReport {
    /// Nothing differs.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// One line per item, for the prompt and sysmon.
    pub fn lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut push = |label: &str, items: &[String]| {
            for i in items {
                out.push(format!("{label}: {i}"));
            }
        };
        push("added (session)", &self.added_master);
        push("added (mine)", &self.added_mine);
        push("deleted", &self.deleted);
        push("changed on both sides, session's kept", &self.conflicts);
        push("deleted on one side, edited on the other, kept", &self.edit_beats_delete);
        push("mine moved to a free slot", &self.moved_slots);
        push("moved", &self.moves);
        out
    }
}

/// Why Merge isn't offered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeBlocked {
    /// The two sides hold different shows (`show_id`).
    OtherShow,
    /// No show both sides had at the split.
    NoBase,
    /// Both sides changed the patch since the split (blocked, as consoles do).
    PatchOnBothSides,
}

impl MergeBlocked {
    pub fn reason(&self) -> &'static str {
        match self {
            MergeBlocked::OtherShow => "the stations hold different shows",
            MergeBlocked::NoBase => "no common show to merge from (no split base)",
            MergeBlocked::PatchOnBothSides => "the patch changed on both sides",
        }
    }
}

/// Can these three be merged?
pub fn can_merge(base: Option<&ShowData>, master: &ShowData, mine: &ShowData) -> Result<(), MergeBlocked> {
    if master.show_settings.show_id != mine.show_settings.show_id {
        return Err(MergeBlocked::OtherShow);
    }
    let Some(base) = base else {
        return Err(MergeBlocked::NoBase);
    };
    if base.show_settings.show_id != master.show_settings.show_id {
        return Err(MergeBlocked::NoBase);
    }
    let at_split = base.session.patch_version;
    if master.session.patch_version != at_split && mine.session.patch_version != at_split {
        return Err(MergeBlocked::PatchOnBothSides);
    }
    Ok(())
}

/// Merge `master` and `mine` from their common `base`.
pub fn three_way(base: &ShowData, master: &ShowData, mine: &ShowData) -> Result<(ShowData, MergeReport), String> {
    let to = |s: &ShowData| serde_json::to_value(s).map_err(|e| e.to_string());
    let (b, m, y) = (to(base)?, to(master)?, to(mine)?);
    let mut report = MergeReport::default();
    let mut merged = merge_value(Some(&b), Some(&m), Some(&y), "", &mut report).unwrap_or(Value::Null);
    // Running playbacks: always the session's (the stage keeps going).
    merged["playbacks"] = m["playbacks"].clone();
    // The patch version only rises.
    let pv = master
        .session
        .patch_version
        .max(mine.session.patch_version);
    merged["session"]["patch_version"] = pv.into();
    let mut show: ShowData = serde_json::from_value(merged).map_err(|e| format!("merged show: {e}"))?;
    show.after_load();
    Ok((show, report))
}

/// The GUID of a JSON object (`guid` or `meta.guid`).
fn guid_of(v: &Value) -> Option<&str> {
    v.get("guid")
        .and_then(Value::as_str)
        .or_else(|| v.get("meta")?.get("guid")?.as_str())
}

fn is_object_node(v: &Value) -> bool {
    v.is_object() && guid_of(v).is_some()
}

/// A map or list whose entries are objects with GUIDs.
fn is_pool(v: &Value) -> bool {
    match v {
        Value::Object(map) => !map.is_empty() && map.values().all(is_object_node),
        Value::Array(list) => !list.is_empty() && list.iter().all(is_object_node),
        _ => false,
    }
}

fn any_pool(vs: [Option<&Value>; 3]) -> bool {
    vs.into_iter().flatten().any(is_pool)
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

/// 3-way merge of one value. None = absent.
fn merge_value(
    base: Option<&Value>,
    master: Option<&Value>,
    mine: Option<&Value>,
    path: &str,
    report: &mut MergeReport,
) -> Option<Value> {
    if master == mine {
        return master.cloned();
    }
    // Pools and objects are walked even when one side alone changed, so the
    // report lists everything that differs.
    let (Some(m), Some(y)) = (master, mine) else {
        return if mine == base {
            master.cloned()
        } else if master == base {
            mine.cloned()
        } else {
            // One side removed a field the other changed: keep the change.
            master.or(mine).cloned()
        };
    };
    if any_pool([base, master, mine]) {
        return Some(merge_pool(base, m, y, path, report));
    }
    let objects = m.is_object() && y.is_object();
    if !objects && mine == base {
        return master.cloned();
    }
    if !objects && master == base {
        return mine.cloned();
    }
    match (m, y) {
        (Value::Object(_), Value::Object(_)) if is_object_node(m) || is_object_node(y) => {
            Some(merge_object(base, m, y, path, report))
        }
        (Value::Object(mm), Value::Object(ym)) => {
            // A plain container: merge field by field.
            let bm = base.and_then(Value::as_object);
            let mut out = Map::new();
            for k in mm.keys().chain(ym.keys()).collect::<BTreeSet<_>>() {
                let child = merge_value(
                    bm.and_then(|b| b.get(k)),
                    mm.get(k),
                    ym.get(k),
                    &join(path, k),
                    report,
                );
                if let Some(c) = child {
                    out.insert(k.clone(), c);
                }
            }
            Some(Value::Object(out))
        }
        _ => {
            report.conflicts.push(path.to_string());
            Some(m.clone())
        }
    }
}

/// Fields of an object that hold pools (merged on their own) vs its own data.
fn split_fields(v: &Value, pools: &BTreeSet<String>) -> (Map<String, Value>, Map<String, Value>) {
    let mut own = Map::new();
    let mut nested = Map::new();
    if let Some(map) = v.as_object() {
        for (k, val) in map {
            if pools.contains(k) {
                nested.insert(k.clone(), val.clone());
            } else {
                own.insert(k.clone(), val.clone());
            }
        }
    }
    (own, nested)
}

/// Both sides changed one object: its own data is the master's when both
/// changed it; the pools inside merge entry by entry.
fn merge_object(base: Option<&Value>, m: &Value, y: &Value, path: &str, report: &mut MergeReport) -> Value {
    let mut pools = BTreeSet::new();
    for v in [base, Some(m), Some(y)].into_iter().flatten() {
        if let Some(map) = v.as_object() {
            for (k, val) in map {
                if is_pool(val) {
                    pools.insert(k.clone());
                }
            }
        }
    }
    let (b_own, b_nested) = base.map(|b| split_fields(b, &pools)).unwrap_or_default();
    let (m_own, m_nested) = split_fields(m, &pools);
    let (y_own, y_nested) = split_fields(y, &pools);
    let mut out = if m_own == y_own || y_own == b_own {
        m_own
    } else if m_own == b_own {
        y_own
    } else {
        report.conflicts.push(path.to_string());
        m_own
    };
    for k in &pools {
        let child = merge_value(
            b_nested.get(k),
            m_nested.get(k),
            y_nested.get(k),
            &join(path, k),
            report,
        );
        if let Some(c) = child {
            out.insert(k.clone(), c);
        }
    }
    Value::Object(out)
}

/// Pool entries by GUID: guid → (slot, value).
fn index(v: Option<&Value>) -> BTreeMap<String, (String, &Value)> {
    let mut out = BTreeMap::new();
    match v {
        Some(Value::Object(map)) => {
            for (k, val) in map {
                if let Some(g) = guid_of(val) {
                    out.insert(g.to_string(), (k.clone(), val));
                }
            }
        }
        Some(Value::Array(list)) => {
            for (i, val) in list.iter().enumerate() {
                if let Some(g) = guid_of(val) {
                    out.insert(g.to_string(), (format!("{:08}", i), val));
                }
            }
        }
        _ => {}
    }
    out
}

/// Where a merged entry goes and whether mine decided it.
struct Placed {
    guid: String,
    slot: String,
    value: Value,
    mine_slot: bool,
}

fn merge_pool(base: Option<&Value>, m: &Value, y: &Value, path: &str, report: &mut MergeReport) -> Value {
    let (bi, mi, yi) = (index(base), index(Some(m)), index(Some(y)));
    let is_list = m.is_array() || y.is_array();
    let mut placed: Vec<Placed> = Vec::new();
    // The master's order first, then mine's additions.
    let mut order: Vec<&String> = mi.keys().collect();
    order.sort_by(|a, b| mi[*a].0.cmp(&mi[*b].0));
    let mut mine_only: Vec<&String> = yi.keys().filter(|g| !mi.contains_key(*g)).collect();
    mine_only.sort_by(|a, b| yi[*a].0.cmp(&yi[*b].0));
    order.extend(mine_only);
    for g in order {
        let b = bi.get(g);
        let here = |slot: &str| join(path, slot);
        match (mi.get(g), yi.get(g)) {
            (Some((ms, mv)), Some((ys, yv))) => {
                let value = merge_value(b.map(|x| x.1), Some(mv), Some(yv), &here(ms), report)
                    .unwrap_or_else(|| (*mv).clone());
                let base_slot = b.map(|x| x.0.as_str());
                let (slot, mine_slot) = if ms == ys {
                    (ms.clone(), false)
                } else if base_slot == Some(ms.as_str()) {
                    // Only mine moved it.
                    report.moves.push(format!("{} → {}", here(ms), slot_label(ys, is_list)));
                    (ys.clone(), true)
                } else {
                    if base_slot != Some(ys.as_str()) {
                        // Both moved it: the master's move.
                        report.conflicts.push(here(ms));
                    } else {
                        report.moves.push(format!("{} → {}", here(ys), slot_label(ms, is_list)));
                    }
                    (ms.clone(), false)
                };
                placed.push(Placed {
                    guid: g.clone(),
                    slot,
                    value,
                    mine_slot,
                });
            }
            (Some((ms, mv)), None) => match b {
                // Mine deleted it: gone unless the master edited it.
                Some((_, bv)) if *bv == *mv => report.deleted.push(here(ms)),
                Some(_) => {
                    report.edit_beats_delete.push(here(ms));
                    placed.push(Placed {
                        guid: g.clone(),
                        slot: ms.clone(),
                        value: (*mv).clone(),
                        mine_slot: false,
                    });
                }
                None => {
                    report.added_master.push(here(ms));
                    placed.push(Placed {
                        guid: g.clone(),
                        slot: ms.clone(),
                        value: (*mv).clone(),
                        mine_slot: false,
                    });
                }
            },
            (None, Some((ys, yv))) => match b {
                // The master deleted it: gone unless mine edited it.
                Some((_, bv)) if *bv == *yv => report.deleted.push(here(ys)),
                Some(_) => {
                    report.edit_beats_delete.push(here(ys));
                    placed.push(Placed {
                        guid: g.clone(),
                        slot: ys.clone(),
                        value: (*yv).clone(),
                        mine_slot: true,
                    });
                }
                None => {
                    report.added_mine.push(here(ys));
                    placed.push(Placed {
                        guid: g.clone(),
                        slot: ys.clone(),
                        value: (*yv).clone(),
                        mine_slot: true,
                    });
                }
            },
            (None, None) => {
                if let Some((bs, _)) = b {
                    report.deleted.push(here(bs));
                }
            }
        }
    }
    if is_list {
        return merge_list(placed, &mi, &yi);
    }
    // Slots: the master's entries keep theirs; mine move on when taken.
    let mut taken: BTreeSet<String> = placed
        .iter()
        .filter(|p| !p.mine_slot)
        .map(|p| p.slot.clone())
        .collect();
    let mut out = Map::new();
    for mut p in placed {
        if p.mine_slot {
            if taken.contains(&p.slot) {
                let free = next_free(&p.slot, &taken, &out);
                report
                    .moved_slots
                    .push(format!("{} → {}", join(path, &p.slot), free));
                renumber(&mut p.value, &p.slot, &free);
                p.slot = free;
            }
            taken.insert(p.slot.clone());
        }
        let _ = &p.guid;
        out.insert(p.slot, p.value);
    }
    Value::Object(out)
}

fn slot_label(slot: &str, is_list: bool) -> String {
    if is_list {
        slot.trim_start_matches('0').parse::<usize>().map_or(slot.to_string(), |i| (i + 1).to_string())
    } else {
        slot.to_string()
    }
}

/// An ordered list (macro lines, modes): the master's order; mine's
/// additions follow the entry they followed on my side.
fn merge_list(
    placed: Vec<Placed>,
    mi: &BTreeMap<String, (String, &Value)>,
    yi: &BTreeMap<String, (String, &Value)>,
) -> Value {
    let mut ordered: Vec<Placed> = Vec::new();
    let (from_master, from_mine): (Vec<Placed>, Vec<Placed>) =
        placed.into_iter().partition(|p| mi.contains_key(&p.guid));
    ordered.extend(from_master);
    // My order, to find where each of mine goes.
    let mut mine_order: Vec<&String> = yi.keys().collect();
    mine_order.sort_by(|a, b| yi[*a].0.cmp(&yi[*b].0));
    for p in from_mine {
        let pos = mine_order.iter().position(|g| **g == p.guid).unwrap_or(0);
        let after = mine_order[..pos]
            .iter()
            .rev()
            .find_map(|g| ordered.iter().position(|o| &o.guid == *g));
        let at = after.map_or(0, |i| i + 1);
        ordered.insert(at.min(ordered.len()), p);
    }
    Value::Array(ordered.into_iter().map(|p| p.value).collect())
}

/// The first free slot after `slot` (numbers count up; names get " 2", …).
fn next_free(slot: &str, taken: &BTreeSet<String>, placed: &Map<String, Value>) -> String {
    let used = |s: &str| taken.contains(s) || placed.contains_key(s);
    if let Ok(n) = slot.parse::<u64>() {
        return (n + 1..)
            .map(|k| k.to_string())
            .find(|k| !used(k))
            .expect("u64 slots never run out");
    }
    (2..)
        .map(|i| format!("{slot} {i}"))
        .find(|k| !used(k))
        .expect("names never run out")
}

/// An object that names its own slot (`id`, `no`, `meta.id`) follows the move.
fn renumber(v: &mut Value, old: &str, new: &str) {
    let (Ok(old), Ok(new)) = (old.parse::<u64>(), new.parse::<u64>()) else {
        return;
    };
    for field in ["id", "no"] {
        if v.get(field).and_then(Value::as_u64) == Some(old) {
            v[field] = new.into();
        }
    }
    if v.get("meta").and_then(|m| m.get("id")).and_then(Value::as_u64) == Some(old) {
        v["meta"]["id"] = new.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datapool::{Datapool, Preset};
    use crate::node::NodeMeta;

    /// A show whose preset pool holds presets at `slots`.
    fn show_with_presets(slots: &[u32]) -> ShowData {
        let mut show = ShowData::new();
        let mut pool = BTreeMap::new();
        for &n in slots {
            pool.insert(
                n,
                Preset {
                    meta: NodeMeta::new(Some(format!("P{n}"))),
                    mode: Default::default(),
                    values: Vec::new(),
                },
            );
        }
        show.datapools
            .entry(1)
            .or_insert_with(|| Datapool::new(Some("Default".into())))
            .presets = pool;
        show
    }

    fn presets(show: &ShowData) -> &BTreeMap<u32, Preset> {
        &show.datapools[&1].presets
    }

    fn name(show: &ShowData, n: u32) -> Option<&str> {
        presets(show).get(&n).and_then(|p| p.meta.identity.name.as_deref())
    }

    fn add(show: &mut ShowData, n: u32, label: &str) {
        show.datapools.get_mut(&1).unwrap().presets.insert(
            n,
            Preset {
                meta: NodeMeta::new(Some(label.into())),
                mode: Default::default(),
                values: Vec::new(),
            },
        );
    }

    fn pool_mut(show: &mut ShowData) -> &mut BTreeMap<u32, Preset> {
        &mut show.datapools.get_mut(&1).unwrap().presets
    }

    // The data-negotiation case "store new presets while disconnected".
    #[test]
    fn merge_new_presets_on_both_sides_are_all_kept() {
        let base = show_with_presets(&[1, 2, 3]);
        let mut master = base.clone();
        add(&mut master, 5, "remote 5");
        let mut mine = base.clone();
        add(&mut mine, 4, "local 4");
        let (out, report) = three_way(&base, &master, &mine).unwrap();
        assert_eq!(presets(&out).keys().copied().collect::<Vec<_>>(), vec![1, 2, 3, 4, 5]);
        assert_eq!(name(&out, 4), Some("local 4"));
        assert_eq!(name(&out, 5), Some("remote 5"));
        assert_eq!(report.added_master.len(), 1);
        assert_eq!(report.added_mine.len(), 1);
    }

    // "deleting presets while disconnected": both sides' deletes apply.
    #[test]
    fn merge_deletes_on_both_sides_apply() {
        let base = show_with_presets(&[1, 2, 3]);
        let mut master = base.clone();
        add(&mut master, 5, "remote 5");
        pool_mut(&mut master).remove(&1);
        let mut mine = base.clone();
        pool_mut(&mut mine).remove(&3);
        let (out, _) = three_way(&base, &master, &mine).unwrap();
        assert_eq!(presets(&out).keys().copied().collect::<Vec<_>>(), vec![2, 5]);
    }

    // "merge all after moving presets on remote": the move applies, no copy.
    #[test]
    fn merge_a_move_applies_without_a_duplicate() {
        let base = show_with_presets(&[1, 2, 3]);
        let master = base.clone();
        let mut mine = base.clone();
        let p = pool_mut(&mut mine).remove(&1).unwrap();
        pool_mut(&mut mine).insert(5000, p);
        let (out, report) = three_way(&base, &master, &mine).unwrap();
        assert_eq!(presets(&out).keys().copied().collect::<Vec<_>>(), vec![2, 3, 5000]);
        assert_eq!(report.moves.len(), 1);
        assert_eq!(presets(&out)[&5000].meta.identity.guid, presets(&base)[&1].meta.identity.guid);
    }

    #[test]
    fn merge_same_slot_on_both_sides_moves_mine_on() {
        let base = show_with_presets(&[1]);
        let mut master = base.clone();
        add(&mut master, 4, "remote 4");
        let mut mine = base.clone();
        add(&mut mine, 4, "local 4");
        let (out, report) = three_way(&base, &master, &mine).unwrap();
        assert_eq!(name(&out, 4), Some("remote 4"));
        assert_eq!(name(&out, 5), Some("local 4"));
        assert_eq!(presets(&out)[&5].meta.name(), Some("local 4"), "the object follows its slot");
        assert_eq!(report.moved_slots.len(), 1);
    }

    #[test]
    fn merge_conflicts_keep_the_masters_and_an_edit_beats_a_delete() {
        let base = show_with_presets(&[1, 2]);
        let mut master = base.clone();
        pool_mut(&mut master).get_mut(&1).unwrap().meta.identity.name = Some("session".into());
        pool_mut(&mut master).remove(&2);
        let mut mine = base.clone();
        pool_mut(&mut mine).get_mut(&1).unwrap().meta.identity.name = Some("mine".into());
        pool_mut(&mut mine).get_mut(&2).unwrap().meta.identity.name = Some("edited".into());
        let (out, report) = three_way(&base, &master, &mine).unwrap();
        assert_eq!(name(&out, 1), Some("session"));
        assert_eq!(name(&out, 2), Some("edited"));
        assert_eq!(report.conflicts.len(), 1);
        assert_eq!(report.edit_beats_delete.len(), 1);
    }

    #[test]
    fn merge_is_blocked_by_patch_on_both_sides_or_another_show() {
        let base = show_with_presets(&[1]);
        let mut master = base.clone();
        let mut mine = base.clone();
        assert_eq!(can_merge(Some(&base), &master, &mine), Ok(()));
        master.session.patch_version += 1;
        assert_eq!(can_merge(Some(&base), &master, &mine), Ok(()));
        mine.session.patch_version += 1;
        assert_eq!(
            can_merge(Some(&base), &master, &mine),
            Err(MergeBlocked::PatchOnBothSides)
        );
        assert_eq!(can_merge(None, &master, &mine), Err(MergeBlocked::NoBase));
        let other = ShowData::new();
        assert_eq!(can_merge(Some(&base), &master, &other), Err(MergeBlocked::OtherShow));
    }
}
