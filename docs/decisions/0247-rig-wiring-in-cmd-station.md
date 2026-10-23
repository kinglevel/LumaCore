---
id: 0247
title: Station, Interface, Network in cmd-station; DMX lines on it
area: station
status: built
date: 2026-10-17
built_in: 34g4 cuts 7–9, 34f7c
supersedes: [archive §34 L8411–8414 "cmd-protocols holding Network"]
superseded_by: —
confirmed_by: [`objects::object_of(Node)`, tidy `crate_edges_point_down`]
source: [archive §34 L8542–8567 (item 25.17), archive §34 L9084–9127, archive §34 L9284–9290]
---
# Station, Interface, Network in cmd-station; DMX lines on it

## Context
The protocols cut found Station, Interface and Network named all over the kernel (Station's Lum shape, the keyed settings, three menus, DeskLock Station, Claim LumaSession Station). Every console studied keeps the station's own network apart from its DMX protocol lines. The earlier plan of `cmd-protocols` holding Network is superseded.

## Decision
`lumacore-cmd-station` holds the station and its network: Station (the table, its identity settings, the wiring lines, remote station edits), Interface (the NIC and its services) and Network (the logical network and its NIC map), their settings (`settings`, `wiring`) and the Network / LumaSessions menus. `lumacore-cmd-protocols` sits on it and holds what every line of a kind shares: `line_object!`, `dmx_line_verbs!`, the kinds' shared properties, DMX In and the DMX Protocols menu; since 34f7c each protocol's object is its own crate under `commands/objects/protocols/` (`cmd-sacn`, `cmd-artnet`, `cmd-osc`) on `cmd-protocols`, and `Output` / `Input` as words are gone. The "is it the Station?" checks moved up with their words as hooks (`Hook::SetCheckedElsewhere`, `Hook::ImportByGuid`); devices and the LumaSession words sit on `cmd-station` and register it first. Assign / Unassign stay the kernel's: shared verb words like Store and Delete; the Universe's Assign finds the lines through the row's declared tree node (`objects::object_of(Node)`), never a static. The kernel keeps `network_id` and the shared Store / Delete helpers.

## Consequences
A new station setting or network word goes in `cmd-station`; a new property of every line in `cmd-protocols::props`; what one protocol has in its own crate's `SettingDef`s. The kernel never names a protocol or the Station; a lower layer reads a declared value. Cross-crate wiring tests live in `tests/catalog` (`wiring`, `lines`, `import`).
