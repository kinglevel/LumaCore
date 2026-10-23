//! UniverseAddressMap: per-universe 512-slot DMX address occupancy tracker.
//!
//! Wraps a [Option<Uuid>; 512] array tracking which fixture GUID occupies
//! each DMX channel slot. Provides conflict detection and boundary enforcement.

use crate::errors::PatchError;
use lumacore_fixtures::Universe;
use uuid::Uuid;

/// Tracks which fixture GUID occupies each of the 512 DMX channel slots
/// in a single universe.
///
/// Addresses are always 1-indexed in the public API (matching DMX convention).
/// Internal storage uses 0-indexed arrays.
pub struct UniverseAddressMap {
    slots: Box<[Option<Uuid>; 512]>,
}

impl UniverseAddressMap {
    /// Create a new, empty address map (all 512 slots free).
    pub fn new() -> Self {
        Self {
            slots: Box::new([None; 512]),
        }
    }

    /// Attempt to occupy `count` consecutive slots starting at `start` (1-indexed).
    ///
    /// # Errors
    /// - `PatchError::InvalidAddress` if `start` is 0 or above 512
    /// - `PatchError::ExceedsBoundary` if `start - 1 + count > 512`
    /// - `PatchError::Conflict` if any slot in range is already occupied
    ///
    /// On success, writes `owner` GUID to all affected slots.
    pub fn try_occupy(&mut self, start: u16, count: u16, owner: Uuid) -> Result<(), PatchError> {
        if !(1..=512).contains(&start) {
            return Err(PatchError::InvalidAddress { address: start });
        }
        // A mode with no DMX channels occupies nothing (R1-54, Q24).
        if count == 0 {
            return Ok(());
        }
        // Boundary check: 1-indexed start + count - 1 must be <= 512
        let available = 512u16.saturating_sub(start.saturating_sub(1));
        if (start as u32 - 1 + count as u32) > 512 {
            let suggestion = if count <= 512 { 512 - count + 1 } else { 1 };
            return Err(PatchError::ExceedsBoundary {
                needed: count,
                available,
                suggestion,
            });
        }

        // Conflict check: scan each slot before writing any
        for i in 0..count {
            let idx = (start - 1 + i) as usize;
            if let Some(occupant_guid) = self.slots[idx] {
                return Err(PatchError::Conflict {
                    slot: start + i,
                    occupant_guid,
                    occupant_name: None,
                });
            }
        }

        // All clear — write owner to all slots
        for i in 0..count {
            let idx = (start - 1 + i) as usize;
            self.slots[idx] = Some(owner);
        }

        Ok(())
    }

    /// Mark `count` slots from `start` as `owner`'s without any check,
    /// skipping address 0 and slots past 512 (building the map from breaks).
    pub fn mark(&mut self, start: u16, count: u16, owner: Uuid) {
        let start = u32::from(start);
        for addr in (start..start + u32::from(count)).filter(|a| (1..=512).contains(a)) {
            self.slots[addr as usize - 1] = Some(owner);
        }
    }

    /// Release those of `count` slots from `start` (1-indexed) that `owner`
    /// holds. Another fixture's slots in the range stay taken, and address 0
    /// or slots past 512 (a broken loaded break) are skipped (R1-52).
    pub fn release(&mut self, start: u16, count: u16, owner: Uuid) {
        let start = u32::from(start);
        for addr in (start..start + u32::from(count)).filter(|a| (1..=512).contains(a)) {
            let idx = addr as usize - 1;
            if self.slots[idx] == Some(owner) {
                self.slots[idx] = None;
            }
        }
    }

    /// Check if a specific DMX address slot is free (1-indexed).
    ///
    /// Returns `false` for out-of-range addresses (0 or > 512).
    pub fn is_slot_free(&self, addr: u16) -> bool {
        if addr == 0 || addr as usize > 512 {
            return false;
        }
        self.slots[addr as usize - 1].is_none()
    }

    /// Count the number of occupied slots.
    pub fn occupied_count(&self) -> u16 {
        self.slots.iter().filter(|s| s.is_some()).count() as u16
    }

