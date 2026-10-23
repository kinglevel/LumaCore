---
id: 0068
title: Continuous controls ride the gesture channel, one Oops step
area: playback
status: built
date: 2026-10-14
built_in: 12f parts 1, 4a, 4c
supersedes: [archive §30 L5739 "a /Quiet option"]
superseded_by: —
confirmed_by: [test "one entry per gesture", e2e `kit-12f.spec.ts`, `CLIENT_PROTOCOL` / `SESSION_PROTOCOL` bumps]
source: [archive §30 L5739–5746, archive §30 L6453–6458, archive §30 L6501–6512, archive §30 L6563–6572]
---
# Continuous controls ride the gesture channel, one Oops step

## Context
A fader move is a stream of command lines; echoing, journaling and filing an Oops step per move would bury the history and the undo list. A `/Quiet` option was rejected because anyone could hide any line with it: quiet is a property of the transport, not of the line.

## Decision
A kit control sends `gesture { id, line, phase: move | end, window?, label?, from?, value? }` on the page's shell socket `/ws/shell` (`wsGesture`); it arrives as `ShellClientPacket::Gesture` and runs through `run_gesture` (`AppState::gesture`, `core::gesture::GestureLine::Move | End`). The line is a real command (`Master 2.1 At 63.5`): the same rights gate, dispatch and replication. The channel makes it quiet: no echo, command history, Usage, journal or `Cmd` line per move; while the shell queue waits only the newest move of a gesture stays (the end is never dropped); the end files one journal entry and one System Monitor line under the hideable "Faders" filter ("Fader Master 2.1: 100 → 63.5 · Admin"); a refusal is told once per gesture. A member forwards the mark (`StreamMsg::Command.gesture`), so the master stays quiet too. The same line typed is a normal command. One Oops step per drag: `gesture::before_line` and `gesture::remember` keep the first `before` of every subtree the moves change and put them back before the end's line runs, so the end's step spans the drag; moves are Untracked; open gestures are station-local `AppState::gesture_starts` (its cap in the code). Master moves file no step (see group-masters).

## Consequences
Only kit controls send gestures; never a `/Quiet` flag or a per-move quiet path. OSC / MIDI faders take the same channel later. A new continuous control emits `gesture` per move and one `end`, nothing else.
