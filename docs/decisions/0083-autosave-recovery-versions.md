---
id: 0083
title: Every save keeps a version; autosave writes recovery copies
area: backup
status: built
date: 2026-10-13
built_in: 12d part 5
supersedes: [archive §9 L417–419 "master-initiated network-synchronized save", archive §9 L422–430 "backups newest-10 scheme"]
superseded_by: —
confirmed_by: [backups_tests, plan_cleanup unit tests, E-CMD-012, E-SHOW-006]
source: [archive §9 L415–430, archive §21 L1654–1657, archive §23 L2147–2150, archive §30 L4722–4737, archive §30 L5242–5276, archive §30 L5338–5346, archive §30 L5361–5366]
---
# Every save keeps a version; autosave writes recovery copies

## Context
§9 wanted a master-initiated, network-synchronised save. The bugfixes branch built a `backups/` folder (newest ten); at the merge main's Versions won and the branch's hardening was kept.

## Decision
`core::backups`, station-local, everything through `core::drive`. The show file changes only when someone saves; every save keeps the file it replaces in `library/shows/.versions/<show>/<stamp>.luma` (`keep_version`, copied by the OS, never over a version from the same second). Recovery copies go to `.autosave/{show}_{reason}_{station}_{stamp}.luma`: the timed copy (the show's `Autosave` setting — the timer stays the show's, untitled shows too; `autosave_tick` never writes the show file), `before-load` (a load over unsaved work), `shutdown`, signal and lost-merge copies. A recovery copy that can't be written refuses to replace the show (E-CMD-012). Delete moves to `.trash`; pins live in `library/shows/.pins.json` (`Set Version|Recovery … Pinned|Note`); a Snapshot is a pinned version.

Cleanup (`plan_cleanup`, pure) runs after every save or copy with tiered rules; the figures are the station settings `RecoveryKeepDays` and `RecoveryCap`; signal / merge copies are kept longer, pinned files forever; a file whose manifest doesn't say why it was written is never removed. Lists read only manifests (`backups::manifest`, cached): saved_at, reason, station, user, note, counts.

Startup: the station writes `running` and removes it on a clean stop; the `.starting` marker (R1-17) skips a startup show that died loading (E-SHOW-006) and counts as an unclean stop; after one, with a newer recovery copy, ONE station ask: Open copy · Show Recovery… · Open saved show — never loads by itself, never asks twice.

## Consequences
Never run cleanup against the real library in tests (`backups::use_dir`). `Version` and `Recovery` are station-scoped objects (List / Info / Set / Delete; `Delete Recovery /Cleanup` asks), reach Drive. Opening copies and the history: see load-show-dialog-history.
