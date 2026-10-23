---
id: 0103
title: Nested objects are addressed owner first
area: commands
status: built
date: 2026-10-15
built_in: 12a2 / 12a3 (media words), lum part 3d7 (owners in the resolver)
supersedes: [archive §33 L7013–7020 "dotted Theme.AccentHue", archive §30 L3843–3845 "Set UserProfile Theme.Preset", archive §33 L7669–7670 "child-first cue timing"]
superseded_by: —
confirmed_by: [E-SYN-003, E-SYN-004, `Props::within` / `scoped`, `resolve::resolve_owned` / `slot_owned`]
source: [archive §30 L3801–3804, archive §30 L3839–3846, archive §33 L7027–7038 (item 9), archive §33 L7307–7325 (item 49), archive §33 L7479–7487 (items 59, 60), archive §33 L7664–7671 (item 87), archive §33 L7745–7790 (3d1–3d8)]
---
# Nested objects are addressed owner first

## Context
The media words wrote the rule down first; the lum discussion made it the only nesting form. A reference console nests owner first too; its child-first forms silently mean something else — not copied. Dotted property names (`"Theme.AccentHue"`) were tried and dropped.

## Decision
A nested object is written in tree-path order, keywords spaced: `DataPool 2 Group 3`, `Store Sequence 4 Cue 2`, `Goto Sequence 2 Cue 3`. A child alone uses your selected owner (`Cue 3` = a cue of your selected sequence); a bare `UserProfile` is your own. Child-first is refused with the owner-first form as hint (E-SYN-004: `Goto Cue 3 Sequence 2`); a keyword after a path that is neither a declared child nor a slot of the signature is E-SYN-003. A word naming another kind inside a line (`Park Fixture 1 Attribute "Pan" At 30`) is a declared helper slot, never a path.

A group of settings is a one-of-a-kind child object, written the same way: `Set UserProfile Theme "AccentHue" -30`, `Set UserProfile Theme Paint "Accent" "#9a7a08"` — never a dot inside the quotes. A cue's timing for one feature group is that group as the cue's child (`Set Sequence 2 Cue 3 FeatureGroup "Position" "Fade" 4`).

The resolver knows scoping owners: an owner whose Props declare `within` resolves to one row and scopes the next segment (`resolve::resolve_owned`); Store's slot under owners is `resolve::slot_owned` (`Store Sequence 9 Cue 1` may name an owner that doesn't exist yet).

## Consequences
A new child object needs no grammar change; it is a registered word in the one namespace. Every line, ask button, kit template and test was rewritten owner first in 3d7. The dotted `Root.` form is the other way to name a nested object — see tree-paths-typed-node.
