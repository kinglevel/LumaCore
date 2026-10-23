---
id: 0113
title: Options are declared; an unknown flag is refused
area: flags
status: built
date: 2026-10-14
built_in: code review 2026-10-14 (`core::commands::flags`), lum part 1 (`/Name value`), 3d3 (one name per option)
supersedes: [archive §31 L6805–6808 "flags::has from args", archive §31 L6809–6810 "every /x swallows", archive §33 L7136–7137 "the Rest form", archive §33 L7138–7144 "quoted-line examples"]
superseded_by: —
confirmed_by: [E-CMD-018, E-SYN-006, `core::commands::flags`, `FlagDef::choices`, the sweep, help Options and the reference]
source: [archive §31 L6783–6814, archive §33 L7134–7144 (items 19, 20)]
---
# Options are declared; an unknown flag is refused

## Context
A reference console silently ignores an unknown option (`Delete Group 9999 /Foo` deletes), so a typo like `/Overwite` changes nothing visible; LumaCore's handlers each swallowed the word after any `/x` and some accepted a bare `nc`.

## Decision
Every option is declared on its word (today a field of the `#[derive(Args)]` struct, see typed-handlers-derive-args): with or without a value, its rights, its doc, its choices (`FlagDef::choices`). Completion after `/`, help's Options, the reference and the sweep read the declaration. A `/Name` word changes how a line runs, never what it does; the value is the next single word or quoted text, no `=` (`/Port 8000`, `/Tab "Lines In"`); multi-word values are quoted; an option declared without a value never takes the next word (`/Overwrite 3` leaves the 3). One name per option: `/o` is refused, `/Overwrite` runs.

The global options valid on every line are the ones lum.md's Options section lists (`/NoConfirmation`, `/NoOops`, `/Verbose`, `/Debug`, `/R`); a word's own option wins. Any other unknown option is refused for every statement before any runs — before forwarding, the asks and a key line's save — naming the options the word takes (E-CMD-018 in dispatch, E-SYN-006 from the parser). A quoted `"/x"` is always a value (see quoting-values-options); a bare `nc` is never `/nc`.

One list of things per statement; every extra carries a `/label`: files, sources and descriptions are options (`Import Patch + Users /From "Festival"`, ``Bind "F5" `List Fixture` /Description "Lists all fixtures"``); no verb keeps its own positional sub-shape.

## Consequences
No `/word` string comparison lives in `core/` outside `flags.rs`; never a handler-local option list. The old global `/Watch` is gone (lum.md's list is the globals). E-CMD-018 retires into E-SYN-006 with the shim (see lum-grammar-one-parser).
