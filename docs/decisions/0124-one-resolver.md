---
id: 0124
title: Every target becomes objects only through core::resolve
area: resolve
status: built
date: 2026-10-15
built_in: 3b1–3d7
supersedes: [archive §33 L7421–7427 "five resolver families"]
superseded_by: —
confirmed_by: [tidy `old_resolvers_only_go_down`, E-CMD-007, E-CMD-008, E-CMD-020]
source: [archive §33 L7421–7450, archive §33 L7745–7777]
---
# Every target becomes objects only through core::resolve

## Context
A survey after 3b1 found five resolver families with different grammars (`parse_selection`, `number_list`, `set_rows::split`, `playback::parse_target`, per-object owner parsers), some twenty hand-written "number or name" lookups and six private `unquote` copies — so `Group 1 Thru 4` and a line's `1 Thru 4` followed different rules.

## Decision
The parser gives a `Target`; `core::resolve` turns it into objects and nothing else does. `resolve` gives each item's object kind, owner and members `(no, guid, name)` in typed order by the one range rule (see selection-ordered-one-range-rule), names case-insensitively, from what the registries declare (`ObjectDef::lum`, `TreeDef::children_in`); `resolve_one` takes one kind; `fixtures` is the typed view (`Group 1 + Fixture 7 - Group 2`, group members in their order). Scoping owners (`resolve_owned` / `with_owners`: an owner whose Props declare `within` — Sequence, Cue, UserProfile, Station) resolve to one row whose scope the next segment enters; rows come back with their owner chain. Store names a slot (`resolve::slot`, `slot_owned`: a number that may not exist, an existing name, a new name = the next free number, none = the next free; a range is refused; `Store Sequence 9 Cue 1` may name an owner not yet there). Nothing matched is one refusal, E-CMD-007; a destination past the last number E-CMD-008; another kind E-CMD-020. Quoted `*` patterns are its rule (see door-words-insert-compact-patterns). `Create` is gone — one name per word, Store.

## Consequences
No handler, command or module resolves ids or names any other way; a new lookup or `unquote` copy fails the ratchet `old_resolvers_only_go_down`, which only goes down and reaches zero in part 5. Multi-part ids and remaining owners join the resolver as their areas move, never as a handler-side lookup.
