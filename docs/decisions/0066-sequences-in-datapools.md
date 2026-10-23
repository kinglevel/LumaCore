---
id: 0066
title: Sequences, Timing and Curve presets live in DataPools
area: playback
status: built
date: 2026-10-14
built_in: 12f part 6a
supersedes: [archive §32 L6913–6915 "global PlaybackData Playback.json"]
superseded_by: —
confirmed_by: [`FORMAT_VERSION` bump]
source: [archive §30 L5828–5832, archive §30 L6246–6248, archive §30 L6265–6267, archive §30 L6413–6426]
---
# Sequences, Timing and Curve presets live in DataPools

## Context
§32 names the destination: Sequences, Effects and Executors are DataPool objects like Groups, while the code kept them in one global `PlaybackData`. At the 12f go the user chose what moves with Sequences: the Timing and Curve presets the sequence window uses; Effects and Executors wait for their own steps.

## Decision
`Datapool` carries `sequences`, `timing_presets` and `curve_presets` (numbered maps); the tree is `Root.ShowData.DataPools.<n>.Sequences | TimingPresets | CurvePresets`; a sequence's identity is a `NodeMeta` (name, note, colour, icon). The show file writes them as the generic per-pool files (`DataPools/<n>/Sequences.json`, `TimingPresets.json`, `CurvePresets.json`). `Sequence 3` names your selected DataPool's sequence, `DataPool 2 Sequence 3` another pool's, owner first; bare words use the selected sequence wherever it lives, and that selection stays a GUID link. Readers go through `ShowData::sequences()` (every pool), `ShowData::sequence_by_guid` → (datapool, number, sequence), `sequence_mut(dp, n)` and `AppState::selected_sequence()` → (datapool, number). The Sequence pool shows the user's selected DataPool or a GUID setting. The Sequence object's settable properties are NodeMeta's plus Tracking, CueZeroMode, MaxEchoes, EchoMax. Effects, Executors and Park stay show-wide in `PlaybackData` until their steps.

## Consequences
The move was a forward-only format bump (`FORMAT_VERSION` in `lumacore-luma`, the demo regenerated); no migration exists. Copy / Move work across pools by decision; Copy / Move / Delete of a Sequence were not built in 12f (roadmap). Pages and executors follow the same rule when they come. Never add a show-wide sequence list or a number-keyed sequence link.
