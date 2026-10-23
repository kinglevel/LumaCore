---
title: Metrics & Health
category: docs
---
# Metrics & Health

The mechanism behind the operator's view in
[health-and-clock.md](../../user/health/health-and-clock.md): what is measured, where it is
written, and what is enforced (archive §24 "Step 9 hardening", §26).

## The render budget

The render thread runs at a fixed **60 Hz** (16.67 ms per frame, like an
established console's realtime loop). Output lines sample its newest frame at their own `Rate`
(default 30 Hz, up to 60). A reference console's realtime view on a 35,451-parameter
rig shows CalcChannel at 2.7 ms per frame; LumaCore's budget for the same size
is **< 3 ms per frame in release** (checked by a test).

## Runtime.Health — the one health source

`Root.Runtime.Health` (`lumacore-runtime`, `HealthBoard`) holds everything;
writers use atomics only, so even the render tick reports.

| Part | What | Written by |
|---|---|---|
| `Render` (`RenderStats`) | per stage (Plan, Inputs, Channels, Park, InputMerge, Publish): start and duration in the last frame, max in the last second; frame time, jitter, headroom (min), late / skipped frames, overload (episodes), busy buffers, panics, restarts, down, plan generation, compile time (channel + value tier), channels, universes | the render thread, the compiler |
| `Lines` (`LineStats`) | per output line: configured / used (node-capped) / actual rate, send time (max), frames, packets, errors | the line drivers |
| `Clock` (`ClockSync`) | role, state (Unlocked / Acquiring / Locked / Stable / Holdover), offsetFromMaster, meanPathDelay, jitter, frequency and skew (ppm), correction (ppm), samples used / rejected, steps (last time, size), master + epoch; on the master every member's report; linuxptp when it runs | the clock sync thread, the UDP port, `lumacore_net::ptp` |
| `Threads` (`ThreadHealth`) | every long-lived thread: alive, last activity, iterations, last / max loop µs, queue depth, panics | each thread |

Where it shows:

- `List Health`, `Info Health Render|Lines|Clock|Threads`, `Info Clock`
- the **Realtime** window (window type `realtime`: every render frame over the `health.render` feed, the tables over `health` at 4 Hz):
  stage bars on the frame timeline
- `/api/health` (ok / degraded) and `/api/metrics` (threads)
- the System Monitor: overload episodes, engine panics / down, clock steps —
  once per episode (the engine never logs)

## Developer attach: Tracy and Perfetto

- **Tracy** (`--features tracy`): a frame mark per render frame, a zone per
  render stage (`stage_span`, `lumacore_trace::held_zone!`), the `frame_us` plot, `late_frame`
  messages.
- **Perfetto** (`--features perfetto,http`, its port `REMOTE_PERFETTO_PORT` in `lumacore-wire`): each render stage is
  recorded as a complete event without waiting (`try_record_complete`); the
  `trace_zone!` macro's guards live to the end of their scope.

Both compile to nothing without their feature.

## Enforcement

- **No allocation in the tick or the send** — `assert_no_alloc` (warn mode) in
  debug builds: `lumacore-server` installs the allocator; an allocation in the
  tick makes the engine panic (held frame, restart, reported), one in a send is
  logged once.
- **Tidy rule** (lumacore-tidy `render_paths_never_lock_log_or_allocate`): the regions
  between `// tidy: render-begin` and `// tidy: render-end` in lumacore-render, lumacore-lines, lumacore-artnet and lumacore-sacn
  (`render.rs`, `engine.rs`, `lines.rs`, `sacn/source.rs`, `artnet/source.rs`)
  may not use locks, logging, `format!`, strings, collections or `Box`/`Arc::new`;
  `// tidy: render-ok (<reason>)` excuses one line.
- **Budget** — `tests::render_tests::a_35k_parameter_frame_renders_under_3_ms`
  (release) and `cargo bench -p lumacore-render --bench render`
  (`render_tick_35k`, `compile_value_tier_35k`).

## Rule: if it runs every frame, it is measured and checked

A new per-frame stage gets a `STAGES` entry (timed by the engine), a Tracy span,
a place in a tidy render region, and stays inside the 35k budget.
