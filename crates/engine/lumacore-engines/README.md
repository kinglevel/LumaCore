# lumacore-engines

The engine registry: capability-gated engines that start, stop and join against one deadline —
the render engine, the Perfetto trace engine and the HTTP UI engine.

## Purpose

The long-running services a station starts and stops as one set (decisions §26, §34 20.4). An
engine declares the platform capabilities it needs; the registry starts the ones the platform
offers with an `EngineContext` (the published show, the render hub, the command channel, the
health board), stops them in order and joins every thread against the shutdown deadline. The
wrappers live here because the context carries the render hub: a wrapper inside the renderer
would be a cycle. Protocols are not engines (`lumacore-lines`).

## Owns / Doesn't own

- **Owns:** `Engine`, `EngineContext`, `EngineRegistry`, `PublishedShow` (`registry`);
  `RenderEngine` (`render`); `HttpUiEngine`, `HttpConfig` (`http`, feature `http`);
  `PerfettoEngine` (`perfetto`, feature `perfetto`).
- **Doesn't own:** the render thread and plan compiler (`lumacore-render`), REST / WebSocket and
  the feeds (`lumacore-http`), the trace collector (`lumacore-trace`), which engines a station
  registers and the console bridge (the server).

## Public surface

- `EngineRegistry::{register, start_all, stop_all, engine_info}` — the set, started and stopped.
- `Engine` — what an engine implements (`start` / `stop` / `join(deadline)`).
- `RenderEngine`, `HttpUiEngine`, `PerfettoEngine` — the engines.
- `PublishedShow` — the show the state thread publishes, read by the engines.

## Rules

- Every engine thread is joined against the one shutdown deadline; a late one is named. [unchecked]
- An engine starts only when the platform has the capabilities it declares. [test registry_start_all_skips_unsupported]
- It depends on the renderer, the tree and the wire; HTTP only under its features. [compiler + tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-base, lumacore-tree, lumacore-wire, lumacore-trace, lumacore-render, lumacore-http.
- **Used by:** lumacore-station.

## Decisions

- [§26](../../../docs/archive/decisions-2026-10.md) — engine ownership
- [§27](../../../docs/archive/decisions-2026-10.md) — the render engine
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (20.4, 24.2), its README (18)

## Testing

- `cargo test -p lumacore-engines` — the registry (`tests/engine_registry_tests.rs`).
