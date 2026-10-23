---
id: 0270
title: Renumber is Move: /Skip, /By column, a previewing dialog
area: door
status: built
date: 2026-10-19
built_in: round 3 1b, 3c
supersedes: []
superseded_by: —
confirmed_by: [`<luma-grid>.setPreview`, `kit/value.js`]
source: [archive §34 L9953–9974, archive §34 L10122–10133, archive §34 L10603–10618]
---
# Renumber is Move: /Skip, /By column, a previewing dialog

## Context
Round 3's address tools asked for renumbering and swapping from the grid. A docked renumber panel and a `/Sorted` flag were drawn and declined; the door's rule that Move onto a taken number swaps (see one-object-door) stands, so renumbering needed only two flags and one dialog.

## Decision
Renumber is Move. `/Skip` writes consecutive numbers skipping taken ones (`Move Fixture 101 Thru 103 At 107 /Skip`; rows that aren't moving hold); Swap is the bare line — no `/Swap` on Move. Order is as typed (`Move Fixture 105 + 101 + 103 At 107`) or `/By <column>` (any declared column, the pool's declared orders; `Move Fixture 101 Thru 140 At 201 /By Address`); no `/Sorted`. Both flags sit on the Move row, so every pool's Move takes them. The value dialog (`kit/value.js`, `openValue`; `ContextItem::value` names the kind an item opens: Renumber… = `number`, Fixture's Patch to… = `address`) sits beside the selection (a bottom sheet on a phone): the field with − / +, one hint line, free values (`freeNumbers` / `freeAddresses`), the kind's toggles (Skip taken = `/Skip` by default, Swap; Selection order = the rows as picked, `+`-joined when not ascending, Grid order = as the list shows them; for addresses Bunch up by default or `/KeepSpacing`), Apply; every keystroke previews old → new in the grid (`<luma-grid>.setPreview`: both in the gutter, rows in the way red); nothing is sent until Apply. Universes renumber through the same Move (see universe-move-delete-ask).

## Consequences
Never a renumber verb or a per-pool renumber rule. Open: the free values come from the list's own rows, not a free-value index in the suggestion engine (roadmap); a pool's Renumber… has no preview yet.
