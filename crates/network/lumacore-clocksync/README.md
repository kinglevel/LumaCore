# lumacore-clocksync

The session clock sync: a member disciplines its session clock to its master's from `Time` pings
on the master's UDP command port.

## Purpose

How every station of a LumaSession shares one time for fades, group-master fades and effect
anchors (decisions §24 "Session clock (researched)", §34 24.2: Ableton Link, Snapcast, nqptp,
PipeWire, FPP, RFC 5905, chrony, ntpd-rs, statime, linuxptp and game netcode were studied). A
burst of pings when it starts following, then one every 500 ms; a 32-sample filter (RTT 5σ
outliers dropped once, an RTT gate, the median offset, the RTT-weighted regression for the
frequency); the session clock overlay runs at 1 + frequency + correction, slows down rather than
running backwards, and steps only on its first lock or beyond 1 s; holdover when the master is
quiet. The stats are in PTP terms (`Runtime.Health.Clock`). Pure apart from its one socket and
thread: it knows nothing of the show, the state thread or LumaSession's rules.

## Owns / Doesn't own

- **Owns:** `answer` (the master stamps a `Time` ping at the socket), `follow` / `set_role` (whom
  to follow), `spawn_clock_sync` (the sync thread), `Filter`, `Sample`, `Estimate`,
  `correction_for` (`clock`).
- **Doesn't own:** the clock overlay itself (`lumacore_base::clock`), who the master is
  (`lumasession`), the command port (the server), linuxptp (`lumacore-net`'s `ptp`, read only),
  the System Monitor line it asks the caller to write (`warn`).

## Public surface

- `answer` — on the master's UDP command port, never through the state thread.
- `follow`, `set_role` — called by the state thread when the role or master changes.
- `spawn_clock_sync(stop)` — the sync thread, once per process; a step is logged to the System Monitor (`lumacore-sysmon`).
- `Filter`, `Sample`, `correction_for` — the filter and discipline (property-tested by the sweep).

## Rules

- Session time never runs backwards except the one decided step (first lock, or more than 1 s off). [unchecked]
- A ping is answered at the socket, never behind the state thread. [unchecked]
- It depends on base, tree and the wire only. [compiler + tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-base, lumacore-tree, lumacore-wire, lumacore-sysmon.
- **Used by:** lumacore-station.

## Decisions

- [§24 "Session clock (researched)"](../../../docs/archive/decisions-2026-10.md) — the design and its research
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (24.2), its README (18)

## Testing

- `cargo test -p lumacore-clocksync` — the filter, the discipline, a simulated master.
- The filter's properties: the sweep's `property_tests`.
