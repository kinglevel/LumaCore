---
id: 0228
title: Every crate is lumacore-category; folder equals package
area: crates
status: built
date: 2026-10-16
built_in: 34f8; 34g4 (the cmd- crates)
supersedes: [archive §34 L8043–8046 "the two that"]
superseded_by: —
confirmed_by: [crates_sit_in_their_group]
source: [archive §34 L8041–8047 (item 17), archive §34 L8287–8288 (23.3), archive §34 L8390–8395 (25.6)]
---
# Every crate is lumacore-category; folder equals package

## Context
Two crates broke the pattern on 2026-10-15: the station was plainly `lumacore`, and `lumacore-session` held `Root.Runtime`'s types while never holding LumaSession code ("session" alone is no name of ours). Deno showed the cost of a folder drifting from its package name (`ext/bundle` holding `deno_bundle_runtime`).

## Decision
Every workspace crate is `lumacore-<category>`, and its folder's last segment is the full package name, so `-p`, grep, error paths and the README title all read the same. The station is `lumacore-station` (its binaries keep their names), the runtime types are `lumacore-runtime`. A crate that holds commands is `lumacore-cmd-<domain>` (after nushell's `nu-cmd-*`), paired with what it edits (`lumacore-patch` ↔ `lumacore-cmd-patch`); a crate without commands beside dispatch (the suggestion engine) carries no `cmd-`. "Rig" and "wiring" are not crate names: in lighting a rig is the hung fixtures. `crates_sit_in_their_group` checks the name, the folder and the group (see layers-point-down).

## Consequences
A rename of a crate renames its folder, its `EDGES` row and its `GROUPS` entry in one commit; a new command crate takes the `cmd-` prefix and its domain's name. Open: none.
