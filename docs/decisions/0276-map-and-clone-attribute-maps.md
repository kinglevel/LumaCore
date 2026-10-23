---
id: 0276
title: Map & clone: AttributeMap pool, AttributeRule, Clone verb
area: patch
status: decided
date: 2026-10-18
built_in: —
supersedes: [archive §34 L9813–9818 "scoped to all or to a pair"]
superseded_by: —
confirmed_by: none
source: [archive §34 L9733–9745, archive §34 L9803–9836, archive §34 L10033–10070, archive §34 L10173–10184]
---
# Map & clone: AttributeMap pool, AttributeRule, Clone verb

## Context
A type change leaves values without a target; A defined-attributes dialog seen on an older console inspired a cross-clone. The window went through seven mockup rounds; per-pair scoped rules were replaced by map lines.

## Decision
One window, Map & clone (`map-clone.html` v7 is the first build): a type / revision / mode change and `Clone` share it, no switch; it opens on a plain-language summary listing only what needs a choice, the details as steps; a decisions bar with Meaning | DMX output (Meaning default), Remember for this pair, Referenced presets too, and Keep theirs · Take mine · Overwrite when targets already hold values. Maps are objects: an `AttributeMap` pool in Devices — a pair (from type / mode → to type / mode) with its lines; `/Map n` names one; `/Remember` makes it the pair's default, else it is deleted after the apply in the same Oops step. Defined attributes are `AttributeRule`s with an `Attribute` target (`Store AttributeRule "Dim*2" Attribute "Dimmer"`; ordered, first match wins, show-wide; a profile's rules only group — E-CMD-006 otherwise). `Clone Fixture 1 Thru 12 At Fixture 101 Thru 112 [If Sequence 1 Thru 10] [/Merge | /Overwrite]`: values in cues, presets, programmers and parked, places in groups, tags and selections, the fixture itself; never the patch; Program rights, no claim; the map opens only when the types differ. Colour converts as a whole through CIE (`palette` plus LumaCore's own lighting-colour module; ΔE reported, always "approximate"); unmapped values are kept unused and listed, never dropped silently.

## Consequences
Not built (round 3 part 6). Parts join the window after their own hardening (see cells-on-fixture-type); the built-in rules from community.smidn.com's data are reviewed before they ship.
