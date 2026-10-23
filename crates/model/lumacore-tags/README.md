# lumacore-tags

The tag tree: tags with child tags, their paths and moves, and the member index.

## Purpose

A tag is a node in one tree (decisions §12 "Tags hardened"): it has exactly one parent and is numbered within it, so its number is its path (`Tag 1.2.1` = All › Lx1 › Right; never a decimal, `1.10` ≠ `1.1`). Every show object carries tags as GUID links in its `NodeMeta.tags`; a move renumbers a tag at its new place and nothing that carries it notices. The tree lives in `ShowData.tags` (`lumacore-tree`); what carries a tag is read from the show's identity walk (links of kind `Tag`). The words — Store, Label, Delete, Move, Assign / Unassign, List, Info — are `lumacore-cmd-tags`'.

## Owns / Doesn't own

- **Owns:** `Tag`, `TagAction`, `TagPath`, `TagTree`, `Member`, the tree operations (find, names among siblings, Thru on one parent, insert, remove, move, lift children), `members`, the E-TAG catalog.
- **Doesn't own:** where the tree is stored (`ShowData.tags`), the words and their asks (`lumacore-cmd-tags`), the resolver that expands a tag target (`lumacore-objects`).

## Public surface

- `register` — the E-TAG error catalog.
- `tag::{find, find_mut, level, all, path_of, named, named_one, subtree, siblings, next_free, insert, remove, move_under, lift_children, members}`.
- `Tag`, `TagAction`, `TagPath`, `TagTree`, `Member`, `TagError`, `ERRORS`.

## Rules

- A tag has one parent and never goes under itself. [test a_move_keeps_the_guid_and_takes_the_next_free_number]
- Names are unique among siblings; a number is used once per parent. [test names_are_unique_among_siblings_and_numbers_once]
- A tag's number is a path, never a decimal. [test a_path_is_parts_never_a_decimal]
- Thru walks the children of one parent. [test thru_walks_the_children_of_one_parent]

## Edges

- **Depends on:** lumacore-base, lumacore-registry.
- **Used by:** lumacore-tree, lumacore-cmd-tags.

## Decisions

- [§12 "Tags hardened"](../../../docs/archive/decisions-2026-10.md#12-tags--visual-browser-future) — the tag tree, links, verbs
- [§21 "6d"](../../../docs/archive/decisions-2026-10.md#21-undo--replication-coverage-2026-10-10) — every stored link is a GUID

## Testing

- `cargo test -p lumacore-tags` — the tree's operations and the member index.
