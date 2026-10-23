---
id: 0296
title: Paint is a verb; a theme's colours are its Slot
area: commands
status: built
date: 2026-10-24
built_in: the Paint verb (CHANGELOG 2026-10-24)
supersedes: [0294 "Theme Paint" — the theme's colour object's name only: it is `Slot` now]
superseded_by: —
confirmed_by: [paint_opens_the_picker_on_the_objects_with_their_set_line, kit-paint.spec.ts "Paint Group 2 typed opens the picker", every_name_means_one_thing, lum_words.golden.txt, the_programmers_keys_are_lumacores_rows]
source: [docs/plans/active/2026-10-23-profile-and-key-editor.md, the user in session 2026-10-24 ("Paint group 1 and the color picker opens directly"; the choice "Rename slots to Slot")]
---

## Context

The user wants to type `Paint Group 1` and get the colour picker at once, for any object, and Paint
as Alt+A's third press. `Paint` was the theme's colour object (`Set UserProfile Theme Paint "Accent"
…`), and a name means one thing — an object is never also a command. Keeping `Paint` for the object
and calling the verb something else was rejected: the user chose to rename the object.

## Decision

`Paint <object> <ids>` is a verb (typed handler `PaintObject`, the media family): it resolves the
objects through the one resolver, finds their `PropKind::Paint` property (none: E-SET-001) and
sends the `paint_picker` UI event with `Set <target> "Paint" {value}` and the first one's Paint now;
the page opens `<luma-paint>`, whose Please runs that one line. A terminal gets the Paint and the line.
The theme's colour object is `Slot`: `Set [UserProfile …] Theme Slot "<colour|slot>" …`, `Store … Theme
Slot "<slot>"`, `Reset … Theme Slot …`. The Paint value `Theme "<slot>"` is unchanged. PRG's Alt+A
cycles Assign › Label › Paint.

## Consequences

Nothing saved changes shape (the object's name is a word, not data). A client never builds the
picker's line: it comes in the event. `CLIENT_PROTOCOL` bumped for the event.
