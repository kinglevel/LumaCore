# 08 — Show words walk into `Drives` through the context

**Severity:** medium · **Found:** sweep 2026-10-13 (`drive`: `Enter /broken 3 root` → "a Show
line reached the disk: list /").

## Repro
`Admin  cd Drives`, then `Admin  Enter /broken` → E-SET-004 after reading `/`.

## Cause
`registry::line_reach` is context-free — it sees only text naming Drives. `Enter` is
`Reach::Show`; its target is resolved against the navigation path `Root.Drives`, and
`resolve_drive_path` (`core/vfs.rs:~255`) `read_dir`s `/` to match the segment.

## Docs
§29: Quicksheet, List, Enter and Watch "never walk into it unless the path names it" — here the
context names it.

## Options
1. Reach includes the context (`line_reach(line, state)`): under Drives a Show word counts as Drive.
2. Show words never resolve relative to a Drives context: Enter / List / Info refuse ("use the
   tree shell", E-CMD-010) unless the path itself names Drives. **Recommended** (matches §29).
3. Sweep-only: reset the context after Drive lines — hides it.

## Decision
Option 2. **Awaiting go.** Related: issue 01 (who may enter Drives at all).
