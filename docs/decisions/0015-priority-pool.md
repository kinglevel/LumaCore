---
id: 0015
title: A user-defined Priority pool replaces fixed bands
area: playback
status: decided
date: 2026-10-10
built_in: —
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §7 L349–363, archive §27 L2586–2589, archive §11 L648–652]
---
# A user-defined Priority pool replaces fixed bands

## Context
Established consoles order playbacks by fixed bands (Super, Highest, High, LTP, Low, Lowest). The March design preferred levels the operator defines and reorders; the playback engine hardening (§27, 2026-10-10) kept that choice when it adopted the established merge order, and the DMX input work (§11, 4c) placed input bands on today's layers until the pool exists.

## Decision
Priorities are a DataPool pool, `DataPool.Priorities`, user-defined and ordered (`Priority 1 "Background"` lowest … `Priority 4 "Overrides"` highest): users reorder, insert and rename; a sequence references a Priority by GUID. Those bands are the pool's default content, not a hard-coded list. Until the pool is built, every sequence merges LTP in the one merge (`merge::resolve`: park > frozen programmers > LTP of programmers and running sequences > defaults) and DMX input bands map onto today's layers — Super above everything but Park, Prog LTP with frozen programmers above playbacks, Highest / High above playbacks and programmers, LTP latest change wins, Low / Lowest only where nothing of ours gives a value — so that they slot into the pool later with no show change.

## Consequences
Not built; today every sequence is LTP. When it comes: a pool object on the one object door with a `PoolDef`, a new `ShowData` subtree with its arms and file entry (see change-path-undo-subtrees), a GUID link on `Sequence` (see guids-everywhere-links), the bands compiled into the plan's value tier (see the render-plan record) — never an ordering computed on the state thread or in the tick. Never add a fixed band enum to a sequence; an input line's `input_priority` band keeps its meaning and maps onto the pool.
