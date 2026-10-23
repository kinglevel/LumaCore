---
id: 0065
title: Store Cue between CueZero and OffCue; Cue is Store's default object
area: playback
status: built
date: 2026-10-14
built_in: 12f (Store Cue, Empty pool objects by Store)
supersedes: [archive §30 L6280–6281 "Store Sequence did nothing"]
superseded_by: —
confirmed_by: [E-PLAY-004, E-CMD-008, E-CMD-009, E-CMD-010]
source: [archive §30 L6279–6342, archive §30 L6362–6373]
---
# Store Cue between CueZero and OffCue; Cue is Store's default object

## Context
`Store Sequence 1` did nothing useful, `Store Group 3` without a selection refused, and storing a new cue number with an empty programmer was refused (E-PLAY-002). Decided with the user one question at a time, against the Store of established consoles.

## Decision
CueZero is Cue 0 and nothing goes before it (a number below 0 is E-CMD-008); OffCue is last and stores no values (E-CMD-010); every new cue lands between them. Cue is Store's default object (`CommandDef::default_object`): bare `Store`, `Store 2` and `Store "name"` run as `Store Cue` (`run::store_default`); the suggestion engine offers `Store Cue <n>` first. `Store Cue` without a number stores the next cue after the last (`next_cue_key`); an empty programmer stores an empty cue; a new number creates it. Storing over a cue holding values asks (`playback::store_cue`, `StoreMode`): Merge · Overwrite · Remove · Create Second Cue (`second_key`, a point number before the next cue; no room = E-CMD-010) · Cancel; the flags `/Merge /Overwrite /Remove /CreateSecondCue` (`flags::STORE_CUE`) skip the ask, and a key / macro line without one is refused naming them (E-PLAY-004); an empty cue just takes what is stored. With no selected sequence, Store creates one at `next_sequence_no` (after the highest in your DataPool), selects it and says so — never Sequence 1. `Store Sequence <n> ["name"]` creates the sequence with CueZero + OffCue and stores active programmer values as the next cue; an existing one with an empty programmer is E-CMD-009. `Store Group <n>` with nothing selected creates an empty group; an existing group is never wiped (E-CMD-009). `TimingPreset` / `CurvePreset` are DataPool objects (`core/objects/presets.rs`), owner first. A quoted name labels the object in the same line.

## Consequences
Every built pool object can be created empty by Store; Preset, Macro, Layout, World, Filter, Timecode stay planned. Never a second ask path or a silent overwrite.
