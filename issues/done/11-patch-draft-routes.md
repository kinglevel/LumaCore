# 11 — Routes and merge modes are drafted while Patch is claimed

**Type:** question · **Found:** 2026-10-13, side note of the replication analysis.

CLAUDE.md / §11: "Networks, lines, routes and merge modes are live and need no claim." But routes
and merge modes live on `patch.universes`, so while a Patch draft is open
(`core/patch_draft.rs`) they are drafted for the owner and refused for others (E-PATCH-008).

Decide: move routes/merge modes out of the drafted patch (keep them live), or change the doc.

## Decision (2026-10-17)

Hardened one question at a time, a reference console probed — decisions §11 "Issue 11 hardening" and
§34 "34f7c hardened":

- Fixtures **and universes** (Store, Delete, Move, Label, MergeMode) are the draft; others are
  refused (E-PATCH-008) while it is open.
- The wiring is the protocol lines: a protocol is an object and a line one row (`Sacn 1`,
  `LocalUniverse` + `Amount`). Lines name slots by number (a wiring point, not a link), stay
  live for anyone with the rights, and the draft never moves them.
- Move Universe in the draft asks when input lines stay behind on the old slot; a universe the
  draft deletes leaves the lines on its slot (`no universe`), Keep names them.

Not built yet: 34f7c builds the protocol objects, then 34g4's patch overhaul builds the draft rules.
