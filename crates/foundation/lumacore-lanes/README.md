# lumacore-lanes

A state thread's two job lanes: the `StateHandle` every other thread uses to reach the one
context, its shell and machine lanes, the waits, and the busy watchdog.

## Purpose

The one way into a state thread (decisions §26: one thread owns the context, no global lock;
§31: two lanes). A job waits in the shell lane (unbounded, never dropped, served first: a
person's keys, lines and answers, the station's own work) or the machine lane (bounded at 256,
refused at once when full: HTTP, feeds, the UDP command port). `call` waits for the result,
`call_within` at most a while — a job not started by then never runs (R1-67, Q40) — and `send`
doesn't wait; a call from the state thread itself is refused instead of a deadlock. The watchdog
warns once per episode when one piece of work runs past 5 s. Generic over its context: the
station's `ServerContext` is the one `Host`, and the waits are the caller's policy.

## Owns / Doesn't own

- **Owns:** `StateHandle` (`call`, `call_within`, `send`, `machine`, `shell`, `busy_for`,
  `on_busy_change`), `Lane`, `MACHINE_LANE`, `Lanes` (the receiving end), `Busy`, `CallError`,
  `Job`, `Host`, `enter_state_thread` (`handle`); `spawn_watchdog` (`watchdog`).
- **Doesn't own:** the loop that runs the jobs and what it does between them (the server's state
  thread), how long a caller waits (`HTTP_WAIT` in the server, `SHELL_WAIT` and `SUGGEST_WAIT` in `lumacore-shell`), the
  panic report (the host's guard), where the watchdog's line goes (the caller's `warn`).

## Public surface

- `lanes(on_refused)` — a new state thread's handle and receiving end.
- `StateHandle::{call, call_within, send}` — reach the state thread.
- `Host` — what a context does around a job (its panic guard, its health entry).
- `spawn_watchdog(handle, alive)` — one System Monitor warning per busy episode (`lumacore-sysmon`).

## Rules

- A call from the state thread itself is refused, never waits on itself. [test a_call_from_the_state_thread_is_refused_not_a_deadlock]
- A full machine lane refuses at once and counts it; the shell lane never refuses. [test a_full_machine_lane_refuses_at_once_and_counts_it]
- A job not started within its wait never runs late. [test a_call_that_waits_too_long_never_runs_late]
- It depends on base only. [compiler + tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-base, lumacore-sysmon.
- **Used by:** lumacore-station.

## Decisions

- [§26](../../../docs/archive/decisions-2026-10.md) — one state thread
- [§31](../../../docs/archive/decisions-2026-10.md) — two lanes
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (19.2, 24.2), its README (18)

## Testing

- `cargo test -p lumacore-lanes` — calls, panics, a full lane, a late job, a reentrant call, on a
  small test host.
- The station's own state thread: its `server::state` tests.
