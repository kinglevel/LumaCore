---
id: 0188
title: Every list of objects is the kit's one grid
area: ui-kit
status: built
date: 2026-10-13
built_in: 12d part 3
supersedes: []
superseded_by: —
confirmed_by: [tidy frontend_tables_only_in_the_grid, e2e kit-12d.spec.ts]
source: [archive §30 L4769–4822, archive §30 L4835–4840, archive §30 L5003–5046]
---
# Every list of objects is the kit's one grid

## Context
"We need grid cells and columns and rows … so all grids looks the same." A reference console's object grid gave the model; our touch gestures were decided on the boards.

## Decision
`<luma-grid>` (`kit/grid.js`) is the one grid. Rows are keyed by GUID; columns come from the registry's declared props (kind, width, priority, rights, section). The selection is a set of row keys and cells that survives scroll, sort and live updates: a tap or lasso replaces it, the gutter selects rows, a header the column; `+` / `−` latch, `Thru` is one-shot; All / None / Invert and the count sit in a sticky bar. One-finger drag follows the device setting `Drag: Auto | Lasso | Scroll`; hold-then-drag always lassos (see pointer-core-one-lift-gesture). Editors are kit elements by kind; on touch a two-finger tap or the Edit chip edits, so a scroll never edits. The edit card (`editcard.js`) edits the whole selection: tabs from the sections, each field the shared value or ‹different values› with its range, no OK. Every change is one line naming its rows (another object's rows become a second statement) and one Oops step; refused rows are named with their codes. A user's column state (sort, hidden, width, column set, the new-object line) goes as `Set GridColumns "<key>" …` on the profile (Untracked). Rows never re-sort under a finger or an open editor. `view="mini"` draws the whole list in miniature (see phone-desk-miniature).

## Consequences
No table markup outside `grid.js` — tidy `frontend_tables_only_in_the_grid` (`tidy: table-ok (<reason>)` only where a window still waits for its grid). Never a hidden grid model or a pool-specific endpoint. Its actions are the slab (see slab); its context menu is one-context-menu-from-the-registry.
