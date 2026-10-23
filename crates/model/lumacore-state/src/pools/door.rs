// ██████╗  ██████╗  ██████╗ ██████╗
// ██╔══██╗██╔═══██╗██╔═══██╗██╔══██╗
// ██║  ██║██║   ██║██║   ██║██████╔╝
// ██║  ██║██║   ██║██║   ██║██╔══██╗
// ██████╔╝╚██████╔╝╚██████╔╝██║  ██║
// ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═╝
// FILE: door

//! The one object door (decisions §35): every Store / Delete / Copy / Move / Insert / Compact /
//! Paste writes a pool through here, whatever the object. A pool declares itself once as a typed
//! [`BTreePool`] — its map (a row's number is its key), its [`PoolRules`] (every rule a field, so a
//! pool that forgets one doesn't compile) — and the verbs work on it through the erased
//! [`PoolOps`] (Kubernetes' generic store with per-type strategies). New GUIDs are minted only
//! here: a copy renews every GUID its `Walk` marks own and keeps its links; a move, a swap and an
//! overwrite keep them.
//!
//! Owns:          PoolOps, BTreePool, PoolRules; next_free, next_free_block, spacing; copy,
//!                relocate (swap), insert (shift), compact, delete; renewed; users_of.
//! Doesn't own:   which rows a line names (the resolver), the asks and replies' wording (the verbs
//!                in lumacore-objects), what Store captures (each object).
//! Entry points:  PoolOps, BTreePool, copy, relocate, insert, compact, delete, next_free.
//! Decisions:     §35 1–12, §34 13.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use lumacore_base::{Identified, Walk, Walker};
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::AppState;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One row taken out of its pool, its type erased (a `T` of the pool it came from).
pub type Item = Box<dyn Any + Send>;

/// A pool's map in a DataPool (or the show's), to write and to read.
pub type MapFn<T> = fn(&mut AppState, Option<u32>) -> Option<&mut BTreeMap<u32, T>>;
pub type MapRefFn<T> = fn(&AppState, Option<u32>) -> Option<&BTreeMap<u32, T>>;
/// A named order for `Compact … /By` and its key.
pub type OrderDef<T> = (&'static str, fn(&AppState, &T) -> String);

/// What a pool must answer (decisions §35 2). No defaults: a pool spells every rule out.
pub struct PoolRules<T: 'static> {
    /// Why this row can't be deleted (the locked Default profile), or Ok.
    pub can_delete: fn(&AppState, &T) -> Result<(), String>,
    /// After a row left the show for good (a group's master level goes with it).
    pub deleted: fn(&mut AppState, &T),
    /// After any write to the pool (a fixture bumps the patch generation).
    pub changed: fn(&mut AppState),
    /// Merge `from` into `into` (Store / Copy / Paste `/Merge`), its reply; None = can't merge.
    pub merge: Option<fn(&mut T, &T) -> String>,
    /// Orders `Compact … /By <name>` knows besides Number and Name (`Address` for fixtures).
    pub orders: &'static [OrderDef<T>],
}

/// A pool kept as a `BTreeMap<u32, T>` (every pool of the show today).
pub struct BTreePool<T: 'static> {
    /// Its object word (`Group`).
    pub object: &'static str,
    /// It lives in a DataPool: `None` names the user's selected one.
    pub in_datapool: bool,
    /// The highest number a new row may take (the pools: `pools::MAX_NO`; fixtures the u32 range
    /// but the last, which an ID pool counts past).
    pub max: u32,
    pub map: MapFn<T>,
    pub map_ref: MapRefFn<T>,
    /// A row's name, if it has one (`Group "Front"`; ordering and asks).
    pub name: fn(&T) -> Option<String>,
    /// Give a row a GUID (an overwrite keeps the target's).
    pub identify: fn(&mut T, Uuid),
    pub rules: PoolRules<T>,
}

