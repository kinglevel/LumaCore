# Profiling

LumaCore has two optional, compile-time-gated profiling backends:

- **Tracy** — real-time frame profiler for live performance monitoring
- **Perfetto** — offline trace capture in Chrome Trace Event format

Both compile to zero overhead when their feature flags are disabled. They can
be enabled independently or simultaneously.

## Build

```bash
# Tracy only (live profiling)
cargo build --release -p lumacore-station --features tracy

# Perfetto only (offline trace capture)
cargo build -p lumacore-station --features perfetto,http

# Both backends simultaneously
cargo build --release -p lumacore-station --features tracy,perfetto,http
```

The `perfetto` feature pulls in `lumacore-http` itself (the `PerfettoEngine`
serves its trace handlers), so `--features perfetto` alone works; add `http` for
the web UI and REST API on the HTTP port.

The `start_developer` script builds `lumacore-server` with all three features
automatically.

> **Important:** The workspace declares `tracy-client` with default features
> enabled. The default features include `enable`, which links the Tracy C
> library and allows the profiler to connect. If you ever set
> `default-features = false` on `tracy-client`, the macros will compile but
> Tracy will not be able to discover or connect to the process.

## Tracy (Live Profiling)

Tracy is a real-time frame profiler that connects to the running process via
shared memory and displays per-frame timing, plots, and zone breakdowns.
It does **not** use a TCP port — the Tracy GUI discovers instrumented processes
automatically on the local machine.

### Setup

