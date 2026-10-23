---
title: Engines
category: docs
---
# Engines

What an engine is in LumaCore, which boundaries the code keeps, and which of them are
registry engines. Status lives in the roadmap ([roadmap.md](../../meta/roadmap.md)), not here.
Decisions: the archive's §26 (engine ownership) and §27 (the render engine),
`docs/archive/decisions-2026-10.md`.

**Registry engines today:** `RenderEngine`, `PerfettoEngine` and `HttpUiEngine`
(`crates/engine/lumacore-engines`). Everything else below is a **boundary** in the code — a
crate, a module, a thread — not an `Engine`.

## Principle

One runtime owner and many specialized engines:

- an engine is a long-lived runtime subsystem with a clear responsibility; it may own live
  state, worker loops or a protocol / service boundary
- helper tools, clients and scripts are not engines
- engines plug into the runtime owner instead of becoming parallel applications

`lumacore-server` is the single runtime owner; `lumacore-cli`, the app and the browser are
client surfaces.

## The boundaries

### The runtime owner

`lumacore-server`: one **state thread** (`state.rs`, archive §26) owns `ServerContext` — the
show, the runtime, the station state. Shell clients, the UDP command port, discovery, OSC and
HTTP reach it only through `StateHandle::call` / `send` (`lumacore-lanes`: the shell lane and
the bounded machine lane); it publishes `Arc<ShowData>` and a `StationView` after each change
and never does network I/O inside a job. It starts and stops the engines (`ServerBuilder`,
`Server::wait`) and reports thread health.

### LumaSession (not an engine)

Station ⇄ station: membership, join / invite / dismiss / leave, replication, election, merge,
the session clock. State-thread jobs plus listener threads that only do I/O
(`crates/server/lumacore-lumasession`, the command port in `lumacore-server`, the kernel's
outbox `lumacore_state::session_link`, the words in `lumacore-cmd-lumasession`). Runtime at
`Root.Runtime.LumaSessions`. [lumasession.md](../networking/lumasession.md),
[replication.md](../networking/replication.md).

### Fixture definitions

GDTF import, validation and channel resolution: `lumacore-gdtf` (the vendored reader in
`vendor/gdtf`), the derived types in `lumacore-fixtures`. The fixture metadata authority; it
never owns live DMX output.

### Patch

Device-to-address placement and conflict-safe mutation: `lumacore-patch` (`PatchEngine`,
stateless), the staged Devices draft (`lumacore_state::patch_draft`) and the patch objects in
`lumacore-objects`. A major boundary, not a thread.

### The render engine (registry engine)

The one renderer (archive §27, §24 "Step 9"): `lumacore-render`, wrapped by `RenderEngine`.

```
state thread ── publishes EngineInput (show + users' modes + generations), RenderHub::publish
      │
      ▼
plan compiler thread: channel tier (the patch as dense slots, only when
  AppState::patch_generation moved) + value tier (merge::resolve per change)
      │  Plan + frame buffers, swapped at frame start; retired ones come back to be freed
      ▼
render thread (60 Hz, spin_sleep, no tokio, no locks, no allocation, frees nothing)
  Plan · Inputs · Channels (cue fades, group masters by the session clock, grand, Highlight,
  Solo) · Park · InputMerge · Publish — each stage timed into Runtime.Health.Render
      │  RenderedFrame (ArcSwap) + the plan it came from
      ▼
line drivers sample the newest frame at each line's Rate, Full or Changes, PacketGap,
Sync — and only send
```

Merge order, highest first: park > frozen programmers > LTP of programmers and running
sequences > defaults (`merge::resolve`); Highlight above playback and below park; input
layers merge per DMX channel in the tick. Values are `lumacore_base::Value` everywhere; DMX
bytes exist only in the tick. Running playback state is `ShowData.Playbacks` (saved,
replicated, never undone). A panic holds the last frame and restarts; a late frame is skipped,
never caught up; a plan-compiler panic keeps the last plan. The tick's rules and their checks
are in [metrics-and-health.md](../build/metrics-and-health.md).

### Output transports (protocols, not engines)

Protocols are catalog rows, not engines (archive §11, §34 22): `lumacore-protocols` is the
contract, `lumacore-artnet` / `-sacn` / `-osc` declare themselves, and `lumacore-lines`'
`ProtocolRuntimes` reconciles one driver per (protocol, NIC) and one listener per Data line to
`protocol_plan::desired` after every show or station change. Nothing protocol-related starts
at boot. A line on an unmapped network never sends; only standalone stations or the
LumaSession master send and listen; transports only send — the render engine decides what the
frame is.

