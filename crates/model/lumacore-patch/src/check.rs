//! The patch consistency check (R1-52): what a loaded (or synced, merged,
//! restored) show's patch gets wrong. Like the reference check
//! (`lumacore_objects::references`) it only **lists** — load still succeeds
//! and nothing is dropped or moved.

use lumacore_fixtures::{Fixture, Slot};
use lumacore_tree::ShowData;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// One line per problem, fixtures in number order.
pub fn problems(show: &ShowData) -> Vec<String> {
    let patch = &show.devices.dmx.patch;
    let mut out = Vec::new();
    // Per universe GUID: which fixture holds each slot (first one wins here).
    let mut slots: BTreeMap<Uuid, Box<[Option<Slot>; 512]>> = BTreeMap::new();
    let mut overlaps: BTreeSet<(Slot, Slot, u32)> = BTreeSet::new();
    for (&key, f) in &patch.fixtures {
        // How a line names it (decisions §40 7): "Fixture 849", "Fixture Media 7003".
        let no = f.handle();
        for brk in &f.mvr.breaks {
            let Some(universe) = show.universe_id_for_guid(brk.universe_id) else {
                out.push(format!(
                    "{no}: break on a missing universe ({})",
                    brk.universe_id
                ));
                continue;
            };
            let (start, count) = (brk.start_address, brk.channel_count);
            if start == 0 {
                out.push(format!("{no}: start address 0 in Universe {universe}"));
            }
            if u32::from(start) + u32::from(count) > 513 {
                out.push(format!(
                    "{no}: {universe}.{start} + {count} channels runs past 512"
                ));
            }
            let taken = slots
                .entry(brk.universe_id)
                .or_insert_with(|| Box::new([None; 512]));
            let first = u32::from(start);
            for addr in (first..first + u32::from(count)).filter(|a| (1..=512).contains(a)) {
                let slot = &mut taken[addr as usize - 1];
                match *slot {
                    Some(other) if other != key => {
                        overlaps.insert((other, key, universe));
                    }
                    Some(_) => {}
                    None => *slot = Some(key),
                }
            }
        }
        match patch
            .fixture_types
            .values()
            .find(|t| t.meta.identity.guid == f.mvr.fixture_type)
        {
            None => {} // a broken link: the reference check lists it
            Some(t) => match t.mode_of(f.mvr.revision, f.mvr.mode) {
                None => out.push(format!(
                    "{no}: its mode isn't in its fixture type's revision"
                )),
                Some((_, mode)) => {
                    let patched: u32 = f.mvr.breaks.iter().map(|b| u32::from(b.channel_count)).sum();
                    if patched != u32::from(mode.dmx_footprint) {
                        out.push(format!(
                            "{no}: patched with {patched} channels, its mode now has {}",
                            mode.dmx_footprint
                        ));
                    }
                }
            },
        }
    }
    for (a, b, universe) in overlaps {
        let handle = |k: Slot| patch.fixtures.get(&k).map(Fixture::handle).unwrap_or_default();
        out.push(format!("{} overlaps {} in Universe {universe}", handle(b), handle(a)));
    }
    out
}

/// A fixture whose mode's footprint changed (a re-derived GDTF): it takes
/// the new channel count when the slots it would need are free, else it
/// keeps its count and `problems` lists the conflict (R1-52, Q20). Only
/// single-break fixtures are adjusted. One line per adjusted fixture.
pub fn follow_footprints(show: &mut ShowData) -> Vec<String> {
    let patch = &show.devices.dmx.patch;
    // Every slot taken, by universe GUID → owner fixture number.
    let mut taken: BTreeMap<Uuid, Box<[Option<Slot>; 512]>> = BTreeMap::new();
    for (&no, f) in &patch.fixtures {
        for b in &f.mvr.breaks {
            let slots = taken.entry(b.universe_id).or_insert_with(|| Box::new([None; 512]));
            let first = u32::from(b.start_address);
            for a in (first..first + u32::from(b.channel_count)).filter(|a| (1..=512).contains(a)) {
                slots[a as usize - 1].get_or_insert(no);
            }
        }
    }
    let mut changes: Vec<(Slot, u16)> = Vec::new();
    for (&no, f) in &patch.fixtures {
        let [brk] = f.mvr.breaks.as_slice() else {
            continue;
        };
        let Some(footprint) = patch
            .fixture_types
            .values()
            .find(|t| t.meta.identity.guid == f.mvr.fixture_type)
            .and_then(|t| t.mode_of(f.mvr.revision, f.mvr.mode))
            .map(|(_, m)| m.dmx_footprint)
        else {
            continue;
        };
        if footprint == brk.channel_count {
            continue;
        }
        let start = u32::from(brk.start_address);
        let end = start + u32::from(footprint);
        let fits = start >= 1 && end <= 513;
        let free = fits
            && taken.get(&brk.universe_id).is_none_or(|slots| {
                (start..end).all(|a| slots[a as usize - 1].is_none_or(|owner| owner == no))
            });
        if free {
            // The grown slots are this fixture's now: a later one can't
            // take them too.
            let slots = taken.entry(brk.universe_id).or_insert_with(|| Box::new([None; 512]));
            for a in start..end {
                slots[a as usize - 1] = Some(no);
            }
            changes.push((no, footprint));
        }
    }
    let mut notes = Vec::new();
    for (no, footprint) in changes {
        if let Some(f) = show.devices.dmx.patch.fixtures.get_mut(&no) {
            let was = f.mvr.breaks[0].channel_count;
            f.mvr.breaks[0].channel_count = footprint;
            notes.push(format!("{}: its mode now has {footprint} channels (was {was}): taken, the slots were free", f.handle()));
        }
    }
    if !notes.is_empty() {
        show.rebuild_occupancy();
    }
    notes
}
