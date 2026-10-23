# 02 — A bare number skips the change path

**Severity:** high (replication drift, no Oops) · **Found:** sweep 2026-10-13 (`read-only-changed`
on `Join ; 101 …`, `Library ; 101 …`, `User Universe Window ; 101 …`; `member` on
`GlobalVar Off ScreenConfig -1`).

## Repro
`Admin  101` → changed `Selection(self)`; `Admin  101 At 50` → changes the programmer;
`Info ; 101` likewise. `Fixture 101 At 50` works correctly.

## Cause
`undo_of` (`core/commands/dispatch.rs:~198`) resolves the first word with `registry::resolve`;
`101` doesn't resolve, so the line is `Undo::ReadOnly` and dispatch (`dispatch.rs:~99`) opens no
change pass: no Oops step, no replication op, no journal entry, no Usage. The line still runs
`context_number` (`dispatch.rs:~679`) → `cmd_select_add` / `cmd_fixture_at`.
`registry::statement_scope` (`registry.rs:~3572`) already knows "a bare number selects in Fixture
context" and forwards it to the master — which then doesn't replicate the result. The rights
gate already treats it as Programmer.

## Docs
§26 / §21: every write goes through the one change path. Same class as the earlier finding
"changes the programmer while declared ReadOnly" (decisions ~line 2416).

## Options
1. In `undo_of`, `statement_scope` and `statement_reach`, treat a leading number as the word
   `Fixture` (in Fixture context). **Recommended.** Add a word test for `101 At 50`.
2. Rewrite a bare number to `Fixture <n> …` early in dispatch — context-dependent (in Universe
   context a number means Info Universe).

## Decision
Option 1. **Awaiting go.**
