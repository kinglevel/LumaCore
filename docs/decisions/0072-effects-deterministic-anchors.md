---
id: 0072
title: The cue is the effect sync point; handover slides phase
area: effects
status: decided
date: 2026-10-11
built_in: —
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §28 L2909–2951]
---
# The cue is the effect sync point; handover slides phase

## Context
Effects started in any order must give the same output on every station of a LumaSession. Phaser-style sync and morph and any start-order merge are ruled out. The full design prose lives in `visions/effects.md`.

## Decision
The effect band is a pure function of (running calls, show content, clock). Blend modes apply in a fixed order — Replace → Add / Subtract → Multiply → Min / Max — never by start order. The cue is the sync point: a starting cue grabs the session clock as its anchor, zero time for every effect it calls; a line's position is `(clock − anchor) × BPM × Rate + phase rule`, so calls added later run in sync when they share BPM. A tracking call keeps its anchor; size or Rate changes never re-anchor; a Go or Goto on the call's cue takes a new one. The programmer is one running playback with one anchor: the selected running sequence's current cue anchor (moving on every Go), else its own, taken at its first effect and dropped on Clear. Handover is a phase slide, not a crossfade: an effect stored into a running cue moves from the programmer's anchor to the cue's when the programmer lets go — snapped with Program Time off, otherwise sliding the shortest way, the amplitude never dipping; the same between any two sources with different anchors. Anchors are replicated runtime playback state; a show load takes fresh ones. `Random` forms are seeded per effect, line, fixture, cell and cycle; the clock is `clock::session_ms()`. Every cue is its own clock: no cross-sequence sync. Speed masters change tempo only, never phase.

## Consequences
Not built; it lands with the effect band (see effects-model-and-control). Never anchor on start order, wall clock or a per-station time; the build adds a two-station same-frame test.
