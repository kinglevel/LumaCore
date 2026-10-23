//  █████╗ ██████╗ ██████╗ ██████╗ ███████╗███████╗███████╗
// ██╔══██╗██╔══██╗██╔══██╗██╔══██╗██╔════╝██╔════╝██╔════╝
// ███████║██║  ██║██║  ██║██████╔╝█████╗  ███████╗███████╗
// ██╔══██║██║  ██║██║  ██║██╔══██╗██╔══╝  ╚════██║╚════██║
// ██║  ██║██████╔╝██████╔╝██║  ██║███████╗███████║███████║
// ╚═╝  ╚═╝╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝
// FILE: address

//! Where fixtures land on `At DMX …` (decisions §34 round 3 19), for Store and Assign alike: an
//! exact place is exact (one fixture at a taken address overlaps, and the reply says whose), a
//! run skips taken slots (several fixtures, `DMX n`, a bare `DMX`); addresses are packed unless
//! `/Every n` (start to start, never under the footprint) or `/KeepSpacing` (the block keeps its
//! shape); the whole selection fits in one universe or nothing changes. A fixture's own slots
//! are free to it. `plan` decides, `apply` writes.
//!
//! Owns:          the placement rules, writing a fixture's first break, taking addresses away.
//! Doesn't own:   reading `DMX …` (patch::dmx), the words (Store / Assign / Unassign Fixture),
//!                parking an overlap (part 4).
//! Entry points:  Spacing, plan, apply, unassign, footprint.
//! Decisions:     §34 round 3 19.

use std::collections::BTreeMap;

use lumacore_fixtures::{FixtureBreak, Universe};
use lumacore_tree::ShowData;

use crate::errors::render_code;
use crate::objects::patch::dmx::Dest;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// How a run is spaced: packed by default.
#[derive(Clone, Copy, Debug, Default)]
pub struct Spacing {
    /// `/Every n`: each fixture starts n after the one before.
    pub every: Option<u16>,
    /// `/KeepSpacing`: the gaps the fixtures already have.
    pub keep: bool,
}

/// One fixture to place: its number, footprint and where it is now (universe, start).
#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub no: u32,
    pub footprint: u16,
    pub now: Option<(u32, u16)>,
}

/// Where each fixture goes, and what the reply says about it.
#[derive(Debug, Default)]
pub struct Plan {
    /// (fixture, universe, start).
    pub places: Vec<(u32, u32, u16)>,
    /// Universes the plan needs that don't exist yet (created by `apply`).
    pub created: Vec<u32>,
    /// "2.033–2.060 (Fixture 150 Thru 151)": slots a run jumped over.
    pub skipped: Vec<String>,
    /// "2.001 (Fixture 150)": what an exact place overlaps.
    pub overlaps: Vec<String>,
}

/// Who holds each slot, per universe (fixture numbers), the placed fixtures left out.
type Grid = BTreeMap<u32, Vec<Option<u32>>>;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Where `items` go for `dest`; nothing is written.
pub fn plan(show: &ShowData, items: &[Item], dest: Dest, spacing: Spacing) -> Result<Plan, String> {
    if items.is_empty() {
        return Err(render_code("No fixtures to place.", "E-CMD-009"));
    }
    if spacing.keep && spacing.every.is_some() {
        return Err(render_code("/KeepSpacing or /Every, not both.", "E-CMD-006"));
    }
    let widest = items.iter().map(|i| i.footprint).max().unwrap_or(1);
    if let Some(every) = spacing.every
        && every < widest
    {
        return Err(render_code(format!("/Every {every} is under the footprint {widest}: the fixtures would overlap."), "E-CMD-008"));
    }
    let grid = grid(show, items);
    let exists = |u: u32| show.devices.dmx.patch.universes.contains_key(&u);
    let mut plan = match dest {
        // `At Fixture 301 Thru 304 DMX`: each at its partner's address, exactly (pairs up to the
        // shorter list).
        Dest::Of { fixtures, brk } => {
            let mut plan = Plan::default();
            for (item, other) in items.iter().zip(&fixtures) {
                let (u, a) = break_of(show, *other, brk).ok_or_else(|| render_code(format!("Fixture {other} has no address there."), "E-CMD-009"))?;
                let one = exact(&grid, *item, u, a)?;
                plan.places.extend(one.places);
                plan.overlaps.extend(one.overlaps);
            }
            plan
        }
        Dest::Exact(u, a) if items.len() == 1 => exact(&grid, items[0], u, a)?,
        Dest::Exact(u, a) => run(&grid, items, u, a, spacing)?,
        Dest::Universe(u) => run(&grid, items, u, 1, spacing)?,
        Dest::Any => {
            // The universes there are, in order, then the first number that is free.
            let taken: Vec<u32> = show.devices.dmx.patch.universes.keys().copied().collect();
            let first_new = crate::pools::door::next_free(&taken);
            let candidates = show.devices.dmx.patch.universes.keys().copied().chain(std::iter::once(first_new));
            let mut found = None;
            for u in candidates {
                if let Ok(p) = run(&grid, items, u, 1, spacing) {
                    found = Some(p);
                    break;
                }
            }
            found.ok_or_else(|| render_code("No universe has a free run for them all.", "E-CMD-010"))?
        }
    };
    let mut created: Vec<u32> = plan.places.iter().map(|(_, u, _)| *u).filter(|u| !exists(*u)).collect();
    created.dedup();
    plan.created = created;
    Ok(plan)
}

