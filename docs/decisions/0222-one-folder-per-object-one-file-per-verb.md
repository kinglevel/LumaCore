---
id: 0222
title: One folder per object, one file per verb
area: structure
status: built
date: 2026-10-15
built_in: 34c; each area as it moves
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §34 L7999–8002 (item 8)]
---
# One folder per object, one file per verb

## Context
Before the structure round an object's verbs, its registry rows, its views and its tests sat in one long file per area, and thirteen families of near-duplicate code hid in them. The layout was taken from nushell's commands and rust-analyzer's assist handlers: a small registry file and one handler file each.

## Decision
An object lives in one folder of its command crate: `objects/<object>.rs` holds the REGISTRY Part (the `ObjectDef`, its `Props`), and `objects/<object>/{store,delete,copy,…}.rs` hold one typed command each with its logic; `views.rs` holds its tree children and Info text, `tests.rs` its tests (see tests-in-foo-tests-rs). A file is named after the one thing it holds, never `misc.rs` or a crate-root `utils.rs`; helpers only that object uses go in its folder, helpers two categories share get a named door (see shared-code-one-door). The registries stay explicit lists: the family's `OBJECTS` and `ROWS` name every object and verb, no inventory or linkme.

## Consequences
A new verb is a new file in its object's folder plus a row in the family's list; a new object is a new folder. A file that needs both COMMANDS and IO, or a Part that grows long, is a sign to split (see file-size-limits). The shape is held through the header rules and the size ratchet rather than a folder rule of its own; review holds the one-verb-per-file part. Old areas take the shape when they are next touched.
