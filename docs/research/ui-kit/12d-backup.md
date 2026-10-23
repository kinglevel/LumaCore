# 12d research: Backup menu (Show · Library · Recovery)

Date: 2026-10-13. Read-only research. Sources at the end.

## 0. What LumaCore has today (from code + decisions)

- `apps/lumacore-station/src/core/commands/backup.rs`: SaveShow / LoadShow / NewShow / DeleteShow /
  CleanStart, `Show /Description|/Autosave <min>|/Startup Last|None|"x"`, every ask a
  server-held dialog (`confirm`, "The show has unsaved changes."), `/NoConfirmation`.
- **Timed autosave saves the show in place** (`autosave_tick` → `save_show(state, None)`),
  only for a named, dirty show. There is **no copy of the previous file** on any save: a
  bad state autosaved at minute 15 overwrites the last good save. This is the biggest gap
  versus every project studied (the reference console, Blender, REAPER, Kdenlive all keep the previous file).
- `.autosave/` copies (`session_link::autosave_show`): reasons `signal`, `invite`,
  `use-master`, `use-mine`, `before-merge`, `split-base`, plus format copies
  `{name}_format{N}_{time}` beside the shows. **No retention**: the folder grows forever
  (today: 7 files, 4 × ~1 MB `signal` copies of one show in 6 hours).
- **Name stamps are inconsistent**: `demo_signal_T12_1791837123404` (unix ms) vs
  `demo_user-save-before-12c_20261013_0015` (hand-made) vs `NewShow_2026.10.11_13.32.51UTC`
  vs format copies `_YYYY.MM.DD_hh.mm.ss` UTC. Station names may contain dots and
  underscores (`Mac.halladalbo.smidn.com`), so `{show}_{reason}_{station}_{time}` cannot be
  split back reliably from the file name alone.
- `list_show_files()`: name, size, mtime, newest first. `List Show` marks the loaded one.
- The `.luma` zip's `manifest.json` is written first and already holds `format`, `show_id`,
  `saved_with_commit`, `show_name` — the list can read metadata cheaply **without loading
  the show** (open zip, read first entry). It does not yet hold why/who/where a file was
  written.
- Crash (SIGKILL, power loss, panic outside a guarded job) leaves nothing newer than the
  last timed autosave; there is no startup check for a newer recovery copy. §9 says "on
  crash recovery, offer to restore from autosave" — not built.
- Known UI bug (§29 findings): Backup menu "AutoSave Interval Off" while the header counts
  down "Autosave in 14:45".

## 1. Projects studied — what they do

