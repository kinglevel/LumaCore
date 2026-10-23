---
id: 0262
title: Claiming Devices edits a draft; live columns stay live
area: patch
status: built
date: 2026-10-19
built_in: 34g4 part 2a
supersedes: [archive §11 L621–626 "Unclaim Patch still clears", archive §11 L730–735 "claiming Patch takes a draft"]
superseded_by: —
confirmed_by: [E-PATCH-008, E-CMD-010, `Props::live`, `commands::live_draft::runs_live`, `patch_draft::rebase`, `claims::HolderWatch`, `tests/e2e/tests/kit-34g4.spec.ts`]
source: [archive §11 L621–626, archive §11 L727–770, archive §24 L2684–2690, archive §34 L9435–9441, archive §34 L9542–9555, archive §34 L9668–9672, archive §34 L9683–9688, archive §34 L10402–10404, archive §34 L10420–10427, archive §34 L10575–10587, archive §34 L10705–10712]
---
# Claiming Devices edits a draft; live columns stay live

## Context
Established consoles have Patch (claimed, staged) and Live Patch (a few columns, live). §11 first put it on a `Claim Patch` word whose Unclaim cleared every undo stack; issue 11 split the draft from the wiring; 34g4 moved the claim up to every device kind and made the live columns a declared set.

## Decision
`Claim Root.ShowData.Devices` (an area is its tree path, never a word; `enter patch` claims it too) takes a draft of the whole `ShowData.devices` (`lumacore_state::patch_draft`); the published show keeps the live devices. The holder's edits skip undo and replication; with no draft open, `Set` on patch data that isn't a live column is refused (E-CMD-010). Live columns are declared per object (`Props::live`; Fixture: Name, Note, Address, PosX…RotZ, MasterReact; a universe's Name and Note); the meta level stays live. While a draft is open, another user's live line — and every Oops / Redo — runs against the live devices swapped in for that statement (`commands::live_draft::runs_live`), and the draft follows cell by cell (`patch_draft::rebase`: a leaf both changed refuses the line, E-PATCH-008 naming the holder); such cells carry the violet dot (`PatchDraft.marks`). `Assign` / `Unassign … At DMX` need no claim; a place in the MVR tree does. `Unclaim Root.ShowData.Devices` asks Keep / Discard / Cancel (`/Keep`, `/Discard`); Keep is one Oops step; Unclaim never clears undo. A holder absent past the holder watch's timeout loses the draft, discarded, no copy. Lines and networks are never in the draft (see lines-one-type-runs).

## Consequences
A new patch column decides live or draft at its property, nowhere else. See universe-move-delete-ask, patch-dialog-one-tile-claim-button, import-mvr-reviews-against-the-draft (an open review is a draft change).
