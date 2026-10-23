---
id: 0151
title: "The attribute resolver is data: table, family rules, templates"
area: attributes
status: built
date: 2026-10-14
built_in: 12f-a part 1
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §30 L5987–5991, archive §30 L6017–6026, archive §30 L6574–6579]
---
# The attribute resolver is data: table, family rules, templates

## Context
New attributes arriving with a patched fixture type need a feature group at once, and the user prefers an "FX" grouping over GDTF's nine. A reference console's standard attribute set was studied before the rules were written.

## Decision
LumaCore ships the standard attribute table (Pretty, Special, unit, activation group, MainAttribute) and an ordered family-rule list (pattern → Group.Feature, most specific first, keyword rules for vendor names) as data files in `lumacore-tree`'s `data/`; the GDTF importer keeps each revision's `<AttributeDefinitions>` (`Revision.attributes`). A template says which group each family goes to: "LumaCore FX" (Dimmer · Position · Color · Shutter · FX · Control · Shapers · Video) and "GDTF standard"; New Show offers both, the show keeps the one picked and edits it afterwards (`Set ShowData "AttributeTemplate" "LumaCore FX"`, asks). Rules are show data — editable, exportable, learning: after a Setup user moves one, "Apply to similar names?" stores an `AttributeRule`, numbered in order, first match wins. Deviation from the first design: a learned rule wins before the exact standard name, or learning a standard family would never take. Every placement records its step and confidence (exact · family · parent · context · hint · guess); hint and guess placements land in the patch's attribute editor.

Each profile has its own family rules, a copy of the show template's at first; the user's new-attribute dialog is prefilled by the user's rules with the show's grouping beside it, and every answer can be learned. The resolver works on either layer (`AttributeLayer`, see attributes-two-layers).

## Consequences
Never hard-code a grouping or a vendor name in Rust: a new rule is a row of the data files or a learned `AttributeRule`. A template library beyond the two shipped ones is on the roadmap; applying a template asks first.
