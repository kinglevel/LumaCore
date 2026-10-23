---
id: 0016
title: Freeze lifts the LTP programmer above playbacks
area: modes
status: built
date: 2026-10-10
built_in: step 4a
supersedes: [archive §7 L338–347 "Freeze Toggle flips", archive §7 L325–328 "masters as a layer"]
superseded_by: —
confirmed_by: none
source: [archive §7 L317–347, archive §27 L2620–2621]
---
# Freeze lifts the LTP programmer above playbacks

## Context
The March priority stack put the programmer in a fixed layer above playbacks and masters as a layer above both, with "Freeze Toggle" flipping the programmer below the playbacks. Step 4 hardening (2026-10-10) replaced that with the model of established consoles: the programmer merges LTP with playbacks, Freeze lifts it, and masters are output modifiers applied after the merge, not a layer.

## Decision
The programmer and the running playbacks merge LTP — the latest change wins, so a playback can take an attribute back from an unfrozen programmer. `On Freeze` lifts that user's programmer above every playback except Super; `Off Freeze` hands over with a snap. The one merge is `lumacore_render::merge::resolve`: park > frozen programmers > LTP of programmers and running sequences > defaults, compiled into the plan's value tier; the tick then applies cue fades, Highlight, the grand master, group masters, Solo and raw parked channels last. Masters never sit in the merge; they modify its output. Freeze is a per-user mode like Blind, Highlight and Solo (see mode-objects-verb-first), its state in `Runtime.Users.{User}.Modes`, never saved.

## Consequences
Never add a programmer "layer" above or below playbacks, a Freeze flag on a playback, or a master inside `merge::resolve`; where a value ends up is decided once, in the compiler, and the tick only applies what the plan holds (see the render-tick record). Priority bands between playbacks are priority-pool. Program Time fading programmer changes is decided, not built (see programmers-ltp-fixture-claims).
