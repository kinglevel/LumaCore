# lumacore-timecode

The timecode data model: timelines with their tracks, clips, regions, markers, events, tempo and
signature entries, as the tree saves them.

## Purpose

Plain data below the tree for the timecode pool (decisions §1): a `TimecodeItem` is one
timeline with a `TimeBase` (frames, beats, ticks or SMPTE), its `TimecodeTrack`s — each with
clips, regions and typed `TrackEvents` (note, control change, pitch bend, program change) —,
its marker, tempo and signature tracks, its loop region and duration. A user's selected Timecode is a GUID
link on the user (`Select Timecode n`, `lumacore-objects`), so the item's identity is its
`NodeMeta` and every type `Walk`s its fields. The crate holds shapes only: nothing here reads a
clock, follows a transport or fires an event — that comes with its own step and its own record.

## Owns / Doesn't own

- **Owns:** `TimecodeItem`, `TimecodeTrack`, `TimecodeClip`, `TimecodeRegion`,
  `TimecodeMarker`, `TimecodeEvent`, `TrackEvents`, `TempoEntry`, `SignatureEntry`,
  `LoopRegion`, `TimeBase`, and their `Identified` / `Walk` implementations.
- **Doesn't own:** where the pool sits (`Datapool` in `lumacore-tree`), the selection
  (`User.selected`), the session clock (`lumacore_base::clock`), any playback of a timeline
  (not built), the words (`lumacore-objects`).

## Public surface

- `TimecodeItem { meta, tracks, marker_track, tempo_track, signature_track, loop_region,
  duration_ticks, time_base }` — the timeline as saved.
- `TimecodeTrack`, `TimecodeClip`, `TimecodeRegion`, `TrackEvents`, `TimecodeEvent`.
- `TimeBase` — `Frames` (default), `Beats`, `Ticks`, `Smpte`.
- `TempoEntry`, `SignatureEntry`, `LoopRegion`, `TimecodeMarker`.

## Rules

- `TimecodeItem` is `Identified` only through its `NodeMeta`; every type `Walk`s its fields
  with an exhaustive destructure. [compiler]
- Nothing here runs: no clock read, no transport. [tidy crate_edges_point_down]
- It depends on base only. [tidy crate_edges_point_down]
- Tracks are a `BTreeMap<u32, _>` so a saved timeline serializes the same every time. [unchecked]

## Edges

- **Depends on:** lumacore-base.
- **Used by:** lumacore-tree (re-exported into the datapool types).

## Decisions

- `docs/decisions/0009-selections-one-only.md` — one selected Timecode per user, a GUID link
- `docs/decisions/0007-guids-everywhere-links.md`, `docs/decisions/0006-node-meta-levels.md` — identity
- `docs/decisions/0005-state-homes-by-type.md` — show data, replicated and saved
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §1, §21; timecode playback is an open item in [docs/meta/roadmap.md](../../../docs/meta/roadmap.md)

## Testing

- `cargo test -p lumacore-timecode` — the crate has no tests of its own today; it compiles its
  `Walk` impls.
- Its types are walked on every load and in the identity tests of `cargo test -p lumacore-tree`
  and `cargo test -p lumacore-objects` (`identity_tests.rs`); `Select Timecode` is covered by
  the catalog's words in `cargo test -p lumacore-catalog-tests`.
