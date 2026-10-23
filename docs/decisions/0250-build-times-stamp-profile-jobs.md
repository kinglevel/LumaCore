---
id: 0250
title: Git stamp in the station, local profile, jobs, bench rows
area: build
status: built
date: 2026-10-18
built_in: Build times built
supersedes: [archive §31 L6854–6857 "jobs = 2", archive §34 L8929–8931 "lumacore-luma build.rs stamp"]
superseded_by: —
confirmed_by: [`lumacore_station::stamp()`, `docs/dev/build-times.md`]
source: [archive §34 L10721–10774]
---
# Git stamp in the station, local profile, jobs, bench rows

## Context
A commit alone rebuilt most of the workspace in release: `lumacore-luma`'s `build.rs` stamped the git commit and re-ran on `.git/HEAD`, so everything above it recompiled. Real projects stamp at the top of the graph or outside compilation. A tiny build-info crate and a ship-only stamp were declined.

## Decision
The stamp lives in the station: `apps/lumacore-station/build.rs` stamps the commit (`-dirty` kept) → `lumacore_station::stamp()` (called by `install_catalog`, and first in the server's `main` so `--version` reads it before the catalog) → `lumacore_luma::set_git_commit` (a `OnceLock`; `build_info()` reads `unknown` without it, so tests and tools that skip the station say `unknown`). Never a `build.rs` or `env!` stamp lower in the graph. `[profile.local]` in the workspace `Cargo.toml` inherits release with incremental codegen; `scripts/unix/lib.sh` `lc_profile` builds it unless `--debug` (dev) or `--ship` (full release, `SHIP=1`); the launcher's default is `local`. Jobs are all logical cores minus two: `jobs` in `.cargo/config.toml`. Every build ends with one line (finished, duration, crates, profile), in the script's log too. `./scripts/unix/build bench [local|release|dev] [--yes]` asks, cleans that profile and times a full build of server, CLI and app, appending one row to `docs/dev/build-times.md` (committed; `--remote` runs it on the build servers). A build history and a launcher status row were declined.

## Consequences
A commit recompiles the station and relinks; a kernel change still rebuilds what is above it — the crate graph is the third lever (see objects-split-kernel-and-domain-crates). Measurements live in the rows of `docs/dev/build-times.md`, never in prose.
