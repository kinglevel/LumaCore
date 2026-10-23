---
id: 0044
title: Lines sample the engine's frame at their own Rate
area: lines
status: built
date: 2026-10-13
built_in: step 9
supersedes: [archive §24 L2277–2287 "has its own timer"]
superseded_by: —
confirmed_by: [R1-54 Q21, `LineStats.doubled` / `skipped`, `Info Health Lines`]
source: [archive §24 L2277–2287, archive §11 L573–574, archive §11 L605–610, archive §27 L2637–2645, archive §31 L6699–6708]
---
# Lines sample the engine's frame at their own Rate

## Context
A reference console runs one realtime loop and lets DMX leave slower per row; Art-Net 4 nodes report their maximum rate and expect unchanged data resent. Step 9 put pacing on the line; the code review amended the drivers' free-running timers after video's lesson that whole-number ratios don't judder.

## Decision
The engine renders once per tick; each output line's driver takes the newest frame at its own `Rate` (bounds and default on the declared property in the registry; a line is never faster than the engine and never above a node's reported `RefreshRate`, read through the `Discovery` capability), `Send` Full | Changes (changed universes at once, a few repeats, then a keep-alive — the counts live in `lumacore_lines`), `PacketGap` in µs between universes (the gaps of all lines on one driver never take more than half of the fastest line's period, R1-54 Q21) and `Sync` (ArtSync / E1.31 sync after the frame, only where the row declares the `Sync` capability). Drivers obey the tick rules: preallocated packets rewritten in place, no per-frame collections, health to `Runtime.Health` (see render-tick-rules). Line timing is measured, then locked: step 1 counts frames sent twice and skipped (`LineStats.doubled` / `skipped`, `Info Health Lines`); step 2 makes a line whose Rate divides the engine's rate send every Nth engine frame on a frame sequence the engine bumps, other rates and the engine-down fallback keeping their timer.

## Consequences
Pacing is show data on the line, replicated, never a station setting. Step 2 of the lock is decided, not built: it waits for numbers from real shows and slow fades. A new pacing field is a declared property, read by `Set`, the grid and the reference.
