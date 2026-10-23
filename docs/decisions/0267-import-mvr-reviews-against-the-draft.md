---
id: 0267
title: Import MVR keeps a review on the draft; Apply writes
area: mvr
status: built
date: 2026-10-20
built_in: 34g4 part 4a–4c, 38c, 40c
supersedes: [archive §34 L9452–9460 "File | Xchange switch", archive §34 L9504–9506 "write straight into the draft", archive §34 L10833–10836 "/Missing Recreate | Parent"]
superseded_by: —
confirmed_by: [`PatchDraft.review`, `Action.when`, E-CMD-010, lum.md "### MVR import" (its PENDING shrank), `tests/fixtures/rig.mvr`]
source: [archive §34 L9452–9472, archive §34 L10775–10918, archive §40 L11690–11699]
---
# Import MVR keeps a review on the draft; Apply writes

## Context
Point 10 drew a File | Xchange dialog and, briefly, an import written straight into the draft; review → Import ticked replaced that. Hardened again on 2026-10-20 (declined: a station ask, chips on the review, a page-side review, an `MvrPreset` pool).

## Decision
`Import Mvr "file" [/Drive "usb1"] [/Scope <target>] …` (the `Mvr` object, `objects/patch/mvr`; the Devices draft; the disk door) runs with the profile's "MVR import" preset — declared `UserProfile` properties (`MvrMatch`, `MvrLost`, the None / Take / Offer groups, offsets, `MvrMissing`, `MvrIdType`), each option on the line overriding once. Nothing is written: the review lives on the holder's `PatchDraft.review` (a row per file fixture: Same · Changed · New · Lost · Outside · Clash); the `patch` feed carries `mode: review`; `List Mvr` / `Info Mvr` read it; the grid's selection is the ticks, a Clash row refuses a tick. `Import Mvr {rows} /Apply` writes the ticked rows into the draft (the holder's own cell edit wins; a Clash refuses); `Import Mvr /Cancel` drops it; a bare `Import Mvr` asks with the files element. A ticked New row whose MVR group is gone asks per group as `/Gone Recreate | Parent | None` [`/Group "name"`]; `MvrMissing` other than Ask silences it; a missing layer is made, never asked. An open review is a draft change: Keep is refused (E-CMD-010), Discard and a lost holder drop it. The two-step dialog (`kit/mvrimport.js`: the files element, then the preset as settings) builds and runs the line.

## Consequences
Never write during a review, never a page-only review. Not built: Bring into scope, the two-line review cell, the Xchange side (see export-mvr-and-xchange). Nested file fixtures: see nested-fixtures-folders.
