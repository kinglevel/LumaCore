---
id: 0280
title: The function block: Sort is grid order, Offset spare slots
area: webui
status: built
date: 2026-10-19
built_in: round 3 4b, the left-out items
supersedes: []
superseded_by: —
confirmed_by: [`Set GridColumns "patch.fixtures" "Offset" n`, `GridState.offset`, `<luma-grid>.sortBy`]
source: [archive §34 L9757–9758, archive §34 L10358–10367, archive §34 L10410–10419, archive §34 L10575–10587]
---
# The function block: Sort is grid order, Offset spare slots

## Context
Round 3 item 11 listed the Patch tab's function block; Sort and Offset had no decided meaning when 4b was built. An artifact offered three Sorts (the grid's order, renumber by a column, no cell) and the user chose the grid's order; Offset took the recommended meaning.

## Decision
The Patch tab's function block: Insert fixtures… (claim), Patch to… (the address value dialog), Edit Patch…, Next free address (`Assign Fixture … At DMX`), Multipatch… (claim, value kind `copies`), Info, Sort, Offset, Clone… (planned). An `Action` declares `claim` (greyed in a claim area's Live with the one reason) and `value` (the value dialog it opens); Patch to…, Next free address, Info, Sort and Offset work in Live because the address is a live column. Sort is the grid's order: a view cell showing the column the patch grid sorts by; a tap picks another (`<luma-grid>.sortBy`; the user's grid state like a header tap, its `Set GridColumns` line) — no show change, no Oops step; rejected: renumber by a column and no Sort cell. Offset is spare slots between fixtures: a view cell kept as the patch grid's state (`Set GridColumns "patch.fixtures" "Offset" n`, `GridState.offset`, on the profile like a window setting); Patch to…, Next free address, Multipatch… and Insert fixtures pack a selection with n spare slots after each fixture by writing `/Every <the selection's largest footprint + n>`; 0 = packed, no `/Every`. Greyed cells stay in place and say why on a tap; Paste counts the station's clipboard.

## Consequences
A new patch action declares its claim and value on its row; never a client-only Offset or a Sort that writes the show. Clone… waits for map-and-clone-attribute-maps.
