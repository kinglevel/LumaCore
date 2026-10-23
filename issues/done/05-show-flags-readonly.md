# 05 — `Show` flags run as ReadOnly

**Severity:** medium (silent drift in a LumaSession) · **Found:** sweep 2026-10-13
(`read-only-changed`: `Show 1 /save` changed ShowSettings).

## In plain words
Every command declares whether it changes the show. That decides whether it gets an Oops step,
whether it is replicated, and whether a member forwards it to the master. `Show 1 /save`,
`Show … /load`, `Show /Description "…"`, `/Autosave`, `/Merge…` are declared "only reads" but
change ShowSettings (saving stamps format version, commit, name). On one station Oops doesn't
know; in a LumaSession the member changes only its own copy and the stations quietly disagree
until the next merge.

## Cause
`backup.rs:~841` `call_undo: ReadOnly`; `show_call` (`:~782`) routes `/save` to `cmd_save_show`
(stamps at `:~206-213`), `/load` to LoadShow, `cmd_show_option` (`:~590`) writes description,
autosave, merge settings. `statement_scope` makes a ReadOnly object Client (`registry.rs:~3571`).

## Options
1. `call_undo: Untracked` (like `SaveShow`): forwarded to the master, replicated, no Oops step.
2. Move `/Description`, `/Autosave`, `/Merge*` to writable `Set Show …` tree props
   (`core/props.rs`), keep `/save` `/load` as Untracked spellings — the usual pattern.

## Decision
Proposed: 1 now, 2 on the roadmap. **Fixed** (2026-10-13, on branch `bugfixes`; 2 is on the roadmap).
