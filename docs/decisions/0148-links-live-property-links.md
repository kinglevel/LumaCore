---
id: 0148
title: "Live links: Wire, typed nodes, computed where used"
area: links
status: decided
date: 2026-10-15
built_in: —
supersedes: [archive §33 L7527–7533 "Assign makes Link", archive §33 L7607–7614 "Assign reads source"]
superseded_by: —
confirmed_by: none
source: [archive §33 L7519–7555, archive §33 L7556–7562, archive §33 L7563–7579, archive §33 L7580–7585, archive §33 L7586–7606, archive §33 L7607–7621, archive §33 L7622–7648, archive §33 L7649–7655]
---
# Live links: Wire, typed nodes, computed where used

## Context
The registry raised a property driven by another object's property (a preset's Dimmer driving a theme Paint). Node tools were compared.

## Decision
A property pointing at another property is always a live link, never a copy. `Link` is an object made by `Wire <source> "<P>" At <destination> "<P>"` (source first, one direction; → `Link 7`), removed by `Unwire <destination> "<P>"`. A quoted property sticks to the target before it on either side of `At`. Links live in `ShowData.Links`, `UserProfile.Links`, `DataPool.Links`; the line's owner names the home, else what it writes does. Ends are GUID plus address; an import reconnects by GUID, by address only as a reviewed ask, else kept broken and listed.

A link is a typed node declared in the Rust registry, never stored: `Inlet` / `Outlet` children (`Wire Link 8 Outlet 2 At Link 9 Inlet 1`), mapping settings named like Blender's Map Range (`"FromMin"` … `"TriggerOn"`), a declared conversion table per kind pair (Exact / Lossy / Refused, never a guess), a loop refused. Pure parts in `lumacore-links`, station side `core::links`, ends via `core::resolve`.

A link is computed where its destination is used: DMX-ending links in the sending station's tick under the render rules, screen-ending on each station showing it. Driven values never travel, never write show data or an Oops step, never overwrite the stored value. Setting a linked property asks Remove link / Pause link / Cancel; in a stored key line it is refused (an E-LINK code). Making a link needs the destination property's level; it is an Oops step. Only users make links.

## Consequences
Not built; command line only at first. When built: a test that no shipped show, Default profile or demo holds a link; the E-LINK catalog; lum.md's rule sentence; a node editor is a new kit element, mockup first.