/// A pool as the verbs see it, whatever it holds.
pub trait PoolOps: Sync {
    fn object(&self) -> &'static str;
    /// The highest number a new row may take.
    fn max_no(&self) -> u32;
    /// The DataPool a line's rows live in (`None` = the user's selected one; None for show pools).
    fn datapool(&self, state: &AppState, dp: Option<u32>) -> Option<u32>;
    fn numbers(&self, state: &AppState, dp: Option<u32>) -> Vec<u32>;
    fn guid(&self, state: &AppState, dp: Option<u32>, no: u32) -> Option<Uuid>;
    fn name(&self, state: &AppState, dp: Option<u32>, no: u32) -> Option<String>;
    /// A clone of a row.
    fn read(&self, state: &AppState, dp: Option<u32>, no: u32) -> Option<Item>;
    /// A row out of its pool (no rule runs).
    fn take(&self, state: &mut AppState, dp: Option<u32>, no: u32) -> Option<Item>;
    /// A row into its pool at `no`; an item of another pool is refused.
    fn put(&self, state: &mut AppState, dp: Option<u32>, no: u32, item: Item) -> Result<(), String>;
    /// The same row with new GUIDs for everything its `Walk` marks own, links kept.
    fn renewed(&self, item: &Item) -> Item;
    /// Content of `from` over row `no`, which keeps its own GUID and number.
    fn overwrite(&self, state: &mut AppState, dp: Option<u32>, no: u32, from: &Item) -> Result<(), String>;
    fn can_merge(&self) -> bool;
    fn merge(&self, state: &mut AppState, dp: Option<u32>, no: u32, from: &Item) -> Result<String, String>;
    fn can_delete(&self, state: &AppState, dp: Option<u32>, no: u32) -> Result<(), String>;
    fn deleted(&self, state: &mut AppState, item: &Item);
    fn changed(&self, state: &mut AppState);
    /// The order names `Compact … /By` takes here (Number and Name always).
    fn orders(&self) -> Vec<&'static str>;
    fn order_key(&self, state: &AppState, dp: Option<u32>, no: u32, by: &str) -> Option<String>;
}

/// What a write does where a row already is (decisions §35 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnExisting {
    /// Nothing is written; the verb asks (`Exists`).
    Ask,
    Merge,
    Overwrite,
}

/// A write that would land on rows that exist, with `OnExisting::Ask`: nothing was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exists {
    pub taken: Vec<u32>,
}

