---
paths:
  - crates/engine/**
  - crates/network/protocols/lumacore-lines/**
  - crates/foundation/lumacore-base/src/clock*
  - crates/network/lumacore-clocksync/**
---

# Render engine, lines, clock, health

Records: `render-tick-rules`, `playback-merge-order`, `render-engine-is-the-renderer`, `line-rates-send-pacing`, `dmx-input-merge`, `session-clock-rate-overlay`, `tracking-cues-parts`, `group-masters`, `sequence-cuezero-offcue`, `gesture-channel`, `one-line-costs-two-fingerprints`, `presets-link-into-cues-update-recast-extract`.

## The tick

- One render thread at the engine's fixed tick rate (the constant in `lumacore-render`): show snapshot → plan compiler → tick (masters, Solo, park, DMX input) → lines sample frames at their own `Rate` and only send. Presets, programmer and cues share one `Value` type (`lumacore_base::Value`, normalized / physical); DMX bytes exist only in the tick.
- Inside `// tidy: render-begin … render-end` (`lumacore_render::render::render_into`, `engine::Engine::tick`, the lines' send): no allocation, no lock, no logging, no map or string lookup, no `SystemTime`, and nothing is freed; replaced plans, frames and inputs go back to the compiler thread as `Retired`. An exception is `// tidy: render-ok (<reason>)`. Checked three ways: `assert_no_alloc` in debug (the server installs the allocator; an allocation in the tick panics), the tidy rule, the budget test and `cargo bench -p lumacore-render --bench render`.
- No tokio in the DMX path; `spin_sleep` on an absolute deadline; ArcSwap hand-off. The engine never logs.

## The plan

- `lumacore_render::plan` is built on the `plan_compiler` thread: the channel tier (the patch as dense slots) is rebuilt only when `AppState::patch_generation` moves, so bump it wherever the patch can change; the value tier (`merge::resolve`) per change. The state thread publishes `EngineInput` with `RenderHub::publish`; never compile or render on the state thread. Commands read output through `AppState::output_frame` / `output_plan` (they wait, bounded, for the frame carrying what was published; without an engine the plan is compiled once there).
- `merge::resolve` is the one merge: park > frozen programmers > LTP of programmers and running sequences > defaults. The tick applies cue fades, Highlight, grand master, group masters (fading by the session clock), Solo and raw parked channels last. Highlight uses only the fixture's GDTF `DMXChannel@Highlight`, per user's selection, above playback, below park.
- A cue value is plain steps or a preset link (`CueData`); the compiler resolves a link each compile through `lumacore_tree::PresetIndex` (the fixture's own entry, else the universal one; none = no value). The sequence cache keeps positions in the sequence, never a value, so a changed preset needs no invalidation; never a copy of a preset's value in a cue, never a lookup in the tick.
- Cue timing: each cue part times itself (`CuePart` fade / delay in and out; dimmer up and non-dimmer = in, dimmer down = out; GDTF snap jumps after the delay); the compiler keeps the previous plan and fades a channel from where it is at `clock::session_ms()` when a sequence's newer value arrives (`Plan::compile_after`); a release a Go brings fades by the part's timing (`merge::releases`); handovers (Clear, Unpark, Off Freeze, Blind, Off Sequence) snap.
- Running state is `ShowData.Playbacks` (replicated, never in undo frames): `Live` (saved: Sequences, Masters) and `Echoes` (never saved; `Top On|Off Sequence n`; intensity HTP in `ValueTier::echoes`, the rest LTP; `echo_tick` ends them). Masters are `Master 2.1` grand (Off = blackout), `2.2` / `2.4` cap everybody's Highlight / Solo, `Master Highlight|Solo` your own; relative input; never undone.

## Panic and overload

- An engine panic holds the last frame and restarts; a second panic soon after leaves it down. A late frame is skipped, never caught up. A plan-compiler panic keeps the last plan (never blackout), latches `stale_since`, and repeated panics stop the compiler until a show load or restart. The state thread reports each episode once to the System Monitor (`RenderWatch`).

## Lines and input

- Every protocol's lines are one `Line` type and one set of handlers; never duplicate per protocol or direction. One driver per (protocol, NIC) sends outputs and receives inputs (`lumacore_lines::lines`, `input::InputMerger`: E1.31 priority then HTP, loss timeout, own CID and IPs ignored; sequence numbers and sync as the spec says); nothing allocates per received packet (`SourceName`, reused buffers).
- Input layers go to `RenderHub::inputs` and merge per DMX channel in the tick (`Universe.merge_mode`, band = `Line.input_priority`). Park always wins; masters and Solo never touch input. `DMX In <u>` reads raw input; the `dmx` window reads the `dmx` feed (binary frames while visible; ArcSwap only).
- Output lines have `Rate`, `Send Full|Changes`, `PacketGap` and `Sync`; an Art-Net node's `RefreshRate` caps a line; a line never sends faster than the render. Only standalone stations or the session master send; a line on an unmapped network never sends, never add an "auto" fallback. `protocol_plan::desired` is the pure plan; `ProtocolRuntimes::reconcile` makes it so.

## Clock and health

- `lumacore_base::clock` is a rate overlay on `Instant` that never runs backwards except one step (first lock after joining, or far off); ahead of the master it slows down. Fades, group-master fades and effect anchors use `clock::session_ms()`, never wall clock. Members discipline it from `Time` pings on the master's command port (`lumacore_clocksync`); stats in PTP terms in `Runtime.Health.Clock`.
- `Runtime.Health` (`HealthBoard::view`) is the one source: render stages per frame (`RenderStats`, atomics), lines' send numbers, clock, threads; `List Health`, `Info Health Render|Lines|Clock|Threads`, the `realtime` window on the `health.render` feed. Trace zones per stage: `lumacore_trace::trace_zone!`; every trace macro lives in `lumacore-trace`.
- The gesture channel: a kit control's line (`/ws/shell` `gesture`, `AppState::gesture`) runs quietly, newest move wins while busy, one journal and one "Faders" sysmon entry at its end; never a `/Quiet` flag.
