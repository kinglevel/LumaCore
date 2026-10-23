---
id: 0023
title: Three-way merge by GUID on rejoin
area: lumasession
status: built
date: 2026-10-11
built_in: step 6d
supersedes: [archive §10 L514–526 "Merge All Keep Master", archive §21 L1598–1600 "Merge keeps both copies"]
superseded_by: —
confirmed_by: [E-LUMA-005, lumacore_tree::merge]
source: [archive §10 L514–526, archive §21 L1626–1657, archive §21 L1681–1692, archive §21 L1721–1726]
---
# Three-way merge by GUID on rejoin

## Context
A reference console merges only the same show file, matches by slot (a move duplicates) and offers Cancel / Use Master / Merge / Use Mine. LumaCore goes further on purpose: GUIDs on every object and a split base, so a move is a slot change of one GUID. The prompt uses the established words.

## Decision
At a split a station keeps its show as of the last applied `(journal, seq)` — in memory and as an autosave copy, with its `patch_version`. On rejoin, or when two masters meet, the merge is **3-way** (base, the session's show, mine) in `lumacore_tree::merge`, the smallest GUID object the unit: added on one side → added; both added in one slot → the master's keeps it, mine moves to the next free slot; deleted on one side and untouched → deleted; deleted on one side, **edited on the other → the edit wins**; changed on both → **the master's wins**; a move applies, nothing duplicates; `ShowData.Playbacks` always the master's. Only the same `show_id` can merge. Patch changed on both sides blocks Merge (E-LUMA-005; a timeout with default Merge falls back to Cancel). The prompt is Merge / Use Master / Use Mine / Cancel, listing what differs; `ShowSettings.merge_default` and `merge_timeout` (Admin); `Pause LumaSession` holds it; the decision previews the master's show (`Hello.peek`); `Join LumaSession … /Merge` asks the master to merge. **Every losing side autosaves first** (`.autosave/{show}_{reason}_{station}_{time}`). After merge, load, join and Oops / Redo the reference check runs (`core::references`): a broken link is kept and listed, never dropped.

## Consequences
A new object kind merges only once it has a GUID and `Walk` (see strict identity). No tombstones: the split base tells "deleted" from "never existed". The 6d tests follow a reference console's data-negotiation test cases.
