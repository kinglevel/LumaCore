// ██████╗ ███╗   ███╗██╗  ██╗    ███████╗██╗      ██████╗ ████████╗███████╗
// ██╔══██╗████╗ ████║╚██╗██╔╝    ██╔════╝██║     ██╔═══██╗╚══██╔══╝██╔════╝
// ██║  ██║██╔████╔██║ ╚███╔╝     ███████╗██║     ██║   ██║   ██║   ███████╗
// ██║  ██║██║╚██╔╝██║ ██╔██╗     ╚════██║██║     ██║   ██║   ██║   ╚════██║
// ██████╔╝██║ ╚═╝ ██║██╔╝ ██╗    ███████║███████╗╚██████╔╝   ██║   ███████║
// ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝    ╚══════╝╚══════╝ ╚═════╝    ╚═╝   ╚══════╝
// FILE: dmx_slots

//! Who sits on which DMX slot (decisions §34 round 3, 18, `dmx-map.html` v2): every patched run —
//! a fixture's breaks and its multipatch copies' —, the overlaps between them and the runs past
//! 512, the fixtures that are **parked** for it (both sides of an overlap: they keep their address
//! and programming but send nothing until resolved; nothing asks at patch time) and each
//! universe's slots in use. The render plan skips a parked fixture; the patch feed, the DMX map
//! and the conflict list read the same answer.
//!
//! Owns:          SlotMap (runs, conflicts, parked, usage).
//! Doesn't own:   patching (the Assign / Store lines), the plan (`lumacore-render`), the rows
//!                (`lumacore-views`).
//! Entry points:  SlotMap::of.
//! Decisions:     §34 round 3, 18 (an overlap parks both).

use crate::tree::PatchData;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One patched run of slots: a fixture's break, or a break of one of its multipatch copies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    /// The fixture's number and GUID.
    pub fixture: u32,
    pub guid: Uuid,
    /// The multipatch copy it belongs to (1 = the first, its place in the fixture), or none.
    pub copy: Option<u32>,
    /// Its break (1 = the first).
    pub brk: u32,
    pub universe: Uuid,
    /// The first slot (1-indexed) and how many.
    pub start: u16,
    pub count: u16,
}

impl Run {
    /// The last slot it would take (past 512 when it runs over).
    pub fn end(&self) -> u32 {
        u32::from(self.start) + u32::from(self.count.max(1)) - 1
    }
}

/// A conflict: two runs sharing slots (`other` = the second run), or one run past 512 (none).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conflict {
    pub run: usize,
    pub other: Option<usize>,
}

/// The slot map of a patch.
#[derive(Debug, Clone, Default)]
pub struct SlotMap {
    /// Every run on an existing universe, by fixture number then break (copies after).
    pub runs: Vec<Run>,
    /// Each overlap once (the earlier run first), then each run past 512.
    pub conflicts: Vec<Conflict>,
    /// The fixtures (GUIDs) that send nothing until resolved.
    pub parked: BTreeSet<Uuid>,
    /// Slots in use per universe GUID, a shared slot counted once.
    pub used: BTreeMap<Uuid, u16>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl SlotMap {
    /// The slot map of `patch`. A break on a universe that is gone, or at address 0, takes no
    /// slot (the row says "no universe").
    pub fn of(patch: &PatchData) -> Self {
        let mut map = Self::default();
        for f in patch.fixtures.values() {
            let guid = f.meta.identity.guid;
            // 0 = a fixture without a number (decisions §40 7); a folder has no address anyway.
            let no = f.mvr.fid.unwrap_or(0);
            let own = f.mvr.breaks.iter().map(|b| (None, b));
            let copies = f.luma.multipatch.iter().zip(1u32..).flat_map(|(c, n)| c.breaks.iter().map(move |b| (Some(n), b)));
            let mut brk_of: BTreeMap<Option<u32>, u32> = BTreeMap::new();
            for (copy, b) in own.chain(copies) {
                let brk = brk_of.entry(copy).or_insert(0);
                *brk += 1;
                if b.start_address == 0 || !patch.universes.values().any(|u| u.meta.identity.guid == b.universe_id) {
                    continue;
                }
                map.runs.push(Run { fixture: no, guid, copy, brk: *brk, universe: b.universe_id, start: b.start_address, count: b.channel_count });
            }
        }
        // Who sits on each slot: two runs on one slot are one overlap, found once per pair.
        let mut slots: BTreeMap<(Uuid, u16), Vec<usize>> = BTreeMap::new();
        for (i, r) in map.runs.iter().enumerate() {
            for s in u32::from(r.start)..=r.end().min(512) {
                slots.entry((r.universe, s as u16)).or_default().push(i);
            }
        }
        let mut pairs: BTreeSet<(usize, usize)> = BTreeSet::new();
        for ((universe, _), on) in &slots {
            *map.used.entry(*universe).or_insert(0) += 1;
            for (k, a) in on.iter().enumerate() {
                for b in &on[k + 1..] {
                    pairs.insert((*a.min(b), *a.max(b)));
                }
            }
        }
        map.conflicts.extend(pairs.into_iter().map(|(a, b)| Conflict { run: a, other: Some(b) }));
        let past = map.runs.iter().enumerate().filter(|(_, r)| r.end() > 512).map(|(i, _)| Conflict { run: i, other: None });
        map.conflicts.extend(past.collect::<Vec<_>>());
        for c in &map.conflicts {
            map.parked.insert(map.runs[c.run].guid);
            if let Some(o) = c.other {
                map.parked.insert(map.runs[o].guid);
            }
        }
        map
    }

    /// Whether fixture `guid` is parked.
    pub fn is_parked(&self, guid: Uuid) -> bool {
        self.parked.contains(&guid)
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
mod tests;
