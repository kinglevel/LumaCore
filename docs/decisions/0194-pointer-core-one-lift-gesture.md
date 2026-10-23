---
id: 0194
title: Every drag is the pointer core's one lift gesture
area: pointer
status: built
date: 2026-10-20
built_in: 4d4
supersedes: []
superseded_by: —
confirmed_by: [e2e kit-4d4.spec.ts, tidy frontend_gestures_only_in_the_kit]
source: [archive §34 L9917–9931, archive §34 L10194–10275, archive §40 L11719–11728]
---
# Every drag is the pointer core's one lift gesture

## Context
Round 3 of the patch dialog decided the desk's drag; the user asked for the lasso in every list. The phone round made the slot grid scroll with one finger, so 4d4 decided, one question at a time, how touch lifts; 40d gave the Patch tree the same drop.

## Decision
`lift` in `kit/pointer.js` is the one drag engine for any surface of items: a tap selects; a drag on an item lifts it — on touch the hold is the press: held still for the HoldTime, the item lifts and follows the finger; a drag on empty ground lassos; hold-and-release without moving is the context menu; a plain finger scrolls in `Drag: Auto` / `Scroll`, two fingers in `Lasso`. A lifted selection moves as one and the drop is one command line — `Assign Fixture … At DMX u.aaa`, `… At <node>`, a node's Move through the nesting ask. The lift cue: the item rises, the rest dims, a chip above the finger names it and where it lands (red where it would park); a wrapping ghost is a ribbon, red per run. A lifted thing held still over a pager or a tab springs it (`.lc-sprung`) — tabs, pool pages, every ‹ ›. Today: `<luma-dmxmap>` in both zooms, the Patch tree's sidebar drop, `<luma-grid>`'s lasso; every other surface takes it as it gets it.

## Consequences
Never a per-window pointer handler, HTML5 drag or own lasso (tidy). A window that pages without the spring, or a list without the lasso, is a kit gap, not a window choice. Which tool arms lasso and move in a pool, where a tap runs its action, is runtool-edittool-tool-object (decided, not built).
