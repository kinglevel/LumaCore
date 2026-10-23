# lumacore-patch

The patch engine: fixture-to-universe addressing on `ShowData`, conflict-aware, with the
per-universe address map and the consistency check a loaded show gets.

## Purpose

Stateless operations on the patch (decisions §11, §40): `PatchEngine` works on `&mut ShowData`
and carries nothing of its own — it patches a fixture of a type at an address, in batches too,
unpatches it and lists what is patched, refusing a conflict with the occupant named, an address
past the universe's end, or a type or universe that isn't there. `UniverseAddressMap` is the
occupancy of one universe's slots by fixture GUID, the piece the engine and the DMX views agree
on. `check` is the patch consistency check a loaded, synced, merged or restored show gets: like
the reference check it only lists what is wrong — load still succeeds and nothing is dropped or
moved. The one decided load-time rewrite, a changed footprint followed where slots are free,
lives here.

## Owns / Doesn't own

- **Owns:** `PatchEngine` and `FixtureListEntry` (`patch`), `UniverseAddressMap`
  (`address_map`), the consistency check (`check`), `PatchError` and the `E-PATCH` catalog
  (`errors`).
- **Doesn't own:** the fixture, its three ids and the slot-keyed patch map
  (`lumacore-fixtures`, `PatchData`), the universes and the slot map of overlaps and parked runs
  (`lumacore_tree::dmx_slots`), the words `Store` / `Assign` / `Unassign` / `DMX` and the
  staged draft (`lumacore-objects`, `lumacore-state`), the fixture type (`lumacore-gdtf`).

## Public surface

- `PatchEngine::patch_fixture`, `batch_patch`, `unpatch_fixture`, `list_fixtures` — on
  `&mut ShowData`, returning `PatchError`s.
- `UniverseAddressMap::try_occupy`, `release`, `is_slot_free`, `occupied_count`,
  `first_free_block` — one universe's slots.
- `check::problems` — what a show's patch gets wrong; `check::follow_footprints` — the one
  rewrite at load.
- `register(&mut Registry)` — the error catalog.

## Rules

- A conflict is refused naming the occupant; an address past the universe's end is a boundary
  error, never a wrap. [test occupy_conflict_returns_error_with_occupant_guid + test patch_fixture_boundary_exceeded_returns_error]
- A batch rolls back on the first conflict, and a huge count is refused before anything is
  allocated. [test batch_patch_rollback_on_conflict + test batch_patch_of_a_huge_count_is_refused_before_allocating]
- A changed footprint is followed where the slots are free. [test a_changed_footprint_is_followed_where_free]
- Unpatching frees the slots and returns the id to its pool.
  [test unpatch_fixture_removes_fixture_and_frees_slots + test unpatch_fixture_id_returned_to_pool]
- The check lists; it never drops or moves a fixture. [unchecked]
- Its codes are in the one catalog. [test catalog_is_valid + tidy crates_on_the_registry_register_and_install]

## Edges

- **Depends on:** lumacore-fixtures, lumacore-registry, lumacore-tree.
- **Used by:** lumacore-state, lumacore-objects, lumacore-cmd-station, catalog.

## Decisions

- `docs/decisions/0271-patch-slot-key-and-three-ids.md` — the patch keyed by a slot; FID, IDType, CID
- `docs/decisions/0269-patch-words-store-assign-at-dmx.md` — Store creates, Assign addresses
- `docs/decisions/0262-staged-patch-draft.md`, `docs/decisions/0262-staged-patch-draft.md` — the draft
  the engine's writes land in
- `docs/decisions/0082-show-format-forward-only.md` — a changed footprint followed where free, the only
  rewrite at load
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §11, §20, §34, §40

## Testing

- `cargo test -p lumacore-patch` — `src/patch.rs` (the engine, batches, the footprint),
  `src/address_map.rs` (occupancy), `src/errors.rs` (the catalog).
- On the rig: `cargo test -p lumacore-objects` (its `commands/patch_draft_tests.rs`) and the demo
  golden in `cargo test -p lumacore-catalog-tests`.
