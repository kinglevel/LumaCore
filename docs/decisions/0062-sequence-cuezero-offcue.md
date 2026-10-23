---
id: 0062
title: Every sequence has CueZero and OffCue; Top starts echoes
area: playback
status: built
date: 2026-10-14
built_in: 12f part 2
supersedes: [archive §30 L6234–6235 "Top sends Goto"]
superseded_by: —
confirmed_by: [E-CMD-010, `CommandDef::hold`, "load refuses a sequence without them"]
source: [archive §32 L6930–6955, archive §30 L6259–6278, archive §30 L6529–6552, archive §33 L7105–7110]
---
# Every sequence has CueZero and OffCue; Top starts echoes

## Context
Established consoles create CueZero and OffCue with every sequence. 12f first planned Top as a `Goto Sequence 3 Cue <first>` button; §32 replaced that with echoes. They have no echoes (Top jumps to the first cue); the naming is left open.

## Decision
`cue_zero` and `off_cue` are fixed `CueObject` fields of every Sequence (no serde default: a file without them is refused at load); never deleted, moved or renumbered. Words: `Cue 0` / `CueZero`, `Cue OffCue` (`playback::cue_ref`); `Store Cue 0` stores into CueZero; the OffCue refuses values (E-CMD-010) and takes timing; neither is a Goto target. `Set Sequence n CueZeroMode Off | AllUsedAttributes | OnlyUsedDimmers` (default Off) fills CueZero part 1 with fixture-type defaults; part 0 is what you stored and wins. Nothing hardcodes "cue 1": the top is CueZero, then the first cue.

`Top On Sequence n` / `Top Off Sequence n` is a hold verb (the press / release slot right after the verb, `CommandDef::hold`; the bare form is refused naming `On|Off`; Playback rights, Untracked). `Top On` starts an echo: a run from CueZero into the first cue with cue timing, the live playback untouched; `Top Off` runs its OffCue. Echoes live in `Playbacks.Echoes.Sequences` by GUID, replicated, stripped from the file. The sequence props `MaxEchoes` (a full list refuses the newest `Top On`, never drops an old echo) and `EchoMax` (starts the OffCue) limit them; a holder gone is `Top Off` at once — `playback::echo_tick` on the station owning the show. An echo's intensity is HTP in `ValueTier::echoes`, every other attribute LTP; `Off Sequence` ends them.

## Consequences
Temp and Flash are decided, not built. Not built: echoes advancing through Follow / time cues, `Kill`, the live run's Off still snaps, non-intensity attributes snap back when an echo goes. Effects' echoes come with effects-model-and-control.
