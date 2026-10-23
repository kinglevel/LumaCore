# lumacore-tree

The object tree: `Root` with `ShowData`, `StationSettings` and the runtime, every show type
with its identity, and the stateless operations on universes, lines and merges.

## Purpose

One tree holds everything a station knows (decisions §1, §10): `ShowData` is the show file,
replicated and changed only by the master; `Root.Runtime` is session-wide runtime state;
`StationSettings` is this machine's. Every object under
`ShowData` has a `NodeMeta`, is `Identified` only through it and `Walk`s its fields exhaustively,
so a new field doesn't compile until it is classified. It also holds
what the tree's shape decides: `after_load` and `identity`, the 3-way merge by GUID, the DMX
slot map, the attribute resolver, window types, UI keys and profiles. Playback, effects,
timecode, tags and fixtures are crates below it, re-exported here.

## Owns / Doesn't own

- **Owns:** `Root`, `ShowData`, `ShowSettings`, `FORMAT_VERSION`, `StationSettings`,
  `DevicesData`, `ProtocolsData` (`tree`); `Datapool`, `Group`, `Preset` and the other pool
  types (`datapool`); `User`, `UserProfile`, `Programmer`, `UserRights` (`programmer`);
  `ShowData::identity`; `merge`; `SlotMap` (`dmx_slots`); `UniverseOps`, `LineOps`; the
  attribute resolver and its data; `window_types`; `ui_keys`; `Usage`; `MediaFile`, `Theme`.
- **Doesn't own:** `AppState` and the change path (`lumacore-state`), the words that write the
  tree (`lumacore-objects`), the show file on disk (`lumacore-luma`), fixtures
  (`lumacore-fixtures`), the protocol contract (`lumacore-protocols`), any lookup from a line
  (`lumacore_objects::resolve`).

## Public surface

- `Root::new`, `ShowData::after_load`, `identity`, `programmer_of`, `sequence_by_guid`.
- `FORMAT_VERSION`, `ShowSettings`, `BuildInfo` — the forward-only stamp.
- `merge`, `MergeChoice` — the rejoin merge.
- `LineOps`, `UniverseOps`, `SlotMap` — lines, universes, who sits on which slot.
- `window_types`, `ui_keys`, `attribute_resolver`, `Usage`; the re-exports `runtime`, `tags`.
- `register(&mut Registry)` — window types, built-in keys, errors.

## Rules

- Rights and profiles come only from users; a profile copy gets new GUIDs and keeps its links.
  [test rights_and_profiles_come_only_from_users + test a_profile_copy_gets_new_guids_and_keeps_its_links]
- Merge: the master keeps conflicts, an edit beats a delete, a patch changed on both sides blocks.
  [test merge_conflicts_keep_the_masters_and_an_edit_beats_a_delete + test merge_is_blocked_by_patch_on_both_sides_or_another_show]
- A line carries one run of universes; two outputs never send one protocol universe to one
  destination. [test a_line_carries_one_run_of_universes + test two_output_lines_never_send_one_protocol_universe_to_one_destination]
- An overlap parks both sides and counts a shared slot once.
  [test an_overlap_parks_both_and_counts_a_shared_slot_once]
- Saved windows of an unknown type are dropped at load; core keys win over profile keys.
  [test saved_windows_of_an_unknown_type_are_dropped + test core_wins_over_plugin_and_profile_keys]
- A new `ShowData` field doesn't compile until it has a `Walk` arm and a `ShowSubtree`. [compiler]
- Code names a `Node`, never a path string. [tidy tree_paths_are_typed + test tree_paths_are_unchanged]

## Edges

- **Depends on:** lumacore-fixtures, lumacore-protocols, lumacore-registry, lumacore-base,
  lumacore-playback, lumacore-effects, lumacore-timecode, lumacore-runtime, lumacore-tags.
- **Used by:** lumacore-state, lumacore-patch, lumacore-luma, lumacore-render, lumacore-http,
  lumacore-objects and everything above.

## Decisions

- `docs/decisions/0005-state-homes-by-type.md` — the three homes
- `docs/decisions/0006-node-meta-levels.md`, `docs/decisions/0007-guids-everywhere-links.md` — identity
- `docs/decisions/0023-merge-on-rejoin.md` — the 3-way merge
- `docs/decisions/0082-show-format-forward-only.md` — `FORMAT_VERSION`
- `docs/decisions/0042-lines-one-type-runs.md`, `docs/decisions/0002-programmer-owned-by-user.md`,
  `docs/decisions/0202-window-types-rust-json.md`, `docs/decisions/0151-attribute-resolver-data-driven.md`
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §1, §10, §11, §17, §20, §21, §30, §34

## Testing

- `cargo test -p lumacore-tree` — `tree.rs`, `merge.rs`, `output_ops.rs`, `ui_keys.rs`,
  `attribute_resolver.rs`, `src/dmx_slots/tests.rs` (`lumacore-sacn` is a dev-dependency for a
  protocol row).
- The paths' golden and the demo show: `cargo test -p lumacore-catalog-tests`.
