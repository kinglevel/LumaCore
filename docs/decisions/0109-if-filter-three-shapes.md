---
id: 0109
title: If and EndIf in three shapes, no comparisons; planned
area: syntax
status: decided
date: 2026-10-14
built_in: —
supersedes: [archive §33 L7169–7170 "half-built comparison filter"]
superseded_by: —
confirmed_by: [E-CMD-003, test `planned_words_reply_planned_and_change_nothing`]
source: [archive §33 L7161–7170 (item 23), archive §21 L3266–3271]
---
# If and EndIf in three shapes, no comparisons; planned

## Context
The selection parser carried a half-built `> < = ==` filter nobody had decided. The established console `If` has no comparisons and three positions; §21 had already published `If` as a planned word (new, so the PRG `I` shortcut stays).

## Decision
`If` has exactly three shapes: (1) a verb at the start — `If Group 5` deselects what is not in group 5, `Group 3 If Group 5` keeps the overlap; (2) a filter clause at the end of a statement — `Delete Cue 3 If Fixture 4`, `Delete Cue 3 If Fixture 4 Attribute "Pan"`, Clone's scope `Clone Fixture 1 At Fixture 2 If Sequence 1`; (3) `EndIf` for an If in the middle — `Store If Group 5 EndIf Preset 1.1` runs as `Store Preset 1.1 If Group 5`. `print` writes the If at the end: one spelling. lum.md's grammar carries `if-early` and `if`; the If section's examples are the forms. No comparisons: the old filter is gone.

Until built, `If` is a Planned word: running it replies "isn't built yet (planned)" (E-CMD-003) and changes nothing; help, hints, completion and the reference show it.

## Consequences
Not built. Building it is a registry change (the verb's handler and the `If` slot on signatures that take a filter), not a grammar change; a comparison operator would be a lum.md hard discussion (see lum-grammar-one-parser). Never a stub that acknowledges.