1. Install the [Tracy Profiler](https://github.com/wolfpld/tracy) release that matches the
   `tracy-client` pinned in the root `Cargo.toml` (its `tracy-client-sys` bundles one Tracy
   release — see below)
2. Build with `--features tracy` (release mode recommended for realistic timings)
3. Start the LumaCore server
4. Open Tracy — it should auto-discover the running process
5. If using "Connect" manually, enter `127.0.0.1` (no port needed)

> **Version matching:** the Tracy GUI must be the **exact release** the bundled client
> library speaks, or it refuses with a protocol mismatch. Do not use `brew install tracy` —
> Homebrew ships whatever is newest. `./scripts/unix/dev_init.sh` builds the matching
> release from source ([dev-init.md](dev-init.md)).

### What You See

- **Frame view** — 60 Hz render frames as frame boundaries (`tracy_frame_mark!`)
- **Zone timeline** — one span per render stage (`Plan`, `Inputs`, `Channels`,
  `Park`, `InputMerge`, `Publish`), plus `plan_compile` and `state_job`
- **Plot** — `frame_us`: the work of one frame in µs
- **Late frame markers** — `late_frame` messages when a frame misses its
  16.67 ms deadline (`FRAME_PERIOD` = 1 s / `RENDER_HZ` 60). A late frame is
  skipped, never caught up.

### Macros

| Macro | Purpose |
|-------|---------|
| `tracy_zone!(name)` | Scoped span — zone opens on creation, closes on drop |
| `tracy_plot!(name, value)` | Plot a named value (cast to `f64`) |
| `tracy_frame_mark!()` | Mark a frame boundary (called once per 60 Hz render frame) |
| `tracy_message!(text)` | Emit a text marker on the timeline |

Every macro lives in `lumacore-trace` (`macros.rs`); `tracy_frame_mark!`, `tracy_plot!` and
`tracy_message!` are used by `lumacore-render` only. All of them compile to nothing when the
`tracy` feature is disabled.

## Perfetto (Offline Trace Capture)

Perfetto captures trace events into a ring buffer and serves them as
Chrome Trace Event JSON over HTTP. You load the resulting file in
[Perfetto UI](https://ui.perfetto.dev) for offline analysis.

### Architecture

The `PerfettoEngine` is a runtime engine that:

- Runs its own single-threaded tokio runtime on a dedicated OS thread
  (`perfetto-engine`)
- Serves the trace endpoints on its own port (`REMOTE_PERFETTO_PORT` in `lumacore-wire`)
- Is decoupled from the main HTTP engine

The trace collector (`TRACE_COLLECTOR` in `lumacore-trace`) is a global singleton holding a
ring buffer of `MAX_EVENTS` events. When the buffer is full, the oldest events are evicted.
The render thread records with `try_record_complete`, which never waits on the lock.

### PerfettoGuard (RAII)

`PerfettoGuard` is the primary recording mechanism. It captures the start time
on creation and records a complete (`'X'` phase) trace event on drop:

```rust
let _guard = PerfettoGuard::new("my_operation", "category");
// ... work happens here ...
// event recorded automatically when _guard drops
```

### HTTP Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/trace?seconds=N` | JSON snapshot of events (optionally filtered to last N seconds) |
| `GET /api/trace/download?seconds=N` | Downloadable `lumacore-trace.json` with `Content-Disposition` header |

Both return Chrome Trace Event format:
```json
{
  "traceEvents": [
    {"name": "Plan", "cat": "render", "ph": "X", "ts": 1234, "dur": 56, "pid": 1, "tid": 2}
  ],
  "displayTimeUnit": "us"
}
```

### Workflow

1. Build with `--features perfetto,http`
2. Start the server — `PerfettoEngine` binds to the Perfetto port (`server status` prints it)
3. Exercise your workload (run commands, trigger DMX output, etc.)
4. Open [ui.perfetto.dev](https://ui.perfetto.dev)
5. Download the trace: `curl http://localhost:<perfetto port>/api/trace/download -o trace.json`
6. Load `trace.json` in Perfetto UI (or use "Open trace file" → paste the URL)

To capture only the last 30 seconds:
```bash
curl "http://localhost:<perfetto port>/api/trace/download?seconds=30" -o trace.json
```

## Unified `trace_zone!` Macro

The `trace_zone!` macro emits to both Tracy and Perfetto simultaneously. Each
backend is independently compiled in or out based on feature flags:

```rust
lumacore_trace::trace_zone!("plan_compile");
// Expands to (statements, so the guards live to the end of the scope):
//   let _tracy_zone = lumacore_trace::tracy_zone!("plan_compile");       // a Tracy span, or ()
//   let _perfetto_zone = lumacore_trace::perfetto_zone!("plan_compile"); // a PerfettoGuard, or ()
```

Use `trace_zone!` for new instrumentation in hot paths — it automatically
covers both backends with a single call.

Every trace macro lives in one crate, `lumacore-trace` (`macros.rs`, decisions §34 20; tidy counts a
`tracy_client::` anywhere else). Which backend records is `lumacore-trace`'s own `tracy` / `perfetto`
feature — each macro has a recording form and an empty one — so a crate that zones depends only on
`lumacore-trace`, and the station's `tracy` / `perfetto` features turn them on for every crate.

## Instrumentation Points

| Location | Type | Name | Notes |
|----------|------|------|-------|
| `lumacore-render` `engine.rs` render frame | Tracy span per stage (`stage_span` → `held_zone!` into a `HeldZone`) | `Plan`, `Inputs`, `Channels`, `Park`, `InputMerge`, `Publish` | 60 Hz render thread (decisions §24) |
| `engine.rs` render frame | Perfetto complete event per stage (`lumacore_trace::record_complete`, never waits) | same names, category `render` | |
| `engine.rs` render frame | `tracy_plot!` | `frame_us` | Work per frame |
| `engine.rs` render frame | `tracy_frame_mark!` | — | Frame boundary |
| `engine.rs` late frame | `tracy_message!` | `late_frame` | Deadline missed (no catch-up) |
| `lumacore-render` `compiler.rs` | `trace_zone!` | `plan_compile` | Plan compiler thread |
| `lumacore-server` `state.rs` | `trace_zone!` | `state_job` | One job on the state thread |

The same numbers, always on and without a profiler, are `Runtime.Health.Render`
(`List Health`, `Info Health Render`, the Realtime window over the `health.render` and `health` feeds;
`frame_us` is also a `RenderStats` field). `trace_zone!` expands to statements, so
its guards live to the end of the caller's scope (a block form dropped them at
once and measured nothing).

## Adding New Instrumentation

**Hot path (compiler, state thread):** Use `trace_zone!` to cover both
backends. In `lumacore-render`, pair with `tracy_plot!` if the value is useful as a
time-series.

```rust
fn my_hot_path(&mut self) {
    lumacore_trace::trace_zone!("my_hot_path");
    let start = std::time::Instant::now();
    // ... work ...
    lumacore_trace::tracy_plot!("my_hot_path_us", start.elapsed().as_micros());
}
```

**Render tick:** the tick must not allocate, lock or log (decisions §24). It
already times its stages into `RenderStats` and records them with
`lumacore_trace::record_complete`; add a stage there instead of a new zone.

**One-off measurement:** Use `PerfettoGuard::new(name, category)` directly
when you only need offline trace capture without Tracy.

**Frame boundaries:** Only the render thread calls `tracy_frame_mark!()`.

**Event markers:** Use `tracy_message!` for rare events (missed frames, errors)
that should appear on the Tracy timeline without a duration.

See [Metrics & Health](metrics-and-health.md) for frame time budgets and
health check invariants.

## Key Files

| File | Role |
|------|------|
| `crates/foundation/lumacore-trace/src/macros.rs` | Every Tracy + Perfetto macro, `HeldZone`, `record_complete` |
| `crates/foundation/lumacore-trace/src/collector.rs` | `TRACE_COLLECTOR` ring buffer (`MAX_EVENTS`) |
| `crates/foundation/lumacore-trace/src/guard.rs` | `PerfettoGuard` RAII duration recording |
| `crates/foundation/lumacore-trace/src/format.rs` | Chrome Trace Event JSON serialization |
| `crates/model/lumacore-runtime/src/render_stats.rs` | `RenderStats` (always-on stage timings, `STAGES`) |
| `crates/ui/lumacore-http/src/endpoints/trace.rs` | HTTP endpoints (`/api/trace`, `/api/trace/download`) |
| `crates/engine/lumacore-engines/src/perfetto.rs` | `PerfettoEngine` (its own HTTP server on the Perfetto port) |
