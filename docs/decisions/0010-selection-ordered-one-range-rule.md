---
id: 0010
title: Selection is ordered; the one range rule
area: commands
status: built
date: 2026-10-11
built_in: R1-34 / Q6
supersedes: []
superseded_by: —
confirmed_by: [parser::number_list]
source: [archive §2 L194–212]
---
# Selection is ordered; the one range rule

## Context
Established consoles keep selection order, and a console with gaps in its numbers (the rig has fixtures 1–9999 with gaps) must not fail a range because one number is missing. The R1 review (R1-34, Q6) made one range rule for every object word after the words had each parsed ranges their own way.

## Decision
The fixture selection is an ordered list, never a set: `Fixture 1 Thru 6` is 1…6, `Fixture 6 Thru 1` is 6…1; selection expressions keep the typed order; `Shuffle` randomizes it, `Next` / `Previous` walk it, and a group recalls its stored order. One range rule for every object word, `parser::number_list`: `3`, `1 Thru 4`, `1 Thru` (to the last), `Thru 4` (from the first), a bare `Thru` is every number, joined by `+` and taken out by `-` (`Thru - 3`); numbers that do not exist are skipped — the list closes up, no holes — and only none at all is an error; only existing numbers are visited, so an absurdly large upper bound costs nothing. Every verb uses it: select, Delete, Copy, Move, Assign, On / Off, row deletes. Copy, Move and Assign keep the sources' spacing (a gap stays a gap, counted from the first in typed order), and a Copy or Move checks every target first — it never leaves half a copy. A pool's number cap (`pools::MAX_NO` for Groups) applies only to new numbers; an older show's object above it is named, labelled, moved down and deleted like any other.

## Consequences
Never add a range parser, a sorted selection or a hole-preserving copy; a word that takes numbers goes through `parser::number_list` and the one resolver (see the resolver record). Which selections add and which replace is groups-in-datapool-selection-rule.
