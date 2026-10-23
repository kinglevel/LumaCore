# lumacore-drives

This station's drives (decisions §38 "Drives and the one files element"): the volumes the OS has
mounted, the library tree on a drive, the ids, and the drive list against the remembered table.

## Purpose

A drive is a whole volume — the station's own `library/` (`local-library`), a stick or disk the
station finds (`usb1`, `ssda`), a folder added by hand (`folder1`) — and its library is
`LumaCore/library/` on it, the station library's folders, created on the first write and never by
a read. The same stick keeps its id: the first time a volume is seen it gets the next free id of
its kind and the station remembers volume key → id. Network drives as LumaCore's own clients
(SMB, FTP, SFTP, never the OS's mounts) are planned here.

## Owns / Doesn't own

- **Owns:** the probe (`probe`: Linux `/proc/mounts` + `/sys/block` + `/dev/disk/by-uuid`, macOS
  `/Volumes`, Windows the drive letters; `LUMACORE_DRIVES_PROBE=<dir>` for tests; free space from
  one `df -kP`), the library tree on a drive (`library`), the id rule (`ids`), the drive list and
  the table rule (`table::reconcile`: ids assigned and remembered, present / Out, the sysmon
  lines).
- **Doesn't own:** the state that holds the list and saves the table (`lumacore_state::drives`),
  the watcher thread (the server, every 2 s), the `Drive` object and the Drives menu
  (`lumacore-cmd-station`), `/Drive` on the file words (`lumacore-cmd-showfile`).

## Public surface

- `probe::{probe, spaces, space_in, space, Volume, VolumeKind}` — what is mounted now.
- `library::{library_root, has_library, init_library, LIBRARY_DIR, LIBRARY_FOLDERS}` — the tree.
- `ids::next_id` — the next free id of a kind.
- `table::{reconcile, Drive, Reconciled, LOCAL, gb}` — the list and the table.

## Rules

- A read never creates the library tree; the first write does, never an ask. [unchecked]
- An id is bound to the volume's key and remembered; a new volume never takes an absent one's id. [tested: table]
- The OS's mounts are read on a thread of the server, never on the state thread. [unchecked]

## Edges

- **Depends on:** lumacore-luma, lumacore-tree.
- **Used by:** lumacore-state, lumacore-objects, lumacore-server.

## Decisions

§38 "Drives and the one files element" (2026-10-19).

## Testing

`cargo test -p lumacore-drives` (the probe folder, the tree, the ids, the table); the station's
side in `cargo test -p lumacore-state drives`; end to end in `tests/e2e/tests/kit-drives.spec.ts`
(a probe folder as the sticks).
