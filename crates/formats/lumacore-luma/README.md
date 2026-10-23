# lumacore-luma

The `.luma` show file and the station's disk: the show file format, the one disk door, the
backups, the media store and the station's local settings file.

## Purpose

A `.luma` is a zip of the ShowData tree (decisions §30 "12a2"): `manifest.json` first (format,
every entry with its SHA-256, the save's facts, counts, the GUID → hash index, the last history
rows), one JSON file per top-level field, a folder per pool member, every fixture type's `.gdtf`
and every embedded picture. This crate writes and reads it (`show_file`, `parse_show`: a newer or
older format is refused, every object needs a unique GUID), keeps every replaced save, the
recovery copies and the trash (`backups`, §30 "12d part 5 built"), the content-addressed media
store (`media`) and the station config file (`station_config`). Every file a command touches goes
through the one door, `drive`, which the command sweep can shut or sandbox; long disk work runs on
the one `background` worker behind the same door. It knows nothing of AppState (decisions §34
25.10): who and why a copy is written and the startup ask stay in `lumacore-state`'s `recovery`.

## Owns / Doesn't own

- **Owns:** the format (`show_file`), the disk door (`drive`), the worker (`background`), versions,
  recovery copies, trash, pins, manifests, cleanup, the Backup menu's rows (`backups`), the media
  store (`media`), the station config file (`station_config`), `build_info`, the E-SHOW and
  E-MEDIA codes (`errors`).
- **Doesn't own:** loading a show into a station, the change path (`lumacore-state`); the facts of
  a save and the startup ask (`lumacore-state`'s `recovery`); the commands (SaveShow, LoadShow,
  Import, media objects: `lumacore-objects`); GDTF parsing (`lumacore-gdtf`).

## Public surface

- `show_file::{encode_with, decode, from_bytes, manifest_of, parse_show}` — the format.
- `drive::{read, write, write_atomic, library_file, set_off, set_sandbox}` — the disk door.
- `backups::{keep_version, versions, recovery, trash, cleanup, library_view}` — the copies.
- `media::{store, stored, sha}` — the media store.
- `build_info` — the build that writes a show.

## Rules

- Show files only move forward: newer and older formats are refused, no migrations. [unchecked]
- Every disk access of `core/` goes through `drive`. [tidy core_reaches_the_disk_only_through_drive]
- Never run cleanup against the real library in tests (`backups::use_dir`). [unchecked]

## Edges

- **Depends on:** lumacore-registry, lumacore-base, lumacore-wire, lumacore-sysmon, lumacore-tree, lumacore-gdtf.
- **Used by:** lumacore-state, lumacore-objects, lumacore-views, lumacore-lumasession, lumacore-server, lumacore-sweep, lumacore-drives.

## Decisions

- [§20, §21 "6d"](../../../docs/archive/decisions-2026-10.md) — show files only move forward, strict identity
- [§30 "12a2", "12d part 5 built"](../../../docs/archive/decisions-2026-10.md) — the format, backups
- [§34 25.10](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate

## Testing

- `cargo test -p lumacore-luma` — the door, the format, the station config file.
- Backups end to end: `lumacore-objects`' `backups_tests`.
