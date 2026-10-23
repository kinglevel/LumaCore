---
id: 0234
title: The trace macros have one door, lumacore-trace
area: crates
status: built
date: 2026-10-15
built_in: 34f2
supersedes: []
superseded_by: —
confirmed_by: [shared_doors_have_no_copies]
source: [archive §34 L8113–8116 (20.9), archive §34 L8630–8639]
---
# The trace macros have one door, lumacore-trace

## Context
The Tracy and Perfetto zone macros were copied into the station and the DMX crate as two `telemetry.rs` files, each carrying its own `tracy` / `perfetto` features and a `tracy-client` dependency; a macro's `#[cfg]` is read in the caller's crate, so every crate that zoned had to forward the features.

## Decision
Every trace macro lives in `lumacore-trace` (`#[macro_export]`), with a recording and an empty form chosen by the trace crate's own `tracy` / `perfetto` features, so a caller needs no feature of its own; the station forwards its features to `lumacore-trace/…`. A render stage's Tracy zone is a `HeldZone` field set with `held_zone!`; `trace_zone!` lives to the end of its scope; a Perfetto event records with `record_complete` and never blocks. Render-thread zones stay allocation-free (see render-tick-rules). `shared_doors_have_no_copies` counts `tracy_client::` outside the crate and holds it at zero.

## Consequences
A crate that wants a zone depends on `lumacore-trace` and calls its macros; it never adds `tracy-client` or a telemetry module of its own. A new kind of trace event is a new macro here with both forms. Open: none.
