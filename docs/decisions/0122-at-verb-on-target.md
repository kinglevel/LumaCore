---
id: 0122
title: Target At value is the At verb; levels only with At
area: commands
status: built
date: 2026-10-15
built_in: 3d8
supersedes: [archive §33 L7778–7790 "child-first Master forms"]
superseded_by: —
confirmed_by: [`Props::hint`, E-SYN-004, E-SYN-001]
source: [archive §33 L7021–7026, archive §33 L7272–7278, archive §33 L7690–7697, archive §33 L7778–7790]
---
# Target At value is the At verb; levels only with At

## Context
`At` is one word for a value and a destination, and the old handlers guessed which from what followed. Masters were reached child-first (`Group 3 Master At …`), their mode by a bare word (`Master Highlight Mode …`), and `Set` could write a level.

## Decision
A verb's signature declares its `At` slot as `Value` or `Destination`, never both; nothing is guessed from what follows (`Fixture 1 Thru 4 At 50` · `Copy Group 1 At 2`). `<target> At <value>` is the At verb on that target — a default verb applies only when there is no `At` (see bare-object-default-verb). `At` declares its effect per target kind: fixtures, groups and presets are selected, then set (a following `At 70` sets the same selection); `Attribute "Pan" At 45` sets that attribute on your selection; `Master 2.1 At 80` sets its level; `At 50` alone acts on your selection. Levels are set only with `At` (`Master Group 4 At 50 Fade 5`, owner first: the Master object owns the group masters); `Set Group 4 "Master" 50` is refused with a hint to the At form (`Props::hint`); a child-first line is E-SYN-004; a level that isn't a number is an unknown word, E-SYN-001. A level stays a declared live property (`"Level"`) — readable, a `Wire` source and destination — never written by `Set`. A master's mode is a property, `Set Master "Highlight" "Mode" AlwaysOn`; the user's own masters are `Master "Highlight"` / `Master "Solo"`.

## Consequences
A typed `At` handler takes its line as `<target> At <value>`; an object that takes `At` declares the effect, never a second parse. Master rows are keyed rows on the registry; other numbers reply planned or none. Relative values: see relative-numbers-on-station.
