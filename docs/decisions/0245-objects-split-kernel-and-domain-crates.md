---
id: 0245
title: A command kernel plus one cmd- crate per domain
area: crates
status: built
date: 2026-10-17
built_in: 34g3, 34g4 cuts 1–9, 34f7c, Tags part 3a
supersedes: [archive §34 L8355–8359 "one OBJECTS list"]
superseded_by: —
confirmed_by: [tidy `crate_edges_point_down`, tidy `crates_sit_in_their_group`, tidy `every_command_states_what_decides_it`]
source: [archive §34 L8346–8368 (item 25.1), archive §34 L8377–8385 (25.3–25.4), archive §34 L8396–8425 (25.7), archive §34 L8433–8437 (25.9), archive §34 L8568–8587 (25.13), archive §34 L8955–9127, archive §34 L9284–9290]
---
# A command kernel plus one cmd- crate per domain

## Context
After 34f11 the objects crate was the hub above effects, playback and patch. Research over large projects found that size never decides a split; the recipe is a small shared kernel, then a crate per family with one file per verb. What blocked a cut were the kernel's edges up into the domains: one handler table, one `OBJECTS` list naming every module, and dispatch's special cases.

## Decision
The objects crate splits into a kernel and one crate per domain. The kernel (dispatch, resolve, rights, claims, the typed `Command`, flags, lum, the tree shell's vfs / navigation, the verbs every object shares) stays `lumacore-objects` until the last cut renames it `lumacore-cmd-dispatch` in `crates/commands/`. Each domain crate sits in `crates/commands/objects/` as `lumacore-cmd-<domain>`, pushes its own command rows, object rows and hooks in its `register` (`Registry::rows`), and names another domain only downward. Dispatch's special cases are typed `Hook` rows (`commands/hooks.rs`: Gate, Rewrite, Intercept, Used, ShowReplaced, Changed, PatchKept, DeleteNeverAsks, SetCheckedElsewhere, ImportByGuid, SetUntracked) a family pushes beside its rows; dispatch names no family. Programming and playback are two crates: playback reads the programmer through programming's API, programming never names a sequence. Keys sit below users. `lumacore-suggest` is beside dispatch, not `cmd-`. Cut so far: suggest, treeshell, keys, showfile, lumasession, station, devices, protocols, tags, and one crate per protocol object (sacn, artnet, osc). The station's wiring: see rig-wiring-in-cmd-station.

## Consequences
Pending, in this order: patch, media, windows, users, programmer, playback, then the kernel's rename. If two families still need each other through dispatch, they stay one crate with folders. A new word goes in its domain's crate, never the kernel; a new dispatch special case is a `Hook` row. Tests move with their words (see three-homes-for-a-test).
