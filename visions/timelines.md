---
title: Timelines
category: docs
---
# Timelines

**Design, not built:** this page is the timecode timeline as decided — tracks, markers, transports — written before it was built. Today the station has sequences and cues (`docs/user/playback/sequences.md`), a `Timecode` pool you can select, and nothing that runs a timeline; `Info Timecode` is `Planned` in the generated reference (`docs/user/reference/objects.md`). The decision records are `docs/decisions/<slug>` — the related ones today are `scenes-stateful-logic`, `selections-one-only`, `playback-state-saved` and `sequence-pool-and-window`; timelines get their own record when their step is picked up (see `docs/plans/active/md-upgrade.records.md` for the slugs). `§N` below names a section of the frozen log `docs/archive/decisions-2026-10.md`.

This document captures the intended direction for
`Root.ShowData.DataPools.DataPool("SongName").Timecodes`: the shape and meaning
of timecode timelines before they are built. The current syntax of `Timecode`
is in the [object reference](../docs/user/reference/objects.md#timecode); sequences and
cues are under [`Sequence`](../docs/user/reference/objects.md#sequence) and
[`Cue`](../docs/user/reference/objects.md#cue).

## Status

Markers: **Built** = in the code today · **Planned** = decided, not built.

- **Built (playback, not timecode):** sequences and cues on the 60 Hz playback
  engine — `Store Cue`, `Go`, `Go-`, `Goto Cue`, `On|Off|Toggle Sequence`,
  running state saved in `ShowData.Playbacks` and resumed on load; the grand
  master and group masters; cue part timing (fade/delay in and out) and the
  selected sequence — see [sequences.md](../docs/user/playback/sequences.md).
- **Built (tree only):** the `Timecodes` branch and its sub-nodes (`Tracks`,
  `MarkerTrack`, `TempoTrack`, `SignatureTrack`, `LoopRegion`, `Duration`,
  `TimeBase`) can be browsed; `List Timecode` lists the pool. Nothing stores a
  timecode yet.
- **Planned:** timecode objects and their editing, the Timecode Engine
  (transport: play, stop, position, chase/lock to external clock), tracks,
  markers, events, MIDI event families, and every example sentence on this page.
  The Timecode verbs are marked Planned in the reference.

## Timecodes

`Timecodes` belong to the datapool layer.

They are stored show objects and should travel with the show as authored truth.

They are not:

- runtime transport state
- generic playback `Sequence` objects
- machine-local timing services

Important:

- `Timecode`
  - is the authored show object
- `Timecode Engine`
  - is the runtime subsystem that runs authored timecode structures

These are related, but they are not the same thing.

## DataPool Timecodes

```text
Root.ShowData.DataPools
└── DataPool("SongName")
    └── Timecodes
        └── Timecode
            ├── Tracks
            │   └── Track
            │       ├── Events
            │       │   ├── NoteEvents
            │       │   ├── ControlChangeEvents
            │       │   ├── PitchBendEvents
            │       │   ├── ProgramChangeEvents
            │       │   └── MetaEvents
            │       ├── Clips
            │       └── Regions
            ├── MarkerTrack
            ├── TempoTrack
            ├── SignatureTrack
            ├── LoopRegion
            ├── Duration
            └── TimeBase
```

Notes:

- `Timecode` is the stored datapool timing/program noun.
- `Tracks` are the main lane structure for timed content.
- `MarkerTrack`, `TempoTrack`, and `SignatureTrack` are timeline-level
  concerns.
- `LoopRegion` should be explicit rather than hidden in runtime state.
- `Duration` should be authored state, not only inferred at runtime.
- `TimeBase` should be explicit so tick-based and clock-based workflows remain
  teachable and stable.
- `Events` is the stable parent branch for typed event families.

## Timecode Tracks

`Track` is an ordered timed lane inside a `Timecode`.

Tracks should:

- hold timed content
- preserve ordering/index
- support typed timeline content where needed
- remain authored objects in `ShowData`

Examples of likely track roles:

- trigger track
- event track
- marker-supporting track
- media-sync track
- automation/value track

## Timecode Content

The recurring timeline content family from the DAW references is:

- `Event`
  - the smallest timed item
- `Clip`
  - reusable timed content
- `Region`
  - a placed timed span

Notes:

- typed events are the most important content type for MIDI-capable timecode
  workflows
- `Clip` is useful when identity or reuse matters more than placement.
- `Region` is useful when placement matters more than reuse.
- some `Timecode` implementations may need only event collections
- the model should still allow `Clips` and `Regions` when they help

Recommended event families for later MIDI support:

- `NoteEvents`
  - note-on/note-off style musical note data
- `ControlChangeEvents`
  - MIDI CC style continuous controller data
- `PitchBendEvents`
  - pitch bend data
- `ProgramChangeEvents`
  - instrument/program selection changes
- `MetaEvents`
  - textual, marker, cue, or other non-note metadata events

Recommended structure:

- `Events`
  - the general event container branch
- typed event collections
  - the specific timed data families inside `Events`

Avoid collapsing all future MIDI data into one flat untyped event list.

## Timeline-Level Timing Objects

These should belong to the `Timecode` itself, not to arbitrary content tracks:

- `MarkerTrack`
  - named reference points on the timeline
- `TempoTrack`
  - timeline tempo changes where relevant
- `SignatureTrack`
  - meter/grid changes where relevant
- `LoopRegion`
  - the authored loop span
- `Duration`
  - the authored timeline length
- `TimeBase`
  - the authored timing base such as frames, beats, ticks, or SMPTE-oriented
    resolution

This keeps the time domain explicit and consistent.

## Authored Vs Runtime

This distinction should stay hard and obvious.

Authored `Timecode` state belongs in `ShowData`, such as:

- tracks
- note events
- control change events
- pitch bend events
- program change events
- meta events
- clips
- regions
- markers
- tempo changes
- signature changes
- loop region
- duration
- time base

Runtime timing state belongs elsewhere, such as:

- current play position
- playing/stopped state
- chase/lock state
- active loop pass
- external clock lock state

Important:

- `Timecode` is authored structure
- runtime transport belongs to the `Timecode Engine`

These should not collapse into the same saved object.

## Timecode Object Vs Timecode Engine

This distinction should stay explicit across the definitions.

`Timecode` object:

- lives under `Root.ShowData.DataPools.DataPool(...).Timecodes`
- is authored truth
- stores tracks, markers, tempo/signature data, and timing structure

`Timecode Engine`:

- lives conceptually in `Root.Runtime`
- owns live play/stop state
- owns current position and clock lock
- executes time-based triggers against authored `Timecode` objects

Rule:

- store structure in `Timecode`
- run structure in `Timecode Engine`

## Lum Alignment (Planned)

The language should keep stable nouns that map cleanly to this branch:

- `Timecode`
- `Track`
- `Marker`
- `Event`

Important distinction:

- these are the likely Lum-facing nouns
- typed branches like `NoteEvents` and `ControlChangeEvents` may remain
  internal tree nouns unless Lum later chooses to expose them directly

Examples:

```text
Store Timecode 1
Info Timecode 1
Timecode 1 Track 3
Info Track 3
Store Marker 5
Set Marker 5 Time 01:00:00:00
Assign Timecode 1 Sequence 4
```

These should map conceptually to:

```text
Timecode  -> Root.ShowData.DataPools.DataPool(...).Timecodes
Track     -> Timecode.Tracks
Marker    -> Timecode.MarkerTrack
Event     -> Timecode.Tracks.Track.Events
```

Internal tree mapping for typed families:

```text
NoteEvent           -> Timecode.Tracks.Track.Events.NoteEvents
ControlChangeEvent  -> Timecode.Tracks.Track.Events.ControlChangeEvents
PitchBendEvent      -> Timecode.Tracks.Track.Events.PitchBendEvents
ProgramChangeEvent  -> Timecode.Tracks.Track.Events.ProgramChangeEvents
MetaEvent           -> Timecode.Tracks.Track.Events.MetaEvents
```

This gives `Lum` a simpler noun story while keeping the tree extensible.

## Background

### Reference direction

The local DAW repos in the `SKILL` workspace suggest the same recurring shape:

- timeline-level markers
- timeline-level tempo/signature data
- track-owned timed content
- explicit authored loop and duration
- typed event families scale better than one flat event list when MIDI enters
  the picture

The strongest references for this were:

- `openDAW`
- `Stargate`
- `LMMS`
- `daw`
- `butterDAWg`
- `cacophony`

These were used as structural references only, not as naming authority.

### Improvement suggestion

The clearest improvement would be to make the `Timecode` branch more explicit
inside [object-tree.md](../docs/user/object-tree/object-tree.md).

Recommended improvement:

```text
Root.ShowData.DataPools
└── DataPool("SongName")
    └── Timecodes
        └── Timecode
            ├── Tracks
            │   └── Track
            │       ├── Events
            │       │   ├── NoteEvents
            │       │   ├── ControlChangeEvents
            │       │   ├── PitchBendEvents
            │       │   ├── ProgramChangeEvents
            │       │   └── MetaEvents
            │       ├── Clips
            │       └── Regions
            ├── MarkerTrack
            ├── TempoTrack
            ├── SignatureTrack
            ├── LoopRegion
            ├── Duration
            └── TimeBase
```

Why this improves the current design:

- it gives `Timecode` a clear internal family shape
- it gives `Lum` stable nouns like `Track`, `Marker`, and `Event`
- it keeps typed MIDI-capable event families without forcing all of them into
  first-line Lum syntax immediately
- it keeps authored timecode structure separate from runtime transport state
- it leaves room for typed MIDI note, CC, pitch bend, program change, and
  meta-event support later
- it leaves room for simple or advanced timecode workflows without changing the
  top-level tree
