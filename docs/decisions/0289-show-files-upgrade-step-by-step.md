---
id: 0289
title: Older show files upgrade step by step
area: showfile
status: built
date: 2026-10-23
built_in: show upgrades (2026-10-23)
supersedes: [show-format-forward-only]
superseded_by: —
confirmed_by: [every_format_from_the_oldest_has_its_step, an_older_show_upgrades_step_by_step_to_today, E-SHOW-001, E-SHOW-002]
source: [session 2026-10-23, "it must be possible to load older showfiles … do the recommended"]
---
# Older show files upgrade step by step

## Context
Record 0082 refused every older format (E-SHOW-002). Three bumps in one day left the user's saved shows and the demo unreadable; the demo rebuilt itself only when a station started with it. A tolerant load (defaults for what is missing) was not taken: a renamed or reshaped field would be lost without a word.

## Decision
`OLDEST_FORMAT` (in `show_file::upgrade`) is the oldest format this build reads. A show from `OLDEST_FORMAT` up to `FORMAT_VERSION` is read whole and goes through one step per bump, each rewriting the show's JSON from format N to N+1 before it is parsed; every step says what it changed. A show older than `OLDEST_FORMAT` is refused (E-SHOW-002), a newer one too (E-SHOW-001). An upgraded show opens as unsaved changes and says so (the reply, the System Monitor); its file stays as it was until a save, which keeps it as a version. `Import Show`, a profile file and the `library/ui` files read through the same door. `LoadShow "demo"` builds the demo from its sources when it is stale, as a start with it does.

## Consequences
A `FORMAT_VERSION` bump ships with its step (an empty step when nothing saved changed shape); a test fails when a format has none, and each step is tested by taking today's demo back to the older shape and up again. Raising `OLDEST_FORMAT` deletes the steps below it. A change to the file layout itself (the files and folders) still needs its own reading code.
