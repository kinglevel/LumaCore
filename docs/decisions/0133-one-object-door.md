---
id: 0133
title: One pool door for every write; next free first unused
area: door
status: built
date: 2026-10-15
built_in: 35a–35g3
supersedes: [store-cue-default-object, archive §34 L8017–8020 "after the last", archive §35 L10919–10985 "per-object insert/remove copies", archive §35 L11027–11040 "E-GROUP-002 /Force", archive §35 L11078–11087 "move_cmd gone"]
superseded_by: —
confirmed_by: [E-CMD-020, E-USER-002, `pools::door`, `PoolRules`]
source: [archive §35 L10919–10985, archive §35 L11027–11040, archive §35 L11060–11087, archive §34 L8017–8020, archive §35 L10963–10966]
---
# One pool door for every write; next free first unused

## Context
Every object wrote its own insert / remove / rename, the next free number came in two flavours (first unused, "after the last"), and each verb's occupied-slot rule differed by object. Research (consoles, PowerShell providers, `rename(2)`, Kubernetes' store) pointed one way: a generic verb, per-type rules declared once, ids minted only in the generic layer, a move never copy + delete.

## Decision
`pools::door` in `lumacore-state` is the one write path for every collection (`BTreePool<T>`, named by `TreeDef.door`): `next_free`, `next_free_block`, `spacing`, `insert`, `duplicate`, `relocate`, `compact`, `delete`, `users_of`, the coded replies and the one existing-object ask; `commands/door` in `lumacore-objects` holds one file per verb. Per-pool rules are declared and compile-checked (`PoolRules`: `can_delete` — the locked Default profile, E-USER-002 —, `after_change`, `capture`, optional `merge`, the key type); a verb a pool can't take isn't published. Identity is minted only in `insert` and `duplicate`: a copy renews every GUID `Walk` marks own, links kept; Move, Swap, Label and Overwrite keep the GUID. Move onto a taken slot swaps, between owners of one kind too; another kind is E-CMD-020. Store / Copy / Paste onto an existing object ask once from the door — Merge (where the pool declares it) / Overwrite / Remove / Cancel — and `/Merge` `/Overwrite` `/Remove` answer it; a stored line without a flag is refused. Delete in use asks once naming its users (see oops-categories-statements). The next free number is the first unused one for every pool, sequences included; a cue appends after the last; `next_free_block` finds the first block a range fits.

## Consequences
A new pool declares its rules and gets every verb; no object keeps an insert, next-free or Move of its own (`move_cmd` is gone). Fixture and Universe have no Copy yet and keep their own Delete — the patch area decides.
