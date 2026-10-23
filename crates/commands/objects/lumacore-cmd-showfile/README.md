# lumacore-cmd-showfile

The show file's words, objects and menu, and the service the server starts with.

## Purpose

What saves, loads, starts, deletes and takes parts of shows (decisions §30 "12d part 5"):
`SaveShow` (every save keeps the file it replaces as a version; over a newer-format file a copy
first, E-SHOW-004), `LoadShow` (a show, a version or a recovery copy — the copies open as unsaved
changes of their show), `NewShow`, `DeleteShow` (to the trash), `CleanStart`, `Import Show` (parts
by GUID, one Oops step); the Show, Version, Recovery and Library objects; the Backup menu. Before
replacing or leaving the show it asks "save first?" — "Don't save" still keeps a recovery copy —
which `Restart` and `Shutdown` (in `cmd-station`, on this crate) ask through `cmd_power`. The server starts with its service: the startup show, the Admin user, the
timed recovery copy (`autosave_tick`), the show's status (`show_info`). The `.luma` file, the disk
and the copies' rules are `lumacore-luma`'s; the recovery copy and the startup ask the state's.

## Owns / Doesn't own

- **Owns:** the rows (`rows`: `ROWS`, `OBJECTS`, `HOOKS`), the shared options and asks (`ask`),
  saving (`save`), loading (`load`), the words (`words`), the Show object and autosave (`show`),
  Version / Recovery (`copies`), Import Show (`import`), Library (`library`), the Backup menu
  (`menu`).
- **Doesn't own:** the file format, the disk and the copies (`lumacore-luma`), recovery's rules
  and the startup ask (`lumacore-state` `recovery`), the show's settings (the kernel's
  `settings_props`), the grid builders (the kernel's `menu_defs`).

## Public surface

- `register` — the rows, the hook (`Delete Show` asks itself) and the Backup menu.
- `load_show`, `save_show`, `clean_start`, `ensure_admin_user`, `autosave_in`, `autosave_tick`,
  `show_info`, `list_shows`, `parts_view`, `show_path`, `list_show_files`, `SHOWS_DIR`.

## Rules

- Every save keeps the file it replaces. [unchecked]
- "Don't save" keeps a recovery copy; one that can't be written stops the line. [unchecked]
- Show settings take their declared level. [test show_settings_are_typed_and_need_their_level]

## Edges

- **Depends on:** lumacore-registry, lumacore-base, lumacore-sysmon, lumacore-wire, lumacore-tree,
  lumacore-luma, lumacore-objects.
- **Used by:** lumacore-cmd-station (Restart / Shutdown), lumacore-server, lumacore-terminal,
  lumacore-sweep, lumacore-tidy.

## Decisions

- [§20](../../../../docs/archive/decisions-2026-10.md) — show files only move forward
- [§30 "12d part 5"](../../../../docs/archive/decisions-2026-10.md) — versions, recovery, the Backup menu
- [§34 25.7](../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate

## Testing

- `cargo test -p lumacore-cmd-showfile` — versions, recovery copies, the trash, cleanup, the
  startup ask and Import Show in a temporary library (never the real one).
