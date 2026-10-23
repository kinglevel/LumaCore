---
id: 0071
title: Effects are DataPool objects with lines, forms, calls, stomp
area: effects
status: decided
date: 2026-10-14
built_in: —
supersedes: [archive §27 L2562 "phaser steps in Value", archive §28 L2888–2889 "stores no fixture list"]
superseded_by: —
confirmed_by: [`status: Planned` words, `E-EFX` catalog]
source: [archive §28 L2881–2908, archive §28 L2952–2976, archive §28 L2993–3015, archive §28 L3016–3073, archive §32 L6956–6972]
---
# Effects are DataPool objects with lines, forms, calls, stomp

## Context
From established console manuals and GDTF: the right structure is a cue calling a living effect; phaser-style effects are what LumaCore avoids. §32 replaced "the effect stores no fixture list" with targets per line. The full design prose lives in `visions/effects.md`.

## Decision
`Effect` is a DataPool object; a line is `Effect 5.2`. A line has attributes, a `Form` (a pool of waveforms, GUID-linked), BPM, width, direction, NShot, phase rule, layout (`Linear` by order or `Grid`), a Pawtern, absolute / relative mode and a `Blend` (Replace · Add · Subtract · Multiply · Min · Max). The pattern is shared; only Lo / Hi and Disable are sparse per-fixture overrides. A line's target is a raw fixture selection or a live pointer to a Group; `Assign Fixture 1 Thru 5 | Selection | Group 3 At Effect 5.2` replaces it. Two line kinds: wave lines and Prowl lines (a GUID link to a Prowl, a beat-snapped gradient timeline of stops); steps never exist in values, and `lumacore_base::Value` has no phaser field. Edits go into the effect, never the cue; a cue stores the call (effect, size, Rate, timing, pause), never fixtures. Effects merge as their own band: Park > Stomp > effect band > static LTP > defaults; size is a master envelope. `Effect 5` selects its fixtures, `Go Effect 5` starts a programmer call, `Off Effect 5 [Fixture …]`, `Top On|Off Effect 5`; `Stomp` is a tracked layer; `Unstomp` stores a tracked removal. New words are `status: Planned` until built; errors go in an `E-EFX-NNN` catalog; `Lo` / `Hi` are line properties, not registry words.

## Consequences
Not built; the build order is the archive's, the effect band in `resolve` first. Sync and handover: see effects-deterministic-anchors. A static value in a later cue never stops an effect.
