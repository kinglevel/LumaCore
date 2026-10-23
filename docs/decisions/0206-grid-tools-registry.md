---
id: 0206
title: Grid tools are one registry; the list button marks a menu
area: pools
status: built
date: 2026-10-14
built_in: 12e part 3
supersedes: [archive §30 L5476 "the tool row"]
superseded_by: —
confirmed_by: [grid_tools_change_the_group_layout_one_step_each, E-CMD-003]
source: [archive §30 L5442–5447, archive §30 L5501–5507, archive §30 L5519–5524, archive §30 L5528–5530, archive §30 L5553–5567, archive §30 L5572–5576, archive §30 L5653–5683]
---
# Grid tools are one registry; the list button marks a menu

## Context
The 12e group editor needed selection grid tools (Linearize, Remove gaps, Align, Rotate, …). Round 1 drew them as a plain tool row; the artifact rounds (2026-10-13) put them behind a Tools button so a mis-tap cannot reorder a group, reshaped the drill-down sheet into a grouped toolbar, and made the buttons one fixed grid after the icons misaligned.

## Decision
Tools are one registry, `core::tools` (`TOOLS`): id, label, Lucide icon, toolbar group (Order · Gaps · Align · Transform · Scale · Arrange · Pawterns), question (`ToolAsk::Choice` with an icon per value, `ToolAsk::Number`), status, transform. The one form is `Grid [DataPool d] Group n "<Tool>" ["<Arg>"]`, owner first, tool names quoted values (no new keywords); one statement, one Oops step, computed on the station; `Grid` with an object needs Program (`arg_rights`). `GET /api/tool-types` is open and static. A planned tool is listed greyed and refuses E-CMD-003. ↑ ↓ and a row drag stay `Set Group g Fixture f Order n`.

`kit/tools.js` draws them: on a desk `toolbar(spec)`, a grouped bottom toolbar docked in the editor that grows sideways and stays open after a tool; on a phone `openToolSheet(spec)`, a low sheet in thumb reach that closes after each tool; `ask(tool)` asks a parametered tool first; a one-shot tool runs on tap.

A button that opens a list or menu carries the kit's yellow `===` mark (`listMark()`, `<luma-button list>`): the one way to say "this opens a list" — never a hand-made marker or the word "asks"; planned tools keep "planned".

## Consequences
A new tool is a `TOOLS` row with its icon and group, never a page handler; the group editor (`kit/groupeditor.js`) and `<luma-grid reorder>` send only lines. Applying the list mark to the older pick buttons (settings links, window-bar choices) is a follow-up.
