---
id: 0086
title: Root.Drives is the station's drive list, ids remembered
area: drives
status: built
date: 2026-10-19
built_in: 38a
supersedes: [archive §38 L11334–11337 "Root.Drives = /"]
superseded_by: —
confirmed_by: [cargo test -p lumacore-drives, tests/e2e/tests/kit-drives.spec.ts, LUMACORE_DRIVES_PROBE]
source: [archive §38 L11334–11355, archive §38 L11429–11451]
---
# Root.Drives is the station's drive list, ids remembered

## Context
`Root.Drives` used to be the raw filesystem root. §38 asked how a desk sees a stick: the user chose a found-and-added drive list and ids of kind + ordinal bound to the volume. Not chosen: fixed slots (a macro's `usb1` would mean "whatever is in first") and the label as the id.

## Decision
`crates/formats/lumacore-drives` owns it: `probe` (the OS's mounted volumes per platform; `LUMACORE_DRIVES_PROBE` lets a test present a folder as a volume), `library` (`init_library` makes `LumaCore/library/`), `ids` (`next_id`), `table` (`Drive`, `reconcile`).

`Root.Drives` is this station's drive list — `local-library` (its own `library/`, always Drive 1), every volume the station finds, and folders someone added — never the raw disk; `Root.Drives.<id>` is the volume root (`vfs`) with `LumaCore` marked ★ library. An id is the kind plus an ordinal: `ssda` / `hdda` (fixed disks beyond the system one), `usb1` (removable), `folder1` (added), `smb1` / `ftp1` / `sftp1` reserved for LumaCore's own connections. The first sight of a volume takes the next free id of its kind; the station remembers volume key (UUID / serial) → id, kind, label, last seen in `StationSettings::drives` — the same stick is `usb1` every day; absent drives stay listed Out; Forget frees the id; the label is the display name, never the id; a misreported kind is corrected once (`Set Drive "ssdb" "Kind" "USB"` re-ids). Station-local, never replicated; a show never names a drive; a LumaSession member's `usb1` is its own stick.

## Consequences
A line names a drive only as `/Drive "<id>"` (see drive-object-and-slash-drive-option); the watcher and the first write: see drive-service-and-network-drives; every file dialog opens on a drive's library folder (see one-files-element-on-one-feed). An AssetRoot's station map names a drive and a folder, never a bare path.
