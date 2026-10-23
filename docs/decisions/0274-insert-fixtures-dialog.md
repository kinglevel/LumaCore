---
id: 0274
title: Insert fixtures: Choose, then Place, one Store line
area: webui
status: built
date: 2026-10-19
built_in: round 3 4c2, §37 4
supersedes: []
superseded_by: —
confirmed_by: [`kit/insert.js`, `tests/e2e/tests/phone.spec.ts`]
source: [archive §34 L9719–9724, archive §34 L9790–9802, archive §34 L10023–10032, archive §34 L10368–10378, archive §34 L10543–10559, archive §37 L11234–11242]
---
# Insert fixtures: Choose, then Place, one Store line

## Context
"i dont have new line so i can patch a fresh new fixture": the patch had no way to add a fixture from the UI. An established Insert New Fixtures flow (a picker, then a form) was taken into the kit; the mockup went through six rounds.

## Decision
The Patch tab's Insert fixtures… (an Action of value kind `insert`, `claim`: greyed in Live) and the slab's New open one dialog (`kit/insert.js`), two steps. 1 Choose: the No list (a `<luma-grid>` of the show's types) or Manufacturer · Type · Revision columns; Show (default) / Library / Both; one frame — the Mode column and the read-only channel map never move; Details (the file's facts, the thumbnail there only); Upload from this device into `library/gdtf/` (on a member the file reaches the master over the stream first). 2 Place: Name, Quantity, First FID and Address with free chips, Every (from the Offset), the MVR node, Class; where the block lands, overlaps or runs past the universe's end is said before Create. Create runs `Store Fixture a Thru b At DMX u.aaa /FixtureType <no | "file.gdtf"> /Mode n [/Revision n] [/Every n]; Label Fixture … "name"; Assign Fixture … At <node>`, each statement its own Oops step, shown live in the footer. On a phone Choose splits into Type, then Mode, each a full screen with ‹ Back and step dots; a tablet keeps the columns.

## Consequences
Every field is part of the line; never a page-side create path. The IDType · CID field was dropped with "No CIDs", since reversed (see patch-slot-key-and-three-ids): its return in Place is open. The DMX sheet is Edit Patch's slot grid (see dmx-map-two-zooms-park-on-conflict).
