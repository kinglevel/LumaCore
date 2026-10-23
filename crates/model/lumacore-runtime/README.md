# lumacore-runtime

`Root.Runtime`: the session-wide and station runtime state — users, claims, LumaSessions,
network, health, the journal and the live screens — replicated where it must be, never saved.

## Purpose

The second of the three homes (decisions §10): what a running station knows but no show file
keeps. `RuntimeState` holds the logged-in users with their modes and undo lists, the claims by
GUID, the joined LumaSessions and their members, the data networks' interfaces and lines, the
engines' status, the health board every long-lived thread reports to, the change log and each
user's screens with their windows. The render engine writes its frame numbers here as atomics, the
thread health is atomics too, so even the render loop reports without a lock or an allocation. Nothing in this crate is serialized into a `.luma`.

## Owns / Doesn't own

- **Owns:** `RuntimeState`, `RuntimeUser`, `Modes`, `ModeMaster`, `Claim`, `ClaimAsk`,
  `Clipboard`, `LumaSessionsRuntime` and its member / station / client rows, `NetworkRuntime`
  and its interface / listener / connection rows, `EngineRuntime`, `WatchTarget`; `HealthBoard`,
  `ThreadHealth`, `ThreadGuard` (`health`); `RenderStats`, `FrameRing`, `LineStats`, `ClockSync`,
  `STAGES` (`render_stats`); `Journal`, `JournalEntry`, `JOURNAL_CAPACITY` (`journal`);
  `ScreenStore`, `Screen`, `Window`, `LayoutNode`, `EncoderState`, `CaptureRequest`
  (`live_screens`).
- **Doesn't own:** the show (`lumacore-tree`), who writes the claims and modes (the commands
  in `lumacore-objects`), the render tick that fills `RenderStats` (`lumacore-render`), the
  state thread's lanes (`lumacore-lanes`), the sysmon log on disk (`lumacore-sysmon`), what a
  window type is (`lumacore_tree::window_types`).

## Public surface

- `RuntimeState` and its fields — `Root.Runtime` as the tree shows it.
- `HealthBoard::register` / `snapshot`, `ThreadHealth`, `ThreadGuard` — a thread reports once
  registered, by atomics.
- `RenderStats`, `FrameRing`, `RenderStatsView`, `ClockView`, `LineStatsView`, `STAGES`,
  `STAGE_COUNT` — the Realtime numbers.
- `Journal::push`, `entries`, `JOURNAL_CAPACITY` — the last commands that changed the show.
- `ScreenStore`, `SharedScreens`, `default_layout`, `CaptureRequest` — the live screens the
  UI engines render and the screenshot capture board.

## Rules

- The journal keeps the last `JOURNAL_CAPACITY` entries with an increasing `seq`.
  [test keeps_the_last_entries_with_increasing_seq]
- The frame ring hands a reader only what it has not seen, and a reader that fell behind gets
  what the ring still holds. [test the_ring_hands_out_what_a_reader_has_not_seen + test a_reader_past_the_ring_gets_only_what_it_holds]
- A thread registers once, ticks by atomics and is marked stopped. [test registers_ticks_and_stops]
- A screenshot capture asks the viewers of a screen and counts their uploads.
  [test a_capture_asks_the_viewers_and_counts_their_uploads]
- Replacing a layout drops orphan windows and rejects a dangling reference.
  [test replace_layout_drops_orphans_and_rejects_dangling]
- Nothing here goes into a show file; `RuntimeState` has no `Serialize`. [unchecked]
- It depends on base only. [tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-base.
- **Used by:** lumacore-tree (re-exported as `lumacore_tree::runtime`), lumacore-protocols,
  lumacore-net.

## Decisions

- `docs/decisions/0005-state-homes-by-type.md` — runtime is replicated, never saved
- `docs/decisions/0004-claims-by-tree-path.md` — claims by GUID in the runtime
- `docs/decisions/0026-one-state-thread.md`, `docs/decisions/0049-render-tick-rules.md` — health and frame
  numbers as atomics
- `docs/decisions/0027-session-stream-json-zstd-journal-cap.md` — the journal's cap
- `docs/decisions/0174-screenshots-client-capture.md` — the capture board
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §10, §21, §24, §26, §29, §30

## Testing

- `cargo test -p lumacore-runtime` — `journal.rs`, `render_stats.rs`, `health.rs`,
  `live_screens.rs`.
- On a live station: `Info Health Render|Lines|Clock|Threads` in `tests/systemtest`.