    /// Find the first contiguous block of `size` free slots.
    ///
    /// Returns the 1-indexed start address of the first suitable block,
    /// or `None` if no contiguous block of that size is available.
    /// Used for address suggestions.
    pub fn first_free_block(&self, size: u16) -> Option<u16> {
        if size == 0 || size > 512 {
            return None;
        }
        let size = size as usize;
        let mut run_start = 0usize;
        let mut run_len = 0usize;

        for (i, slot) in self.slots.iter().enumerate() {
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

    /// Reconstruct an address map from a Universe's current occupancy state.
    ///
    /// Used by PatchEngine to perform conflict checking against existing
    /// occupancy before mutating the universe directly.
    pub fn from_universe(universe: &Universe) -> Self {
        let mut map = Self::new();
        for (i, slot) in universe.occupied.iter().enumerate() {
            map.slots[i] = *slot;
        }
        map
    }
}

impl Default for UniverseAddressMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_guid() -> Uuid {
        Uuid::new_v4()
    }

    // --- try_occupy: success cases ---

    #[test]
    fn occupy_empty_universe_succeeds() {
        let mut map = UniverseAddressMap::new();
        let guid = make_guid();
        assert!(map.try_occupy(1, 24, guid).is_ok());
        assert_eq!(map.occupied_count(), 24);
    }

    #[test]
    fn occupy_single_slot_succeeds() {
        let mut map = UniverseAddressMap::new();
        let guid = make_guid();
        assert!(map.try_occupy(1, 1, guid).is_ok());
        assert_eq!(map.occupied_count(), 1);
        assert!(!map.is_slot_free(1));
    }

    #[test]
    fn occupy_last_valid_address_succeeds() {
        let mut map = UniverseAddressMap::new();
        let guid = make_guid();
        // 24-channel fixture at 489 uses 489..512 (exactly 24 slots)
        assert!(map.try_occupy(489, 24, guid).is_ok());
        assert_eq!(map.occupied_count(), 24);
    }

    #[test]
    fn occupy_full_universe_succeeds() {
        let mut map = UniverseAddressMap::new();
        let guid = make_guid();
        assert!(map.try_occupy(1, 512, guid).is_ok());
        assert_eq!(map.occupied_count(), 512);
    }

    // --- try_occupy: conflict cases ---

    #[test]
    fn occupy_conflict_returns_error_with_occupant_guid() {
        let mut map = UniverseAddressMap::new();
        let guid1 = make_guid();
        let guid2 = make_guid();

        map.try_occupy(1, 24, guid1).unwrap();
        let result = map.try_occupy(13, 12, guid2); // overlaps with guid1 at slot 13

        match result {
            Err(PatchError::Conflict {
                slot,
                occupant_guid,
                ..
            }) => {
                assert_eq!(slot, 13);
                assert_eq!(occupant_guid, guid1);
            }
            other => panic!("expected Conflict, got {:?}", other),
        }
    }

    #[test]
    fn occupy_adjacent_non_overlapping_succeeds() {
        let mut map = UniverseAddressMap::new();
        let guid1 = make_guid();
        let guid2 = make_guid();

        map.try_occupy(1, 24, guid1).unwrap();
        assert!(map.try_occupy(25, 24, guid2).is_ok()); // starts right after
        assert_eq!(map.occupied_count(), 48);
    }

    // --- try_occupy: boundary cases ---

    #[test]
    fn occupy_exceeds_512_returns_boundary_error() {
        let mut map = UniverseAddressMap::new();
        let guid = make_guid();
        // 24-channel fixture at 490 would need slots 490..513 — over 512
        let result = map.try_occupy(490, 24, guid);

        match result {
            Err(PatchError::ExceedsBoundary {
                needed,
                available,
                suggestion,
            }) => {
                assert_eq!(needed, 24);
                assert_eq!(available, 23); // 512 - (490-1) = 23 slots available
                assert_eq!(suggestion, 512 - 24 + 1); // = 489
            }
            other => panic!("expected ExceedsBoundary, got {:?}", other),
        }
    }

    #[test]
    fn occupy_at_exactly_512_single_channel_succeeds() {
        let mut map = UniverseAddressMap::new();
        let guid = make_guid();
        assert!(map.try_occupy(512, 1, guid).is_ok());
    }

    #[test]
    fn occupy_at_512_with_count_2_exceeds_boundary() {
        let mut map = UniverseAddressMap::new();
        let guid = make_guid();
        let result = map.try_occupy(512, 2, guid);
        assert!(matches!(result, Err(PatchError::ExceedsBoundary { .. })));
    }

    // --- release ---

    #[test]
    fn release_frees_slots() {
        let mut map = UniverseAddressMap::new();
        let guid = make_guid();
        map.try_occupy(1, 24, guid).unwrap();
        map.release(1, 24, guid);
        assert_eq!(map.occupied_count(), 0);
        assert!(map.is_slot_free(1));
        assert!(map.is_slot_free(24));
    }

    #[test]
    fn release_and_reoccupy_succeeds() {
        let mut map = UniverseAddressMap::new();
        let guid1 = make_guid();
        let guid2 = make_guid();

        map.try_occupy(1, 24, guid1).unwrap();
        map.release(1, 24, guid1);
        // Now guid2 should be able to occupy the same space
        assert!(map.try_occupy(1, 24, guid2).is_ok());
        assert_eq!(map.occupied_count(), 24);
    }

    #[test]
    fn release_frees_only_the_owners_slots() {
        let mut map = UniverseAddressMap::new();
        let (a, b) = (make_guid(), make_guid());
        map.try_occupy(1, 4, a).unwrap();
        map.try_occupy(5, 4, b).unwrap();
        map.release(1, 8, a);
        assert_eq!(map.occupied_count(), 4);
        assert!(!map.is_slot_free(5));
        // Nothing occupied, address 0, past 512: no panic.
        map.release(0, 2, a);
        map.release(510, u16::MAX, a);
        assert_eq!(map.occupied_count(), 4);
    }

    // --- is_slot_free ---

    #[test]
    fn is_slot_free_returns_true_for_empty_universe() {
        let map = UniverseAddressMap::new();
        assert!(map.is_slot_free(1));
        assert!(map.is_slot_free(256));
        assert!(map.is_slot_free(512));
    }

    #[test]
    fn is_slot_free_returns_false_for_occupied_slot() {
        let mut map = UniverseAddressMap::new();
        let guid = make_guid();
        map.try_occupy(10, 5, guid).unwrap();
        assert!(!map.is_slot_free(10));
        assert!(!map.is_slot_free(14));
        assert!(map.is_slot_free(9));
        assert!(map.is_slot_free(15));
    }

    #[test]
    fn is_slot_free_out_of_range_returns_false() {
        let map = UniverseAddressMap::new();
        assert!(!map.is_slot_free(0));
        assert!(!map.is_slot_free(513));
    }

    // --- occupied_count ---

    #[test]
    fn occupied_count_empty() {
        let map = UniverseAddressMap::new();
        assert_eq!(map.occupied_count(), 0);
    }

    #[test]
    fn occupied_count_after_multiple_occupants() {
        let mut map = UniverseAddressMap::new();
        map.try_occupy(1, 24, make_guid()).unwrap();
        map.try_occupy(100, 50, make_guid()).unwrap();
        assert_eq!(map.occupied_count(), 74);
    }

    // --- first_free_block ---

    #[test]
    fn first_free_block_empty_universe_returns_1() {
        let map = UniverseAddressMap::new();
        assert_eq!(map.first_free_block(1), Some(1));
        assert_eq!(map.first_free_block(24), Some(1));
        assert_eq!(map.first_free_block(512), Some(1));
    }

    #[test]
    fn first_free_block_full_universe_returns_none() {
        let mut map = UniverseAddressMap::new();
        map.try_occupy(1, 512, make_guid()).unwrap();
        assert_eq!(map.first_free_block(1), None);
    }

    #[test]
    fn first_free_block_skips_occupied_start() {
        let mut map = UniverseAddressMap::new();
        map.try_occupy(1, 10, make_guid()).unwrap(); // 1-10 occupied
        assert_eq!(map.first_free_block(5), Some(11));
    }

    #[test]
    fn first_free_block_finds_correct_gap() {
        let mut map = UniverseAddressMap::new();
        map.try_occupy(1, 10, make_guid()).unwrap(); // 1-10
        map.try_occupy(15, 10, make_guid()).unwrap(); // 15-24
        // Free: 11-14 (4 slots), 25-512
        assert_eq!(map.first_free_block(4), Some(11));
        assert_eq!(map.first_free_block(5), Some(25));
    }

    #[test]
    fn first_free_block_size_zero_returns_none() {
        let map = UniverseAddressMap::new();
        assert_eq!(map.first_free_block(0), None);
    }

    #[test]
    fn first_free_block_size_exceeds_512_returns_none() {
        let map = UniverseAddressMap::new();
        assert_eq!(map.first_free_block(513), None);
    }
}

#[cfg(test)]
mod zero_tests {
    use super::*;

    /// R1-54 (Q24): a mode with no DMX channels patches and occupies nothing.
    #[test]
    fn a_zero_channel_mode_occupies_nothing() {
        let mut map = UniverseAddressMap::new();
        assert!(map.try_occupy(1, 0, Uuid::new_v4()).is_ok());
        assert!(map.try_occupy(512, 0, Uuid::new_v4()).is_ok());
        assert_eq!(map.occupied_count(), 0);
    }
}