/// How `Compact` orders the rows (decisions §35 16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    Number,
    Name,
    /// A pool's own order (`Address`).
    By(String),
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl<T> PoolOps for BTreePool<T>
where
    T: Identified + Walk + Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn object(&self) -> &'static str {
        self.object
    }

    fn max_no(&self) -> u32 {
        self.max
    }

    fn datapool(&self, state: &AppState, dp: Option<u32>) -> Option<u32> {
        self.in_datapool.then(|| dp.unwrap_or_else(|| state.selected_datapool()))
    }

    fn numbers(&self, state: &AppState, dp: Option<u32>) -> Vec<u32> {
        let dp = self.datapool(state, dp);
        (self.map_ref)(state, dp).map(|m| m.keys().copied().collect()).unwrap_or_default()
    }

    fn guid(&self, state: &AppState, dp: Option<u32>, no: u32) -> Option<Uuid> {
        let dp = self.datapool(state, dp);
        (self.map_ref)(state, dp)?.get(&no).map(Identified::guid)
    }

    fn name(&self, state: &AppState, dp: Option<u32>, no: u32) -> Option<String> {
        let dp = self.datapool(state, dp);
        (self.map_ref)(state, dp)?.get(&no).and_then(self.name)
    }

    fn read(&self, state: &AppState, dp: Option<u32>, no: u32) -> Option<Item> {
        let dp = self.datapool(state, dp);
        (self.map_ref)(state, dp)?.get(&no).map(|t| Box::new(t.clone()) as Item)
    }

    fn take(&self, state: &mut AppState, dp: Option<u32>, no: u32) -> Option<Item> {
        let dp = self.datapool(state, dp);
        (self.map)(state, dp)?.remove(&no).map(|t| Box::new(t) as Item)
    }

    fn put(&self, state: &mut AppState, dp: Option<u32>, no: u32, item: Item) -> Result<(), String> {
        let dp = self.datapool(state, dp);
        let t = *item.downcast::<T>().map_err(|_| format!("Not a {}.", self.object))?;
        let map = (self.map)(state, dp).ok_or_else(|| format!("No {} pool there.", self.object))?;
        map.insert(no, t);
        Ok(())
    }

    fn renewed(&self, item: &Item) -> Item {
        let t = item.downcast_ref::<T>().expect("an item of this pool");
        Box::new(renewed(t))
    }

    fn overwrite(&self, state: &mut AppState, dp: Option<u32>, no: u32, from: &Item) -> Result<(), String> {
        let dp = self.datapool(state, dp);
        let from = from.downcast_ref::<T>().ok_or_else(|| format!("Not a {}.", self.object))?;
        let mut new = renewed(from);
        let map = (self.map)(state, dp).ok_or_else(|| format!("No {} pool there.", self.object))?;
        let old = map.get_mut(&no).ok_or_else(|| format!("{} {no} doesn't exist.", self.object))?;
        (self.identify)(&mut new, Identified::guid(old));
        *old = new;
        Ok(())
    }

    fn can_merge(&self) -> bool {
        self.rules.merge.is_some()
    }

    fn merge(&self, state: &mut AppState, dp: Option<u32>, no: u32, from: &Item) -> Result<String, String> {
        let merge = self.rules.merge.ok_or_else(|| format!("A {} can't merge.", self.object))?;
        let dp = self.datapool(state, dp);
        let from = from.downcast_ref::<T>().ok_or_else(|| format!("Not a {}.", self.object))?;
        let map = (self.map)(state, dp).ok_or_else(|| format!("No {} pool there.", self.object))?;
        let into = map.get_mut(&no).ok_or_else(|| format!("{} {no} doesn't exist.", self.object))?;
        Ok(merge(into, from))
    }

    fn can_delete(&self, state: &AppState, dp: Option<u32>, no: u32) -> Result<(), String> {
        let dp = self.datapool(state, dp);
        match (self.map_ref)(state, dp).and_then(|m| m.get(&no)) {
            Some(t) => (self.rules.can_delete)(state, t),
            None => Ok(()),
        }
    }

    fn deleted(&self, state: &mut AppState, item: &Item) {
        if let Some(t) = item.downcast_ref::<T>() {
            (self.rules.deleted)(state, t);
        }
    }

    fn changed(&self, state: &mut AppState) {
        (self.rules.changed)(state);
    }

    fn orders(&self) -> Vec<&'static str> {
        let mut out = vec!["Number", "Name"];
        out.extend(self.rules.orders.iter().map(|(n, _)| *n));
        out
    }

    fn order_key(&self, state: &AppState, dp: Option<u32>, no: u32, by: &str) -> Option<String> {
        let dp = self.datapool(state, dp);
        let t = (self.map_ref)(state, dp)?.get(&no)?;
        if by.eq_ignore_ascii_case("Name") {
            return Some((self.name)(t).unwrap_or_default().to_lowercase());
        }
        let (_, key) = self.rules.orders.iter().find(|(n, _)| n.eq_ignore_ascii_case(by))?;
        Some(key(state, t))
    }
}

/// A copy of `item` with a new GUID for every object its `Walk` reports as its own (itself, its
/// cues, its views …); a link inside it that names one of those follows the copy, every other
/// link is kept (decisions §35 3, `UserProfile::copy_renewed` the first of its kind).
pub fn renewed<T: Identified + Walk + Serialize + DeserializeOwned>(item: &T) -> T {
    let mut w = Walker::new();
    w.object("Item", item);
    let mut json = serde_json::to_string(item).expect("a show object serializes");
    for o in &w.objects {
        if !o.guid.is_nil() {
            json = json.replace(&o.guid.to_string(), &Uuid::new_v4().to_string());
        }
    }
    serde_json::from_str(&json).expect("its copy deserializes")
}

