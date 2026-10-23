---
id: 0001
title: The object tree: ShowData, Runtime, station-local subtrees
area: tree
status: superseded
date: 2026-10-11
built_in: —
supersedes: []
superseded_by: state-homes-by-type
confirmed_by: none
source: [archive §1 L63–169]
---
# The object tree: ShowData, Runtime, station-local subtrees

## Context
The first design (March) drew one definitive tree for the whole console: `Root` with `MessageCenter`, `ShowData` (Patch, 3D, DataPools with Groups / Sequences / Scenes / Macros / Pawterns / Priorities / Timecodes / Worlds, UserProfiles with their Programmer, LivePatch, Protocols, Media, Masters, Tags, Agendas, Playbacks, Variables, ShowSettings), `RDMData`, `Temp` (runtime only) and the station's own branches (`StationSettings`, `ColorTheme`, `Menus`, `WebServer`). The step 11a audit (2026-10-11) marked it as the sketch it was: MessageCenter, Scenes, Agendas, Media and RDMData were never built, and the programmer had already left the profile.

## Decision
The §1 diagram is history. The tree as built is documented in `docs/user/object-tree/object-tree.md`, and its shape follows two later rules rather than a drawing: every field has one home among show data, session runtime and station-local state (see state-homes-by-type), and every node carries its meta level (see node-meta-levels). `Root` splits into `ShowData` and `RuntimeState` in `lumacore-tree`; the paths code names are typed `Node`s (see the tree-path record), so the diagram is never the reference for a path.

What the diagram got right and still holds: Patch is claimable as an area (`Claim Root.ShowData.Devices`, see claims-by-tree-path); DataPools hold the pool objects; `Playbacks` is saved show data; station settings are never in the show.

## Consequences
Nobody adds a branch from the §1 drawing to the code; a branch is added when its feature is decided, as a `ShowSubtree` variant with its `Walk` and `show_file` entries (see change-path-undo-subtrees, guids-everywhere-links). Scenes, Pawterns, Priorities and Variables keep their own decided-not-built records (scenes-stateful-logic, pawtern-pool, priority-pool, variables-three-tiers). MessageCenter, Agendas, Media as a manifest and RDMData have no record and no roadmap item unless one is opened. A path change shows in `tests/catalog/testdata/tree_paths.golden.txt`.
