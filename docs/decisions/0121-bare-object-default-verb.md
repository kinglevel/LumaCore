---
id: 0121
title: A bare object runs its declared default verb
area: commands
status: built
date: 2026-10-15
built_in: 3b1
supersedes: [archive §33 L7924–7942 "old catch-all Call", archive §33 L7291–7299 "Show action options"]
superseded_by: —
confirmed_by: [`ObjectDef::default_verb`, `ObjectDef::bare_verb`, the generated object reference]
source: [archive §33 L7008–7012, archive §33 L7189–7194, archive §33 L7240–7271, archive §33 L7291–7299, archive §33 L7505–7509, archive §33 L7924–7942]
---
# A bare object runs its declared default verb

## Context
Every object once carried its own `Call` handler parsing a bare line its own way: a bare `Universe 3` created a universe, `Show /Save` saved, `Clock` printed. Research over each object's `Call` gave one table of default verbs; a per-object catch-all `Call` was rejected and `Call` became a real verb (bring a stored thing back here: `Call WindowPreset 2`).

## Decision
Every `ObjectDef` declares one registered `default_verb`. A bare object runs `ObjectDef::bare_verb`: the At verb when the line has `At` and the object takes it (see at-verb-on-target); a named owner-first child's own bare verb where the owner passes it on (`DataPool 2 Group 3`); else the default verb, run by the object's handler or by the verb's own command. So `Group 3` = `SelectFixtures Group 3`, `Sequence 3` = `Select Sequence 3`, `View 3` = `Call View 3`, `Menu "Network"` = `Toggle Menu "Network"`, `Universe 3` = `Info Universe 3`; a one-of-a-kind settings object alone runs `EditSetting` (`Theme`), a mode alone its default verb (`Blind` = `Toggle Blind`). A counted keyword alone does not act — it sets the default object (see default-object-per-user). A bare line never creates (`Store Universe 3`) and never replaces the show (`LoadShow "demo" /Version "…"`); `Show`'s `/Save`, `/Load`, `/Autosave`, … are gone (`SaveShow`, `LoadShow`, `Set Show "Autosave" 5`).

## Consequences
A new object names its default verb in its row and gets no `Call` handler; changing a default is a lum.md hard discussion. `print` writes the verb out; the generated object reference lists each default verb. The one leftover: `Watch` keeps `Call` as its default until Watch becomes a verb (see watch-is-a-verb).
