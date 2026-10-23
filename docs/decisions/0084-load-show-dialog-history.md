---
id: 0084
title: Load show is one dialog; Import Show by GUID; history matrix
area: backup
status: built
date: 2026-10-13
built_in: 12d part 5
supersedes: [archive §30 L4737 "Partial load later", archive §30 L4864–4872 "separate load parts"]
superseded_by: —
confirmed_by: [backups_tests, core::show_import, GET /api/show-parts]
source: [archive §30 L4864–4903, archive §30 L5277–5309]
---
# Load show is one dialog; Import Show by GUID; history matrix

## Context
The user refused a separate "load parts" word ("we rather have it say load show then show the load parts dialog") and asked for a show history with more than save dates. The GUID merge of `lumacore_tree::merge` replaces a partial-show-read wizard.

## Decision
Backup › Library › Load… opens the Load show dialog: the file's facts from its manifest, then its five parts as All / Pick / None (counts from `GET /api/show-parts`), Pick… a toggle per object. Everything All = `LoadShow "x"` (the unsaved-changes ask follows); anything less = ONE `Import Show "x" <parts | objects>` line.

`Import Show` is the Show object's Import verb (Setup, reach Drive, one Oops step, `core::show_import`): parts Data · Patch · Protocols · Users · Stations, or objects by the file's numbers or names, merged by GUID — the same GUID updated in place, a new one added at its number or the next free; broken links kept and counted; refused while Patch has a draft. `LoadShow "x" /Version <stamp>` and `LoadShow /Recovery "<copy>"` open a copy as unsaved changes of its show.

Show · History is a grid, one row per save, List | Matrix (`Grid.matrix`: a column per object kind with `+a ~c −r` by GUID against the save before, from the manifest's GUID → hash index, no full load). A copied file keeps its history (`AppState::saved`); `Library › History…` reads without loading; reading needs no right.

## Consequences
Never a second partial-load path or a client-side merge. A new object kind that belongs in the matrix joins the manifest's index. The dialog's lines carry `/Drive` (see drive-object-and-slash-drive-option).