/// The first number nobody uses (decisions §34 13): Groups 1, 2, 4 → 3.
pub fn next_free(taken: &[u32]) -> u32 {
    let taken: BTreeSet<u32> = taken.iter().copied().collect();
    (1..).find(|n| !taken.contains(n)).expect("u32 has room")
}

/// The first start where a block with these offsets from its first row fits (a range keeps its
/// spacing: offsets 0, 1, 3 need n, n+1 and n+3 free).
pub fn next_free_block(taken: &[u32], offsets: &[u32]) -> Option<u32> {
    let taken: BTreeSet<u32> = taken.iter().copied().collect();
    (1..=crate::pools::MAX_NO).find(|n| offsets.iter().all(|o| n.checked_add(*o).is_some_and(|d| d <= crate::pools::MAX_NO && !taken.contains(&d))))
}

/// The destinations of `sources` from `at`, the spacing kept (`Copy Group 1 + 3 At 10` = 10, 12).
pub fn spacing(sources: &[u32], at: u32) -> Result<Vec<u32>, String> {
    let Some(&first) = sources.first() else { return Ok(Vec::new()) };
    if at == 0 {
        return Err("Numbers start at 1.".into());
    }
    sources
        .iter()
        .map(|&s| at.checked_add(s.abs_diff(first)).filter(|&d| d < u32::MAX).ok_or_else(|| format!("{at} + {} is out of range.", s.abs_diff(first))))
        .collect()
}

/// The offsets of `sources` from the first (for `next_free_block`).
pub fn offsets(sources: &[u32]) -> Vec<u32> {
    let first = sources.first().copied().unwrap_or(0);
    sources.iter().map(|s| s.abs_diff(first)).collect()
}

/// Copy rows `from` (in `from_dp`) to `to` (in `to_dp`), pairwise: new GUIDs, links kept. Every
/// source is read before anything is written (`Copy Group 1 Thru 3 At 2` never reads its own
/// copy). Onto an existing row by `on`: Ask writes nothing and names them.
pub fn copy(ops: &dyn PoolOps, state: &mut AppState, from_dp: Option<u32>, from: &[u32], to_dp: Option<u32>, to: &[u32], on: OnExisting) -> Result<Vec<String>, Exists> {
    let present: BTreeSet<u32> = ops.numbers(state, to_dp).into_iter().collect();
    let taken: Vec<u32> = to.iter().copied().filter(|d| present.contains(d)).collect();
    if on == OnExisting::Ask && !taken.is_empty() {
        return Err(Exists { taken });
    }
    let sources: Vec<(u32, u32, Item)> = from.iter().zip(to).filter_map(|(&f, &t)| ops.read(state, from_dp, f).map(|i| (f, t, i))).collect();
    let mut lines = Vec::new();
    let object = ops.object();
    for (f, t, item) in sources {
        let result = if present.contains(&t) {
            match on {
                OnExisting::Merge if ops.can_merge() => ops.merge(state, to_dp, t, &item).map(|r| format!("{object} {f} merged into {object} {t}: {r}")),
                _ => ops.overwrite(state, to_dp, t, &item).map(|()| format!("{object} {f} copied over {object} {t}.")),
            }
        } else {
            ops.put(state, to_dp, t, ops.renewed(&item)).map(|()| format!("{object} {f} copied to {object} {t}."))
        };
        lines.push(result.unwrap_or_else(|e| e));
    }
    ops.changed(state);
    Ok(lines)
}