/// One fixture exactly at `u`.`a`: a taken slot is an overlap the reply names.
fn exact(grid: &Grid, item: Item, u: u32, a: u16) -> Result<Plan, String> {
    fits(u, a, item.footprint)?;
    let mut holders: Vec<u32> = slots(grid, u, a, item.footprint).flatten().collect();
    holders.sort_unstable();
    holders.dedup();
    let mut plan = Plan { places: vec![(item.no, u, a)], ..Plan::default() };
    if !holders.is_empty() {
        plan.overlaps.push(format!("{u}.{a:03} (Fixture {})", numbers(&holders)));
    }
    Ok(plan)
}

/// Several fixtures from `u`.`start` on, over taken slots, within the one universe.
fn run(grid: &Grid, items: &[Item], u: u32, start: u16, spacing: Spacing) -> Result<Plan, String> {
    let mut taken = grid.get(&u).cloned().unwrap_or_else(|| vec![None; 512]);
    let mut plan = Plan::default();
    let free = |taken: &[Option<u32>], at: u16, len: u16| at >= 1 && at as u32 + len as u32 - 1 <= 512 && taken[at as usize - 1..(at + len - 1) as usize].iter().all(Option::is_none);
    let no_room = || render_code(format!("They don't fit in universe {u} from {u}.{start:03}: a run never splits across universes."), "E-CMD-010");
    if spacing.keep {
        // The block keeps its shape: offsets from the first fixture's start, in one universe.
        let now: Vec<(u32, u16)> = items.iter().map(|i| i.now).collect::<Option<_>>().ok_or_else(|| render_code("/KeepSpacing needs fixtures that have an address.", "E-CMD-008"))?;
        if now.iter().any(|(nu, _)| *nu != now[0].0) {
            return Err(render_code("/KeepSpacing keeps the gaps inside one universe.", "E-CMD-008"));
        }
        let first = now.iter().map(|(_, a)| *a).min().unwrap_or(1);
        let base = (start..=512)
            .find(|b| items.iter().zip(&now).all(|(i, (_, a))| free(&taken, b + (a - first), i.footprint)))
            .ok_or_else(no_room)?;
        if base > start {
            plan.skipped.push(span(grid, u, start, base));
        }
        plan.places = items.iter().zip(&now).map(|(i, (_, a))| (i.no, u, base + (a - first))).collect();
        return Ok(plan);
    }
    let mut at = start;
    for item in items {
        let found = (at..=512).find(|p| free(&taken, *p, item.footprint)).ok_or_else(no_room)?;
        if found > at {
            plan.skipped.push(span(grid, u, at, found));
        }
        taken[found as usize - 1..(found + item.footprint - 1) as usize].fill(Some(item.no));
        plan.places.push((item.no, u, found));
        at = found + spacing.every.unwrap_or(item.footprint);
    }
    Ok(plan)
}

/// Write `plan`: missing universes made, each fixture's first break set (a fixture without an
/// address gets one), the occupancy rebuilt.
pub fn apply(show: &mut ShowData, plan: &Plan, footprints: &BTreeMap<u32, u16>) {
    let patch = &mut show.devices.dmx.patch;
    for u in &plan.created {
        patch.universes.insert(*u, Universe::new(None));
        patch.universe_id_pool.reserve(*u);
    }
    for (no, u, start) in &plan.places {
        let Some(guid) = show.universe_guid(*u) else { continue };
        let Some(f) = show.devices.dmx.patch.fixture_mut(no) else { continue };
        let len = footprints.get(no).copied().unwrap_or(1);
        match f.mvr.breaks.first_mut() {
            Some(b) => {
                b.universe_id = guid;
                b.start_address = *start;
            }
            None => f.mvr.breaks.push(FixtureBreak::new(guid, *start, len)),
        }
    }
    show.rebuild_occupancy();
}

