---
id: 0130
title: Dispatch's special cases are Hook rows families push
area: catalog
status: built
date: 2026-10-16
built_in: 34g3c (more hooks in 34g4 cuts 2 and 8)
supersedes: [archive §34 L8974–8985 "dispatch naming families"]
superseded_by: —
confirmed_by: [the objects crate's tests, `tests/catalog`]
source: [archive §34 L8974–8985, archive §34 L8997–9017, archive §34 L9097–9113]
---
# Dispatch's special cases are Hook rows families push

## Context
Dispatch had grown a list of exceptions that named families by hand: a locked device's allowed keys, `<line> /Watch`, Usage ranking, windows following their profiles, the programmer's attributes after a kept draft, the words whose Delete never asks, the untracked `Set` of grid columns. Splitting the objects crate into command crates (§34 25) needed dispatch to know no family.

## Decision
`commands/hooks.rs` holds one typed catalog row, `Hook`; every family pushes its hooks beside its command rows (`Family::hooks`, `HOOKS` in its file), and dispatch runs the rows it finds, naming no family. The kinds: `Gate` (devices: a locked device runs only its lock keys), `Intercept` (station: `<line> /Watch`), `Used` (suggest: Usage ranks the words), `Changed` (windows following their profiles), `PatchKept` (programmer: attributes follow a kept draft, `/Standard` / `/Mine`), `DeleteNeverAsks` (showfile: Show; windows: ViewButton), `SetUntracked` (windows: GridColumns), `Rewrite` (the tree shell's `mv` / `cp` / `rm` write-out, see tree-shell-mv-cp-rm-through-doors), `SetCheckedElsewhere` (the rights gate's "`Set Station "Tower" …` is checked on that station"), `ImportByGuid` (Import Show's single objects, Network and Station among them), `ShowReplaced` (keys checked on a show load). The Default profile's lock stays dispatch's own rule (`DEFAULT_LOCKED`, E-USER-002), as do the kernel's pieces: claims, the change path, the System Monitor's change report, the tree walk.

## Consequences
A new special case in dispatch is a new `Hook` kind pushed by the family that needs it, never a match on a family or keyword in the kernel (the keyword rule, see one-registry-one-namespace). A command crate registers its hooks in its `register` like its rows; the test catalogs that need them register the crate.
