# 07 — Quoted arguments: options vs paths and strings

**Severity:** medium (syntax) · **Found:** sweep 2026-10-13 (`drive-escape`: `export Fixture 101
/user` → write `/user`; `Import /save Reference` → read `/save`).

## Cause
Dispatch hands `/flags` to handlers unfiltered; each handler filters them (`backup.rs:65`,
`users.rs:134` reply "Unknown option"). `handle_export` (`core/commands/info.rs:~990`) takes
`args[2]` as the filename, `handle_import` (`core/commands/import.rs:~49`) `args[0]` as the path.
On unix `/user` is a valid absolute path. The lexer knows `Kind::Word { quoted }` but dispatch
drops it (`dispatch.rs:~755`), so no handler can tell `/user` from `"/home/x.gdtf"`.

## Direction (user, 2026-10-13)
One quoting rule for every string a function takes — filenames, paths, names, free text: carry
the lexer's `quoted` mark through dispatch. A bare `/word` is always an option; a quoted value is
always a value. Handlers take paths and free text only from quoted arguments (or the value after
an option). `SetUserVar` (later) uses the same rule.

## Before building
- Probe a reference console: quoting of names, filenames (`Export Group 1 "file"`), option
  values (`/Path "…"`), `SetUserVariable "name" "value"`, `$name` use. From memory consoles
  quote anything with spaces/specials and filenames.
- Write it as a decision in §15 (grammar), then build: dispatch arg type with `quoted`, every
  handler that takes a path/name/text, completion and hints, generated reference.

Interim (optional): Export / Import refuse an unknown `/x` with E-CMD-006.

## Decided (2026-10-13, after the probe)
- **As established consoles do**: the lexer's `quoted` mark is carried through dispatch; a quoted word is always a
  value (`"All"`, `"/nc"` are names); a bare `/word` is an option; a one-word name may be unquoted
  (keywords win over unquoted names); multi-word names must be quoted.
- **`;` inside quotes is kept** (one statement, the text as typed). Proper nesting of `"…"` and
  `'…'` matters for a later `Lua "…"` command (a quote of the other kind inside).
- **Filenames** are relative to the right library folder; sub-folders are allowed
  (`"test/hej.json"` → `<library folder>/test/hej.json`, created); `..` and absolute paths are
  refused (E-CMD-008). Other places stay reachable through the tree shell / Drives.
- **Variables** (`SetUserVar`, `$name`): roadmap — variable syntax and a syntax hardening pass after
  the step 12 UI work.

## Built (2026-10-13, branch `bugfixes`)
Quoted words reach handlers with their quotes (`tokens::args`, `tokens::unquote`); Label refuses an unquoted multi-word name; `drive::library_file` for Import, Export, Import Image and UserProfile files. Tests: `core/commands/quoting_tests.rs`. Decision text: decisions §15 "Quoting".
