---
id: 0269
title: Store creates, Assign addresses, Unassign removes; DMX is slots
area: patch
status: built
date: 2026-10-19
built_in: round 3 1a, 1b, 2
supersedes: [archive §34 L9705–9718 "Patch Fixture <sel> At", archive §34 L10189–10193 "DMX In 1"]
superseded_by: —
confirmed_by: [E-PATCH-001, E-CMD-006, E-CMD-008, E-CMD-010, lum.md's patch examples]
source: [archive §34 L10071–10120, archive §34 L10189–10193, archive §34 L10648–10657, archive §34 L10672–10689, archive §34 L10690–10704]
---
# Store creates, Assign addresses, Unassign removes; DMX is slots

## Context
Round 3 first decided `Patch` / `Unpatch` verbs with `/Type`, `/Offset`, `/Address`; item 19 corrected that to §33 48 (no new verbs), after the old `Patch Nx …`, `Import "x.gdtf"` and `DMX In` were found still adding fixtures outside the Lum syntax.

## Decision
`Store Fixture 201 Thru 206 At DMX 2.001 /FixtureType <slot | "name" | "x.gdtf"> [/Mode] [/Revision n] [/Every n]` creates (`objects/patch/fixture/store.rs`; a `.gdtf` from `library/gdtf/` is imported in the same step; no `/Address` — a flag never carries a target). `Assign Fixture … At DMX [u[.aaa]] [/Every n | /KeepSpacing]` addresses; `Unassign Fixture … At DMX | DMX 2.001 | Fixture 201 DMX 2 | MvrGroup [n] | MvrLayer [n] | MvrClass | Tag` removes — a bare `Unassign Fixture 201` is E-CMD-006. One placement rule (`fixture/address.rs`): an exact address is exact (taken = overlap, E-PATCH-001, both parked); several fixtures, `At DMX n` or a bare `At DMX` take the first free run, skipping taken slots and naming them; packed unless `/Every n` (under the footprint E-CMD-008) or `/KeepSpacing` (with `/Every` E-CMD-006); one universe or nothing (E-CMD-010); a missing universe is created and said. `DMX` is the kernel's object (`objects/patch/dmx.rs`): `DMX 1` a universe's slots, `DMX 1.001` an address, Info by default, `/Input` the input lines' values; `Universe` owns name, MergeMode, lines and renumbering. `DMX` is `Fixture`'s child: `Fixture 3 DMX 2` is break 2; `Assign Fixture 201 Thru 204 At Fixture 301 Thru 304 DMX /Swap` trades pair by pair. `Import FixtureType "file.gdtf"` reads `library/gdtf/` (Devices claim, reach Drive); a newer revision changes no fixture. The UI keeps the labels Patch to… / Unpatch.

## Consequences
Planned: `DMX 1.001 At 255` (the tester). See renumber-is-move-skip-by and multipatch-inside-the-fixture.