/// Move rows `from` to `to` (decisions §35 4, 6): GUIDs kept. A destination that holds a row that
/// isn't moving swaps — the displaced rows go, in order, to the source slots nothing lands on
/// (`Move Group 3 At 7`: 3 ↔ 7). Between owners of one kind it is the same: they swap owners.
pub fn relocate(ops: &dyn PoolOps, state: &mut AppState, from_dp: Option<u32>, from: &[u32], to_dp: Option<u32>, to: &[u32]) -> Vec<String> {
    let object = ops.object();
    let same = ops.datapool(state, from_dp) == ops.datapool(state, to_dp);
    let moving: Vec<(u32, u32)> = from.iter().copied().zip(to.iter().copied()).filter(|(f, t)| !(same && f == t)).collect();
    let sources: BTreeSet<u32> = moving.iter().map(|(f, _)| *f).collect();
    let dests: BTreeSet<u32> = moving.iter().map(|(_, t)| *t).collect();
    // Slots the moving rows leave that nothing lands on, and rows in the way (not moving).
    let freed: Vec<u32> = sources.iter().copied().filter(|s| !(same && dests.contains(s))).collect();
    let in_way: Vec<u32> = {
        let present: BTreeSet<u32> = ops.numbers(state, to_dp).into_iter().collect();
        dests.iter().copied().filter(|d| present.contains(d) && !(same && sources.contains(d))).collect()
    };
    let items: Vec<(u32, u32, Item)> = moving.iter().filter_map(|&(f, t)| ops.take(state, from_dp, f).map(|i| (f, t, i))).collect();
    let displaced: Vec<(u32, Item)> = in_way.iter().filter_map(|&d| ops.take(state, to_dp, d).map(|i| (d, i))).collect();
    let mut lines = Vec::new();
    for (f, t, item) in items {
        match ops.put(state, to_dp, t, item) {
            Ok(()) => lines.push(format!("{object} {f} moved to {object} {t}.")),
            Err(e) => lines.push(e),
        }
    }
    for ((d, item), slot) in displaced.into_iter().zip(freed) {
        match ops.put(state, from_dp, slot, item) {
            Ok(()) => lines.push(format!("{object} {d} moved to {object} {slot} (swapped).")),
            Err(e) => lines.push(e),
        }
    }
    ops.changed(state);
    lines
}

/// Put rows `from` at `at` (their spacing kept) and push the rows in the way up to the first free
/// number, in order (Insert, decisions §35 15). GUIDs kept.
pub fn insert(ops: &dyn PoolOps, state: &mut AppState, from_dp: Option<u32>, from: &[u32], to_dp: Option<u32>, at: u32) -> Result<Vec<String>, String> {
    let object = ops.object();
    let dests = spacing(from, at)?;
    let items: Vec<(u32, u32, Item)> = from.iter().copied().zip(dests.iter().copied()).filter_map(|(f, t)| ops.take(state, from_dp, f).map(|i| (f, t, i))).collect();
    let block: BTreeMap<u32, (u32, Item)> = items.into_iter().map(|(f, t, i)| (t, (f, i))).collect();
    let last = *block.keys().next_back().unwrap_or(&at);
    let mut block = block;
    let mut queue: VecDeque<(u32, Item)> = VecDeque::new();
    let mut lines = Vec::new();
    let mut p = at;
    while p <= ops.max_no() {
        let occupied = ops.numbers(state, to_dp).contains(&p);
        if let Some((f, item)) = block.remove(&p) {
            if occupied {
                let was = ops.take(state, to_dp, p).expect("occupied");
                queue.push_back((p, was));
            }
            ops.put(state, to_dp, p, item)?;
            lines.push(format!("{object} {f} inserted at {object} {p}."));
        } else if occupied && !queue.is_empty() {
            let was = ops.take(state, to_dp, p).expect("occupied");
            let (n, item) = queue.pop_front().expect("not empty");
            ops.put(state, to_dp, p, item)?;
            lines.push(format!("{object} {n} moved up to {object} {p}."));
            queue.push_back((p, was));
        } else if !occupied && let Some((n, item)) = queue.pop_front() {
            ops.put(state, to_dp, p, item)?;
            lines.push(format!("{object} {n} moved up to {object} {p}."));
        }
        if queue.is_empty() && p >= last {
            break;
        }
        if p >= ops.max_no() {
            break;
        }
        p += 1;
    }
    if !queue.is_empty() {
        return Err(format!("No room above {object} {} — nothing more moved.", ops.max_no()));
    }
    ops.changed(state);
    Ok(lines)
}

