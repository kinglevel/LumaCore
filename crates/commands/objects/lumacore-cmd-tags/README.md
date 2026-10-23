# lumacore-cmd-tags

The `Tag` object: the tag tree's words.

## Purpose

A tag is a node in one tree (decisions §12 "Tags hardened"): one parent, numbered within it, so its number is its path (`Tag 1.2.1`). Any show object carries tags as GUID links in its `NodeMeta.tags`. This crate holds the words:

- `Store Tag`: at a path, under an owner by name, or at the next free number.
- `Label Tag`: a new name, unique among the tag's siblings.
- `Delete Tag`: one question naming the child tags and what carries the tag; With children / Keep children (they move up one level). Objects lose only their links.
- `Move Tag … At Tag | Root | <n>`: nests a tag with its subtree, or renumbers it; the GUID stays.
- `Assign` / `Unassign Tag … At <objects>`: tags or untags any show object, a tag included.
- `List Tag`, `Info Tag [/R]`.
- The default verb: `Tag 1.1 [/R]` selects the tag's fixtures (`/R` adds the tags under it).
- A tag's own properties: `Action` (Default | Select, what Run action does in the Tags pool window) and its Meta (`Name`, `Note`, `Paint`), keyed by its GUID so a move never changes the row (`Set Tag 1.1 "Action" Select`).
- EditSetting's Tags tab for every show object (`settings`: a `SettingsTab` row, the tag tree as `FieldInput::Tags`).

The tree and its operations are `lumacore-tags`'. The one resolver walks the tree through this crate's `PathDef` row (`IdReading::Path`), so the kernel never names tags.

## Owns / Doesn't own

- **Owns:** `TAG`, `PATHS`, `TABS`, each verb's file, the Tag's properties (`props`), the tag links' edits (`links`).
- **Doesn't own:** the tree (`lumacore-tags`), the resolver and the selection rule (`lumacore-objects`), where the tree is stored (`ShowData.tags`, `lumacore-tree`).

## Public surface

- `register`: the Tag object and its path row, after the kernel's.
- `TAG`, `PATHS`, `TABS`, `OBJECTS`.

## Rules

- A tag reaches its own objects; `/R` adds those of every tag under it. [test assign_tags_fixtures_and_a_tag_selects_them]
- A move keeps the tag's GUID, so what carries it never notices. [test move_nests_with_the_guid_kept]
- Delete asks once for the child tags and never deletes an object. [test delete_asks_for_children_and_keeps_the_objects]
- Thru walks the children of one parent. [test thru_walks_one_parents_children]
- A tag's settings are its own row, a nested tag too; a name a sibling has is refused. [test a_tag_has_its_own_settings]

## Edges

- **Depends on:** lumacore-registry, lumacore-syntax, lumacore-syntax-derive, lumacore-wire, lumacore-tags, lumacore-tree, lumacore-objects.
- **Used by:** lumacore-terminal, lumacore-sweep, lumacore-tidy.

## Decisions

- [§12 "Tags hardened"](../../../../docs/archive/decisions-2026-10.md#12-tags--visual-browser-future): the tag tree, links, verbs, `/R`
- [§34 21](../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15): rows in the catalog

## Testing

- `cargo test -p lumacore-cmd-tags`: the words against the demo show.
