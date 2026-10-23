---
id: 0241
title: The catalog: every crate pushes its rows at startup, frozen once
area: catalog
status: built
date: 2026-10-16
built_in: 34f3a–34f3g2
supersedes: [archive §34 L8156–8161 "f before g"]
superseded_by: —
confirmed_by: [`freeze()`, tidy `crates_on_the_registry_register_and_install`, tidy `the_station_only_assembles`]
source: [archive §34 L8127–8161 (items 21.1–21.7), archive §34 L8173–8176 (21.9), archive §34 L8640–8730, archive §34 L8862–8868]
---
# The catalog: every crate pushes its rows at startup, frozen once

## Context
34f3 found the menu table pinned in the objects crate because its `Menu` verbs read it, and asked how large projects publish declarations. Large projects push at startup; linker gathering (`inventory` / `linkme`) was rejected: undefined order, rows lost when a crate is dropped, toolchain trouble.

## Decision
`lumacore-registry` (foundation, on base, no `AppState`) holds pure-data rows, the store and `freeze()`. Every crate on it has `pub fn register(r: &mut Registry)` in its `lib.rs`: first the `register` of the crates it builds on, then `r.enter(env!("CARGO_PKG_NAME"))`, then its rows. Every declaration is a row: errors, flags, feeds, window types, tools, settings, UI keys, commands, objects, menus, hooks, protocols. Declaration and behaviour split: `CommandDef` is `{ run, decl: CommandDecl }`, `ObjectDef` is `{ decl: ObjectDecl, … }`, each deref'ing to its row; dispatch runs `registry::run_of(row)`, `objects::def_of(row)` finds behaviour by the row's address. A row names another row by its static (`&X.decl`), so a typo doesn't compile; commands and objects therefore became rows before menus (f-before-g is superseded). `freeze()` checks once across tables: one namespace, a grid's object exists, a key line resolves, a window's feed exists; a duplicate is refused. One process-wide frozen catalog: `lumacore_station::install_catalog()` installs it; `lumacore_registry::get()` reads it; the server only checks one is installed (`require_catalog`). The catalog never crosses the wire; `/api/menu-types`, `/api/window-types` and the feed list are dumps of it.

## Consequences
Never add a static list of another crate's rows. A new table is a row type plus its freeze check, never a hand-kept lookup. Test binaries: see test-binary-installs-catalog-via-ctor; rows typed above the registry: see rows-typed-above-registry; properties: see properties-stay-beside-row. Words added at run time (plugins) are a later layer checked against this one, not built.
