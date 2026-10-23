---
id: 0014
title: Pawtern replaces Distribute; slot reserved
area: effects
status: decided
date: 2026-10-10
built_in: —
supersedes: [archive §3 L214–217 "the Distribute verb"]
superseded_by: —
confirmed_by: none
source: [archive §3 L214–241, archive §22 L1739–1740]
---
# Pawtern replaces Distribute; slot reserved

## Context
Spreading values across a selection was first a `Distribute` verb. The revision of 2026-10-10 made it a pool object named Pawtern (LumaCore's name), both the thing and the action, and the group model of step 7 (§22) reserved room for it so the show format needs no change when it lands.

## Decision
A Pawtern is a DataPool object and an action: `Pawtern 3` or `Pawtern "Arc"` applies it to the selection, `Store Pawtern 3` stores one. Pawterns are typed by context — linear (1D, selection order: `Pawtern "Linear"`), grid (2D: `Grid Pawtern "Arc"`), 3D (`3D Pawtern "Radial"`) — and the pool holds every type with metadata naming the contexts it is valid in. Every `Group` carries a `pawtern: Option<Uuid>` slot, empty today (`None`), a GUID link when the feature comes (see guids-everywhere-links). A node-based distribution editor is a later UI; the pool structure supports it from day one.

## Consequences
Not built. Never add a `Distribute` word or a distribution hidden in another verb; the words are declared in the catalog and in lum.md first (the Lum syntax record), the object on the one object door with a `PoolDef`, its running effect in the effects design (see the effects records). The group slot stays `None` and is never removed; the show format moves only forward, so the slot is why no bump is needed for the field itself. The grid context depends on the selection grid already built (see groups-in-datapool-selection-rule); the 3D context waits for 3D.
