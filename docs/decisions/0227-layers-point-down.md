---
id: 0227
title: Crates sit in domain folders and their edges point down
area: crates
status: built
date: 2026-10-17
built_in: step 8, 34b–34d, 34f8, 34g1, 34g4, 34f7c
supersedes: [code-structure-later-items, archive §23 L2029–2033 "Modules now; separate", archive §34 L7992–7998 "cut in three", archive §34 L8266–8273 "Eight groups under", archive §34 L8278 "control/ holds osc"]
superseded_by: —
confirmed_by: [crate_edges_point_down, crates_sit_in_their_group]
source: [archive §23 L2007–2049, archive §23 L2071–2105, archive §34 L7992–7998 (item 7), archive §34 L8021–8040 (items 15–16), archive §34 L8593–8620, archive §34 L8260–8302 (23.1–23.7), archive §34 L8369–8376, archive §34 L8386–8389, archive §34 L8456–8461, archive §34 L8786–8797, archive §34 L9270–9276]
---
# Crates sit in domain folders and their edges point down

## Context
Step 8 (§23) drew one direction through the station's modules, core importing nothing above it, and closed the network-I/O-in-a-command exception through core's outbox; crates waited "until build times demand". §34 7 cut the station in three; 15–16 settled the cut (the registry's types go down into state, its contents and dispatch up into objects, one re-entrant function with a depth cap); §34 23–25 placed every crate in a folder by domain after comparing flat and grouped workspaces. The names `show/`, `tools/`, `control/` became `model/`, `dev/`, `data/`.

## Decision
Every crate depends only on crates below it; `crate_edges_point_down` holds every allowed edge in `EDGES`, and a new edge is an API change added there with a reason. `lumacore-state` (AppState, change path, Oops, props' types, errors, dialogs, the registry's types) sits below `lumacore-objects` (the tables, every object and verb, dispatch, resolve, rights, claims, lum, menus), which re-exports the state's modules. Core asks LumaSession through its outbox and never calls it; the engines, protocols and LumaSession never depend on the terminals, so a DMX-only build stays possible. Nested lines are capped at `dispatch::MAX_NESTED_DEPTH` (E-CMD-019). Crates live under `crates/<group>[/<family>…]/lumacore-<name>`, grouped by what they are about, not by layer; tidy's `GROUPS` lists the groups with their purposes. A family (siblings on one interface) gets a sub-folder: `network/protocols/{dmx,data}/` for the runtimes, mirrored by `commands/objects/protocols/{dmx,data}/` for their objects; `commands/` holds the kernel, `commands/objects/` the command crates. `tests/` holds systemtest, fuzz, catalog and e2e.

## Consequences
A new crate is born in its group with its row in `EDGES` and `GROUPS` (see crate-naming-lumacore-category, crate-readme); nothing in `core/` names a crate above it; code-structure.md §1's diagram is older than this record. Open: none named.
