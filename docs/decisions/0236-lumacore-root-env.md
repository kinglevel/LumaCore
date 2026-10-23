---
id: 0236
title: Code reaches the repo root through LUMACORE_ROOT
area: build
status: built
date: 2026-10-16
built_in: 34f8
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §34 L8294–8296 (23.6), archive §34 L8786–8797]
---
# Code reaches the repo root through LUMACORE_ROOT

## Context
When the crates moved into their folders (34f8), every `../..` counted from a crate to the repo root broke at once — the demo show's sources, lum.md, the GDTF fixtures, the frontend kit, tidy's workspace scan and systemtest's harness all did it, each from a different depth.

## Decision
Code and tests that read files beside the crates use `env!("LUMACORE_ROOT")`, the one door to the repo root. It is set in `.cargo/config.toml` under `[env]`, relative to the workspace, and never counted as `../..` from a crate. Tidy finds a crate through `GROUPS` (`crate_root`, `crate_roots`) on the same base.

## Consequences
A crate can move between groups without touching its paths; a new test fixture, data file or script path is built on `LUMACORE_ROOT`. A `../..` in new code is a review finding — no tidy rule counts it today. Open: a tidy rule for it, if a copy slips in.
