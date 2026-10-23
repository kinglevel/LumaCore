---
id: 0288
title: One preset pool that follows the programmer
area: ui-kit
status: built
date: 2026-10-23
built_in: the preset pool (2026-10-23)
supersedes: [visions/presets.md "feature group pools", record 0105 Preset Parts(2)]
superseded_by: —
confirmed_by: [objects::preset tests, a_preset_carries_the_tag_assigned_at_it, kit-presets.spec.ts]
source: [session 2026-10-23, preset pool round 1 — docs/dev/ui-kit/mockups/2026-10-23-presets/01-presets.html, approved]
---
# One preset pool that follows the programmer

## Context
LumaCore has preset data but builds only `List Preset`. A reference console keeps a pool and a window per preset type (the user's desk: 25 pools; a Dynamic window follows the feature group). Presets already stay open: a store keeps every active value (record 0150). The old docs disagreed on `Preset 4.1` against `Preset Color 1` and on Delete. Per-type pools, a window type per pool, exact-group filtering, storing only the current group and a store dialog were not taken.

## Decision
Presets live in one numbered pool per DataPool, `Preset <n>`. A preset knows the feature groups its values touch and carries tags. One Presets window type shows that pool. It follows the feature group selected on the encoder bar (a window setting pins one group or All); it shows every preset that touches the group, packed by default (Fixed slots as a setting), narrowed by the tag chips on top. A cell shows how it fits the selection (all / some / none), a live preview drawn from its values, a square per feature group it holds, the fixtures using it, and its mode letter. The tap runs the window's action (At by default; SelFix/At, SelFix, Call, None). Store keeps every active programmer value; Store + a tap on an empty cell stores there, Store alone takes the next free number, and the lit tags are assigned. Modes are Selective or Universal, chosen automatically and shown as a letter. Hold opens the one context menu; Edit opens the preset's editor.

## Consequences
`Preset` changes from two parts to one, so lum.md changes with the build. The show format bumps (the pool and tags), with no migration. Global mode, Update into cues, a live link in cues, embedded presets and recipes are later. Delete keeps broken links listed (§35).