/// Take addresses away (`Unassign Fixture … At DMX …`): every break, those in universe `u`, or
/// the one starting at `u`.`a`. The fixtures that lost one.
pub fn unassign(show: &mut ShowData, fixtures: &[u32], dest: Dest) -> Vec<u32> {
    let universe = |u: u32| show.universe_guid(u);
    let (in_u, at) = match dest {
        // `At Fixture 201 DMX 2`: that break of each (every break without a number).
        Dest::Of { brk, .. } => {
            let mut lost = Vec::new();
            for no in fixtures {
                let Some(f) = show.devices.dmx.patch.fixture_mut(no) else { continue };
                match brk {
                    Some(b) if (1..=f.mvr.breaks.len()).contains(&(b as usize)) => drop(f.mvr.breaks.remove(b as usize - 1)),
                    Some(_) => continue,
                    None if f.mvr.breaks.is_empty() => continue,
                    None => f.mvr.breaks.clear(),
                }
                lost.push(*no);
            }
            show.rebuild_occupancy();
            return lost;
        }
        Dest::Any => (None, None),
        Dest::Universe(u) => (Some(universe(u)), None),
        Dest::Exact(u, a) => (Some(universe(u)), Some(a)),
    };
    let mut lost = Vec::new();
    for no in fixtures {
        let Some(f) = show.devices.dmx.patch.fixture_mut(no) else { continue };
        let before = f.mvr.breaks.len();
        f.mvr.breaks.retain(|b| {
            let here = in_u.is_none_or(|g| g == Some(b.universe_id));
            let start = at.is_none_or(|a| a == b.start_address);
            !(here && start)
        });
        if f.mvr.breaks.len() < before {
            lost.push(*no);
        }
    }
    show.rebuild_occupancy();
    lost
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// A fixture's footprint: its mode's, else its first break's (0 for a virtual mode: it is placed
/// and occupies nothing).
pub fn footprint(show: &ShowData, no: u32) -> Option<u16> {
    let f = show.devices.dmx.patch.fixture(no)?;
    let by_mode = show
        .devices
        .dmx
        .patch
        .fixture_types
        .values()
        .find(|t| t.meta.identity.guid == f.mvr.fixture_type)
        .and_then(|t| t.mode_of(f.mvr.revision, f.mvr.mode))
        .map(|(_, m)| m.dmx_footprint);
    by_mode.or_else(|| f.mvr.breaks.first().map(|b| b.channel_count))
}

/// The fixtures `items` names as placement items (their first break as where they are now).
pub fn items(show: &ShowData, fixtures: &[u32]) -> Result<Vec<Item>, String> {
    fixtures
        .iter()
        .map(|no| {
            let fp = footprint(show, *no).ok_or_else(|| render_code(format!("Fixture {no} has no mode with a footprint."), "E-CMD-010"))?;
            let now = show.devices.dmx.patch.fixture(no).expect("resolved").mvr.breaks.first().and_then(|b| show.universe_id_for_guid(b.universe_id).map(|u| (u, b.start_address)));
            Ok(Item { no: *no, footprint: fp, now })
        })
        .collect()
}

/// Where fixture `no`'s break `brk` (the first without one) starts: (universe, address).
pub fn break_of(show: &ShowData, no: u32, brk: Option<u16>) -> Option<(u32, u16)> {
    let f = show.devices.dmx.patch.fixture(no)?;
    let b = f.mvr.breaks.get(brk.map_or(0, |b| b as usize).saturating_sub(1))?;
    Some((show.universe_id_for_guid(b.universe_id)?, b.start_address))
}

/// `Assign Fixture A At Fixture B DMX /Swap`: pairs trade addresses (every break, or break
/// `brk`), up to the shorter list; the rest stay. The pairs that traded.
pub fn swap(show: &mut ShowData, a: &[u32], b: &[u32], brk: Option<u16>) -> Vec<(u32, u32)> {
    let mut done = Vec::new();
    for (x, y) in a.iter().zip(b).filter(|(x, y)| x != y) {
        let patch = &mut show.devices.dmx.patch;
        let (Some(fx), Some(fy)) = (patch.fixture(x).cloned(), patch.fixture(y).cloned()) else { continue };
        let (mut bx, mut by) = (fx.mvr.breaks.clone(), fy.mvr.breaks.clone());
        match brk {
            None => std::mem::swap(&mut bx, &mut by),
            Some(n) => {
                let i = n as usize - 1;
                let (Some(p), Some(q)) = (bx.get(i).cloned(), by.get(i).cloned()) else { continue };
                // The addresses trade, each keeps its own footprint.
                (bx[i].universe_id, bx[i].start_address, by[i].universe_id, by[i].start_address) = (q.universe_id, q.start_address, p.universe_id, p.start_address);
            }
        }
        if brk.is_none() {
            // Every break trades: each keeps its own channel counts on the other's addresses.
            for (new, old) in bx.iter_mut().zip(&fx.mvr.breaks) {
                new.channel_count = old.channel_count;
            }
            for (new, old) in by.iter_mut().zip(&fy.mvr.breaks) {
                new.channel_count = old.channel_count;
            }
        }
        patch.fixture_mut(x).expect("read above").mvr.breaks = bx;
        patch.fixture_mut(y).expect("read above").mvr.breaks = by;
        done.push((*x, *y));
    }
    show.rebuild_occupancy();
    done
}

/// Every slot's holder, leaving out the fixtures being placed (their own slots are free to them).
fn grid(show: &ShowData, items: &[Item]) -> Grid {
    let moving: Vec<u32> = items.iter().map(|i| i.no).collect();
    let mut grid = Grid::new();
    for f in show.devices.dmx.patch.fixtures.values() {
        // 0 = a fixture without a number (decisions §40 7); one being placed is left out.
        let no = f.mvr.fid.unwrap_or(0);
        if f.mvr.fid.is_some_and(|n| moving.contains(&n)) {
            continue;
        }
        // Its multipatch copies hold slots in its name.
        for b in f.mvr.breaks.iter().chain(f.luma.multipatch.iter().flat_map(|c| &c.breaks)) {
            let Some(u) = show.universe_id_for_guid(b.universe_id) else { continue };
            let slots = grid.entry(u).or_insert_with(|| vec![None; 512]);
            for a in b.start_address..b.start_address.saturating_add(b.channel_count) {
                if (1..=512).contains(&a) {
                    slots[a as usize - 1] = Some(no);
                }
            }
        }
    }
    grid
}

fn fits(u: u32, a: u16, len: u16) -> Result<(), String> {
    if a as u32 + len as u32 - 1 > 512 {
        return Err(render_code(format!("{u}.{a:03}: the footprint {len} runs past 512 (the last start that fits is {u}.{:03}).", 513 - len), "E-PATCH-003"));
    }
    Ok(())
}

fn slots(grid: &Grid, u: u32, a: u16, len: u16) -> impl Iterator<Item = Option<u32>> + '_ {
    (a..a + len).map(move |s| grid.get(&u).and_then(|g| g[s as usize - 1]))
}

