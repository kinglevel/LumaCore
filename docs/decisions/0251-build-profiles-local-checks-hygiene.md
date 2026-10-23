---
id: 0251
title: Build profiles, pinned toolchain, local checks, git hygiene
area: build
status: built
date: 2026-10-13
built_in: —
supersedes: []
superseded_by: —
confirmed_by: [`./scripts/unix/test check`, `rust-toolchain.toml`, `[workspace.lints]`]
source: [archive §31 L6854–6868]
---
# Build profiles, pinned toolchain, local checks, git hygiene

## Context
The code review hardening (§31) met a small box whose `target/debug` had outgrown the disk, no pinned toolchain, and a CLAUDE.md that had become a manual. §25 "No CI" stands: every check runs locally.

## Decision
`[profile.dev]` keeps `debug = "line-tables-only"` with dependencies at `opt-level = 1` without debug info; `.cargo/config.toml` sets the jobs and a fast linker (lld, mold where installed); the station's binaries are `test = false`; unused dependencies are removed (`cargo machete`). Checks stay local: `[workspace.lints]` with a short clippy list, a pinned `rust-toolchain.toml` (stable; nightly only in `tests/fuzz/`), and one `./scripts/unix/test check` (fmt, clippy with warnings as errors, every crate's tests), optionally a pre-push hook. Git hygiene: `library/shows/.autosave/`, `.versions/` and `.trash/` are ignored and the committed autosave files untracked.

Three lines of the archive text are superseded by later records: `jobs = 2` by the cores-minus-two rule (see build-times-stamp-profile-jobs); `unsafe_code = "forbid"` by `deny` plus tidy `no_unsafe_code` (see test-binary-installs-catalog-via-ctor); "`demo.luma` stays" by the gitignored, rebuilt demo (see demo-show-test).

## Consequences
A new crate inherits the workspace lints and profiles; a new toolchain is a change to `rust-toolchain.toml`, never a local override. The trimmed `CLAUDE.md` with path-scoped `.claude/rules/` files was decided here and not done then; the 2026-10-20 md upgrade (`docs/plans/active/md-upgrade.md`) carries it out (see docs-one-plan-archive).
