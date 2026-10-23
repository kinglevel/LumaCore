---
title: System Test
category: systemtest
---
# systemtest

> The demo show (the rig) loads once per worker thread and the cases run on one thread per
> core, so a full run is short; `build.rs` names what it watches.

`systemtest` is a Rust workspace member that tracks the current LumaCore build
and writes its generated metadata under `target/systemtest/`. It is one test suite
among several; see [docs/dev/testing.md](../../docs/dev/testing.md) for the others
(unit, property, sweep and fuzz, Playwright e2e).

## How it gets built

`systemtest` is listed in the workspace members in
[`Cargo.toml`](../../Cargo.toml).
Because of that, it is built by normal Cargo workspace commands:

```bash
cargo build --workspace
cargo build -p systemtest
cargo run -p systemtest
```

The crate has a build script in [`build.rs`](./build.rs). It tells Cargo to rebuild
`systemtest` when the command registry, dispatch or the demo's sources change (the list of
tracked files is in `build.rs`: the kernel's `commands/registry.rs` and `commands/mod.rs`, and
the demo's sources under `library/mvr/`, `library/poolxml/`, `library/demo/` — the show file itself
is built from them, archive §40 15).

## Why it writes into `target/`

The generated files are kept in the active build output tree so they always match
the current build, not stale checked-in files.

Build-time manifest:

```text
target/systemtest/current_build/command_manifest.json
```

Last run report:

```text
target/systemtest/last_run.json
```

## What it tests

`systemtest` has two layers:

- Curated state checks (`src/test_domains/`, one file per concern, assembled into named groups
  by `curated_groups()` in its `mod.rs`; a domain exports `pub fn tests() -> Vec<TestDef>`)
  - a fresh copy of the rig per test (`fresh_state()`), switched to the test's user
  - dispatch known commands
  - verify the resulting object tree, programmer state, watches, and path state
- Registry syntax sweep
  - walk command examples from the registry
  - run them against a fresh state
  - catch syntax regressions when command definitions move

## Running it

```bash
cargo test -p systemtest                  # the suite through #[test]
cargo run -p systemtest                   # the binary with its JSON report
cargo run -p systemtest -- --group=Health # one group
```

If a curated state check or syntax sweep fails, the process exits with a
non-zero code and writes the failure summary to `target/systemtest/last_run.json`
(`metrics.json` and `metrics_history.jsonl` beside it).

To add a domain: a new file in `src/test_domains/`, `mod` it in `mod.rs`, register it in
`curated_groups()` (an existing group or a new `GroupDef`), export `tests()`.
