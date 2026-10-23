---
id: 0081
title: A .luma is a zip of the ShowData tree
area: showfile
status: built
date: 2026-10-12
built_in: 12a2
supersedes: [archive §21 L1676–1677 "plain JSON file", archive §29 L3234–3243 "zip show.json gdtf"]
superseded_by: —
confirmed_by: [show_file::FILES, show_file::FOLDERS, E-SHOW-005, E-SHOW-002, the show_file tests (round trip, pictures, damaged part, old formats, folder names)]
source: [archive §8 L365–379, archive §21 L1676–1677, archive §29 L3234–3243, archive §29 L3294–3300, archive §30 L3762–3769, archive §30 L3783–3788, archive §30 L3827–3833, archive §30 L3850–3852]
---
# A .luma is a zip of the ShowData tree

## Context
A show began as one plain JSON file. Step 11b made the `.luma` a zip holding `show.json` plus the GDTF files, so the original `.gdtf` travels with the show (see gdtf-native). 12a2 replaced the single JSON with a folder tree and a manifest — "a .luma file contains a folder system that represents the showdata tree and each image has a manifest for its GUID".

## Decision
`core::show_file` is the only reader and writer of show files. A `.luma` is a zip: `manifest.json` first (format, kind, show_id, commit, every entry with its SHA-256 and size — read before anything else), one pretty JSON file per top-level ShowData field (`show_file::FILES`), a folder per pool member (`FOLDERS`: `DataPools/1 Default/Groups.json`, `UserProfiles/1 Admin/Images/1 Stage/image.json + image.jpg`, `Devices/Dmx/Patch/FixtureTypes/<n>/Revisions/<r>/source.gdtf`), inside a data pool one file per pool kind, a user's `user.json` + `Programmer.json` + `Selected.json`. Folder names are `<no> <name>` for reading; the loader trusts only the number and the GUIDs inside. `decode` assembles the tree in the show's field order.

A changed or missing JSON part is E-SHOW-005; a damaged picture is left out; the one-file formats are refused with their number (E-SHOW-002, see show-format-forward-only). Load re-derives every fixture type revision from its `.gdtf`, GUIDs kept. Embedded media and asset drives: see asset-drives-media-sync. The format number is `FORMAT_VERSION` in lumacore-tree.

## Consequences
A new ShowData field does not save until it is listed in `FILES` or `FOLDERS` (beside its `ShowSubtree` variant). Never open a show file outside `show_file`; versions, recovery copies and the trash go through `core::backups` on top of it (see autosave-recovery-versions). Pretty JSON is the layout; the window-layout format question of 12c is a different matter.
