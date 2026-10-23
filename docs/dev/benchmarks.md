# Benchmarks

`cargo bench -p lumacore-render --bench render` (decisions §24 "Enforcement", §31 "Render budget").
The show is `lumacore_render::bench_show` at 1,364 fixtures — 35,464 parameters, the size of the
reference rig probed for §24. Save a baseline with `-- --save-baseline main`; a later run with
`-- --baseline main` reports what got slower. The render budget test (`< 3 ms`, release) stays
the hard limit; a run 20 % slower than these numbers is worth a look; 1 ms per tick is the goal.

## Numbers of record

LumaCoreA (16 vCPU QEMU guest, 22 GB), release (`bench` profile), 2026-10-14, commit after
§31 step 8.

| Benchmark | What it measures | Time |
|---|---|---|
| `render_tick_35k` | one 60 Hz tick: masters, park, the cue's values | 0.42 ms |
| `render_tick_35k_input_64_layers` | the tick merging 64 DMX input layers (2 per universe on 32) | 0.70 ms |
| `render_tick_35k_half_fading` | the tick while 17,732 channels fade (a Go with a 600 s fade) | 0.47 ms |
| `compile_value_tier_35k` | a compile after a show change (the value tier; channel tier kept) | 70 ms |
| `compile_full_35k (channel tier rebuilt)` | a compile after a patch change | 92 ms |
| `compile_after_one_change_500_cue_tracking` | one programmer value changed, a 500-cue tracking sequence at its last cue | 75 ms |
| `publish_show_clone_35k` | the deep copy of the show the state thread publishes after every change | 4.9 ms |

After the merge keyed by attribute index (2026-10-14, same machine):

| Benchmark | Before | After |
|---|---|---|
| `compile_value_tier_35k` | 70 ms | 40 ms |
| `compile_full_35k (channel tier rebuilt)` | 92 ms | 66 ms |
| `compile_after_one_change_500_cue_tracking` | 75 ms | 42 ms |
| `resolve_by_attr_index_35k` (the merge alone) | — | 35 ms (`tracked_one_cue_35k` 6.4 ms of it) |

After sharing sequences (`Arc` per sequence, 2026-10-14): `publish_show_clone_35k` 4.9 ms →
0.23 ms (the bench show's 35k cue values are no longer copied per change).

With the compiler's sequence cache (what each running sequence offers, kept while its `Arc`
and current cue stay; values read in place, never copied):

| Benchmark | Start of §31 | Now |
|---|---|---|
| `compile_value_tier_35k` (cold: a Go, a new cue) | 70 ms | 44 ms |
| `compile_full_35k (channel tier rebuilt)` | 92 ms | 58 ms |
| `compile_cached_one_change_500_cue_tracking` (the compiler thread's case: a fader move) | 75 ms | 29.5 ms |

What is left in the merge is mostly the copy of every winning `Value` (its `Vec<Step>`) into
the plan, which readers (`Plan::resolved`) use.

What they say: the tick is well inside its budget (0.4–0.7 ms of 16.7 ms; the test's limit is
3 ms). The cost is per **change**: every published change recompiles the whole value tier
(~70 ms of the compiler thread, coalesced while it runs) after a ~5 ms copy of the show on the
state thread. That is what §31's "programmer snapshot, then `Arc` parts" and "pass the changed
subtrees to the compiler" are for.

## Builds (2026-10-14, LumaCoreA, 16 cores, `-j 6`)

Incremental, after touching one file in the station's `core/` (the usual edit; measured before §34 split the station into crates — its `core/` is now `lumacore-objects` and `lumacore-state`):

| Build | Wall time |
|-------|-----------|
| `cargo build -p lumacore-station` | 8.0 s |
| `cargo build -p lumacore-station --features http --bins` | 8.8 s |
| `cargo test -p lumacore-station --lib --no-run` | 7.3 s |

`cargo build --timings` puts nearly all of it in the `lumacore` crate itself. Fast enough: the
sweep and the terminal stay in the station crate (§23); measure again if this passes ~30 s.
