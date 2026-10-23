---
id: 0135
title: Insert shifts, Compact renumbers, quoted star patterns, no Find
area: door
status: built
date: 2026-10-15
built_in: 35b, 35e
supersedes: []
superseded_by: —
confirmed_by: [`resolve::name_fits`, `resolve::check_name`, E-CMD-008]
source: [archive §35 L11001–11017, archive §35 L11041–11047]
---
# Insert shifts, Compact renumbers, quoted star patterns, no Find

## Context
With Move a swap (see one-object-door) move-with-shift and a pool renumber needed words of their own, and a quoted `*` wildcard was wanted. Declined: `Sort` doing both contents and pool, `Move Group /Compact`, `Shift` (a value: `Set MeowKey Shift`) and `Gap`.

## Decision
`Insert Group 5 At 9` is move with shift: 5 lands on 9 and the old 9, 10, … move up to the first free number, GUIDs kept, one Oops step; a range places the block; between owners of the same kind like Move; a bare `Insert` keeps the keypad ask. `Compact Group` renumbers the whole pool, holes closed, order kept; `Compact DataPool 2 Group` an owner's; `Compact Group 10 Thru 50` those rows from 10; `/By Name`, `/By Number`, `/By Address` give an order. `Sort` keeps its one meaning, an object's contents (see assign-places-objects-actions-are-verbs). A quoted id may hold `*` anywhere: every row whose name fits, ignoring case, through the one resolver (`resolve::name_fits`: `Group "Mac*"`, `Delete Group "Front*"`, `List Group "Front*"`); a value is never a pattern (`Label Group 3 "A*"`); names may not contain `*` (`resolve::check_name` refuses it in Label, `Set … "Name"` and Store, E-CMD-008); a pattern that fits nothing is not found — never a new name; there is no bare `*` id (`Group Thru` is every group). There is no Find verb: patterns in every target do it.

## Consequences
A Store's quoted id with `*` takes the one row it fits; one that fits several is refused for now — storing into each (`Store Cue "BO*" /Merge`) waits until the Store handlers move onto the door. A new pool operation gets its own word in `commands/door`, never a flag on Move or Sort.
