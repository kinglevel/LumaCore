# lumacore-sysmon

The System Monitor: the station's logging system — the log, feedback events and panic reports.

## Purpose

Everything that happens on a station is a System Monitor line: logins,
every user's commands, show changes, warnings and errors (decisions §26, §30 "12b2"). The log
keeps a ring of lines, writes the daily log file, and hands structured lines to subscribers (the
`sysmon` feed on `/ws/feeds`, TCP monitor clients, `--RootMonitor`). Feedback events — progress
pills, flashes, messages, statuses, client presence — ride the same lines, so the log sees
everything the UI shows. A panic anywhere gets a report number, its location and backtrace in the
log, and `guarded` work gets the report back to roll back and reply. It knows no domain, so it
sits in foundation and every crate can write to it (decisions §34 25.10).

## Owns / Doesn't own

- **Owns:** `SysmonLine`, `Level`, the ring, subscribers, the log file, echo (`monitor`); the
  feedback events and their categories, levels and surfaces (`feedback`); the panic hook,
  report numbers, `guarded`, `PanicReport` (`panic_report`).
- **Doesn't own:** what reads the show for feedback (a profile's surfaces, the station statuses:
  `lumacore-state`'s `show_feedback`); rendering a panic's reply with its E-code
  (`lumacore-state`'s `errors`); drawing feedback (the kit, the TUI).

## Public surface

- `sysmon_note`, `sysmon_line`, `sysmon_early`, `subscribe`, `history`, `start_log_file` — the log.
- `feedback::{flash, message, status, Progress}` — feedback events.
- `panic_report::{install, guarded, PanicReport}` — panic reports.

## Rules

- It depends on no workspace crate. [tidy crate_edges_point_down]
- The render tick never writes to it (no logging in the tick). [tidy render_paths_never_lock_log_or_allocate]
- Importance is set by the emitter, never by users. [unchecked]

## Edges

- **Depends on:** nothing.
- **Used by:** lumacore-lanes, lumacore-clocksync, lumacore-state, lumacore-objects, lumacore-views, lumacore-shell, lumacore-lumasession, lumacore-server, lumacore-sweep.

## Decisions

- [§26](../../../docs/archive/decisions-2026-10.md) — internal-error reports
- [§30 "12b2"](../../../docs/archive/decisions-2026-10.md) — the feedback system
- [§34 25.10](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate

## Testing

- `cargo test -p lumacore-sysmon` — the ring, feedback statuses, guarded panics.
