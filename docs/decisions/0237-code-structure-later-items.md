---
id: 0237
title: Crate split after measuring; AppState grouped gradually
area: structure
status: superseded
date: 2026-10-13
built_in: —
supersedes: []
superseded_by: layers-point-down
confirmed_by: none
source: [archive §31 L6843–6851]
---
# Crate split after measuring; AppState grouped gradually

## Context
The code review hardening of 2026-10-13 closed its structure items with three deferrals: a `Walk` derive only if a field without `#[walk(pool|link|data)]` ever fails to compile (classification stays compile-time, see guids-everywhere-links); `AppState`'s fields grouped into sub-structs one area at a time, session link and change bookkeeping first, every field keeping its one home; and crates only after build profiles and `cargo build --timings` showed builds still hurt — then the sweep and the terminal as leaf crates, the small data crates kept to grow, `lumacore-tree`'s one-line re-export shims removed.

## Decision
Superseded two days later. §34 cut the station into crates without waiting on a measurement — state, objects, tidy, terminal, views, the engines, LumaSession, the server and the sweep each became a crate in its group (see layers-point-down, station-code-becomes-crates, station-only-assembles), and the build-time work came after as its own round. The `Walk` derive is still not needed, and grouping `AppState` by area remains the standing manner of working on it; neither is a rule of its own.

## Consequences
Nobody waits for a build measurement before cutting a crate whose edges are clean; the measurement rounds (`./scripts/unix/build bench`, `docs/dev/build-times.md`) stand on their own. An `AppState` area is grouped when it is next worked on, never in a sweep. Open: none.
