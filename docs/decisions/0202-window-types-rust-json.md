---
id: 0202
title: Window types declared in Rust, sent as JSON
area: windows
status: built
date: 2026-10-13
built_in: 12c
supersedes: [archive §30 L3733–3740 "KDL on disk"]
superseded_by: —
confirmed_by: [GET /api/window-types, data-id hooks]
source: [archive §30 L3733–3740, archive §30 L4499–4509, archive §30 L4532–4537]
---
# Window types declared in Rust, sent as JSON

## Context
Mid-build of 12a the user asked for a layout description of our own, plain enough for users and for later Lua plugins. The research (`layout-format.md`) recommended KDL files on disk with JSON on the wire. The 12c hardening (2026-10-13), after studying how a reference console separates a view from a window's build, dropped every hand-written file format.

## Decision
Two models. The floor plan — screens and views as the split tree with ratios and tab stacks, each window's type, label, colour, scroll and typed settings — is show and profile data: JSON inside the `.luma`, JSON when exported. How a window type is built — bar actions (`{id, label, icon, line, state?}`), `bar_default`, `bar_min`, title template, `chips` (`{id, label, shown_when, line}`), minimum size, settings tabs and defaults — ships with LumaCore and is never in a show, so an update can redesign a window without touching a show. It is declared in the Rust registry beside `window_types.rs`, compiler- and test-checked like a command, and sent to clients as JSON by `GET /api/window-types`. No KDL, no hand-written layout file; plugins later describe their UI as Lua tables that become the same JSON. Every drawn element carries a `data-id` (`<window type>/<setting or action id>`, pool cells by GUID) for tests.

## Consequences
A new window type or bar item is a Rust declaration plus its JSON on the wire, never a file in a show or a page-side table; the frontend reads `/api/window-types` and the `windows` feed and builds nothing of its own. Window settings are written only by `Set` lines (see window-bar-and-self); the layout `PUT` carries only splits, resizes and tabs. An unknown window type in a loaded show is dropped (see show-format-forward-only). A renamed `data-id` is a deliberate test change.
