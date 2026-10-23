---
title: Show Files Autosave And Recovery
category: docs
---
# Show Files Autosave And Recovery

## Goal

Make `.luma` shows durable, safe, and recoverable across process restarts and
LumaSessions.

## The file

A `.luma` in `library/shows/` is a zip holding the `ShowData` tree as folders and pretty
JSON files (history: §8 and §30 "12a2 / 12a3 hardening" of
[the frozen decision log](../../archive/decisions-2026-10.md); the rule is the record
`luma-show-file-container` in [docs/decisions/](../../decisions/)). The format has a number, the
current `FORMAT_VERSION` in `lumacore-tree`; shows only move forward (below).

```text
manifest.json                         format, show_id, commit; every entry with SHA-256 and size
Settings.json  Playback.json  Protocols.json  Assets.json …   one file per top-level field
Devices/Devices.json                  fixtures, universes, Metadata.MVR (all but the types)
Devices/Dmx/Patch/FixtureTypes/1 MAC Aura XB/fixturetype.json
                                            /Revisions/2/source.gdtf   the GDTF file, the source of truth
DataPools/1 Default/datapool.json + Groups.json, PresetGroups.json, Macros.json …
Users/1 Admin/user.json + Programmer.json + Selected.json + Usage.json
UserProfiles/1 Admin/profile.json     views, keys, settings, theme, lock screen
                    /Icons/1 Logo/icon.json + icon.svg
                    /Images/1 Stage/image.json + image.jpg
Images/4 Stage/image.json + image.jpg the show's pictures
```

- A pool member's folder is `<no> <name>` for reading; the loader trusts only the number
  and the GUIDs inside.
- `manifest.json` is read first: another format is refused at once (E-SHOW-001 / 002), and
  every entry's SHA-256 is checked — a changed or missing JSON part refuses the file
  (E-SHOW-005); a damaged picture is only left out.
- **Embedded files** (pictures, icons, GDTF) have a size cap per file and per show
  (E-MEDIA-003 / 004; the limits are the media crate's constants). Each sits beside its manifest (GUID, name, SHA-256, type, size, width × height,
  source name). The station keeps one copy per SHA-256 in `library/media/.store/` and
  `library/fixturetypes/`; LumaSession members fetch what they lack from the master by
  hash, browsers read `GET /api/media/<sha256>`.
- **Large media stays on asset drives** (§8): the show names `AssetRoot`s, each station
  maps them to its own folder (`Set AssetRoot "Video" "Folder" …`), an `Asset` is root +
  path + SHA-256 + size; `List Asset /Missing`.
- **Icons are never in the show**: the standard set ships with LumaCore; a profile carries
  its own Icons and Images, its theme and lock screen, so `Export UserProfile "Bob"` →
  `library/userprofiles/Bob.lumaprofile` (the same layout) → `Import UserProfile "Bob"` in
  another show looks the same.
- A GDTF fixture type keeps LumaCore's GUID for links and stores the file's own
  `FixtureTypeID`; importing the same FixtureTypeID again adds a revision.

## What the station does

- `SaveShow`, `LoadShow`, `NewShow`, `DeleteShow` write and read the file above.
- `ShowData.ShowSettings` carries:
  - `format_version` — the build's `FORMAT_VERSION` (`crates/model/lumacore-tree`; `Info Show`
    prints it)
  - `show_id` — a UUID, new on New Show; a merge needs both sides to have the same one
  - `saved_with_commit` — the git commit of the build that saved it, with `-dirty` when
    that build had uncommitted changes (stamped by `apps/lumacore-station/build.rs`)
- **Older shows upgrade** (the record `show-files-upgrade-step-by-step`): a show saved in an
  older format, back to the oldest this build reads (`OLDEST_FORMAT`), opens upgraded step by
  step, as unsaved changes; the reply and the System Monitor say from which format. Its file stays
  as it was until you save, and then it is kept as a version. A file with a **newer**
  `format_version` than the build knows is refused, naming the version and commit that saved it
  (`E-SHOW-001`); one older than `OLDEST_FORMAT` is refused too (`E-SHOW-002`). Saving over a file in a newer format is refused too (`E-SHOW-004`):
  the show goes to `{name}_format{N}_{time}` and `SaveShow "x" /Overwrite` replaces it.
  "A show always has an Admin" stays a rule.
- **Identity**: every object in the show has a GUID and every stored link is a GUID.
  A file with an object missing its GUID, or two objects sharing one, is refused
  (`E-SHOW-003`, naming the paths). Copies get new GUIDs, moves keep them.
- **Reference check**: after a load, join, merge, Oops or Redo the show is checked.
  A link to a missing object is kept and listed (sysmon "References", `Info Show`,
  `List Reference [/Broken]`).
- `Info Show` describes the loaded show (name, show id, format, status, references, merge
  default and timeout); `Info Show "name"` reads a file.
- What is saved:
  - users and their **programmer and selection** (`ShowData.Users.{n}.Programmer`)
  - user profiles: views, screens, keys, settings
  - running playback state (`ShowData.Playbacks`): every running sequence resumes snapped
    at its saved cue on load
  - networks, lines, routes and merge modes (`ShowData.Protocols`)
