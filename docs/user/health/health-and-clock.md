---
title: Health And Clock
category: docs
---
# Health and clock

LumaCore renders DMX in real time: a missed frame is a visible glitch on stage.
This page shows where to see how the engine, the output lines, the session clock
and the threads are doing. Syntax: [`Health`](../reference/objects.md#health),
[`Clock`](../reference/objects.md#clock) in the reference.

The developer side (what is measured, how it is enforced) is in
[metrics-and-health.md](../../dev/build/metrics-and-health.md).

## How output runs

- One render thread computes every frame at a fixed **60 Hz** (16.67 ms per
  frame): masters, Solo, park and DMX input are applied there.
- Each output line samples the newest frame at its own `Rate` (default 30 Hz,
  1–60, `Set Sacn 1 "Rate" 44`). An Art-Net node's reported refresh rate caps its
  line.
- A late frame is skipped, never caught up. If the engine panics it holds the
  last frame and restarts; a second panic within 10 s leaves it down (the lines
  keep the last frame).

## Where to look

| Where | Shows |
|---|---|
| `List Health` | everything below, in one sheet |
| `Info Health Render` | the render engine stage by stage |
| `Info Health Lines` | each output line's rates and send times |
| `Info Health Clock` | the session clock, and on the master every member's |
| `Info Health Threads` | every long-lived thread |
| `Info Clock` | the session clock in detail, with PTP |
| `Clock` / `Clock /Watch` | the wall clock (HH:MM:SS.mmm), live |
| the **Realtime** window | stage bars on a frame timeline, like a console's realtime view |
| the System Monitor | overloads, engine panics, clock steps — once per episode |

Open the Realtime window like any other: `Store Window "Realtime"`. It updates
4 times a second.

All of it lives in `Root.Runtime.Health` — runtime state, never saved.

## Render

`Info Health Render` shows:

- **Rate** — 60 Hz and the frame period
- **Frame** — this frame's time and the max in the last second
- **Headroom** — time left in the frame, and the minimum
- **Jitter**
- **Late** — late frames, skipped frames, overload episodes (and `OVERLOAD now`)
- **Panics** — panics, restarts, and `DOWN` when the engine stopped
- **Plan** — the compiled plan: generation, channels, universes, compile time
- **Frames** — frames rendered, and busy buffers
- one line per **stage** (Plan, Inputs, Channels, Park, InputMerge, Publish):
  when it started in the frame, how long it took, the max

The budget: a 35,000-parameter rig renders in under 3 ms per frame (release
build).

## Lines

`Info Health Lines` shows one row per sending output line: protocol, configured
rate, the rate actually used (capped by the node), the measured rate, send time
(and max), packets and errors. "No output line is sending" means no line is
mapped to a NIC on this station, or this station is a LumaSession member (only
the master sends).

`List Sacn` (or `List ArtNet`) shows each line's universes and live state.

## Session clock

Fades, group-master fades and effects run on the **session clock**, never on
wall-clock time. It is a rate-adjusted clock that never runs backwards except one step (first lock after joining, or more than 1 s off); when it is ahead of the master it slows down instead.

- A standalone station or the master runs its own clock.
- Members follow the master: they ping its UDP command port, keep the good
  samples (round-trip gate, median, a weighted fit) and slew their clock gently.
  They step it only on first lock or when it is more than 1 s off.
- If the master is lost, a member holds its last rate (Holdover).

`Info Clock` (and the Clock section of `List Health`) shows:

| Field | Means |
|---|---|
| Role | Standalone, Master, or Member of a master (with its epoch) |
| State | Unlocked, Acquiring, Locked, Stable or Holdover |
| Session time / Local time | the two clocks now |
| Offset | difference to the master (µs) |
| Path delay | half the round trip to the master |
| Jitter | how much the samples scatter |
| Frequency | the rate correction in ppm |
| Correction | the slew applied right now (ppm) |
| Samples | used and rejected |
| Last sync, Steps | when it last synced; how often it stepped, and by how much |
| PTP | port state, offset, path delay, grandmaster — when linuxptp runs (read-only) |

On the master, every member's report is listed too.

## Threads

`Info Health Threads` lists every long-lived thread: alive or not, last
activity, loop count, last and max loop time, queue depth and panics. A thread
that stopped moving shows here first.

## For developers

- `/api/health` (ok / degraded) and `/api/metrics` on the HTTP port, and
  the `health.render` and `health` feeds on `/ws/feeds` (the Realtime window) — see
  [http-api.md](../../dev/api/http-api.md).
- Tracy and Perfetto zones per render stage — see
  [metrics-and-health.md](../../dev/build/metrics-and-health.md).

## Background

The Realtime view follows a common console's realtime page: on a probed 35,451-parameter
rig the channel calculation takes about 2.7 ms per frame, which set LumaCore's < 3 ms
budget. The clock statistics use PTP terms (offset from master, mean path delay)
so they read like a PTP tool's output (decisions §24).
