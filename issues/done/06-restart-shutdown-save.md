# 06 — Restart / Shutdown save outside the change path

**Severity:** low (process exits right after; shows only on rejoin/merge) · **Found:** sweep
2026-10-13 (`member` after `restart Universe`, `Restart Executor Screen`).

## Repro
A few lines, then `Admin  Restart` (or `Shutdown`) → member differs in ShowSettings.

## Cause
`Restart` / `Shutdown` are `undo: ReadOnly, scope: Station` (`registry.rs:~2088`, `~2119`). Their
save goes through `write_show` (`backup.rs:~198-213`: stamps `show_name`, `created_with_version`,
`format_version`, `saved_with_commit`) and runs `screens::sync` (can change Profiles); nothing
replicates. On a member they write the member's own ShowData copy.

## Options
1. Mark them `Untracked` so a change pass runs and the stamps replicate before exit (flush runs
   after the job). **Recommended.**
2. Make the save stamps station-local (not ShowData) — bigger.
3. Only silence the sweep (skip the member compare when the line ended the process).

## Decision
Option 1. **Fixed** (2026-10-13, on branch `bugfixes`).
