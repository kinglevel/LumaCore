---
id: 0278
title: One DMX map, two zooms; an overlap parks both
area: webui
status: built
date: 2026-10-19
built_in: round 3 4d1, 4d2, 4d3b, §37 5–6
supersedes: [archive §34 L9759–9766 "Address cell or Patch to… opens Edit Patch"]
superseded_by: —
confirmed_by: [`lumacore_tree::dmx_slots::SlotMap`, the `patch` feed's `runs` / `conflicts`]
source: [archive §34 L9759–9766, archive §34 L9909–9916, archive §34 L10428–10464, archive §34 L10516–10542, archive §37 L11243–11255]
---
# One DMX map, two zooms; an overlap parks both

## Context
The round-3 review chose parking both sides of an overlap over Titan's "the later one", and the ways into Edit Patch were decided after the Address cell had been one of them. §37 reshaped both zooms for a phone.

## Decision
One door for who sits on which slot: `SlotMap` (every run, overlap and parked fixture, copies included). An overlap parks both fixtures: each keeps its address, values and programming, the render plan's channel tier skips its channels until resolved, both are red on the conflict list with one-tap fixes as lines (`Assign … At DMX`, `Swap`, `Unassign … At DMX`, `Delete Fixture n Multipatch m` for a copy). One element, `<luma-dmxmap>` (`kit/dmxmap.js`): zoomed out, a row per universe with its usage bar and a block per run coloured by type; a tap selects, a tap on a free run with a selection runs `Assign Fixture … At DMX u.aaa` (the Universes grid's second view, `Grid.dmx_map`). Zoomed in (`zoom: 'slots'`), one universe's slots in rows with `u:aaa` labels; a phone draws shorter rows and a list of universe cards instead of bars. Edit Patch (`kit/editpatch.js`): the fixtures left, the slot grid right, its tools (Patch with Skip patched and Layout, Probe, Find, Identify, Unpatch, Next free address / universe, Move to this universe) each one line. Four ways in: the function block's Edit Patch…, the value dialog's Slot grid…, the context menu's Edit Patch…, the Map's zoom in; the Address cell keeps the value dialog.

## Consequences
Never a client-side conflict rule; the feed reads the same map. Open: Swap addresses and Multipatch… inside Edit Patch, another user's Identify in the strip.
