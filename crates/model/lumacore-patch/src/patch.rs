//! PatchEngine: stateless operations on ShowData for fixture patching.
//!
//! PatchEngine operates on `&mut ShowData` — it carries no state itself.
//! All fixture-to-universe assignments, conflict detection, and address
//! tracking go through this module.

use crate::address_map::UniverseAddressMap;
use crate::errors::PatchError;
use lumacore_tree::ShowData;
use lumacore_fixtures::{Fixture};
use uuid::Uuid;

/// Summary of a patched fixture for listing purposes.
#[derive(Debug, Clone)]
pub struct FixtureListEntry {
    pub fixture_id: u32,
    pub name: Option<String>,
    pub universe_id: u32,
    pub start_address: u16,
    pub mode_name: Option<String>,
    pub footprint: u16,
    pub fixture_type_name: Option<String>,
    pub manufacturer: String,
}

/// Stateless patch engine — all operations take `&mut ShowData`.
///
/// PatchEngine does NOT store any state. It is a namespace for patch
/// operations that mutate ShowData directly.
pub struct PatchEngine;

impl PatchEngine {
    /// Patch a fixture to a universe at the specified start address.
    ///
    /// # Arguments
    /// - `show`: mutable reference to ShowData
    /// - `fixture_type_guid`: GUID of the FixtureType to patch from
    /// - `revision_index`: zero-based index into FixtureType.revisions
    /// - `mode_index`: zero-based index into Revision.modes
    /// - `universe_id`: integer ID of the target universe
    /// - `start_address`: 1-indexed DMX start address
    /// - `name`: optional display name for the fixture
    ///
    /// # Returns
    /// The new fixture's integer ID on success.
    ///
    /// # Errors
    /// - `PatchError::FixtureTypeNotFound` if the GUID is not in show.devices.dmx.patch.fixture_types
    /// - `PatchError::UniverseNotFound` if universe_id is not in show.devices.dmx.patch.universes
    /// - `PatchError::Conflict` if any slots are already occupied
    /// - `PatchError::ExceedsBoundary` if footprint exceeds 512 from start address
    #[allow(
        clippy::too_many_arguments,
        reason = "patch operations move to the root change path (decisions.md §26)"
    )]
    pub fn patch_fixture(
        show: &mut ShowData,
        fixture_type_guid: Uuid,
        revision_index: usize,
        mode_index: usize,
        universe_id: u32,
        start_address: u16,
        name: Option<String>,
    ) -> Result<u32, PatchError> {
        // 1. Look up FixtureType by guid
        let (_ft_manufacturer, _ft_name, footprint, revision_guid, mode_guid) = {
            let ft = show
                .devices
                .dmx
                .patch
                .fixture_types
                .values()
                .find(|ft| ft.meta.guid() == fixture_type_guid)
                .ok_or(PatchError::FixtureTypeNotFound {
                    guid: fixture_type_guid,
                })?;

            let revision =
                ft.revisions
                    .get(revision_index)
                    .ok_or(PatchError::FixtureTypeNotFound {
                        guid: fixture_type_guid,
                    })?;
            let mode = revision
                .modes
                .get(mode_index)
                .ok_or(PatchError::FixtureTypeNotFound {
                    guid: fixture_type_guid,
                })?;

            (
                ft.manufacturer.clone(),
                ft.meta.name().map(|s| s.to_string()),
                mode.dmx_footprint,
                revision.meta.identity.guid,
                mode.meta.identity.guid,
            )
        };

        // 2. Look up Universe and get its GUID
        let universe_guid = show
            .universe_guid(universe_id)
            .ok_or(PatchError::UniverseNotFound { universe_id })?;
        let universe = show
            .devices
            .dmx
            .patch
            .universes
            .get_mut(&universe_id)
            .ok_or(PatchError::UniverseNotFound { universe_id })?;

        // 3. Build a temporary address map from the universe's current occupied state
        //    and check for conflicts
        let fixture_guid = Uuid::new_v4();
        let mut addr_map = UniverseAddressMap::from_universe(universe);
        addr_map.try_occupy(start_address, footprint, fixture_guid)?;

        // A number before anything changes: a full pool changes nothing (R1-12).
        let fixture_id = show.devices.dmx.patch.fixture_id_pool.allocate().ok_or(PatchError::NoFreeNumber)?;

        // 4. Apply the occupancy to the actual universe
        let universe = show
            .devices
            .dmx
            .patch
            .universes
            .get_mut(&universe_id)
            .ok_or(PatchError::UniverseNotFound { universe_id })?;
        universe.mark_occupied(start_address, footprint, fixture_guid);

        // 5. Create the Fixture and insert into show (using universe GUID)
        let mut fixture = Fixture::new(
            name,
            fixture_type_guid,
            revision_guid,
            mode_guid,
            universe_guid,
            start_address,
            footprint,
        );

        // Override the fixture's GUID to match what we registered in the occupied map
        // Note: Fixture::new generates its own guid via NodeMeta::new. We need to
        // update the universe occupancy to use the fixture's actual guid instead.
        let actual_guid = fixture.meta.guid();
        // Re-mark with the fixture's actual GUID (clear the temp guid, set real one)
        let universe = show
            .devices
            .dmx
            .patch
            .universes
            .get_mut(&universe_id)
            .expect("universe confirmed to exist above");
        universe.release_owned(start_address, footprint, fixture_guid);
        universe.mark_occupied(start_address, footprint, actual_guid);

        // Its number is the one allocated above; the slot is the pool's (decisions §40 5).
        fixture.mvr.fid = Some(fixture_id);
        show.devices.dmx.patch.insert_fixture(fixture);

        Ok(fixture_id)
    }

    /// Move a patched fixture to `universe_id`.`start_address` (Live Patch,
    /// decisions §11). Its own slots don't count as taken.
    pub fn readdress_fixture(
        show: &mut ShowData,
        fixture_id: u32,
        universe_id: u32,
        start_address: u16,
    ) -> Result<(), PatchError> {
        let (slot, fixture) = show.devices.dmx.patch.fixture_by_fid(fixture_id).ok_or(PatchError::FixtureNotFound { fixture_id })?;
        let guid = fixture.meta.guid();
        let old = fixture
            .primary_break()
            .cloned()
            .ok_or(PatchError::FixtureNotFound { fixture_id })?;
        let target_guid = show
            .universe_guid(universe_id)
            .ok_or(PatchError::UniverseNotFound { universe_id })?;
        if !show.devices.dmx.patch.universes.contains_key(&universe_id) {
            return Err(PatchError::UniverseNotFound { universe_id });
        }
        // What every *other* fixture takes in the target universe, from the
        // breaks themselves: a loaded show may overlap, and the stored
        // occupancy then names only one owner per slot (R1-52).
        let mut addr_map = UniverseAddressMap::new();
        for (_, other) in show.devices.dmx.patch.fixtures.iter().filter(|(n, _)| **n != slot) {
            for b in other.mvr.breaks.iter().filter(|b| b.universe_id == target_guid) {
                addr_map.mark(b.start_address, b.channel_count, other.meta.guid());
            }
        }
        addr_map.try_occupy(start_address, old.channel_count, guid)?;

        let fixture = show.devices.dmx.patch.fixtures.get_mut(&slot).expect("checked above");
        if let Some(brk) = fixture.mvr.breaks.first_mut() {
            brk.universe_id = target_guid;
            brk.start_address = start_address;
        }
        // Freed slots another (overlapping) fixture also covers stay taken.
        show.rebuild_occupancy();
        Ok(())
    }

    /// Unpatch a fixture, removing it from ShowData and freeing all occupied slots.
    ///
    /// # Errors
    /// - `PatchError::FixtureNotFound` if fixture_id is not in show.devices.dmx.patch.fixtures
    pub fn unpatch_fixture(show: &mut ShowData, fixture_id: u32) -> Result<(), PatchError> {
        // 1. Remove fixture from show
        let slot = show.devices.dmx.patch.fixture_by_fid(fixture_id).map(|(s, _)| s).ok_or(PatchError::FixtureNotFound { fixture_id })?;
        let fixture = show.devices.dmx.patch.remove_fixture(slot).ok_or(PatchError::FixtureNotFound { fixture_id })?;

        // 2. For each break, release slots in the corresponding universe (resolve GUID to key)
        for brk in &fixture.mvr.breaks {
            if let Some(uid) = show.universe_id_for_guid(brk.universe_id)
                && let Some(universe) = show.devices.dmx.patch.universes.get_mut(&uid)
            {
                universe.release_owned(brk.start_address, brk.channel_count, fixture.meta.guid());
            }
            // If the universe is gone, slots are irrelevant — no error
        }

        // An overlapping fixture keeps the slots it shared (R1-52).
        show.rebuild_occupancy();

        // The number went back to the pool with `remove_fixture`.
        Ok(())
    }

    /// List all patched fixtures with their universe, address, mode, and type info.
    pub fn list_fixtures(show: &ShowData) -> Vec<FixtureListEntry> {
        show.devices
            .dmx
            .patch
            .fixtures
            .iter()
            .map(|(_, fixture)| {
                // A fixture without a number (a folder) lists as 0 (decisions §40 7).
                let fixture_id = fixture.mvr.fid.unwrap_or(0);
                let primary = fixture.primary_break();
                let (universe_id, start_address, footprint) = primary
                    .map(|b| {
                        let uid = show.universe_id_for_guid(b.universe_id).unwrap_or(0);
                        (uid, b.start_address, b.channel_count)
                    })
                    .unwrap_or((0, 0, 0));

                let (mode_name, ft_name, manufacturer) = show
                    .devices
                    .dmx
                    .patch
                    .fixture_types
                    .values()
                    .find(|ft| ft.meta.guid() == fixture.mvr.fixture_type)
                    .and_then(|ft| {
                        ft.mode_of(fixture.mvr.revision, fixture.mvr.mode).map(|(_, mode)| {
                            (
                                mode.meta.name().map(|s| s.to_string()),
                                ft.meta.name().map(|s| s.to_string()),
                                ft.manufacturer.clone(),
                            )
                        })
                    })
                    .unwrap_or((None, None, String::new()));

                FixtureListEntry {
                    fixture_id,
                    name: fixture.meta.name().map(|s| s.to_string()),
                    universe_id,
                    start_address,
                    mode_name,
                    footprint,
                    fixture_type_name: ft_name,
                    manufacturer,
                }
            })
            .collect()
    }

    /// Patch `count` fixtures sequentially, auto-incrementing address by footprint.
    ///
    /// Implements the `Patch 4x "Mac Aura XB" At 4.1` syntax.
    ///
    /// # Returns
    /// Vec of new fixture IDs on success.
    ///
    /// # Errors
    /// Any single patch failure causes a full rollback of all previously patched
    /// fixtures in this batch.
    #[allow(
        clippy::too_many_arguments,
        reason = "patch operations move to the root change path (decisions.md §26)"
    )]
    pub fn batch_patch(
        show: &mut ShowData,
        count: u32,
        fixture_type_guid: Uuid,
        revision_index: usize,
        mode_index: usize,
        universe_id: u32,
        start_address: u16,
    ) -> Result<Vec<u32>, PatchError> {
        // Determine footprint first (before any mutations)
        let footprint = {
            let ft = show
                .devices
                .dmx
                .patch
                .fixture_types
                .values()
                .find(|ft| ft.meta.guid() == fixture_type_guid)
                .ok_or(PatchError::FixtureTypeNotFound {
                    guid: fixture_type_guid,
                })?;
            let revision =
                ft.revisions
                    .get(revision_index)
                    .ok_or(PatchError::FixtureTypeNotFound {
                        guid: fixture_type_guid,
                    })?;
            let mode = revision
                .modes
                .get(mode_index)
                .ok_or(PatchError::FixtureTypeNotFound {
                    guid: fixture_type_guid,
                })?;
            mode.dmx_footprint
        };

        // The whole span must fit in the universe before anything is
        // allocated or patched (R1-07: `Patch 4294967295x …` must not abort).
        if !(1..=512).contains(&start_address) {
            return Err(PatchError::InvalidAddress {
                address: start_address,
            });
        }
        let available = 513 - start_address;
        let fits = count
            .checked_mul(u32::from(footprint.max(1)))
            .is_some_and(|span| span <= u32::from(available));
        if !fits {
            let needed = count
                .saturating_mul(u32::from(footprint.max(1)))
                .min(u32::from(u16::MAX)) as u16;
            return Err(PatchError::ExceedsBoundary {
                needed,
                available,
                suggestion: if needed <= 512 { 513 - needed } else { 1 },
            });
        }

        let mut patched_ids: Vec<u32> = Vec::with_capacity(count as usize);
        let mut current_address = start_address;

        for i in 0..count {
            let name = Some(format!("Fixture {}", i + 1));
            match Self::patch_fixture(
                show,
                fixture_type_guid,
                revision_index,
                mode_index,
                universe_id,
                current_address,
                name,
            ) {
                Ok(id) => {
                    patched_ids.push(id);
                    // Fits: the span was checked above.
                    current_address = current_address.saturating_add(footprint);
                }
                Err(e) => {
                    // Rollback all previously patched fixtures
                    for &rollback_id in &patched_ids {
                        let _ = Self::unpatch_fixture(show, rollback_id);
                    }
                    return Err(e);
                }
            }
        }

        Ok(patched_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumacore_tree::node::NodeMeta;
    use lumacore_tree::{ShowData};
    use lumacore_fixtures::{AttributeType, ChannelFunction, FixtureCategory, FixtureType, Mode, Revision, Universe};

    /// Build a minimal ShowData with one FixtureType (1 revision, 1 mode, 24-channel)
    /// and one Universe, ready for patching tests.
    fn make_show_with_fixture_type(channels: u16) -> (ShowData, Uuid, u32) {
        let mut show = ShowData::new();

        // Create fixture type
        let _ft_guid = Uuid::new_v4();
        let mut ft = FixtureType::new(
            "TestMfg".to_string(),
            Some("TestFixture".to_string()),
            FixtureCategory::Wash,
        );
        // Override the auto-generated guid with our known guid for testing
        // We need to find the fixture type by guid, so we set it on the meta.
        // NodeMeta uses Uuid::new_v4() by default — we create a custom one.
        // We'll use the fixture_type_guid returned from this function.
        let actual_ft_guid = ft.meta.guid(); // use the auto-generated one

        // Build mode
        let channel_function = ChannelFunction {
            attribute_name: "Dimmer".to_string(),
            attribute_type: AttributeType::Dimmer,
            dmx_offset: 0,
            byte_count: 1,
            default_value: 0,
            default_fraction: None,
            physical_range: None,
            is_virtual: false,
            resolved: true,
            ..Default::default()
        };
        let mode = Mode {
            meta: NodeMeta::new(Some("Standard".to_string())),
            channel_functions: vec![channel_function; channels as usize],
            dmx_footprint: channels,
            geometry: String::new(),
            cells: Vec::new(),
            relations: Vec::new(),
        };
        let revision = Revision {
            meta: NodeMeta::new(Some("Rev1".to_string())),
            gdtf_version: "1.2".to_string(),
            file_hash: "abc123".to_string(),
            modes: vec![mode],
            geometries: Vec::new(),
            wheels: Vec::new(),
            emitters: Vec::new(),
            filters: Vec::new(),
            attributes: Vec::new(),
        };
        ft.revisions.push(revision);

        show.devices.dmx.patch.fixture_types.insert(1, std::sync::Arc::new(ft));

        // Create universe 1
        let universe = Universe::new(Some("Universe 1".to_string()));
        show.devices.dmx.patch.universes.insert(1, universe);

        (show, actual_ft_guid, 1)
    }

    // --- patch_fixture ---

    #[test]
    fn patch_fixture_succeeds_on_empty_universe() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        let result = PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 1, None);
        assert!(result.is_ok());
        let fixture_id = result.unwrap();
        assert!(show.devices.dmx.patch.has_fid(fixture_id));
    }

    #[test]
    fn patch_fixture_updates_universe_occupancy() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 1, None).unwrap();
        let universe = &show.devices.dmx.patch.universes[&universe_id];
        assert_eq!(universe.occupied_count(), 24);
        assert!(!universe.is_slot_free(1));
        assert!(!universe.is_slot_free(24));
        assert!(universe.is_slot_free(25));
    }

    #[test]
    fn patch_fixture_conflict_returns_error() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 1, None).unwrap();
        let result = PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 13, None);
        assert!(matches!(result, Err(PatchError::Conflict { .. })));
    }

    #[test]
    fn patch_fixture_universe_not_found_returns_error() {
        let (mut show, guid, _) = make_show_with_fixture_type(24);
        let result = PatchEngine::patch_fixture(&mut show, guid, 0, 0, 999, 1, None);
        assert!(matches!(
            result,
            Err(PatchError::UniverseNotFound { universe_id: 999 })
        ));
    }

    #[test]
    fn patch_fixture_type_not_found_returns_error() {
        let (mut show, _, universe_id) = make_show_with_fixture_type(24);
        let unknown_guid = Uuid::new_v4();
        let result =
            PatchEngine::patch_fixture(&mut show, unknown_guid, 0, 0, universe_id, 1, None);
        assert!(matches!(
            result,
            Err(PatchError::FixtureTypeNotFound { .. })
        ));
    }

    #[test]
    fn patch_fixture_boundary_exceeded_returns_error() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        let result = PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 490, None);
        assert!(matches!(result, Err(PatchError::ExceedsBoundary { .. })));
    }

    // --- unpatch_fixture ---

    #[test]
    fn unpatch_fixture_removes_fixture_and_frees_slots() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        let fixture_id =
            PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 1, None).unwrap();
        assert!(show.devices.dmx.patch.has_fid(fixture_id));

        PatchEngine::unpatch_fixture(&mut show, fixture_id).unwrap();

        assert!(!show.devices.dmx.patch.has_fid(fixture_id));
        let universe = &show.devices.dmx.patch.universes[&universe_id];
        assert_eq!(universe.occupied_count(), 0);
        assert!(universe.is_slot_free(1));
    }

    #[test]
    fn unpatch_fixture_not_found_returns_error() {
        let (mut show, _, _) = make_show_with_fixture_type(24);
        let result = PatchEngine::unpatch_fixture(&mut show, 999);
        assert!(matches!(
            result,
            Err(PatchError::FixtureNotFound { fixture_id: 999 })
        ));
    }

    #[test]
    fn unpatch_fixture_id_returned_to_pool() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        let id1 = PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 1, None).unwrap();
        PatchEngine::unpatch_fixture(&mut show, id1).unwrap();
        // Patch again — should get the same ID back (reuse)
        let id2 = PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 1, None).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn unpatch_releases_slots_for_new_patch() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        let fixture_id =
            PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 1, None).unwrap();
        PatchEngine::unpatch_fixture(&mut show, fixture_id).unwrap();
        // Should be able to patch again at same address
        let result = PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 1, None);
        assert!(result.is_ok());
    }

    // --- list_fixtures ---

    #[test]
    fn list_fixtures_empty_show_returns_empty() {
        let show = ShowData::new();
        let list = PatchEngine::list_fixtures(&show);
        assert!(list.is_empty());
    }

    #[test]
    fn list_fixtures_returns_correct_details() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        let fixture_id = PatchEngine::patch_fixture(
            &mut show,
            guid,
            0,
            0,
            universe_id,
            100,
            Some("MyLight".to_string()),
        )
        .unwrap();

        let list = PatchEngine::list_fixtures(&show);
        assert_eq!(list.len(), 1);
        let entry = &list[0];
        assert_eq!(entry.fixture_id, fixture_id);
        assert_eq!(entry.name.as_deref(), Some("MyLight"));
        assert_eq!(entry.universe_id, universe_id);
        assert_eq!(entry.start_address, 100);
        assert_eq!(entry.footprint, 24);
        assert_eq!(entry.fixture_type_name.as_deref(), Some("TestFixture"));
        assert_eq!(entry.manufacturer, "TestMfg");
    }

    #[test]
    fn list_fixtures_multiple_fixtures() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 1, None).unwrap();
        PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 25, None).unwrap();
        PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 49, None).unwrap();

        let list = PatchEngine::list_fixtures(&show);
        assert_eq!(list.len(), 3);
    }

    // --- batch_patch ---

    #[test]
    fn batch_patch_creates_n_fixtures_sequentially() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        let ids = PatchEngine::batch_patch(&mut show, 4, guid, 0, 0, universe_id, 1).unwrap();
        assert_eq!(ids.len(), 4);
        assert_eq!(show.devices.dmx.patch.fixtures.len(), 4);
        // Check addresses: 1, 25, 49, 73
        let list = PatchEngine::list_fixtures(&show);
        let mut addresses: Vec<u16> = list.iter().map(|e| e.start_address).collect();
        addresses.sort();
        assert_eq!(addresses, vec![1, 25, 49, 73]);
    }

    #[test]
    fn batch_patch_rollback_on_conflict() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        // Pre-occupy slot 25 by patching a single fixture there
        PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 25, None).unwrap();

        // Try batch of 4 starting at 1 — the 2nd fixture at 25 will conflict
        let result = PatchEngine::batch_patch(&mut show, 4, guid, 0, 0, universe_id, 1);
        assert!(matches!(result, Err(PatchError::Conflict { .. })));

        // Only the pre-existing fixture should remain (batch was rolled back)
        assert_eq!(show.devices.dmx.patch.fixtures.len(), 1);
        let universe = &show.devices.dmx.patch.universes[&universe_id];
        // Pre-existing fixture at 25 still occupied, batch fixtures gone
        assert!(!universe.is_slot_free(25));
        // Slots 1-24 should be free again (rollback)
        assert!(universe.is_slot_free(1));
    }

    #[test]
    fn batch_patch_of_a_huge_count_is_refused_before_allocating() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(24);
        for count in [u32::MAX, u32::MAX / 24 + 1, 22] {
            let result = PatchEngine::batch_patch(&mut show, count, guid, 0, 0, universe_id, 1);
            assert!(
                matches!(result, Err(PatchError::ExceedsBoundary { .. })),
                "{count}: {result:?}"
            );
        }
        assert!(show.devices.dmx.patch.fixtures.is_empty());
        // 21 × 24 = 504 fits.
        let ids = PatchEngine::batch_patch(&mut show, 21, guid, 0, 0, universe_id, 1).unwrap();
        assert_eq!(ids.len(), 21);
    }

    /// R1-52 (Q20): a changed footprint is taken where the slots are free;
    /// a fixture whose new slots are taken keeps its count (listed).
    #[test]
    fn a_changed_footprint_is_followed_where_free() {
        let (mut show, guid, universe_id) = make_show_with_fixture_type(4);
        let a = PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 1, None).unwrap();
        let b = PatchEngine::patch_fixture(&mut show, guid, 0, 0, universe_id, 9, None).unwrap();
        // The type's mode grows to 6: fixture a (1–6) is free, b (9–14) too.
        show.devices.dmx.patch.fixture_types.values_mut().map(std::sync::Arc::make_mut).next().unwrap().revisions[0].modes[0].dmx_footprint = 6;
        // A third fixture sits at 15: still free for b (9–14).
        let notes = crate::check::follow_footprints(&mut show);
        assert_eq!(notes.len(), 2, "{notes:?}");
        assert_eq!(show.devices.dmx.patch.fixture_by_fid(a).unwrap().1.mvr.breaks[0].channel_count, 6);
        // Grows to 10: a (1–10) would take b's 9 — a stays at 6, listed.
        show.devices.dmx.patch.fixture_types.values_mut().map(std::sync::Arc::make_mut).next().unwrap().revisions[0].modes[0].dmx_footprint = 10;
        crate::check::follow_footprints(&mut show);
        assert_eq!(show.devices.dmx.patch.fixture_by_fid(a).unwrap().1.mvr.breaks[0].channel_count, 6, "a's slots aren't free");
        assert_eq!(show.devices.dmx.patch.fixture_by_fid(b).unwrap().1.mvr.breaks[0].channel_count, 10, "b's are");
        assert!(crate::check::problems(&show).iter().any(|p| p.contains(&format!("Fixture {a}"))));
    }

    #[test]
    fn batch_patch_type_not_found_returns_error() {
        let (mut show, _, universe_id) = make_show_with_fixture_type(24);
        let unknown = Uuid::new_v4();
        let result = PatchEngine::batch_patch(&mut show, 2, unknown, 0, 0, universe_id, 1);
        assert!(matches!(
            result,
            Err(PatchError::FixtureTypeNotFound { .. })
        ));
        assert!(show.devices.dmx.patch.fixtures.is_empty()); // nothing patched
    }
}
