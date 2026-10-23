---
paths:
  - crates/formats/lumacore-luma/**
  - crates/formats/lumacore-drives/**
  - crates/commands/objects/lumacore-cmd-showfile/**
  - library/**
---

# Show files, backup, drives, media

Records: `luma-show-file-container`, `show-files-upgrade-step-by-step`, `autosave-recovery-versions`, `load-show-dialog-history`, `asset-drives-media-sync`, `drives-are-the-os-volumes-probed`, `drive-object-and-slash-drive-option`, `one-files-element-on-one-feed`, `drive-service-and-network-drives`, `reach-drive-door`, `demo-show-test`.

## The file

- A `.luma` is a zip of the `ShowData` tree (`core::show_file`): `manifest.json` first (format, every entry + SHA-256), one JSON file per top-level field (`show_file::FILES`; a new `ShowData` field doesn't save until listed there or in `FOLDERS`), a folder per pool member (`DataPools/1 Default/Groups.json`, `UserProfiles/1 Admin/Images/…`, `Devices/Dmx/Patch/FixtureTypes/<n>/Revisions/<r>/source.gdtf`). Read and write only through `show_file`; load re-derives every fixture type revision from its `.gdtf`, GUIDs kept.
- `ShowSettings.format_version` (`FORMAT_VERSION` in lumacore-tree) + `saved_with_commit` + `show_id`; a show from `OLDEST_FORMAT` to `FORMAT_VERSION` is read whole and upgraded step by step (`show_file::upgrade::STEPS`, one per bump, on the JSON before it is parsed), opens as unsaved changes and says so; older or newer is refused (E-SHOW-002 / 001); a profile file and the `library/ui` files read through the same door (the other load-time rewrites: a changed footprint followed where slots are free; unknown window types dropped). Saving over a newer-format file is refused (E-SHOW-004): the show goes to `{name}_format{N}_{time}` and `SaveShow "x" /Overwrite` replaces it. Load refuses missing or duplicate GUIDs (E-SHOW-003) and a protocol or setting this build lacks (E-SHOW-007 / 008).
- Stores (gitignored caches): `library/fixturetypes/<sha256>.gdtf`, `library/media/.store/<sha256>.<ext>` (`core::media`). Embedded files have size limits (constants in the luma crate; E-MEDIA codes); large media lives on asset drives (`AssetRoot` mapped per station). The show has no icons; a profile carries its own Icons, Images, Theme and LockScreen (`Export|Import UserProfile`).

## Backup and recovery

- The show file changes only when someone saves, and every save keeps the file it replaces in `library/shows/.versions/<show>/<stamp>.luma`. The timed Autosave writes a recovery copy (`.autosave/{show}_{reason}_{station}_{stamp}.luma`, untitled too); so does a load / new show / CleanStart over unsaved work, a stop signal and a lost merge. Deleting moves to `.trash`. The manifest carries the save's facts, `counts`, a GUID → hash `index` and the last history rows; lists read only manifests (`backups::manifest`, cached).
- Cleanup is a station setting (`RecoveryKeepDays`, `RecoveryCap`); pinned copies (`Set Version|Recovery … Pinned|Note`) and files whose manifest doesn't say why they exist are never removed; never run it against the real library in tests (`backups::use_dir`). A recovery copy that can't be written refuses to replace the show (E-CMD-012).
- A station marks itself running; after an unclean stop with a newer recovery copy it asks once (Open copy / Show Recovery… / Open saved show). The `.starting` marker skips the startup show once after a start that died loading it (E-SHOW-006). `LoadShow "x" /Version <stamp>` / `LoadShow /Recovery "<copy>"` open a copy as unsaved changes; `Import Show "x" <Data|Patch|Protocols|Users|Stations | objects>` takes parts by GUID as one Oops step; the Load show dialog is one dialog with parts.
- Signals (SIGTERM / SIGINT / SIGHUP) act like `Shutdown /NoSave` after an autosave copy; a second signal exits at once.

## Drives

- `Root.Drives` is this station's drive list: the OS's volumes probed by a watcher thread, ids `usb1` / `ssda` / `folder1` bound to the volume and remembered; `LumaCore/library/` on a drive is made on the first write. Every file word takes `/Drive "<id>"`; the `Drive` object and menu live in `lumacore-cmd-station`; a show never names a drive. `<luma-files>` on the `files` feed is every load / save dialog. Network drives are LumaCore's own, never an OS mount, and are not built; `Eject` is planned.
- `core` touches the disk only through `core::drive` (`// tidy: drive-ok (<reason>)` for a non-command thread); the sweep runs `Drive` lines in a sandbox copy of `library/`.

## The demo show

- The demo is the rig: `lumacore_objects::demo::build` runs the station's words on the sources in git (`library/mvr/ti2026.mvr`, `library/poolxml/ti2026-groups.xml`, `library/demo/manual.lum`); `library/shows/demo.luma` is gitignored and `demo::ensure` builds it when missing or stale; the catalog test checks counts and a SHA-256 of the normalized JSON against `tests/catalog/testdata/demo.golden.txt` (`LUMACORE_BLESS=1`). Every test runs on it (`demo::fresh_state()`, built once per process, a copy each).
