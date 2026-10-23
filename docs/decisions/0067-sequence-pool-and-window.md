---
id: 0067
title: Sequence pool in executor style; sequence window sheet and timeline
area: windows
status: built
date: 2026-10-14
built_in: 12f parts 6b, 7b, 8
supersedes: []
superseded_by: —
confirmed_by: [e2e `kit-12f.spec.ts`]
source: [archive §30 L5757–5763, archive §30 L5771–5780, archive §30 L5786–5795, archive §30 L5802–5807, archive §30 L5813–5827, archive §30 L6343–6361, archive §30 L6374–6392, archive §30 L6401–6412]
---
# Sequence pool in executor style; sequence window sheet and timeline

## Context
The sequence window came from the user's two references: a classic vertical cue sheet with per-feature-group timing and the fade curve, and a horizontal timeline with cues as columns. The artifact rounds settled the pool look, the vertical slider for every timing field and a graph whose scale never jumps.

## Decision
The Sequences window is the one pool window (`buildPoolPanel`, the `pools` feed, DataPool `"selected"` or a GUID): executor-style cells with the previous / current / next cue lines, the current framed gold while running, the blue mark on a running sequence. Tap actions (`SEQUENCE_POOL`): Select (default), Go, Toggle, Off, Top (held: `Top On` at the press, `Top Off` at the release — `PoolAction.release`), Edit, None; the window's `hold_action` setting is Edit | Sheet.

The Sequence window has the settings `Layout Sheet | Timeline` and `Zoom` (ranges declared on the window type), one element `<luma-cue-sheet>` (`kit/cuesheet.js`) fed by the `sequence` feed (`core::sequence_view`: CueZero first, OffCue last, per feature group its own timing or null, an owner-first `line`). `Edit Sequence <n>` opens the sheet as a pop-up (`UiEvent::SequenceSheet`). Every edit is a `Set Cue …` line; a tap on a timing opens the slider panel, press-and-drag edits live (the curve's handles too); a drag rides the gesture channel (see gesture-channel); the time scale (`niceTotal`) holds during a drag and grows only past its end; the sheet re-renders after the drag, scroll kept. The timeline is a column per cue, a row per timing drawn in its Path's shape; a cue's head opens its options.

## Consequences
The mockup's other cue options wait for their `Set Cue` words; pinch zoom is not built. Never a pool-specific endpoint or client-only state: the kit's pool, sheet and slider panel are the only controls.