### Network I/O (not an engine)

NICs, the command-port and shell sockets, mDNS, discovery state, PTP read-out:
`lumacore-net`; the listener threads and discovery in `lumacore-server`. Shared by the
LumaSession, the shells and HTTP without owning their semantics; per-NIC services are
`lumacore_lumasession::ServiceAccess`.

### Effects, timecode (data only)

`lumacore-effects` and `lumacore-timecode` hold authored data; no runtime exists. The designs
are [visions/effects.md](../../../visions/effects.md) and
[visions/timelines.md](../../../visions/timelines.md); an effect runtime must feed the render
engine, never bypass it.

### UI (one registry engine)

The per-Shell side — line editing, suggestions, the TUI, frames — is `lumacore-terminal` on
`lumacore-shell`; the web side is `HttpUiEngine` (`lumacore-http`, feature `http`) serving the
WebUI (`lumacore-webui`). Neither owns state: both run command lines through the kernel and
read what `lumacore-views` builds. Live screens are in `Root.Runtime`; saved views on the
user's profile in the show.

### Diagnostics

`Root.Runtime.Health` (`HealthBoard`: render, lines, clock, threads;
[metrics-and-health.md](../build/metrics-and-health.md)), the System Monitor
(`lumacore-sysmon`, the `sysmon` feed, the Meownitor;
[server-diagnostics.md](../build/server-diagnostics.md)), `Root.Runtime.Journal`, and the
`PerfettoEngine` (registry engine, feature `perfetto`; [profiling.md](../build/profiling.md)).
Diagnostics observe every other boundary and never become a command side effect.

### Integrations

REST and the WebSockets (`HttpUiEngine`, [http-api.md](../api/http-api.md)), OSC input (an
`Osc` line, not an engine), the command port. Every one feeds the same dispatch, rights gate,
change path and runtime as a typed line ([integrations.md](../integrations/integrations.md)).

### Spatial and preview

Not built: `ShowData.Spatial3D` holds the scene objects; a preview must never endanger DMX
real-time stability.

## The engine registry

`EngineRegistry` (`crates/engine/lumacore-engines/src/registry.rs`):

- **Capability-based gating** — each engine declares the `Capability` values it requires (the
  enum is in `lumacore-wire`); the registry checks availability before starting it.
- **`EngineContext`** — the shared resources handed over at start (`show_data`, `playback` =
  the `RenderHub`, `command_tx`, `health`, all `Option`). The session clock is global
  (`lumacore_base::clock`), not passed in.
- **`Engine` trait** — `name()`, `requires()`, `status()`, `info()`, `start()`, `stop()`,
  `join(deadline)`, `is_running()`; `EngineInfo` is the serialisable snapshot.

| Engine | Feature | What |
|---|---|---|
| `RenderEngine` | always | The render thread + plan compiler; registered first; line drivers sample its `RenderHub` frames |
| `PerfettoEngine` | `perfetto` | Serves `/api/trace` on its own port and tokio runtime |
| `HttpUiEngine` | `http` | REST / WebSocket / the WebUI |

The ports are the constants in `crates/foundation/lumacore-wire/src/lib.rs`.

`ServerBuilder` (`lumacore_server::serve`) creates the registry, registers the engines and
calls `start_all()`. On shutdown (`Shutdown` from any client, `lumacore-shutoff`, a signal) the
state thread leaves its loop and `Server::wait` stops, in order, inputs (OSC) → outputs
(sACN / Art-Net goodbye) → HTTP → listeners, joining every thread against one deadline
(`SHUTDOWN_DEADLINE`); anything late is named in the System Monitor.

## Authored vs runtime

Authored objects are not engines. Authored truth: fixture types, fixtures, universes, lines,
groups, presets, sequences, timecodes, spatial objects — and the running playback state
(`ShowData.Playbacks`, saved). Runtime truth: LumaSession membership and logged-in users,
transport status, frame buffers, engine health, watches, connected clients. Show objects
describe what can run; engines describe what is running and how.

## Naming

Use `engine` only for a runtime subsystem; `PatchEngine` and `RenderEngine` are the names in
code. Never call a client, a script, a file format, a pool or a one-shot helper an engine.