/// Renumber `rows` (all of the pool when None) one after another from their first number (1 for
/// the whole pool), ordered by `order`; GUIDs kept (decisions §35 16).
pub fn compact(ops: &dyn PoolOps, state: &mut AppState, dp: Option<u32>, rows: Option<&[u32]>, order: &Order) -> Result<Vec<String>, String> {
    let object = ops.object();
    let mut rows: Vec<u32> = match rows {
        Some(r) => r.to_vec(),
        None => ops.numbers(state, dp),
    };
    rows.sort_unstable();
    rows.dedup();
    let Some(&first) = rows.first() else { return Ok(Vec::new()) };
    let start = if rows.len() == ops.numbers(state, dp).len() { 1 } else { first };
    let by = match order {
        Order::Number => None,
        Order::Name => Some("Name".to_string()),
        Order::By(n) => {
            if !ops.orders().iter().any(|o| o.eq_ignore_ascii_case(n)) {
                return Err(format!("{object}s order by {}, not '{n}'.", ops.orders().join(", ")));
            }
            Some(n.clone())
        }
    };
    if let Some(by) = &by {
        let keys: BTreeMap<u32, String> = rows.iter().map(|&n| (n, ops.order_key(state, dp, n, by).unwrap_or_default())).collect();
        rows.sort_by(|a, b| keys[a].cmp(&keys[b]).then(a.cmp(b)));
    }
    // The rows' new slots must be free or among the rows themselves.
    let present: BTreeSet<u32> = ops.numbers(state, dp).into_iter().collect();
    let moving: BTreeSet<u32> = rows.iter().copied().collect();
    let targets: Vec<u32> = (start..).take(rows.len()).collect();
    if let Some(t) = targets.iter().find(|t| present.contains(t) && !moving.contains(t)) {
        return Err(format!("{object} {t} is in the way — compact from {object} {t} on, or the whole pool."));
    }
    let items: Vec<(u32, Item)> = rows.iter().filter_map(|&n| ops.take(state, dp, n).map(|i| (n, i))).collect();
    let mut lines = Vec::new();
    for ((n, item), t) in items.into_iter().zip(targets) {
        ops.put(state, dp, t, item)?;
        if n != t {
            lines.push(format!("{object} {n} → {t}"));
        }
    }
    ops.changed(state);
    Ok(lines)
}

/// Delete rows; one its pool won't let go is refused, named. The links left behind are kept
/// (decisions §35 7): the reply lists them.
pub fn delete(ops: &dyn PoolOps, state: &mut AppState, dp: Option<u32>, rows: &[u32]) -> Vec<String> {
    let object = ops.object();
    let mut lines = Vec::new();
    for &n in rows {
        if let Err(e) = ops.can_delete(state, dp, n) {
            lines.push(e);
            continue;
        }
        let Some(guid) = ops.guid(state, dp, n) else { continue };
        let users = users_of(&state.root.show_data, guid);
        let Some(item) = ops.take(state, dp, n) else { continue };
        ops.deleted(state, &item);
        lines.push(if users.is_empty() { format!("{object} {n} deleted.") } else { format!("{object} {n} deleted; now broken: {}", users.join(", ")) });
    }
    ops.changed(state);
    lines
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// Where the show links to `guid` (tree paths), its own running state left out — what a Delete
/// asks about and leaves broken (decisions §35 7).
pub fn users_of(show: &lumacore_tree::ShowData, guid: Uuid) -> Vec<String> {
    let mut out: Vec<String> = show.walk_identity().links.iter().filter(|l| l.target == guid && !l.path.starts_with("Playbacks.")).map(|l| l.path.clone()).collect();
    out.sort();
    out.dedup();
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
