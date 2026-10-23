---
id: 0111
title: One registry, one namespace, one name per word
area: registry
status: built
date: 2026-10-15
built_in: one registry (2026-10-10), BASE rows (code review 2026-10-14), aliases gone per area (lum parts 3b–3d)
supersedes: [archive §14 L1128–1131 "Abbreviation System", archive §15 L1159–1166 "short forms aliases prefix", archive §15 L1151–1152 "not yet implemented", archive §33 L7300–7306 "Patch DMX verbs", archive §33 L7793 "Create alias Store"]
superseded_by: —
confirmed_by: [tidy keyword rule (a match naming two registered words outside the registry), test `planned_words_reply_planned_and_change_nothing`, E-CMD-003, E-SYN-008, E-SYN-010, golden `tests/catalog/testdata/lum_words.golden.txt`, tidy `every_command_states_what_decides_it`]
source: [archive §33 L7057–7076 (item 12), archive §33 L7300–7306 (item 48), archive §33 L7340–7342 (item 52), archive §15 L1143–1170, archive §21 L3244–3247, archive §21 L3266–3271, archive §31 L6838–6842]
---
# One registry, one namespace, one name per word

## Context
Dispatch once matched verb strings, help lived in `legacy_help.rs`, short forms sat in an abbreviation table beside registry aliases and prefix expansion. Prefix plus hand-picked shortcuts, clashes fixed by hand per release, is not copied.

## Decision
`CommandDef` (verbs) and `ObjectDef` (objects) are separate rows in one catalog sharing one namespace: every name means exactly one thing across both; an object is never also a command; one concept is one word (`Patch`, `DMX` are objects only). `freeze()` checks the namespace. Every reader — dispatch, completion, help, rights, undo, TUI hotkeys, the reference — reads the rows; no rogue list, no `match` on keyword strings outside the registry.

One name per word: no built-in short forms, no prefix guessing in a line; a new word never changes what an old line means. `Create` folded into `Store`, `Unpatch` into `Unassign`. Typing is fuzzy, lines are exact: while typing, each word matches fuzzily and the full line shows as ghost text (`str gro 3` → `Store Group 3`); Enter runs the line shown; an ambiguous word is E-SYN-010. Stored and machine lines are exact — a fuzzy word in one is refused on save (E-SYN-008).

A word not yet built is published `status: Planned` (an `ObjectDef` verb too); running it replies "isn't built yet (planned)" (E-CMD-003) and changes nothing; the reference shows it.

Rows state only what differs from `CommandDef::BASE`; every row still names `verb`, `doc`, `run` and the four that decide what a line may do — `undo`, `scope`, `rights`, `reach` (BASE holds the strictest).

## Consequences
Fuzzy Enter (decision 12) is not built; `lumacore-shutoff` folding into `Shutdown` is decided, the binary still ships; rows still filling the registry's `aliases` field are decision 12's remaining debt. `Watch`'s verb-or-object clash: see watch-is-a-verb.
