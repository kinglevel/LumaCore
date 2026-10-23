---
id: 0026
title: One state thread, two lanes, no waiting inside a job
area: engine
status: built
date: 2026-10-19
built_in: step 2 (the thread), the code review's lanes, R1-67 busy coverage
supersedes: [archive §26 L2454–2458 "global Mutex ServerContext", archive §26 L2487–2490 "one queue ten seconds"]
superseded_by: —
confirmed_by: [E-INTERNAL-001, E-CMD-017, CallError::Full, E-WIRE-002, Info Health Threads queue / refused, spawn_named, StateHandle::on_busy_change]
source: [archive §26 L2452–2465, archive §26 L2480–2500, archive §31 L6753–6773, archive §30 L5370–5379]
---
# One state thread, two lanes, no waiting inside a job

## Context
A global `Mutex<ServerContext>` let a slow client or HTTP request freeze the server. Step 2 gave the state to one thread with a job queue; the code review split the queue (a person typing can't outrun the thread, an OSC flood can); R1-67 covered every path that waits on it.

## Decision
One state thread (`lumacore-server`'s `state.rs`; handle and lanes in `lumacore-lanes`, `ServerContext` its `Host`) owns show, runtime and station state and runs jobs one at a time. Everything reaches it through `StateHandle::call` / `send` — no global lock. Two lanes: the **shell lane** (shells' keys, lines, taps and answers, a logout's revoke, the LumaSession stream, elections, signals — unbounded, never dropped, served first) and the **machine lane** (`StateHandle::machine()`: HTTP, feeds, the command port's `udp-requests` worker, discovery — bounded, see `lumacore-lanes`; a full lane refuses at once with `CallError::Full`, E-CMD-017 / HTTP 503, counted `refused` in Health); OSC keeps its own bounded channel. Never call `StateHandle::call` inside a job (E-INTERNAL-001); interactive callers use `call_within` (HTTP → 503, a shell → "busy" E-CMD-017); a feed's menu view and a line on the command port wait the same way, suggestions shorter (the newest waits behind queued keys; a taken `LineEdit` rides the key queue). A watchdog warns on a long turn (`busy` in `/api/station` and the `station` feed via `on_busy_change`). Threads stay one per concern, no tokio outside HTTP; the command port's receive thread never waits on the state thread (it answers `Time` pings at the socket; an oversize reply is E-WIRE-002). Long disk work runs on `core::background`; no network I/O inside a job. After each change it publishes `Arc<ShowData>` plus a `StationView` (see show-snapshot-arc-parts). Every job runs in `panic_report::guarded`.

## Consequences
A new caller picks a lane; a new blocking path gets `call_within` and a busy answer.
