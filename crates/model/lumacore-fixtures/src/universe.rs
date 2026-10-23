// ██╗   ██╗███╗   ██╗██╗██╗   ██╗███████╗██████╗ ███████╗███████╗
// ██║   ██║████╗  ██║██║██║   ██║██╔════╝██╔══██╗██╔════╝██╔════╝
// ██║   ██║██╔██╗ ██║██║██║   ██║█████╗  ██████╔╝███████╗█████╗
// ██║   ██║██║╚██╗██║██║╚██╗ ██╔╝██╔══╝  ██╔══██╗╚════██║██╔══╝
// ╚██████╔╝██║ ╚████║██║ ╚████╔╝ ███████╗██║  ██║███████║███████╗
//  ╚═════╝ ╚═╝  ╚═══╝╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝╚══════╝╚══════╝
// FILE: universe

//! Universe: a 512-channel DMX universe with per-slot occupancy tracking
//! and its input merge mode (decisions §11). The protocol lines name it by its
//! number (a slot), never the other way round (§34 "34f7c hardened").
//!
//! Owns:          Universe: its occupancy and its input merge mode.
//! Doesn't own:   the lines (lumacore-protocols), universe ops over ShowData (lumacore-tree universe_ops.rs).
//! Entry points:  Universe.
//! Decisions:     §11, §34 25.10.

use lumacore_base::node::{NodeMeta, TreeNode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Capacity of a single DMX universe per E1.11/E1.31 specification.
pub const UNIVERSE_CAPACITY: usize = 512;

/// A single DMX universe with 512-channel slot occupancy tracking.
///
/// Each slot records which fixture GUID occupies it, or None if free.
/// The occupied array is skipped on serialization — it is always derived
/// from the patched fixtures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Universe {
    pub meta: NodeMeta,
    /// How DMX input merges into it (the merge mode: its one home, decisions §11 "Issue 11
    /// hardening" 4).
    #[serde(default)]
    pub merge_mode: lumacore_protocols::MergeMode,
    /// Per-slot occupancy: which fixture GUID occupies each DMX address.
    /// Index 0 = DMX address 1 (1-indexed in user interface).
    /// Skipped in serialization — rebuilt from fixtures on load.
    #[serde(skip, default = "Universe::empty_occupied")]
    pub occupied: Box<[Option<Uuid>; UNIVERSE_CAPACITY]>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Universe {
    fn empty_occupied() -> Box<[Option<Uuid>; UNIVERSE_CAPACITY]> {
        Box::new([None; UNIVERSE_CAPACITY])
    }
}

impl Universe {
    pub fn new(name: Option<String>) -> Self {
        Self {
            meta: NodeMeta::new(name),
            merge_mode: lumacore_protocols::MergeMode::default(),
            occupied: Box::new([None; UNIVERSE_CAPACITY]),
        }
    }

    /// Check if a specific DMX address slot is free.
    ///
    /// `addr` is 1-indexed (1-512). Returns false for out-of-range addresses.
    pub fn is_slot_free(&self, addr: u16) -> bool {
        if addr == 0 || addr as usize > UNIVERSE_CAPACITY {
            return false;
        }
        self.occupied[addr as usize - 1].is_none()
    }

    /// Count of occupied slots.
    pub fn occupied_count(&self) -> usize {
        self.occupied.iter().filter(|s| s.is_some()).count()
    }

    /// Count of free slots.
    pub fn free_count(&self) -> usize {
        UNIVERSE_CAPACITY - self.occupied_count()
    }

    /// Find the first contiguous block of `size` free slots.
    ///
    /// Returns the 1-indexed start address of the first free block, or None
    /// if no contiguous block of the requested size is available.
    pub fn first_free_block(&self, size: u16) -> Option<u16> {
        if size == 0 || size as usize > UNIVERSE_CAPACITY {
            return None;
        }
        let size = size as usize;
        let mut run_start = 0usize;
        let mut run_len = 0usize;

        for (i, slot) in self.occupied.iter().enumerate() {
            if slot.is_none() {
                if run_len == 0 {
                    run_start = i;
                }
                run_len += 1;
                if run_len >= size {
                    return Some((run_start + 1) as u16); // 1-indexed
                }
            } else {
                run_len = 0;
            }
        }
        None
    }

    /// The slot indices (0-based) of `count` slots from `start_addr`
    /// (1-indexed), skipping address 0 and anything past 512 — a loaded show
    /// may hold such a break (R1-52); the patch check lists it.
    fn slot_range(start_addr: u16, count: u16) -> impl Iterator<Item = usize> {
        let start = u32::from(start_addr);
        (start..start + u32::from(count))
            .filter(|a| (1..=UNIVERSE_CAPACITY as u32).contains(a))
            .map(|a| a as usize - 1)
    }

    /// Mark `count` slots starting at `start_addr` (1-indexed) as occupied
    /// by the given fixture GUID.
    pub fn mark_occupied(&mut self, start_addr: u16, count: u16, fixture_guid: Uuid) {
        for idx in Self::slot_range(start_addr, count) {
            self.occupied[idx] = Some(fixture_guid);
        }
    }

    /// Free `count` slots starting at `start_addr` (1-indexed).
    pub fn mark_free(&mut self, start_addr: u16, count: u16) {
        for idx in Self::slot_range(start_addr, count) {
            self.occupied[idx] = None;
        }
    }

    /// Free those of `count` slots from `start_addr` that `owner` holds;
    /// another fixture's slots in the range stay taken (R1-52).
    pub fn release_owned(&mut self, start_addr: u16, count: u16, owner: Uuid) {
        for idx in Self::slot_range(start_addr, count) {
            if self.occupied[idx] == Some(owner) {
                self.occupied[idx] = None;
            }
        }
    }
}

impl TreeNode for Universe {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Universe"
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
