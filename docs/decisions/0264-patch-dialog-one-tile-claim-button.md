---
id: 0264
title: One Patch tile, Live by default, a hold-to-claim button
area: webui
status: built
date: 2026-10-19
built_in: 34g4 part 3b, patch entry, round 3 3a
supersedes: [archive §34 L9442–9451 "bottom-right button is Exit", archive §34 L9581–9584 "Exit in the power slot"]
superseded_by: —
confirmed_by: [`tests/e2e/tests/kit-34g4.spec.ts`, `MenuDef.area`, the `patch` feed]
source: [archive §34 L9442–9451, archive §34 L9564–9594, archive §34 L9650–9691, archive §34 L9748–9756, archive §34 L9975–9994, archive §34 L10588–10602, archive §34 L10636–10647]
---
# One Patch tile, Live by default, a hold-to-claim button

## Context
Point 9 drew the Patch as a kit setup dialog with an Exit power button; after 3c the Menu's Patch and Live Patch tiles were still placeholders. Hardened one question at a time; round 3 anchored and animated the button. Declined: asking the holder like other claims; a recovery copy of a lost holder's draft.

## Decision
The Menu has one Patch tile, `Menu "Patch"`; it always opens Live and claims nothing. A menu row declares its claim area (`MenuDef.area`: the crumb Devices › DMX › Patch, the kind rail, the sidebar's tree modes, the `claim` / `exit` / `close` lines); `<luma-setup>` draws it from the `patch` feed (the holder gets `mode: draft`, everyone else `mode: live`). The claim button is the slab's last cell, bottom-right on a phone too: Live · "Hold to claim and edit" — a hold runs `Claim Root.ShowData.Devices`, filling gold over the hold time; a press only shows the hint. Claimed · "Draft · <holder>" (gold, yours) — a press leaves: Save / Discard / Cancel when the draft changed, back to Live at once otherwise. Another's draft greys it; an Admin's `/Force` stays on the command line. The ✕ while claimed runs `Unclaim Root.ShowData.Devices; Menu "Patch"` — the same question — so a draft never outlives its dialog. In Live every non-live column and New / Cut / Delete / Import / Paste lock with the one reason (`areaWhy`); state bars Live green, yours gold, another's violet. Starting or ending a draft republishes, so the feed flips at once.

## Consequences
Never a second entry, a page-side claim state or a "keep claimed" close. A lost holder: see staged-patch-draft; the grid's cells: see patch-function-block-sort-offset.
