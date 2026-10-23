---
id: 0150
title: "Attributes: a show layer and a user layer, never mixed"
area: attributes
status: built
date: 2026-10-14
built_in: 12f-a
supersedes: [archive §30 L5920–5922 "EncoderBarPool on UserProfile"]
superseded_by: —
confirmed_by: [FORMAT_VERSION bump, E-USER-002]
source: [archive §30 L5920–5969, archive §30 L6083–6113, archive §30 L6585–6592]
---
# Attributes: a show layer and a user layer, never mixed

## Context
`ShowData.attribute_defs` linked each attribute to one feature group while the old tree put `EncoderBarPool` on the UserProfile — two owners of one grouping.

## Decision
The show layer is the patch's attribute definitions: each attribute belongs to exactly one `FeatureGroup`, seeded from GDTF and movable (`Assign Attribute "Gobo1Pos" At FeatureGroup "Gobo"`, Setup rights, no Patch claim). Cue timing (`CuePart.group_timing`), the sequence sheet and the encoder bar's Cue timing mode read only the show's groups; timing stored for an old group stays with the cue and is listed by the reference check. A patch change outside a draft defines its new attributes in the same step (`change::finish` → `attribute_resolver::sync`); that sync's write into the locked Default profile is never E-USER-002. The show layer learns only inside its show (`ShowData.attribute_rules`, `ShowSettings.attribute_template`).

The user layer is `UserProfile.attributes` (`ProfileAttributes`: template, groups, rules, `placed`, `encoder_bars`). The encoder bar follows the user's grouping, the show's on request (`Set Screen 1 EncoderBar /ShowData | /UserProfile`). The profile keeps every attribute ever placed: an unused one leaves the bar (slot kept) and returns without a dialog. A new attribute reaches every profile at its suggested spot, marked new; each user answers alone (Follow standard / Edit / Later), the show's Keep never waits, one user's answers never change another profile. A new profile's bar is one layout built from the show's groups, an editable copy. The knowledge travels with `Export` / `Import UserProfile`; an imported profile is checked against the show's definitions.

Presets stay open: a preset stores every active programmer value whatever its group. Individual fade / delay per value may be set through the user's groups, per attribute, never group-wide on a strip.

## Consequences
A show word reads the show's layer, `UserProfile` first reads yours (see featuregroup-attribute-syntax). Store filters are later, on the UserProfile.
