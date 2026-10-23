---
id: 0064
title: Tracking cues with parts; timing per part, group, value
area: playback
status: built
date: 2026-10-14
built_in: step 11 (part timing), 12f parts 4a and 7a
supersedes: [archive §29 L3221 "No per-value timing"]
superseded_by: —
confirmed_by: [E-PLAY-003, E-CMD-006, E-CMD-009]
source: [archive §27 L2567–2582, archive §27 L2610–2616, archive §29 L3216–3221, archive §29 L3285–3287, archive §30 L5918–5919, archive §30 L6393–6400, archive §30 L6501–6518]
---
# Tracking cues with parts; timing per part, group, value

## Context
The playback engine follows the established model of cue tracking and cue parts; later steps added per-feature-group timing with a fade Path and per-value timing.

## Decision
A Sequence holds Cues, a Cue holds Parts; timing and values live on the part. Tracking: a cue stores the programmer's active values; one equal to what already tracks in is kept as a blocked value; a value tracks until a new value or a Release (the OffCue releases by default); with `Set Sequence n Tracking` off, tracked values become releases; Goto asserts tracked values.

Each part times itself (`CuePart` fade_in / fade_out, delay_in / delay_out): dimmer up and every non-dimmer attribute use the in times, dimmer down the out times; GDTF snap channels jump after their delay. Fades run on `clock::session_ms()`; a Go during a fade starts each attribute from where it is (`Plan::compile_after`, `plan::Fade`); a release a Go brings fades by its part's timing (`merge::releases`); handovers snap. Timing layers, innermost first: a value's own `ValueTiming { fade, delay }` (on `ProgValue` and `CueValue`, unset = the cue's; `Attribute "Dimmer" Fade 2 | Delay 0.3 | Fade Cue`, none → E-CMD-009) > its feature group's `GroupTiming` on the part (Fade / Delay / OutFade / OutDelay, `path: Option<FadePath>` Linear · SCurve · EaseIn · EaseOut · Snap) > the part's — `playback::value_timing`. Words: `Set Cue <n> Fade|OutFade|Delay|OutDelay <s>`, `Set Cue <n> [Sequence <m>] FeatureGroup <g> Fade … | Path <shape> | Cue` (a Path without a feature group is E-CMD-006). Live programmer changes snap.

## Consequences
Bare `Go` / `Goto` use the selected sequence (E-PLAY-003; see selections-one-only). Echoes take the same layers. A new timing layer enters `value_timing`, never a second path in the tick. Not built: Assert modes, Follow / time cues, `Kill`.
