---
id: 0082
title: Show files only move forward, no migrations
area: showfile
status: superseded
date: 2026-10-11
built_in: 6a, 9b
supersedes: [archive §20 L1321–1322 "runs migration steps"]
superseded_by: show-files-upgrade-step-by-step
confirmed_by: [E-SHOW-001, E-SHOW-002, E-SHOW-004, the_demo_show_builds_loads_and_matches_its_golden]
source: [archive §20 L1319–1330, archive §21 L1359–1366, archive §21 L1385–1391, archive §21 L1623–1625, archive §23 L2162–2168, archive §23 L2190–2194]
---
# Show files only move forward, no migrations

## Context
§20 first planned per-version migration steps and `#[serde(default)]` on every new field. Step 6 hardening dropped that: the load fix-ups of the day (old users, old key bindings, old cue layout) were removed and the format became forward-only. Step 9b added the save guard after a stale server could have overwritten a newer file.

## Decision
`ShowSettings` carries `format_version` (the value is `FORMAT_VERSION` in lumacore-tree), `saved_with_commit` (the git stamp, set only by the station's `build.rs`) and `show_id`. Every load goes through one parse: a file with a newer format is refused naming the version (E-SHOW-001); an older format is refused naming both (E-SHOW-002). No migrations, no converter, no backward-compatibility code. The only load-time rewrites follow the build, not the format, and report in the System Monitor: a fixture whose type's footprint changed takes the new footprint where the slots are free, else stays and is listed (Q20, `patch::check::follow_footprints`); windows of a type this build doesn't know are dropped with a note, on load and on `Import UserProfile` (Q32, `UserProfile::drop_unknown_windows`). "A show always has an Admin" stays a rule (`ShowData::ensure_admin`).

Save guard: before writing, the server peeks at the existing file's format. A newer one is refused with E-SHOW-004 and the show is saved to `{name}_format{N}_{time}` instead, which becomes the loaded show, so autosave and Quick Save don't pile up copies; `SaveShow "x" /Overwrite` (or the pop-up's answer) writes over it.

## Consequences
Any change to a saved type bumps `FORMAT_VERSION`; the demo is rebuilt from its sources, never converted (see demo-show-test — the manual demo is lines, so a bump never breaks it). Every station of a LumaSession runs the same build (E-LUMA-006). Old shows' programmers start empty; nothing is migrated.
