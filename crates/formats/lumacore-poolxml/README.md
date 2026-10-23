# lumacore-poolxml

A console's pool export as XML, as a format: the pool's groups with their items and grid cells,
read streaming so a 300 MB export costs nothing it doesn't ask for.

## Purpose

The desk exports one XML per DataPool: groups, presets, sequences, macros, layouts. This crate
reads the `Groups` element into data and counts the rest; what a group means for the show — one
LumaCore group per exported group, members by fixture id — is the objects crate's
(`Import PoolXml "file" Groups`, decisions §40 13). Presets, sequences, cues and macros follow
when their LumaCore objects exist.

## Owns / Doesn't own

- **Owns:** the streaming walk (`read`: root › Pool › Groups › Group › SelectionData › Item; the
  root's name is the desk's own), the pool as data (`pool`: name, version, groups in order, an
  item's id text and cell, `fid` for a whole fixture, `sub_fixtures_only`), the refusals (`error`).
- **Doesn't own:** the file's place on a drive (the caller's door), the show's groups
  (`lumacore-tree`), the import and its reply (`lumacore-objects`).

## Public surface

- `read(bytes)`, `read_from(reader)` — `PoolXml { data_version, name, groups, other }`.
- `PoolXmlGroup { name, guid, items }`, `PoolXmlItem { id_type, id, x, y, z }` with
  `PoolXmlItem::fid`.
- `PoolXmlError` — not an export (no root, no pool), XML that doesn't parse.

## Rules

- Streaming: nothing but the Groups element is kept; the other pool children are counted by
  name. [tested: read]
- A dotted id (`849.1`) is a sub-fixture: `fid` is None; a group of those alone says so. [tested: read]

## Edges

- **Depends on:** nothing.
- **Outside the workspace:** quick-xml, thiserror.
- **Used by:** lumacore-objects (`Import PoolXml`).

## Decisions

§40 13 (2026-10-20): the demo's groups come through a small pool-export reader, Groups first.

## Testing

`cargo test -p lumacore-poolxml` (an export in memory: the groups, the items, the counts, the refusals).
