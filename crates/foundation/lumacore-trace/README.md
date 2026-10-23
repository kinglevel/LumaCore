# lumacore-trace

The Perfetto trace collector and the trace macros every crate zones with.

## Purpose

Profiling without a cost when it is off. The collector is a ring buffer of Chrome Trace Events
that the Perfetto engine serves as JSON; the macros put zones, plots, messages and frame marks
into Tracy (live) and Perfetto (offline). Which backend records is this crate's `tracy` /
`perfetto` feature: a crate that zones depends only on this crate, the top crate turns the
features on, and without them every macro expands to nothing — the render tick stays
allocation-free. The DMX crate records into it and the HTTP crate serves it, so neither depends
on the other.

## Owns / Doesn't own

- **Owns:** `TRACE_COLLECTOR` (`TraceCollector`, `TraceEvent`), `PerfettoGuard`, the JSON form
  (`format::event_to_json`), the trace macros (`trace_zone!`, `tracy_zone!`, `perfetto_zone!`,
  `held_zone!`, `tracy_plot!`, `tracy_message!`, `tracy_frame_mark!`), `HeldZone`,
  `record_complete`.
- **Doesn't own:** the Perfetto engine and its port (the station), the trace endpoints
  (lumacore-http), the render stages' names (`lumacore_tree::runtime::STAGES`), the health numbers
  (lumacore-runtime).

## Public surface

- `trace_zone!(name)` — a zone in both backends to the end of the caller's scope.
- `held_zone!("Name")` with a `HeldZone` field — a zone that ends when the field is replaced (a
  render stage).
- `tracy_plot!`, `tracy_message!`, `tracy_frame_mark!` — Tracy's plot, timeline message and frame
  view.
- `record_complete(name, category, start, duration)` — one hand-timed event; never blocks.
- `TRACE_COLLECTOR` (`init`, `snapshot`, `len`) and `format::event_to_json` — what the Perfetto
  engine and the HTTP trace endpoints read.
- `PerfettoGuard` — the RAII guard behind `perfetto_zone!`.

## Rules

- The trace macros live only here; no other crate names `tracy_client`.
  [tidy shared_doors_have_no_copies]
- Every macro expands in a caller with the features on and off. [test every_macro_expands_in_a_caller]
- A zone lives to the end of its scope, not of a block inside the macro.
  [test a_trace_zone_records_when_its_scope_ends]
- The collector never blocks the render thread: a busy collector drops the event.
  [test try_record_complete_drops_when_contended]
- It depends on no workspace crate. [tidy crate_edges_point_down]

## Edges

- **Depends on:** nothing.
- **Used by:** lumacore-render, lumacore-http, lumacore-station.

## Decisions

- [§23 "Step 8"](../../../docs/archive/decisions-2026-10.md) — the collector between dmx and http
- [§24 "Step 9 hardening"](../../../docs/archive/decisions-2026-10.md) — zones per render stage, nothing allocates in the tick
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — the macros' one home (20), this README (18)

## Testing

- `cargo test -p lumacore-trace` — the collector, the guard, the JSON form, the macros' empty forms.
- `cargo test -p lumacore-trace --features perfetto,tracy` — the recording forms.
- The whole station with its zones: `cargo build -p lumacore-station --features perfetto,http` and
  `--features tracy`.