/// "2.033–2.060 (Fixture 150 Thru 151)": the slots from `from` up to `to` and who holds them.
fn span(grid: &Grid, u: u32, from: u16, to: u16) -> String {
    let mut holders: Vec<u32> = slots(grid, u, from, to - from).flatten().collect();
    holders.sort_unstable();
    holders.dedup();
    let who = if holders.is_empty() { String::new() } else { format!(" (Fixture {})", numbers(&holders)) };
    format!("{u}.{from:03}–{u}.{:03}{who}", to - 1)
}

/// Where a plan put them: `2.001` for one, `2.001–2.081` (first start to last start) for several.
pub fn placed_text(plan: &Plan) -> String {
    let at = |(_, u, a): &(u32, u32, u16)| format!("{u}.{a:03}");
    match plan.places.as_slice() {
        [one] => at(one),
        [first, .., last] => format!("{}–{}", at(first), at(last)),
        [] => String::new(),
    }
}

/// The reply's notes on a plan: universes made, slots skipped, overlaps.
pub fn plan_notes(plan: &Plan) -> Vec<String> {
    let mut notes = Vec::new();
    if !plan.created.is_empty() {
        notes.push(format!("Universe {} created.", numbers(&plan.created)));
    }
    if !plan.skipped.is_empty() {
        notes.push(format!("Skipped taken slots: {}.", plan.skipped.join(", ")));
    }
    if !plan.overlaps.is_empty() {
        notes.push(crate::errors::render_code(format!("Overlaps {}: both fixtures share those slots until one moves.", plan.overlaps.join(", ")), "E-PATCH-001"));
    }
    notes
}

/// `150 Thru 152 + 160`.
pub fn numbers(nos: &[u32]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < nos.len() {
        let mut j = i;
        while j + 1 < nos.len() && nos[j + 1] == nos[j] + 1 {
            j += 1;
        }
        parts.push(if j > i { format!("{} Thru {}", nos[i], nos[j]) } else { nos[i].to_string() });
        i = j + 1;
    }
    parts.join(" + ")
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
