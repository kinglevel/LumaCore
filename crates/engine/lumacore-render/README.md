# lumacore-render

The one renderer: the merge, the plan compiler and the 60 Hz render tick on its own thread.

## Purpose

Turns the published show into DMX frames (decisions §27, §24 "Step 9 hardening", §34 20.3). The
state thread publishes an `EngineInput` into the `RenderHub`; the plan compiler thread runs the
merge (`merge::resolve`: park > frozen programmers > LTP of programmers and running sequences >
defaults) and compiles a two-tier `Plan` (the patch as dense slots, the values with fades, masters
and echoes); one 60 Hz thread renders the plan, the session clock and DMX input into
`RenderedFrame`s handed out by ArcSwap. `lumacore-engines`' `RenderEngine` starts it;
the line drivers sample its frames.

## Owns / Doesn't own

- **Owns:** `EngineInput`, `UserLayer`, `Winner`, `Owner`, `resolve`, `releases` (`merge`),
  `Plan`, `ChannelTier`, `ValueTier`, `Fade` (`plan`), the compiler thread (`compiler`), the tick
  `render_into` / `render_now` and `RenderedFrame` (`render`), `RenderHub`, `spawn_engine`,
  `EngineHandle` (`engine`), thread priority (`realtime`), the bench show (`bench_show`).
- **Doesn't own:** the input merger (`lumacore-protocols`), the wires (the protocol crates), sending
  and receiving lines (`lumacore-lines`), the engine registry and its wrapper (`lumacore-engines`), the
  show data and running playbacks (`lumacore-tree`), the session clock (`lumacore-base`).

## Public surface

- `RenderHub` — publish an `EngineInput`, read the newest plan and frame, attach inputs.
- `spawn_engine` / `EngineHandle` — start and stop the render and compiler threads.
- `merge::resolve` — the one merge (also used by readouts).
- `Plan`, `RenderedFrame`, `render_now` — what commands read back (`AppState::output_frame`).
- `RENDER_HZ`, `FRAME_PERIOD`.

## Rules

- The tick allocates, locks, logs and frees nothing; replaced plans and frames go back to the
  compiler thread. [tidy render_paths_never_lock_log_or_allocate + test
  the_tick_allocates_nothing]
- A 35k-parameter frame renders in under 3 ms (release). [test
  a_35k_parameter_frame_renders_under_3_ms]
- DMX bytes exist only in the tick; everything before it is `Value`. [unchecked]
- A late frame is skipped, never caught up; a panic holds the last frame. [unchecked]
- No tokio in the DMX path. [compiler]

## Edges

- **Depends on:** lumacore-protocols, lumacore-base, lumacore-tree, lumacore-trace, lumacore-registry, lumacore-fixtures.
- **Used by:** lumacore-lines, lumacore-state, lumacore-objects, lumacore-views,
  lumacore-station, systemtest.

## Decisions

- [§24 "Step 9 hardening"](../../../docs/archive/decisions-2026-10.md) — the render plan, the audio-style tick
- [§27](../../../docs/archive/decisions-2026-10.md) — the render engine owns DMX, the merge
- [§29, §30 "12f"](../../../docs/archive/decisions-2026-10.md) — cue timing, masters, echoes
- [§31 "Render budget"](../../../docs/archive/decisions-2026-10.md) — the budget and the benchmark
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (20.3, 20.4), its README (18)

## Testing

- `cargo test -p lumacore-render` — the merge, the plan, the tick, the engine (debug: the tick
  allocates nothing).
- `cargo test -p lumacore-render --release a_35k` — the frame budget.
- `cargo bench -p lumacore-render --bench render` — numbers of record in docs/dev/benchmarks.md.
