---
id: 0232
title: lumacore-views is the one source of what any UI shows
area: crates
status: built
date: 2026-10-16
built_in: 34f4
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §34 L8104–8107 (20.7), archive §34 L8731–8739]
---
# lumacore-views is the one source of what any UI shows

## Context
Before 34f4 the menu rows were built in the station's `net/menu_views`, the pool window's content in objects, and the TUI and the HTTP feeds each read a different half. Decision 20.7 gave every row one home when the station's code moved into crates.

## Decision
`lumacore-views`, on objects, builds everything a UI shows: menus (`menu_defs`' JSON — menu types, menus, columns, slabs) and their rows (`menu_rows`), pools, the attribute board, encoder, detail, sequence and window state. The HTTP feeds and the TUI both read it; no client builds its own rows, and no feed sends a shape the views crate did not build. What belongs to a command line (an `Edit Attribute` target, the owner words) stays in objects. The view tests that need the built rows live in the views crate, on the rig; the ones that only drive commands stay with their commands.

## Consequences
A new window, menu or pool column is a view function here plus the kit element that draws it (see pool-window-unified, window-types-rust-json), never a row shaped in `lumacore-http`, the terminal or a page. A change to what a feed shows is a change in this crate and nowhere else. Open: none.
