---
id: 0195
title: One context menu for every list, built from the registry
area: webui
status: built
date: 2026-10-19
built_in: 3b
supersedes: [archive §30 L3553–3557 "hold show me"]
superseded_by: —
confirmed_by: [GET /api/object-menus, ContextDef rows, lumacore_views::context tests, e2e phone.spec.ts]
source: [archive §34 L9935–9952, archive §34 L10619–10635, archive §37 L11256–11265]
---
# One context menu for every list, built from the registry

## Context
"A general context menu with renumber etc. … all types of data a row has in a list, unified in any list / grid in the whole system." It replaced the pools' object sheet and the grid's right-click = Edit; the phone round gave it the phone's sheet.

## Decision
`GET /api/object-menus` (`lumacore_views::context`) gives every object's menu from the registry: the common actions its verbs take — Edit… (the edit card), Renumber…, Label… for every object with a Name, Copy, Move…, Delete, each marked `claim` when it changes more than live columns —, a Set part with every declared property (its FieldInput, level, live flag; the shared value or ‹different values›, greyed with the reason where rights or a claim refuse), then the object's own lines from `ContextDef` rows its crate adds (`ContextItem`: label, icon, verb, line, live — Fixture's Patch to…, Swap addresses…, Unpatch; Group's Select, Edit members; every object Info). The kit's `context.js` draws it on the selection: `<luma-grid>` opens it on right-click, a finger held still and released, the selection chip's ⋯ and the Menu key; a pool's hold opens it too. Renumber… and Address… open the value dialog (see renumber-is-move-skip-by). On a phone it is the kit's sheet rising over the slab, the object's own lines before Set, Set folded to one line; a tablet and a desktop keep the menu at the pointer.

## Consequences
Never a per-window menu list or a second object sheet; an object's extra lines are `ContextDef` rows beside its `ObjectDef`. Both ways stay: Set for one field from the menu, Edit… for the card. Fixture type / Mode / Revision join the Set part with the type-change part (not built).
