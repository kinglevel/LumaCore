---
id: 0273
title: Multipatch copies live inside their fixture, owner first
area: patch
status: built
date: 2026-10-19
built_in: round 3 1c, the left-out items
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §34 L10166–10172, archive §34 L10351–10357, archive §34 L10410–10415, archive §34 L10658–10671]
---
# Multipatch copies live inside their fixture, owner first

## Context
A reference console's Multipatch is a virtual fixture mirroring its primary with its own address, name and place, not in the selection grid, plus a global `Multipatch n` whose id changes with the patch. LumaCore takes the form and leaves the global id.

## Decision
A copy is `Fixture 201 Multipatch 1`, owner first (`Fixture.Luma.multipatch`: each its own GUID, name, address and place). `Store Fixture 201 Thru 206 Multipatch 1 [Thru n] [At DMX …]` creates — the next numbers only; a bare `Multipatch` under Store is each fixture's next copy, packed from the address; `Assign Fixture 201 Multipatch 1 At DMX 5.020` addresses; `Delete Fixture 201 Multipatch 1` (the later ones move down); `Info Fixture 201 Multipatch` lists them; a bare `Multipatch` in Delete or Assign is refused, asking which copy. Selecting a copy selects its fixture; a value goes to the primary and every copy follows; the render plan mirrors the fixture's channels on each copy's address; a copy's slots are taken in the slot map (see dmx-map-two-zooms-park-on-conflict), and an overlapping copy's fix is `Delete Fixture n Multipatch m` (lum.md has no Unassign for a copy). The function block's Multipatch… (value kind `copies`, the draft) is the address dialog, then that line. A global `Multipatch n` is not a form. Import and export map a copy to MVR's separate fixture with `multipatch` set.

## Consequences
Not built: naming a copy (`Set Fixture 201 Multipatch 1 "Name"`), a copy's own place, its row in the grid, Multipatch… inside Edit Patch. Never a copy outside its fixture.