- What is **not** saved: `Runtime` — users' modes (Blind, Highlight, Freeze, Solo start
  off), **undo history** (`Runtime.Users.{n}.Undo`, session-only), LumaSession
  membership, `GlobalVar`, journal, health. Station-local settings (network → NIC map,
  interfaces, station name) live in `library/station_settings.json`, not in the show.
- An invited station keeps its unsaved changes as an autosave copy before it takes the
  LumaSession's show.
- LumaSession autosaves go to `library/shows/.autosave/{show}_{reason}_{station}_{time}.luma`,
  reasons `invite`, `use-master`, `use-mine`, `before-merge` (both sides, before a merge on
  rejoin) and `split-base` (the show at a split, the base of a later merge; forgotten once
  the split is resolved — Use Master, LoadShow, CleanStart — so a restart never brings back a
  merge that was already decided).
- `ShowSettings.merge_default` (Merge) and `merge_timeout` (0 = no prompt; Unlimited
  = waits; the default is the setting's) are saved with the show: `Show /MergeDefault …`, `Show /MergeTimeout …` (Admin).

## Autosave and recovery

Everything below is built; the one open item is at the end. Syntax: [`SaveShow`](../reference/commands.md#saveshow),
[`LoadShow`](../reference/commands.md#loadshow),
[`CleanStart`](../reference/commands.md#cleanstart),
[`Show`](../reference/objects.md#show).

- **Recovery copies:** `Show /Autosave <minutes>` (0 = off,
  saved with the show) is how often a changed show gets a **recovery copy** in
  `library/shows/.autosave/{show}_{reason}_{station}_{stamp}.luma` — untitled
  shows too. The timed copy never writes the show file: that changes only when
  someone saves. Loading another show, New show or CleanStart over unsaved work
  keeps a `before-load` copy first (also after "Don't save").
- **A version on every save:** the file a save replaces goes to
  `library/shows/.versions/<show>/<stamp>.luma`. `List Version ["show"]` is the
  show's history (one row per save: saved, by, station, reason, note, changes,
  kept); `LoadShow "x" /Version <stamp>` opens one as unsaved changes; `Set
  Version <stamp> Note "x"` or `… Pinned On` pins it (never cleaned up) — a named
  snapshot.
- **Cleanup:** the station settings `RecoveryKeepDays` and `RecoveryCap` (defaults in
  the reference) thin the copies out: everything of the last hour, one per hour today,
  one per day for a week, then one per week; signal and merge copies are kept longer;
  pinned never go; copies made by hand are never removed. Deleted shows, versions and
  copies go to `library/shows/.trash/` for `RecoveryKeepDays`. `Delete Recovery /Cleanup` runs the rules now.
- **Crash recovery:** after a station stopped without a clean shutdown,
  when a recovery copy is newer than the show file, it asks once: Open copy (as
  unsaved changes) · Show Recovery… · Open saved show. It never loads by itself.
- **Taking parts of a show:** Backup › Library › Load show… shows the
  file's facts and its parts (Show data, Patch, DMX protocols, Users & profiles,
  Station table), each All / Pick / None. Everything = `LoadShow`; less = `Import
  Show "x" <parts | objects>`, taken by GUID into the loaded show as one Oops step.
- **A version is kept safely:** the replaced file is copied by the
  OS (never read into memory) through a temporary name; a second save in the same
  second gets `-2`, `-3` …; a very long show name still fits; if the old file
  can't be read the save stops and says why — it never goes ahead without its
  version. A console's `backups/` folder maps to Versions here.
- **A startup that died:** a `.starting` marker in the station's folder is
  written before the startup show loads and removed after. If the station died hard
  while loading (abort, out of memory), the next start skips that show once and the
  System Monitor says so (E-SHOW-006). A stop asked for while it loads (SIGTERM,
  Ctrl-C) removes the marker: the next start loads the show as usual. When both
  happen — the start died while loading and a newer recovery copy exists — the show
  is skipped first, then the crash-recovery ask offers the copy.
- **Autosave on a signal:** SIGTERM / SIGINT / SIGHUP keep a copy of
  unsaved changes (`library/shows/.autosave/{show}_signal_…`) before the server
  stops.
- **LumaSession autosaves:** `invite`, `use-master`, `use-mine`,
  `before-merge` and `split-base` copies (above). Every side that loses a show
  keeps a copy first.
- **Save guard:** `E-SHOW-004` (above).
- **Startup show:** `Show /Startup Last|None|"name"` (station-local; the
  last show by default) is loaded when the server starts or restarts.
- **CleanStart:** the way back when a show or the UI is unusable: an
  empty show, startup show None, default station settings. Show files on disk
  stay.
- **Questions before losing work:** `LoadShow`, `NewShow`, `Shutdown`,
  `Restart` and `CleanStart` ask Save / Don't Save / Cancel when there are
  unsaved changes (`/Save`, `/NoSave` answer up front).
- Still planned: a synchronized autosave in a LumaSession, where the master decides when
  to save and every station keeps a recovery copy at the same time (today every station
  writes its own on its own timer) — see [the roadmap](../../meta/roadmap.md).

## Background

The first plan (§9 of [the frozen log](../../archive/decisions-2026-10.md)) was one autosave policy for everything:
master-initiated, network-synchronized, with a recovery copy on every station and
a restore offer after a crash. What got built first were the copies that protect
work at the moments it can be lost — a signal, an invite, a merge decision — then
the timed recovery copies, a version on every save and the Backup menu. The rest
stays on the roadmap.
