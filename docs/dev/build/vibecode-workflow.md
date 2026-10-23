---
title: Vibecode Workflow
category: docs
---
# Vibecode Workflow

Canonical language source: `docs/user/syntax/lum.md`

The working habit for developers in LumaCore: build and test per step, the long run by hand,
and the companion workspaces named in the gitignored `.env.local`
([build-environment.md](build-environment.md)). First setup is
[getting-started.md](../../getting-started.md).

## Build & Verify

While building a step, test only what the step touches: `cargo build` plus the touched
crates' tests, filtered by name, and the e2e files that drive the feature. The gate — the
full e2e, tidy, the catalog's tests — runs once per step before the docs and the commit
([testing.md](../testing.md) "What to run when").

```bash
cargo build                                   # what you changed builds
cargo test -p <crate> <area>::                # that area's tests
cargo clippy -p <crate> --all-targets         # before a commit
```

The demo show (`library/shows/demo.luma`, gitignored) is the rig, built by
`lumacore_objects::demo::build` through the same command lines a user types from the sources in
git (`library/mvr/ti2026.mvr`, `library/poolxml/ti2026-groups.xml`, `library/demo/manual.lum`); the
station's start, the e2e setup and the catalog test build it when it is missing or stale
(decisions §40 15). By hand:

```bash
cargo run -p lumacore-station --bin gen-demo              # always
cargo run -p lumacore-station --bin gen-demo -- --if-stale  # only when missing or stale
```

### The overnight long run

Fuzzing and the command sweep are a **long run only**, never part of per-step tests
(decisions §25 "Step 10 hardening"). Start it by hand, e.g. before leaving for the night:

```bash
./scripts/unix/fuzz_long            # 30 min random + 30 min per fuzz target
./scripts/unix/fuzz_long 60 --fresh # longer, starting over
```

It sweeps every command × keyword × flag × writable property as one user per rights level (generated from
the registries — nothing to update when a command is added), runs random command sequences
and every cargo-fuzz target (nightly), then the property tests with a random seed. Ctrl-C
stops it; running it again continues. Read `target/fuzz-report/summary.md` in the morning;
`docs/dev/testing.md` explains the report.

## Metrics & Health

Every runtime-critical feature must include timing instrumentation.

Before merging code that touches fast loops, network sync, or rendering:

1. Run the area's tests and the render budget test (`cargo test -p lumacore-render --release`)
2. If adding a new fast-loop subsystem, add Tracy zones gated behind `#[cfg(feature = "tracy")]`
3. Check `Info Health` / the realtime window for unexpected jumps

See [`metrics-and-health.md`](metrics-and-health.md) for details.

## Goal

LumaCore should not be developed in isolation.

When working on:

- object tree design
- shell UX
- session management
- realtime/runtime systems
- DMX, media, audio, 3D, or networking engines
- system tests and diagnostics

the developer should first scan related local repos and look for similar or
better solutions before implementing a new approach.

The expectation is:

1. find relevant reference repos from `.env.local`
2. inspect how they solve the same class of problem
3. compare their approach to LumaCore
4. check that the result follows `Lum`, `DataPool`, and `ObjectTree` syntax
   and structure
5. suggest improvements before or while implementing

## Using `.env.local`

`.env.local` (gitignored; copied from `.env.example`) contains developer path variables in
this format:

```text
NAME=/absolute/path
```

Example:

```text
SKILL=../my-reference-workspace
```

These paths point to local companion workspaces that contain:

- research notes
- research docs
- source repos
- shell/runtime/object-tree/tooling references

Developers should read `.env.local` first when they need external local context.

Important:

- companion workspaces may also contain their own `.env.local` or `paths.txt`
- those nested configuration files can point to even more useful source trees
- developers should check them as part of the discovery flow
- the definitions rule: everything follows `lum.md` and the object tree; what doesn't is a
  discussion first, never built on the side

## The `SKILL` Workspace

The companion workspace is your own:

```text
SKILL=../my-reference-workspace
```

Organise it as you like; a layout by domain under `build/` works well.

It also has its own configuration file (`.env.local` or `paths.txt`) that can
expose additional data sources and source trees, so developers should treat it
as part of the normal workflow.

High-value examples:

- `build/tools/shell`
- `build/tools/object tree`
- `build/tools/system`
- `build/system/kernel`
- `build/protocols`
- `build/3D`
- `build/tools/audio`
- `build/tools/integration`

When a feature touches one of these domains, the developer should search this
workspace for comparable repos and patterns before deciding on a design.

Nested path variables follow the same pattern. If a
companion repo has its own `.env.local` or `paths.txt`, developers should inspect it and use
those extra references when they are relevant.

## Required Research Habit

Before making a major design decision, the developer should actively look for:

- a similar solution
- a better solution
- a safer solution
- a more scalable solution

This applies especially to:

- terminal UX
- command completion
- session managers
- runtime kernels
- object tree layout
- multiplayer/network session behavior
- diagnostics and crash handling
- automated testing strategies

The developer should then:

1. summarize what was found
2. compare it to the current LumaCore approach
3. check it against `Lum`, `DataPool`, and `ObjectTree`
4. recommend whether to keep, adapt, or replace the current design

## Expected Output

When using external local repos as references, the developer should give:

- the repo or file used as reference
- the part of the problem it helped with
- what LumaCore should borrow
- what LumaCore should avoid
- a forward-looking suggestion, not just a copy of the old design

## Principle

LumaCore is building the future, but future-facing design should still learn
from strong existing systems.

Use local reference repos often.
Compare before inventing.
Prefer better patterns when they exist.
