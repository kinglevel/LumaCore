# lumacore-mvr

The MVR file (GDTF's My Virtual Rig, 1.6) as a format: the zip's `GeneralSceneDescription.xml`
read into data — layers, nested groups, every fixture with its place, matrix, GDTF spec and mode,
addresses and ids, the classes and positions — and the few facts a file list shows.

## Purpose

A rig drawn elsewhere arrives as an `.mvr`: a zip holding the scene XML and the GDTF files it
names. This crate reads that file and nothing more; what the scene means for the show — the match
against the draft, the review, Apply — is the objects crate's (34g4 part 4), and the file list's
columns (`files` feed, kind `mvr`) read `head`.

## Owns / Doesn't own

- **Owns:** the zip walk (`read`: the entries, the scene entry at the root in any case), the
  element tree (`xml`), the scene as data (`scene`: the tree flattened with parents and layers,
  the address rule — absolute `513` = 2.001, or `universe.channel` —, the matrix text, the head
  with the GDTFs named, embedded or missing).
- **Doesn't own:** the show's MVR data (`lumacore-fixtures` `fixture_mvr`, `devices_meta`), the
  import and the review (`lumacore-objects`), the `.mvr` store, export and MVR-xchange (parts 5, 6).

## Public surface

- `read(bytes)`, `read_from(reader)` — the file: `MvrFile { scene, entries }`.
- `head(bytes)`, `head_from(reader)`, `MvrFile::head` — `MvrHead`: provider, layers, groups,
  fixtures, the GDTFs named / embedded / missing.
- `MvrScene`, `MvrLayer`, `MvrGroup`, `MvrFixture`, `MvrAddress`, `MvrNamed`, `scene::parse_matrix`.
- `MvrError` — not a zip, no scene, the XML doesn't parse, a thing without a uuid.

## Rules

- A big file is never read whole for its head: `read_from` reads only the scene entry. [tested: read]
- Every layer, group, fixture, class and position needs a valid uuid; the file is refused without. [tested: read]
- Other scene objects (trusses, screens…) are counted and kept out until `Root.ShowData.3DSpatial`. [tested: read]

## Edges

- **Depends on:** nothing.
- **Outside the workspace:** quick-xml, zip, uuid, thiserror.
- **Used by:** lumacore-objects (the `files` feed's heads, the import).

## Decisions

§34 "34g4 patch hardened" 10–12 and "Import MVR hardened" (2026-10-20); §38 5 (the file list).

## Testing

`cargo test -p lumacore-mvr` (an MVR built in memory: the tree, the addresses, the matrix, the
GDTFs, the refusals).
