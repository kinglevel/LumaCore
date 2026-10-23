---
id: 0196
title: Two user tools, Run and Edit; the Tool object
area: webui
status: decided
date: 2026-10-19
built_in: —
supersedes: [archive §34 L9921–9925 "lasso in every"]
superseded_by: —
confirmed_by: none
source: [archive §34 L10276–10335]
---
# Two user tools, Run and Edit; the Tool object

## Context
Part 3d's "lasso everywhere" asked what a lasso does in a pool, where a tap runs its action. The user wanted "a unified toolset that works within standard pool windows and are in sync across windows"; decided one question at a time on the artifact `edittool.html`.

## Decision
Two modes for the whole user, everywhere: the Run tool (default: a tap runs the object's action, a hold takes the window full screen) and the Edit tool (a tap selects, a drag on an object moves it, a drag on empty space lassos, right-click / hold opens the one context menu). They are entries of one object, `Tool`, per user in `Runtime.Users.{User}` (never saved, no Oops step): `Tool 1 "Run"` and `Tool 2 "Edit"`, built in — never stored, deleted or renamed. `Tool "Edit"` / `Tool 2` switches (its default verb), `Toggle Tool` flips (UI key `ui.ToggleTool`, Insert), `Info Tool` shows yours and your default; `Set UserProfile "DefaultTool" Tool "Edit"` is a declared link (default Run). What the Edit tool picks is per user per screen, each kind its own picks. The indicator is a tinted orange edge line on the bars; only the line changes. Esc always leads back: each press selects the command line and undoes one layer — a menu or dialog, a full-screen window, the picks, then the tool back to the default.

## Consequences
Not built; on the roadmap after the patch window ("RunTool / EditTool + the lasso"). Until then the lift engine's lasso and move rules stand without the tool gate (see pointer-core-one-lift-gesture). Building it is one object, one UI key, one profile property and the indicator — never per-window handling.
