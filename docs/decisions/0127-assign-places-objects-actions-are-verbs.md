---
id: 0127
title: Assign places objects; Set never acts; actions are verbs
area: commands
status: built
date: 2026-10-15
built_in: 3d (the attribute area)
supersedes: [archive §33 L7656–7663 "Set Attribute FeatureGroup", archive §33 L7672–7689 "Set FeatureGroup Sort|Merge|Split"]
superseded_by: —
confirmed_by: [the typed `Sort` / `Split` rows with `/By`, `Assign` / `Call` in the generated reference]
source: [archive §33 L7656–7663, archive §33 L7672–7689]
---
# Assign places objects; Set never acts; actions are verbs

## Context
The attribute area had grown action words inside `Set`: `Set Attribute … FeatureGroup …` moved an attribute, `Set FeatureGroup "x" Sort|Merge|Split …` ran one-shot operations, `Set ShowData|UserProfile AttributeTemplate "…"` regrouped a whole layer. The two layers of 12f-a (the show's cue-timing groups and the profile's own grouping) must never mix, and `Set` is the one typed property handler (see set-one-typed-handler).

## Decision
`Set` only sets declared properties; an action is a verb. Putting an object into another is `Assign`, owner first per layer: `Assign Attribute "Gobo1Pos" At FeatureGroup "Gobo"` on the show layer (Setup plus the Devices claim), `Assign Attribute "Gobo1Spin" + "ColorWheelSpin" At FeatureGroup "FX" /WithSubs`, and `Assign UserProfile Attribute "Gobo1" At UserProfile FeatureGroup "Beam"` for your own grouping. Merging is objects into objects, `Assign FeatureGroup "Strobe" At FeatureGroup "Shutter"` (asks; the attributes move, the group goes, cue timing set for it is listed as broken). `Sort` and `Split` are general verbs taking `/By`, each object declaring the `/By` keys it takes: `Sort FeatureGroup "Beam" /By Name|Family|MainFirst`, `Split FeatureGroup "Effects" /By Family` (asks). `Sort` keeps one meaning — an object's contents (a pool is renumbered with `Compact`, see door-words-insert-compact-patterns). A template is applied with `Call … At …`: `Call AttributeTemplate "LumaCore FX" At ShowData`, `Call AttributeTemplate "GDTF standard" At UserProfile`, once, with the ask; learned rules stay. A group member's grid spot is its member child's properties, one per statement: `Set Group 3 Fixture 101 "X" 4`.

## Consequences
The attribute board's buttons write these lines; never add an action word or a multi-pair form to `Set`. Later holders of the same verbs: `Sort Group 3 /By Name`, `Split Group "All Spots" /By FixtureType`. `AttributeTemplate` is a listed object (`List AttributeTemplate`).
