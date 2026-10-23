---
id: 0204
title: One pool window from PoolDef; hold opens the object sheet
area: ui
status: built
date: 2026-10-14
built_in: step 7, 12e part 2
supersedes: [archive §22 L1844–1853 "full-screen selector", archive §30 L3550–3556 "PoolHold board A", archive §30 L5430–5437 "candidates B C"]
superseded_by: —
confirmed_by: [pools::tests, kit-pool.spec.ts]
source: [archive §22 L1824–1862, archive §22 L1888–1900, archive §30 L3550–3556, archive §30 L3695, archive §30 L5421–5437, archive §30 L5463–5468, archive §30 L5512–5513, archive §30 L5626–5650]
---
# One pool window from PoolDef; hold opens the object sheet

## Context
Step 7 (2026-10-11) made one window for every pool so later pools only add their extras, with its own `/ws/pools` socket and a hold-drag full-screen selector. The 12e rounds (2026-10-13/14) tried three hold behaviours on an artifact: the full-screen selector (A), A plus store on an empty cell (B), a per-object sheet (C). The user picked C; A, B and §22's drag-through selector are not taken.

## Decision
A pool object declares `ObjectDef.pool: Some(&PoolDef)` — collection, tap actions as command templates, `cells`, optional `detail`. The station builds the generic `PoolCell` (no, GUID, name, colour, icon, another user's claim, referenced, locked, applicability, bar, badges) from the published show; a kind adds only extras. The frontend has one pool window, `buildPoolPanel`, a kit `<luma-pool>` fed by the `pools` feed on `/ws/feeds` (its DataPool setting `"selected"` or a GUID, `pools::resolve_datapool`); pool settings are window settings.

`<luma-pool>` keeps only the rows in view, repaints a cell only when it changed, never under a finger, and keeps its scroll; three looks by place: desk, phone, mini. Every gesture is a command line, owner first: tap runs the window's tap action; hold or right-click on an object opens the object sheet (`kit/sheet.js`, `openObjectSheet`) — preview plus Edit · Label · Store · Delete · Move · Copy · Settings, each a line; Move / Copy close the sheet and arm a target pick (the next tap is the target). A hold on an empty slot does nothing. Delete goes through `dispatch::delete_first` (see oops-categories-statements).

## Consequences
Never a pool-specific endpoint or client-only pool state; a new pool kind is a `PoolDef` and its cell extras. A drag in a pool is the pointer core's one `lift` gesture as pools receive it (see pointer-core-one-lift-gesture), never a per-window handler. Colour comes from NodeMeta (see nodemeta-colour-icon).
