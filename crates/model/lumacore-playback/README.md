# lumacore-playback

The playback data model: sequences, cues and their parts, tracking, park, masters, and the
running state of playbacks that is saved with the show.

## Purpose

Plain data below the tree (decisions §27, §13): a `SequenceObject` holds its cues, every cue its
parts, and values and timing live on the part — part 0 always exists. `tracked` reads a cue's
values as the sequence tracks them, `cue_key` / `cue_label` read and write cue numbers the way
consoles do. `ParkData` is what nothing overrides; `PlaybackStates` is `ShowData.Playbacks` — `Live`
(sequences and masters, saved) and `Echoes` (never saved). Every sequence has its fixed `Cue 0`
and off cue; the grand masters and group masters are `Masters` with `MasterLevel`s that fade by
the session clock. The crate decides nothing at run time: the merge order, fades and the tick are
`lumacore-render`'s, the words are `lumacore-objects`'.

## Owns / Doesn't own

- **Owns:** `PlaybackData` (effects, executors, `ParkData`), `ParkedValue`, `ParkedChannel`;
  `SequenceObject`, `SequenceSettings`, `SequenceAssignment`, `CueObject`, `CuePart`,
  `CueValue`, `CueTrigger`, `GroupTiming`, `FadePath`, `CueZeroMode`, `CUE_ZERO`,
  `MibPreference`, `PlaybackPriority`, `RestartMode`; `Tracked`, `cue_key`, `cue_label`;
  `Effect`, `TimingPreset`, `CurvePreset`, `ExecutorObject`; `PlaybackStates`,
  `LivePlaybacks`, `EchoPlaybacks`, `Echo`, `SequenceState`, `Masters`, `GrandMasters`,
  `MasterLevel`.
- **Doesn't own:** where sequences live (`Datapool::sequences` in `lumacore-tree`), the merge
  and the fades (`lumacore_render::merge`, `plan::Fade`), `Go` / `Store Cue` / `Master`
  (`lumacore-objects`), the `Value` type (`lumacore-base`), the selected sequence (the user).

## Public surface

- `SequenceObject`, `CueObject`, `CuePart`, `CueValue` — the show's sequences as saved.
- `SequenceObject::tracked`, `next_cue`, `previous_cue` — what a cue outputs with tracking.
- `cue_key(no)` / `cue_label(key)` — cue numbers to and from the pool key.
- `PlaybackStates`, `LivePlaybacks`, `EchoPlaybacks`, `SequenceState` — the running state.
- `Masters`, `GrandMasters`, `MasterLevel::go_to` / `at` — levels fading by the session clock.
- `ParkData::park` / `unpark` / `park_channel`, `SequenceState::go_to`, `FadePath::at`.

## Rules

- Values track until changed or released, and a `Go` stamps tracked values older than new ones.
  [test values_track_until_changed_or_released + test go_stamps_keep_tracked_values_older]
- Cue numbers read and print the usual console way. [test cue_numbers_read_like_a_console]
- A master fade near the end of time never overflows. [test a_master_fade_near_the_end_of_time_never_overflows]
- Every type `Walk`s its fields with an exhaustive destructure; a new field doesn't compile until
  classified. [compiler]
- No run-time decision lives here: no merge, no fade curve, no tick. [tidy crate_edges_point_down]
- It depends on base only. [tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-base.
- **Used by:** lumacore-tree (re-exported), and through it lumacore-render and lumacore-objects.

## Decisions

- `docs/decisions/0064-tracking-cues-parts.md` — Sequence → Cue → Part, tracking, timing per part
- `docs/decisions/0061-playback-state-saved.md`, `docs/decisions/0061-playback-state-saved.md` —
  `ShowData.Playbacks`: Live saved, Echoes replicated
- `docs/decisions/0062-sequence-cuezero-offcue.md`, `docs/decisions/0062-sequence-cuezero-offcue.md` — Cue 0, the
  off cue, echoes
- `docs/decisions/0063-group-masters.md`, `docs/decisions/0063-group-masters.md` — the masters
- `docs/decisions/0066-sequences-in-datapools.md`, `docs/decisions/0064-tracking-cues-parts.md`,
  `docs/decisions/0050-playback-merge-order.md`
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §13, §27, §29, §30, §32

## Testing

- `cargo test -p lumacore-playback` — the tests at the end of `src/lib.rs`: tracking, the `Go`
  stamps, cue numbers, the master fade.
- Playback on the rig: `cargo test -p lumacore-objects` (its `commands/playback_tests.rs`) and the
  render budget in `cargo test -p lumacore-render`.
