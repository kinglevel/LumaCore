---
id: 0061
title: Playback state lives in ShowData.Playbacks, never undone
area: playback
status: built
date: 2026-10-14
built_in: 12f part 1
supersedes: [archive §13 L1108–1115 "Playbacks.Sequences, Priority pool", archive §27 L2613–2614 "ShowData.Playbacks.Sequences resumed", archive §32 L6913–6915 "global PlaybackData Playback.json"]
superseded_by: —
confirmed_by: [`ShowSubtree::Playbacks`, test "no master move files an Oops step"]
source: [archive §13 L1100–1115, archive §27 L2613–2614, archive §32 L6911–6928, archive §30 L6553–6562]
---
# Playback state lives in ShowData.Playbacks, never undone

## Context
Step 4 hardening decided that what runs is saved with the show, as a reference console's sequence carries its current cue and user in the show, so a show load recalls where every playback stood. The first shape was one flat subtree per engine kind (`Playbacks.Sequences`) with a Priority pool assignment; the code then drifted into one global `PlaybackData` saved as `Playback.json`. A later study restated the registry and 12f part 1 built it.

## Decision
`ShowData.Playbacks` is the registry of what runs, the playback engine's one source of truth, split at the top by what is saved: `Live` (saved; resumed snapped on load, tracked values asserted, no fades) holding `Sequences` and `Masters` by GUID, later `Effects`; and `Echoes` (replicated, never saved; a load starts with none; see sequence-cuezero-offcue). The tree is `Root.ShowData.Playbacks.Live.Sequences | Masters`; the type is `PlaybackStates { live: LivePlaybacks { sequences, masters } }` plus the echoes. One category per engine kind: a later engine adds one. One live run per object: a sequence or an effect never runs twice as a controllable playback. `ShowSubtree::Playbacks` is replicated and journaled and never enters an undo frame: Oops never undoes a Go, and no master move files a step. The playback objects themselves (Sequences, Effects, Executors) are not here; they are DataPool objects (see sequences-in-datapools).

## Consequences
A new running state goes under `Live` or `Echoes`, never into a pool object or `Runtime`. Nothing hardcodes "cue 1" or `Sequence 1` as a default. The Priority pool of §13 is only decided (see priority-pool); every sequence is LTP today. A test checks that a master move files no Oops step; the change path's exhaustive `ShowSubtree` destructure refuses an unclassified field at compile time.
