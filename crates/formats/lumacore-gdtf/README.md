# lumacore-gdtf

The GDTF reader: a `.gdtf` file becomes a `FixtureType`, deterministically, with the spec's
defaults where a planning tool's export left fields out.

## Purpose

GDTF is native (decisions §29 "11b hardening"): the `.gdtf` is the source of truth and travels
in the show, and the `FixtureType` the station works with is a view derived from it here — the
same every time, so a show re-derives every revision from its bytes on load. Import is warn
only: anything readable is imported and spec problems become `GdtfWarning`s the user reads with
`Info FixtureType`. The XML itself is read by the vendored `gdtf` fork (`vendor/gdtf`), patched
with the spec's defaults and lenient value readers; this crate resolves virtual channels against
the physical ones, turns geometry references into cells at their break offsets, hashes files,
reads a file's own facts for the Insert fixtures dialog and keeps the flat library index.

## Owns / Doesn't own

- **Owns:** `import_gdtf`, `import_gdtf_bytes`, `refresh_revision`, `file_hash` (`importer`);
  `VirtualChannelResolver` and `ResolutionResult` (`resolver`); `ChannelMap` (`channel_map`);
  `validate_din_spec` (`validator`); `FileDetails`, `file_details`, `thumbnail` (`details`);
  `load_index` / `save_index` (`index`); `GdtfError`, `GdtfWarning` and the `E-GDTF` catalog.
- **Doesn't own:** the `FixtureType`, `Revision` and attribute types (`lumacore-fixtures`), the
  XML reader (`vendor/gdtf`), where a file lives and its SHA-256 store (`lumacore-luma`,
  `lumacore-drives`), the `Import FixtureType` word and its dialog (`lumacore-objects`).

## Public surface

- `import_gdtf(path)` / `import_gdtf_bytes(bytes)` → `(FixtureType, Vec<GdtfWarning>)`.
- `refresh_revision(&mut Revision, bytes)` — re-derive a revision, GUIDs kept.
- `file_hash(bytes)` — the SHA-256 a show and the store name a file by.
- `file_details(bytes)`, `thumbnail(bytes)` — `description.xml`'s facts and the PNG beside it.
- `ChannelMap::by_dmx_offset` / `grouped_by_type`, `validate_din_spec`, `load_index`, `save_index`.
- `register(&mut Registry)` — the error catalog.

## Rules

- A revision re-derives from its bytes with its GUIDs kept. [test refreshing_a_revision_keeps_its_guids]
- Import is warn only: a stripped planning export reads with the spec's defaults, a NaN wheel
  colour as white. [test a_stripped_capture_export_reads_with_the_specs_defaults + test a_nan_wheel_colour_reads_as_white_not_a_refusal]
- A broken zip or XML is refused with a code, never half-imported.
  [test import_xml_broken_gdtf_returns_error + test a_broken_file_is_refused]
- Virtual channels resolve against the physical channels of their mode.
  [test import_virtual_channel_gdtf_resolves_virtual_channels]
- Geometry references become cells at their break offset; `Highlight` snaps and emitters are kept.
  [test geometry_references_become_cells_at_their_break_offset + test highlight_snap_sets_wheels_and_emitters_are_kept]
- Its codes are in the one catalog. [test catalog_is_valid + tidy crates_on_the_registry_register_and_install]

## Edges

- **Depends on:** lumacore-fixtures, lumacore-registry, lumacore-base.
- **Used by:** lumacore-luma, lumacore-state, lumacore-objects.

## Decisions

- `docs/decisions/0091-gdtf-native.md` — the `.gdtf` is the source; `FixtureType` is derived
- `docs/decisions/0091-gdtf-native.md` — the vendored fork with the spec's defaults
- `docs/decisions/0092-gdtf-import-revisions-and-file-index.md` — revisions by SHA-256, one file index
- `docs/decisions/0277-cells-on-fixture-type.md` — cells per mode, no sub-fixtures
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §29, §34, §40; the fork's changes in [vendor/README.md](../../../vendor/README.md)

## Testing

- `cargo test -p lumacore-gdtf` — the importer (`src/importer.rs`), the resolver, the channel
  map, the validator, the index, `src/details/tests.rs`.
- The rig's own types: `cargo test -p lumacore-catalog-tests` loads the demo show through this
  crate.
