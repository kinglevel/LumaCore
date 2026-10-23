---
id: 0281
title: Fixture.Luma extras and the other live columns
area: patch
status: decided
date: 2026-10-18
built_in: —
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §34 L10010–10019]
---
# Fixture.Luma extras and the other live columns

## Context
`Fixture.Luma` holds what MVR lacks (see patch-faithful-to-mvr-and-guids); point 2 listed pan / tilt offsets, encoder inverts and the grid spot among them, and a reference console's Live Patch has more columns. The user chose to build them in round 3 rather than wait for the areas that read them.

## Decision
`Fixture.Luma` gains pan / tilt offsets, DMX inverts (pan, tilt), encoder inverts, 3D invert, grid spot (position in the group grid) and grid invert, beam angle, cast shadow, follow target, selectable / visible in 3D, and P1–P4 (free patch values; what they mean is checked against established consoles before building) — each a declared property on the Fixture's row (`ObjectDef::props`) and a patch column in the Full / Extended column sets, live (`Props::live`) wherever a reference Live Patch accepts it — all of these but the grid spot's rotation, which is Patch only. Nothing reads them until their area is built (the programmer and device sheet, the 3D visualizer); the column is there to set and keep them.

## Consequences
Not built (round 3 part 7); the behaviour of `DMXInvertPan` / `Tilt` is open with it. A field goes to `Fixture.Luma` only when MVR has no home for it; its live or draft standing is declared at the property, never in a handler (see staged-patch-draft).
