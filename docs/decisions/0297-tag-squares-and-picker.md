---
id: 0297
title: Tag squares arm Assign; Tag <objects> opens the picker
area: commands
status: built
date: 2026-10-24
built_in: tag assign (CHANGELOG 2026-10-24)
supersedes: [0208 "tags are given only in EditSetting's Tags tab" — the Tags window's squares and the tag picker give them too; Esc clearing the command line (the terminal's Esc chain)]
superseded_by: —
confirmed_by: [tag_with_objects_opens_the_picker_on_them, assigning_what_is_already_carried_changes_nothing, only_assign_or_unassign_tag_path_at_is_armed, a_tags_square_arms_assign_and_the_line_stays_for_the_next_cell, esc_never_clears_the_line, on_gives_each_tag_its_state_over_the_objects, kit-tag-assign.spec.ts, the_programmers_keys_are_lumacores_rows]
source: [docs/dev/ui-kit/mockups/2026-10-24-tag-assign (round 1, approved), the user in session 2026-10-24 ("assign tag x at", "Tag group X should open the tag picker", "Alt-a 4 times should be tag", "Tapping esc should only close open dialogs, never clear the cmdline")]
---

## Context

Tags worked (0207) but were given only in EditSetting's Tags tab (0208); the Tags window only
picked a tag. The user wants a pool square per tag that takes a pending Assign, then a run of pool
taps, and a tag picker like the colour picker. `Tag` is the Tag object, so a verb `Tag` would name
two words (0296's rule); a new verb word was rejected for a new form of the object.

## Decision

Each Tags window row has a pool square (`<luma-tree>` `squares`); a tap is the station's object
tap with the tag's path as `Tap.id`. With an empty line it runs the tag's action (its own, the
window's default, else it picks the tag). With `Assign` / `Unassign` alone it arms the line
`Assign Tag <path> At`; armed, another square switches the tag and any other object runs the line
and arms it again. `lumacore_cmd_tags::armed` says what an armed line is; the terminal sends
`armed_tag` (the tag's carriers by GUID) when it changes and the pools mark them. Assigning what is
already carried answers so and changes nothing.

`Tag <objects>` (a Tag with no path, objects after it) opens `<luma-tag-picker>` (the
`tag_picker` event, the `tags` feed's `on`); each tick is one Assign / Unassign line; a new name is
`Store Tag "x"; Assign Tag "x" At …`. New tags go at the top level. PRG's Alt+A cycles Assign ›
Label › Paint › Tag. Esc only closes (dialogs, suggestions, a panel's focus); it never clears the
command line.

## Consequences

The grammar is unchanged: objects after a Tag were already a value slot. A cell tap on a carrier
never removes the tag. Dropping a cell on a tag row stays open. `CLIENT_PROTOCOL` bumped.
