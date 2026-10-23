---
id: 0226
title: Shared code has one door each; copies are ratchets
area: structure
status: decided
date: 2026-10-15
built_in: —
supersedes: []
superseded_by: —
confirmed_by: [shared_doors_have_no_copies, old_resolvers_only_go_down]
source: [archive §34 L8011–8020 (items 12–13)]
---
# Shared code has one door each; copies are ratchets

## Context
The research round found thirteen families of near-duplicate code: the next free number, not-found and exists replies, the DataPool scope written as a hand-made save / restore that was neither `?`- nor panic-safe, the label / name / delete asks, on / off words, `unquote`, colour and percent text, plain Store / Delete / Label / Move / List steps, list reordering. Each had grown beside the last copy.

## Decision
Every shared thing has one door, named in the table of code-structure.md §5 beside `drive` and `resolve`: `pools::next_free` (the first unused number everywhere, see one-object-door), `errors::{not_found, exists, usage}`, `AppState::in_datapool`, `scope::ScopedCell`, `dialogs::{label_ask, name_ask, delete_ask}`, `props::on_off` (the four words, no `true` / `1`), `tokens::unquote`, the `Collection` trait for plain CRUD, `ObjectDef::BASE` and its builders, `Property::keyed` / `named`, `TreeDef::btree`, `Result<Reply, Coded>`. Everything else calls the door. Tidy bans the copy patterns as ratchets while copies remain: `shared_doors_have_no_copies` counts each door's copies (the `COPIES` table), `old_resolvers_only_go_down` the lookups older than the one resolver.

## Consequences
A new shared helper is added to the table with its door, never beside a copy; a copy found in review is replaced with the door and the count lowered. Two categories sharing a helper get a named door; one category keeps it in its folder. Open: the copies the ratchets still count, taken as their areas move.
