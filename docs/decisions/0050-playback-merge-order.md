---
id: 0050
title: One merge, one Value type, DMX bytes only in the tick
area: render
status: built
date: 2026-10-11
built_in: step 4a
supersedes: [archive §27 L2549–2560 "the three renderers", archive §27 L2558–2562 "phaser steps in Value"]
superseded_by: —
confirmed_by: [`merge::resolve`]
source: [archive §27 L2549–2599, archive §27 L2601–2635, archive §28 L2901–2904, archive §11 L614–617]
---
# One merge, one Value type, DMX bytes only in the tick

## Context
Three renderers each merged their own way. Researched against the priorities and programmer layers of established consoles, one engine replaced them in step 4a; the phaser fields first put into `Value` left again when effects became their own objects.

## Decision
`lumacore_render::merge::resolve` is the one merge, compiled into the plan's value tier: park > frozen programmers > LTP of programmers and running sequences > defaults; DMX input competes at its line's band (see dmx-input-merge). The tick then applies cue fades, Highlight, the grand master, group masters (fading by the session clock), Solo and raw parked channels last; Highlight and Solo sit above playback and the programmer, never above Park; Blind removes only the programmer's layer; masters and Solo never touch input. Commands see the same result through the engine's output snapshot, never a second merge. Presets, programmer and cues share one `lumacore_base::Value` — normalized or physical, static: absolute, relative, a preset reference or a release; no phaser steps, dynamics are effects (see effects-model-and-control). DMX bytes exist only in the tick. Park stores Values per fixture attribute (`Park Fixture 3`, `Park Fixture 3 At 50`); raw channel park (`Park DMX 1.101 At 255`) stays beside it, saved with the show. Running playback state is `ShowData.Playbacks` (see playback-state-saved).

## Consequences
A new layer is a change to `merge::resolve` and the value tier, nowhere else; a new value shape changes `Value` for every store at once. The full ladder (Super, Swap, HTP, Highest, High, Low, Lowest) is the Priority pool's default content, decided and not built — every sequence is LTP today (see priority-pool). Programmer levels and a per-user Preview programmer are decided, not built. A merge change is measured with the budget test (see render-tick-rules).
