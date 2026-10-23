---
id: 0142
title: Set names its target; one typed handler on the registry
area: set
status: built
date: 2026-10-15
built_in: 3c3
supersedes: [archive §30 L5059–5061 "set_rows one Set"]
superseded_by: —
confirmed_by: [E-CMD-006, E-CMD-015, E-SET-001, E-SET-002, E-SYN-007, the_shim_count_only_goes_down]
source: [archive §33 L7111–7115, archive §33 L7791–7802, archive §33 L7889–7909]
---
# Set names its target; one typed handler on the registry

## Context
A survey of `Set` found several `handle_set` routes, objects with their own `Set` handlers, a row router that took Interface and Show before their handlers ran, and a target-less form (`Set ClearBuffer On`) duplicating UserProfile properties.

## Decision
`commands::set::route` (`lumacore-objects`) is the Set command's one handler and takes every `Set` line, typed (`SetProperty`: target, quoted property, value): `Set <target> "<Property>" <value>`. A bare owner keyword is your own / this one (`Set UserProfile "ClearBuffer" On`, `Set Station "Name" "Luma01"`); a named owner scopes (`Set UserProfile "LD" "ClearBuffer" On`, `Set Interface "en1" "HTTP" No`). A target-less line is refused with E-CMD-006 naming the owner-first line; a bare `Set` is a usage error; an object without properties says so (E-CMD-015); a property the object doesn't have lists them (E-SET-001), a read-only one says why (E-SET-002).

Rows come from the one resolver, keyed rows too (interfaces, stations, the show); `Set <Object> 1 Thru 3 + 5 "<Prop>" <value>` writes many rows, every row one statement, one Oops step, refused rows skipped and named with codes. Owner-first paths scope all the way down (`Props::within`: `Set DataPool 2 Sequence 3 Cue 2 FeatureGroup "Dimmer" "Fade" 4`). The property binds from the right; a trailing `+ n` / `- n` after an implicit owner's property is relative; a glued negative there is E-SYN-007 — name the owner. `Set` takes the property's declared rights level. Lines the station writes (asks, `Put back:`, menu templates, remote edits) quote ids and properties.

## Consequences
No object-local `Set` handler, no `handle_set` arm, no `set::legacy_words`. A form that is not "a property and a value" gets its own verb (`Assign` into a feature group, typed `Sort` / `Split`). The value's parsing is the kind's, see property-kinds-values-reset.
