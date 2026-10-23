---
id: 0172
title: The Oops window: select, then Oops n
area: undo
status: built
date: 2026-10-14
built_in: 12f part 5
supersedes: [archive §30 L6236–6238 "round 7 chips filter the view"]
superseded_by: —
confirmed_by: [the `oops` State feed, e2e on desktop / tablet / phone / WebKit]
source: [archive §30 L5749–5752, archive §30 L5770, archive §30 L6236–6238, archive §30 L6427–6442]
---
# The Oops window: select, then Oops n

## Context
A reference console's Oops menu runs `Undo <n> /NoConfirm` from a list; 11c had left the window for the touch kit (see oops-categories-statements). The 12f mockup rounds asked for a window that can never undo on one tap; round 7's chips filtered the view, which the build turned into the profile's own Oops filters.

## Decision
`Menu "Oops"` (a `MenuDef`, alias `undo`; the same line closes it) and the core `oops` UI key (a tap runs `Oops`, a hold opens the window) with its sidebar button open the Oops window; on a phone the line or a key does. It is the kit's pop-up (`kit/oops.js`, tap outside closes, PopupDim dims), fed by the per-user `oops` State feed on `/ws/feeds` (each logged-in user's steps oldest first with line, category, station, time and Main; the undone ones next-redo first; the profile's number and filters). Newest at the bottom; a tap selects a step and every newer one and the button reads "Oops n steps" (it runs `Oops n`, OopsConfirmation still asks); never a one-tap undo. Undone steps stay visible dimmed below a line, a tap there makes "Redo n steps" (`Redo [<n>]` redoes the newest n undone steps and stops at the first that can't). The chips are the profile's filters CreateOops, OopsProgrammer, OopsSelection, OopsViews, each a `Set UserProfile <n> <filter> Yes|No`.

## Consequences
A change to what a step shows is a change to the `oops` feed and the station view's `undo`, never client state; the window stays the kit's pop-up. Open: the Oops window in the TUI (roadmap).
