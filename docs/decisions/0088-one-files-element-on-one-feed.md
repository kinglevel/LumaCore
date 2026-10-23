---
id: 0088
title: One files element on one feed is every file dialog
area: files
status: built
date: 2026-10-20
built_in: 38b, 38c-1
supersedes: [archive §34 L9709–9713 "picker of those files"]
superseded_by: —
confirmed_by: [InputKind::File, Revisions.library, tests/e2e/tests/kit-drives.spec.ts, the Showroom]
source: [archive §38 L11385–11400, archive §34 L10789–10812, archive §38 L11452–11490]
---
# One files element on one feed is every file dialog

## Context
Insert fixtures, Backup › Library and the Import words each grew a list of files. §38 chose one kit element over three lists with three looks; the picker as a station-held dialog was rejected.

## Decision
`<luma-files>` (`kit/files.js`) is every load and save dialog: the drives rail from the `station` feed (This device — the browser upload — last), the crumbs `usb1 › LumaCore › library › shows` with ★ Library as the jump, the folder's rows in a `<luma-grid>` with the kind's columns, the drop zone and a Name row for a save (a taken name is the station's ask).

The rows come from one `files` feed on `/ws/feeds` (`sub files {drive, folder, kind}`), built on the state thread from a cache keyed by path + mtime, heads read on `core::background`; rebuilt when the station's drives or the library's writes move (`Revisions.library`) — never a timer. One upload route, `POST /api/upload?drive=&folder=&name=` (the first write makes the tree; a path leaving the library is refused).

Where: Backup › Library is the element at `shows/` (`Content::Drives(&LIBRARY)`, `{drive}` in a tab's lines becomes ` /Drive "usb1"`), Insert fixtures' Library, Import MVR's File step, Import / Export UserProfile, Import Image, Import FixtureType, and the station's "which file?" ask — `InputKind::File { files: <kind> }` (`Ask::file`), its value `<drive>:<file>` rendered as `"file"` plus ` /Drive "usb1"` (`clean_input`). The TUI has the fence (`ls ~usb1.shows`), no picker.

## Consequences
A new file kind registers its columns and a cached head reader (`heads_in` / `library_in`) — never a new list, endpoint or client-side state. Not built: a profile's user / date and a picture's pixels as columns, the phone's Insert type list, Export on the Name row.
