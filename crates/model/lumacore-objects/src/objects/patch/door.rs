// ██████╗  ██████╗  ██████╗ ██████╗
// ██╔══██╗██╔═══██╗██╔═══██╗██╔══██╗
// ██║  ██║██║   ██║██║   ██║██████╔╝
// ██║  ██║██║   ██║██║   ██║██╔══██╗
// ██████╔╝╚██████╔╝╚██████╔╝██║  ██║
// ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═╝
// FILE: door

//! Fixture and Universe on the object door (decisions §35): Move (swap), Insert, Compact
//! (`/By Address` for fixtures), Cut / Paste. Their own Delete stays (it clears the patch's
//! occupancy), and there is no Copy yet (a copy would sit on the same DMX addresses).
//!
//! Owns:          FIXTURE_DOOR, UNIVERSE_DOOR, their door verbs.
//! Doesn't own:   the verbs (`commands::door`), the patch's own Store / Delete (`objects::patch`).
//! Entry points:  FIXTURE_DOOR, UNIVERSE_DOOR, fixture_door, universe_door.
//! Decisions:     §35, §11.

use crate::AppState;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

// The object door (decisions §35) for the patch: Move (swap), Insert, Compact, Cut / Paste. No
// Copy yet (a copy would sit on the same DMX addresses) and their own Delete (it clears the
// patch's occupancy) — the patch area's pass decides both.
crate::door_verbs!(fixture_door, "Fixture", crate::objects::FIXTURE);
crate::door_verbs!(universe_door, "Universe", crate::objects::UNIVERSE);

fn universes_map(state: &mut AppState, _: Option<u32>) -> Option<&mut std::collections::BTreeMap<u32, lumacore_fixtures::Universe>> {
    Some(&mut state.root.show_data.devices.dmx.patch.universes)
}

fn universes_ref(state: &AppState, _: Option<u32>) -> Option<&std::collections::BTreeMap<u32, lumacore_fixtures::Universe>> {
    Some(&state.root.show_data.devices.dmx.patch.universes)
}

/// The patch moved: its ID pools follow and the engine's channel tier is rebuilt.
fn patch_changed(state: &mut AppState) {
    state.root.show_data.rebuild_id_pools();
    state.patch_generation += 1;
}

/// `Compact Fixture /By Address`: the first break's universe number and DMX address.
fn fixture_address(state: &AppState, f: &lumacore_fixtures::Fixture) -> String {
    let patch = &state.root.show_data.devices.dmx.patch;
    f.mvr.breaks.first().map_or_else(
        || "99999.999".to_string(),
        |b| {
            let universe = patch.universes.iter().find(|(_, u)| u.meta.identity.guid == b.universe_id).map_or(99999, |(n, _)| *n);
            format!("{universe:05}.{:03}", b.start_address)
        },
    )
}

/// The fixture pool's door (decisions §40 5): its numbers are the fixtures' `fid`s, never the
/// slots — `Move Fixture 101 At 201` renumbers, the slot stays; a fixture without a number is
/// outside the door (it has no number to move).
pub struct FixturePool;

impl FixturePool {
    fn patch(state: &AppState) -> &lumacore_tree::PatchData {
        &state.root.show_data.devices.dmx.patch
    }
}

impl crate::pools::door::PoolOps for FixturePool {
    fn object(&self) -> &'static str {
        "Fixture"
    }

    fn max_no(&self) -> u32 {
        u32::MAX - 1
    }

    fn datapool(&self, _: &AppState, _: Option<u32>) -> Option<u32> {
        None
    }

    fn numbers(&self, state: &AppState, _: Option<u32>) -> Vec<u32> {
        Self::patch(state).fids().into_iter().map(|(n, _)| n).collect()
    }

    fn guid(&self, state: &AppState, _: Option<u32>, no: u32) -> Option<uuid::Uuid> {
        Self::patch(state).fixture(no).map(|f| f.meta.identity.guid)
    }

    fn name(&self, state: &AppState, _: Option<u32>, no: u32) -> Option<String> {
        Self::patch(state).fixture(no).and_then(|f| f.meta.identity.name.clone())
    }

    fn read(&self, state: &AppState, _: Option<u32>, no: u32) -> Option<crate::pools::door::Item> {
        Self::patch(state).fixture(no).map(|f| Box::new(f.clone()) as crate::pools::door::Item)
    }

    fn take(&self, state: &mut AppState, _: Option<u32>, no: u32) -> Option<crate::pools::door::Item> {
        state.root.show_data.devices.dmx.patch.remove_fid(no).map(|f| Box::new(f) as crate::pools::door::Item)
    }

    fn put(&self, state: &mut AppState, _: Option<u32>, no: u32, item: crate::pools::door::Item) -> Result<(), String> {
        let f = *item.downcast::<lumacore_fixtures::Fixture>().map_err(|_| "Not a Fixture.".to_string())?;
        state.root.show_data.devices.dmx.patch.insert_fid(no, f);
        Ok(())
    }

    fn renewed(&self, item: &crate::pools::door::Item) -> crate::pools::door::Item {
        let f = item.downcast_ref::<lumacore_fixtures::Fixture>().expect("a fixture");
        Box::new(crate::pools::door::renewed(f))
    }

    fn overwrite(&self, state: &mut AppState, _: Option<u32>, no: u32, from: &crate::pools::door::Item) -> Result<(), String> {
        let from = from.downcast_ref::<lumacore_fixtures::Fixture>().ok_or_else(|| "Not a Fixture.".to_string())?;
        let mut new = crate::pools::door::renewed(from);
        let old = state.root.show_data.devices.dmx.patch.fixture_mut(no).ok_or_else(|| format!("Fixture {no} doesn't exist."))?;
        new.meta.identity.guid = old.meta.identity.guid;
        new.mvr.fid = Some(no);
        *old = new;
        Ok(())
    }

    fn can_merge(&self) -> bool {
        false
    }

    fn merge(&self, _: &mut AppState, _: Option<u32>, _: u32, _: &crate::pools::door::Item) -> Result<String, String> {
        Err("A Fixture can't merge.".to_string())
    }

    fn can_delete(&self, _: &AppState, _: Option<u32>, _: u32) -> Result<(), String> {
        Ok(())
    }

    fn deleted(&self, _: &mut AppState, _: &crate::pools::door::Item) {}

    fn changed(&self, state: &mut AppState) {
        patch_changed(state);
    }

    fn orders(&self) -> Vec<&'static str> {
        vec!["Number", "Name", "Address"]
    }

    fn order_key(&self, state: &AppState, _: Option<u32>, no: u32, by: &str) -> Option<String> {
        let f = Self::patch(state).fixture(no)?;
        if by.eq_ignore_ascii_case("Name") {
            return Some(f.meta.identity.name.clone().unwrap_or_default().to_lowercase());
        }
        by.eq_ignore_ascii_case("Address").then(|| fixture_address(state, f))
    }
}

pub static FIXTURE_DOOR: FixturePool = FixturePool;

pub static UNIVERSE_DOOR: crate::pools::door::BTreePool<lumacore_fixtures::Universe> = crate::pools::door::BTreePool {
    object: "Universe",
    in_datapool: false,

    max: u32::MAX - 1,
    map: universes_map,
    map_ref: universes_ref,
    name: |u| u.meta.identity.name.clone(),
    identify: |u, guid| u.meta.identity.guid = guid,
    rules: crate::pools::door::PoolRules {
        can_delete: |_, _| Ok(()),
        deleted: |_, _| {},
        changed: patch_changed,
        merge: None,
        orders: &[],
    },
};
