---
id: 0165
title: Users & Profiles follows storage: grid, slab, drill-in
area: menus
status: built
date: 2026-10-14
built_in: 12b2 (the window); 12d (grid and slab); after 12f-a part 4b (the detail and the typed profile nodes)
supersedes: [archive §30 L4328–4336 "floating luma-people window", archive §30 L6593 "top-level Encoder bar tab"]
superseded_by: —
confirmed_by: [`tree_paths.golden.txt`, tidy `frontend_actions_only_in_the_slab`, tidy `frontend_tables_only_in_the_grid`]
source: [archive §30 L4328–4336, archive §30 L4396–4405, archive §30 L4841–4850, archive §30 L6609–6664, archive §30 L6668–6671]
---
# Users & Profiles follows storage: grid, slab, drill-in

## Context
12b2 made a floating `<luma-people>` window; 12d moved every list onto `<luma-grid>` with its actions in `<luma-slab>`. After 12f-a put an Encoder bar tab at the top level, the user asked that "all data that is nested in the Users Profiles menu needs to be logical where it is stored"; mockup rounds picked drill-in and a Settings / Content split.

## Decision
The Users & Profiles menu (a Menu tile) shows data where it lives in the tree. A Users grid and a Profiles grid are the lists; New, Delete, Edit and the copy / import / export actions sit in the slab and run command lines (New = the station ask, Delete = the one Delete ask). A picked row drills in: `<luma-setup>` swaps the grid for `<luma-detail>` with a breadcrumb (`‹ Profiles › name`), group tabs and part tabs; under the phone width, tiles → rows → part. `Menu "x" [/Tab t] /Open "<row> [<group> [<part>]]"` opens a detail from a line. The tree follows the same split: `UserProfile` = meta + locked + `settings` (`ProfileSettings`) + `content` (`ProfileContent`), typed nodes `Root.ShowData.UserProfiles.<n>.Settings | Content` with each part a declared node (`Node::entry_schema`); a user drills into Account · Profile · Work · Usage. Every `UserPreferences` field is a declared `UserProfile` property; no hand-made `Set` paths. Rows come from the `detail` State feed.

## Consequences
A new profile setting is a property on the row; a new part is a declared node and shows in the goldens. Never a hand-made table or action button here (tidy). Open: the Content lists are read-only (adds and deletes by their lines); the Look part waits for the theme editor; external logins are roadmap.
