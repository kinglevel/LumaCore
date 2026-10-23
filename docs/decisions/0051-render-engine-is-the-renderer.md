---
id: 0051
title: lumacore-render holds the plan and the tick
area: crates
status: built
date: 2026-10-16
built_in: 34f5
supersedes: [archive §34 L8091–8095 "lumacore-dmx keeps wire"]
superseded_by: —
confirmed_by: [the budget tests in `engine/tests.rs`, `benches/render.rs`]
source: [archive §34 L8091–8099, archive §34 L8740–8757]
---
# lumacore-render holds the plan and the tick

## Context
`lumacore-dmx` carried the wire, the merge, the render tick and the engine thread together, and the engine was still called `PlaybackEngine` from step 4a. The station split asked that crates split along their own imports and that names say what a thing is.

## Decision
`crates/engine/lumacore-render` holds the plan, the compiler, the render tick, the engine thread, realtime scheduling, the bench show, the merge (`merge.rs`), the budget tests (`engine/tests.rs`) and `benches/render.rs`. `RenderEngine` is the one renderer; "playback" names the data (`lumacore-playback`) and running playbacks only. Its hub is `RenderHub` (`EngineContext.render`, `Published.render`), the render thread's health row `Render.Tick`, its OS thread `render_engine`; help and docs say "render engine". The `Engine` wrapper stays in `lumacore-engines` (`engines/render.rs`, name `render`) beside the perfetto and http wrappers, because the `Engine` trait's context carries the render hub — a wrapper inside render would be a cycle. Edges today: protocols ← render ← lines; the state crate sits on render, objects and views on render; the protocol crates never see render (see protocol-crates-declare-themselves). `lumacore-dmx` is gone, folded into `lumacore-protocols`.

## Consequences
Code that compiles or renders lives in `lumacore-render` and nowhere above; code that opens a socket never lives there. A new engine gets its wrapper in `lumacore-engines`. Renaming the tick's stages changes `Runtime.Health` rows and the Realtime window together. Nothing here is open.