### A reference console (closed)
- Backup menu: Save / Load / New / Import-Export …; Load list has a **search field** that
  filters as you type; a **Shows toggle** switches source: Shows · Backup Shows · Demo
  Shows · Template Shows (read-only folders; you can't save into them).
- **A backup is made on every save; up to 10 per show**, named
  `<show>_Backup_<date time UTC>`.
- **Load segments** (checkboxes): Show Data · Local Settings · Output Stations · DMX
  Protocols; unchecked = keep what is loaded now; Check All; Load disabled when none.
- **Partial show read** is separate: a stepped wizard (arrow tabs) to
  pull chosen objects from another show into the current one.
- Template shows = a folder you drop files into.

### Home Assistant (backups redesign, 2025.1 "3-2-1 Backup")
- Overview page: **last backup / next scheduled** at the top, "Backup now", "Show all
  backups".
- Automatic backups: schedule (daily / days + time), **"keep N" retention** (oldest pruned),
  backup before updates, **locations** (toggles: local, NAS, cloud), encryption on by
  default with an **emergency kit** download.
- List: multi-select, download, delete; open a backup → its locations, download per
  location, **restore everything or chosen parts** (folders/add-ons) — restore overwrites
  the selected parts.
- Restore at **onboarding** (fresh install: "restore from backup" on the welcome screen).
- Complaints: a **hidden default filter** ("Automatic" unchecked) made a manual backup
  invisible in "show all"; the manual-backup button vanished behind the automatic setup
  wizard; users asked to **name manual backups** and for the UI to **match the files on
  disk**; core #162678: after an upgrade lost automations, the user couldn't see *what was
  in* a backup or how to get pieces back — only a trash can.

### Proxmox VE (UI reference)
- Backup list per storage: date, notes, size, format, **protected** (lock icon),
  verification state; actions Restore · Show Configuration · Edit Notes · Change
  Protection · Remove.
- **Notes template** (`{{guestname}}` …) fills a note automatically; manual backups can be
  marked protected.
- **Prune options** keep-last / hourly / daily / weekly / monthly / yearly, applied in
  order; within each period only the newest is kept; a **prune simulator** shows which
  would go. Protected backups are never pruned.
- Restore asks "restore over existing" vs "restore to a new ID" (= restore as copy).

### Syncthing file versioning
- Trash can · Simple (keep N) · **Staggered**: first hour 1 per 30 s, first day 1 per hour,
  first 30 days 1 per day, then 1 per week until max age · External script.
- Versions live in `.stversions/` with time-stamped names.

### Nextcloud versions
- Default `auto`: 1 per day for 30 days, then 1 per week; options "keep at least D days",
  "delete after D days". **Named versions are never expired** (user label = pin).
- Version list in the file sidebar: time, size, who edited; restore makes the old version
  current (current becomes a version — nothing is lost).

### Grafana dashboard versions
- Versions tab: version, date, **who**, **notes** (save message); select two → **Compare**
  (human summary + expandable JSON diff); **Restore creates a new version** (history never
  rewritten), noted "Restored from version N". Keeps 20 by default (`versions_to_keep`).

### VS Code Local History (Timeline)
- An entry per save; sources labelled (save, refactoring, undo …), **named entries**,
  compare with current / previous, restore, rename, delete; `maxFileEntries` 50,
  **mergeWindow 10 s** (saves within the window fold into one entry), per-file size limit.

### Blender
- **Save Versions**: on save, the old file becomes `.blend1` (`.blend2` …, count set in
  prefs).
- **Auto Save** timer into the temp dir; **Recover ▸ Auto Save** opens a file browser;
  **Recover ▸ Last Session** opens `quit.blend` written on every quit (can't be disabled).
- Complaints/manual warnings: auto-save files have random-number names, so you need the
  detailed list view to see times; temp dir may be wiped on reboot; recovering an
  auto-save loses changes since then — users don't know which file is newest.

### REAPER
- `.rpp-bak` on overwrite (previous version, on by default); timed autosave every N min to
  a **timestamped file** in the project dir or **an additional directory** (another drive);
  "keep multiple" option. No retention → known disk creep in forums.

### Kdenlive
- A dated backup of the previous file **on every save**; **Project ▸ Open Backup File**
  shows a list of the current project's backups **with a thumbnail and date**.
- Tiered retention: up to 20 from the last hour, 20 from today, 20 from the last 7 days,
  20 older.
- Opening an older-format project makes `<name>_backup.kdenlive` first.

### Ardour
- **Snapshots** = alternate session files in the same folder: "Snapshot (& keep working on
  current version)" vs "Snapshot & switch to new version"; name defaults to date/time;
  editor **Snapshot list** (click to switch; unsaved changes → ask to save first); Open
  Session dialog shows snapshots under an expander per session.
- `.bak` and `.pending` (crash journal) → on start after a crash: "This session appears to
  have been in the middle of recording … Recover / Ignore".
- **Session ▸ Archive**: one compressed file with only the used media (export).

### LibreOffice Document Recovery
- On start after a crash: a list of documents with status, Start Recovery / Discard.
- Bugzilla: **Discard discards all, not the selected one** (114508); recovery loops at
  every start until Discard (118056); a broken recovery file crashes again → users need
  `--norestore`; dialog appears when users don't remember unsaved work; button words
  "Start"/"Discard" unclear (150428); crash notice and recovery dialog confused (135073).

### Obsidian File recovery (core plugin)
- Snapshot interval (default 5 min, only if changed) and history length (7 days); a
  per-file list of snapshots by time, preview, **copy to clipboard / restore**.

### QLC+ (open-source lighting)
- Forum: users keep asking for autosave / numbered saves; "restore last workspace at
  start" can **loop crashing** if that file is corrupt.

### OBS Studio
- Scene collections: Import window for many collections at once (from other apps), Export
  to a file; on load, a **Missing Files** dialog lists media not found (search a folder /
  replace / ignore), with a flag to suppress it. Media paths aren't portable.

### Gitea / GitLab (project export/import)
- Export is **asynchronous**: "export requested, you'll get a link"; the export lists what
  is/isn't included; import maps users and refuses newer export versions; downloaded
  archive is time-limited. Pattern: say what the file contains, refuse incompatible
  versions with a clear message.

## 2. Patterns extracted

| Topic | Pattern seen | Where |
|---|---|---|
| Previous file kept on save | always, rolling N | reference console (10), Blender (.blend1..N), REAPER (.bak), Kdenlive |
| Timed autosave target | a **separate** file, never the user's save | Blender, REAPER, Obsidian, LibreOffice |
| Retention | keep-N, or **staggered/tiered** by age | HA, Grafana, VS Code / Syncthing, Nextcloud, Kdenlive, Proxmox |
| Pin = exempt from cleanup | named / protected never pruned | Nextcloud, Proxmox |
| Restore without loss | restoring makes a new version / current becomes a version | Grafana, Nextcloud, Proxmox "new ID" |
| Metadata | time (relative + absolute), size, who, where (location/station), **reason/source label**, notes, version/format | Grafana, VS Code, Proxmox, HA |
| Grouping | per show (expander), by day sections | Ardour, Kdenlive, Time-Machine-style |
| Search | filter-as-you-type on load list | reference console |
| Contents / preview | thumbnail, "show configuration", summary, compare | Kdenlive, Proxmox, Grafana |
| Partial load | segments checkboxes; object-level import wizard | reference console load segments + partial show read, HA partial restore |
| Crash prompt | at start, list of what can be recovered | LibreOffice, Ardour, Blender (menu, not prompt) |
| Crash prompt pitfalls | loops, discard-all, unclear buttons, crash on bad file | LibreOffice, QLC+ |
| Status header | last backup / next autosave / where | HA overview |
| Hidden filters | cause "my backup vanished" | HA complaint |
| Missing assets | list on load, fix or ignore | OBS |
| Disk | location toggles, size per item | HA, Proxmox |

## 3. Proposals for Show · Library · Recovery

Each: **now (12d)** = in this step; **later** = roadmap. Everything stays a command line;
every ask is a server-held dialog; lists come from one State feed (`backup` or a field on
an existing one) — no backup-specific endpoint.

### A. Safety model (the backend under the menu)

1. **Keep the previous file on every save — "Versions"** — **now (12d).** Before `write_show`
   replaces `<name>.luma`, move the old file to `.versions/<name>/<stamp>.luma` (or
   `.autosave/` with reason `previous`). This is what the reference console (10 per show), Blender, REAPER and
   Kdenlive all do; today an autosave can overwrite the only good copy. Cheap: one rename.
2. **Timed autosave writes a recovery copy, not the show** — **now (12d), recommended.**
   `autosave_tick` writes `.autosave/<name>_timed_<station>_<stamp>.luma`; the show file
   changes only on Save / Quick Save. The header still says "Autosave in 14:45" and the
   show stays "unsaved". Alternative kept as an option (`Autosave InPlace|Copy`), because
   console-trained users may expect in-place. Reason: every studied app separates "the user's
   save" from "the safety copy"; LibreOffice and Blender users trust Save to mean "I chose
   this state". Combined with A1, in-place is also acceptable — see question 1.
   Also autosave **untitled** shows as copies (today untitled never autosaves).
3. **Retention for `.autosave/` and versions** — **now (12d).** Staggered like
   Kdenlive/Syncthing, one rule set, pruned after each write (no thread needed):
   last hour: all (≤ 12) · today: 1 per hour · 7 days: 1 per day · older: 1 per week up to
   a cap (e.g. 30 days or N files) · **reasons `signal`, `before-merge`, `use-master`,
   `use-mine`, `split-base`, `format` never pruned automatically within 30 days** (they
   are the "someone lost" copies) · a total size cap per show (e.g. 500 MB) prunes oldest
   unpinned first. Settings live on the **station** (`StationSettings.recovery`: keep
   days, max MB) — it is disk on this station, not show data. `List Show /Recovery` shows
   what would be pruned (Proxmox's simulator, as a dry-run flag) — later.
4. **Pin (protect) a copy** — **now (12d).** `Pin` / Protected flag (Proxmox/Nextcloud):
   pinned copies are never pruned. Store as a sidecar or as a `pinned` list in station
   settings (keyed by file name) — not inside the zip (no rewrite).
5. **One name scheme, parseable** — **now (12d).** `{show}_{reason}_{station}_{YYYYMMDD-hhmmss}Z.luma`
   with `station` sanitised (dots/underscores → `-`), and **the truth in the manifest**:
   add `saved_at`, `reason`, `station`, `user`, `show_id`, `object counts` (fixtures,
   sequences, cues) to `manifest.json` (FORMAT_VERSION 11; forward-only, no migration —
   old files just show "—"). The list never parses names; names are only for humans in
   Finder.
6. **Crash marker + startup recovery prompt** — **now (12d).** A `library/stations/<id>/running`
   marker (written at start, removed on clean exit). At start, if the marker exists and a
   recovery copy of the startup show is **newer than the show file**, the station opens one
   server-held dialog: "LumaCore stopped unexpectedly at 14:03. A recovery copy from 14:01
   (2 min newer than 'demo' saved 13:30) exists." **Open copy** · **Open saved show** ·
   **Show Recovery…**. Lessons from LibreOffice/QLC+: ask **once** (the marker is cleared
   whatever is chosen, so no loop); never auto-load (a corrupt copy must not crash-loop);
   open a copy as **unsaved changes on the named show** (Save writes over the show, the old
   one goes to Versions) — no "discard all" button. Station starts with an empty show when
   the startup show itself fails (already the CleanStart idea).
7. **Load over unsaved work** — exists (ask). Improve **now**: the ask's third button
   "Save copy and load" is implicit: any load/new/clean start over a dirty show writes an
   `.autosave` `before-load` copy first, always (like the merge-loser rule), so "Don't
   save" never loses work silently. Message says where it went.
8. **Restore never overwrites** — **now (12d).** Opening a recovery copy or a version loads
   it as the current show **under the original name, dirty** (Nextcloud/Grafana: the
   current file becomes a version when you save). `Restore … /AsCopy` saves it under
   `<name>_restored_<stamp>` instead — later if time is short.
9. **Disk space** — **now (12d)**: free space of the `library/` volume in the Show tab
   header (`fs2`/`statvfs` via `core::drive`); warn (yellow) under 1 GB or < 10× show size,
   refuse save with E-CMD-012 + clear text when the write fails. Sysmon note when pruning
   ran because of the size cap.

### B. Show tab (current show)

- **Header card** (HA overview pattern) — now: show name, description, "Saved 13:30 (12
  min ago) · unsaved changes", format 10 · build `abc123`, size, fixtures/sequences counts,
  **Autosave in 14:45 → copy** (fix the "Off" vs countdown bug), free disk.
- **Actions** — now: Save (Quick Save) · Save As… · New… · Rename (later) · Open folder
  (desktop app only, later).
- **Autosave settings** — now: interval (Off / 5 / 10 / 15 / 30 min, named numbers, typed
  `<luma-settings>`), mode (Copy / InPlace if kept). Retention lives under Recovery (station).
- **Startup show** — now: Last / None / "name" (exists as `Show /Startup`).
- **Versions of this show** — now as a section (the list from A1): time, who, station,
  reason, size; Open (as dirty current) · Pin · Delete. **Compare** (Grafana) — later
  (needs a ShowData diff summary; `lumacore_tree::merge` has the machinery).
- **Snapshot** (Ardour "keep working" — a named, pinned version with a note) — now:
  `SaveShow /Snapshot "before rehearsal"` = a pinned version with a note. "Switch to it"
  is just Save As. Cheap and valuable before a show day.

### C. Library tab (show browser)

- **List columns** — now: name · last saved (relative, absolute on hold) · size · format
  (✓ / "newer — can't open" / "older — refused" from the manifest, without loading) ·
  saved on (station) · ★ pinned. Loaded show marked. Sort: Last saved (default) · Name.
- **Search** — now: filter-as-you-type (reference console), kit search field; **no hidden filters**
  (HA lesson): a source switch is visible as segmented tabs, and counts show "3 hidden".
- **Sources** (a source toggle) — now: Shows · Demo/Templates (read-only folder
  `library/shows/templates/`, "New from template") — templates later if not trivial.
- **Favourites / pinned shows** at the top — later (nice, not needed for 12d).
- **Detail pane on tap** — now: manifest metadata + counts (Proxmox "show configuration",
  HA "what's in it"); Load · Delete · Export · Duplicate.
- **Load segments** (reference console) — later, recommended design: Load dialog with toggles
  **Show** (always) · **Station settings in show?** — note: in LumaCore station settings
  aren't in the show at all (§10), so the reference console's "Local Settings / Output Stations / DMX
  Protocols" segments mostly collapse into: Show data · Networks & lines · Patch · Users &
  profiles. Real **partial import** (PSR: pick objects from another show by GUID, copies
  get new GUIDs) is its own feature — later; it shares the merge/GUID code from §21 6d.
- **Import / Export** — now: Import = upload a `.luma` from the browser (kit file drop) or
  pick from a USB/asset drive (`Root.Drives`); conflict on name → ask Rename / Replace
  (old goes to Versions) / Cancel; refuse newer format with E-SHOW-001 shown in the dialog
  *before* copying. Export = download the `.luma` (browser) or copy to a drive. Export
  with versions / "archive" (Ardour) — later. Missing media/GDTF on load (OBS) — the
  warning exists in text; a list in the dialog — later.
- **Delete** — now: ask, and move to `.trash/` (kept 30 days, restorable from Recovery)
  instead of unlinking — Nextcloud trash pattern; cheap insurance on a touch screen.

### D. Recovery tab

- **Grouped by show, then by day** (Ardour/Kdenlive/Time Machine) — now. Rows: time
  (relative + absolute), **reason as a label with a plain word** (Timed · Before load ·
  Shutdown signal · Before merge · Lost merge (use-master) · Split base · Previous save ·
  Format copy · Deleted), station, user, size, ★ pin. The current show's group open first.
- **Actions** — now: Open (as dirty current, A8) · Pin/Unpin · Delete · Export. Later:
  Compare with current, Open as copy.
- **Retention settings + usage** — now: "Recovery uses 41 MB of 120 GB free · keeps last
  hour / hourly today / daily 7 days / weekly 30 days · Max 500 MB" and **Clean up now**
  (asks, shows how many go). Prune preview — later.
- **Trash** (deleted shows) as one more group — now if C-Delete goes to `.trash`.

### E. Multi-station (LumaSession) notes

- §9 says the master decides save timing and every station gets a recovery copy. Proposed:
  the master's timed autosave triggers each member to write its own `timed` copy locally
  (members already hold the full show) — **later** (needs a stream message); for 12d every
  station autosaves its own copy on its own timer (members have the replicated show), which
  already gives "every station has a recovery copy".
- Recovery lists only local files (station-local, never replicated) — now. Station column
  shows where a copy was written so a copy carried over by USB is recognisable.

### F. Touch

- Rows ≥ 48 px, one tap = select + detail pane, actions in the pane (no hover, no
  right-click), long-press = the same pane; relative times; swipe-to-delete **not** used
  (accidental on a desk); confirm destructive actions via the server dialog.

## 4. Known user complaints to design against

1. Autosave/backup names that don't tell when/why (Blender random numbers) → reason +
   time columns from the manifest.
2. Hidden list filters (HA 2025.1) → visible segmented sources, counts.
3. "Discard" discarding everything (LibreOffice) → per-item actions only; no bulk discard.
4. Recovery dialog looping / crashing at every start (LibreOffice, QLC+) → marker cleared
   on first answer; never auto-load.
5. Can't see what's inside a backup / can't get pieces back (HA #162678) → detail pane
   with counts; partial import later.
6. Backups filling the disk (REAPER, HA 45 GB) → staggered retention + size cap + usage.
7. "Recover auto save" loses changes since the last copy without saying so (Blender) →
   the dialog states both times and which is newer.

## 5. Decision questions (also in the summary)

1. Timed autosave: (a) write a recovery copy, show file only on Save — recommended;
   (b) keep in-place, with the previous file kept as a version; (c) setting per show.
2. Versions on save: (a) previous file kept on every save, staggered retention —
   recommended; (b) last 10 per show; (c) none.
3. Retention home + defaults: (a) station setting, staggered + 500 MB cap, merge/signal
   copies kept ≥ 30 days — recommended; (b) per show in ShowSettings; (c) keep-N only.
4. Startup recovery prompt: (a) marker + one dialog when a newer copy exists —
   recommended; (b) only a badge on the Recovery tab; (c) none.
5. Manifest metadata (format 11): (a) add saved_at/reason/station/user/counts now —
   recommended; (b) list from file names + mtime only.
6. Partial load/import: (a) later as its own step (GUID import, a partial show read) — recommended;
   (b) console-style segment toggles in 12d.
7. Delete: (a) move to `.trash` 30 days — recommended; (b) delete at once after ask.

## Sources

- Home Assistant backups: https://www.home-assistant.io/common-tasks/general/#backups, https://www.home-assistant.io/blog/2025/01/03/3-2-1-backup/, https://community.home-assistant.io/t/2025-1-backing-up-into-2025/821339?page=15, https://github.com/home-assistant/core/issues/162678, https://www.home-assistant.io/actions/hassio.restore_partial/
- Proxmox: https://lists.proxmox.com/pipermail/pve-devel/2022-April/052792.html, https://forum.proxmox.com/goto/post?id=638604
- Syncthing versioning: https://docs.syncthing.net/users/versioning.html
- Nextcloud versions: https://docs.nextcloud.com/server/stable/admin_manual/configuration_files/file_versioning.html
- Grafana version history: https://grafana.com/docs/grafana/next/visualizations/dashboards/build-dashboards/manage-version-history.md
- VS Code 1.66 Local History: https://code.visualstudio.com/updates/v1_66
- Blender recovering data: https://docs.blender.org/manual/en/2.83/troubleshooting/recover.html
- REAPER project prefs: https://www.cockos.com/wiki/index.php/Preferences_Project
- Kdenlive backup: https://userbase.kde.org/Kdenlive/Manual/Project_Menu/Open_Backup_File, https://kdenlive.org/news/historic/drupal/feature-week-kdenlive-backups/
- Ardour: https://manual.ardour.org/working-with-sessions/snapshots/, https://manual.ardour.org/working-with-sessions/backup-and-sharing-of-sessions/
- LibreOffice bugs: https://bugs.libreoffice.org/show_bug.cgi?id=114508, https://www.libreoffice.org/bugzilla/show_bug.cgi?id=118056, https://bugs.libreoffice.org/show_bug.cgi?id=150428, https://www.libreoffice.org/get-help/bug/show_bug.cgi?id=135073
- Obsidian file recovery: https://obsidian.md/help/plugins/file-recovery
- QLC+ forum: https://forum.qlcplus.org/viewtopic.php?p=46400
- OBS: https://obsproject.com/kb/portable-mode, missing-files commit in obs-studio UI
