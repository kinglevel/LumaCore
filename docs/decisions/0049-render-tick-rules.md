---
id: 0049
title: The render tick walks arrays only
area: render
status: built
date: 2026-10-13
built_in: step 9
supersedes: [archive §24 L2244–2250 "the first tick rate", archive §24 L2354–2357 "LumaRuntime, HealthReport"]
superseded_by: —
confirmed_by: [`assert_no_alloc`, the tidy render regions, the budget test, `cargo bench -p lumacore-render --bench render`, `Info Health Threads` `scheduling`]
source: [archive §24 L2244–2250, archive §24 L2274–2276, archive §24 L2288–2317, archive §24 L2354–2381, archive §24 L2382–2399, archive §31 L6709–6713, archive §31 L6714–6723]
---
# The render tick walks arrays only

## Context
Step 9 surveyed the render path against a reference console's realtime loop (dense tables rebuilt on change, per-stage timing, overload). Before it the state thread rendered with allocation, and thread priority was only a nice request that failed without rights.

## Decision
One render thread ticks at the engine's fixed rate on an absolute deadline (`spin_sleep`). The tick (`lumacore_render::render::render_into`, `engine::Engine::tick`) and the drivers' per-frame send walk the compiled `Plan`'s arrays only: no allocation, no lock, no logging, no map or string lookup, no `SystemTime` (time is the session clock), and they free nothing — retired plans, frames and inputs go back to the compiler thread. Plans are built on the `plan_compiler` thread in two tiers: the channel tier rebuilt only when `AppState::patch_generation` moves, the value tier per change; the engine swaps at frame start (ArcSwap); the state thread never compiles or renders. Commands read `AppState::output_frame` / `output_plan`, waiting for the frame carrying their generation. A tick panic holds the last frame and restarts, a second within the window leaves it down; a compiler panic keeps the last plan, latches `stale_since`, and repeated panics stop the compiler until a show load (Q39). Overload skips the late frame, never catches up; counted per stage, reported once per episode (`RenderWatch`). The engine asks for `SCHED_FIFO` (`lumacore_render::realtime`, drivers a little lower); `scripts/unix/realtime.conf` grants it; `Info Health Threads` shows what each thread got.

## Consequences
Checked three ways: `assert_no_alloc` in debug, the tidy regions `// tidy: render-begin … render-end`, and the budget test plus the bench — the budget's bound stays where the test holds it; a lower budget is a goal, never a failing test. Decided, not built: baselined benchmarks for the missed paths that warn when slower; RTKit as the desktop fallback.
