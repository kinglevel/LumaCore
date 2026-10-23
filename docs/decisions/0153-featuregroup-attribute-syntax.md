---
id: 0153
title: FeatureGroup, Attribute, AttributeRule, EncoderBar words; scope options are owner keywords
area: attributes
status: built
date: 2026-10-14
built_in: 12f-a part 2
supersedes: [archive §30 L6154–6156 "Attribute is a command"]
superseded_by: —
confirmed_by: [E-CMD-007, the 12f-a flags test (no E-CMD-018)]
source: [archive §30 L6154–6224, archive §30 L6580–6584, archive §30 L6672–6674]
---
# FeatureGroup, Attribute, AttributeRule, EncoderBar words; scope options are owner keywords

## Context
`Group` is the DataPool fixture group and `Attribute` was a command, so "group" could not name an attribute group. The scope words were checked against every decision.

## Decision
One word, owner first: `FeatureGroup` — bare is the show's, `UserProfile FeatureGroup "FX"` yours, `UserProfile "Bob" FeatureGroup "FX"` Bob's. `Attribute` is an object: its Call keeps `Attribute "Pan" At 120`; Set / List / Info / Edit work on the definition. Names join with `+`, a quoted `*` is a whole-name, case-insensitive pattern, no Thru; a pattern matching nothing is E-CMD-007. Into a group: `Assign Attribute "Gobo1Pos" At FeatureGroup "Gobo"` (the archive's `Set … FeatureGroup "x"` is superseded), landing at the bottom; order by `Move Attribute "Zoom" At FeatureGroup "FX" 3`. Group lines reuse verbs: `Store FeatureGroup "Haze"`, `Label`, `"Paint"` and icon by `Set`, typed `Sort` / `Split` with `/By Name | Family | MainFirst`, Merge asks, `Move FeatureGroup "Shutter" At 2`, `Delete` only when empty. `AttributeRule` is numbered, first match wins (`Store AttributeRule "Pixel*" FeatureGroup "FX"`); the template is a property of ShowData or the UserProfile (asks). Answers are options on the asking line: `Unclaim Patch /Keep /Default | /Edit | /Mine <Default|Edit>`, `Default [UserProfile] Attribute /New` — `Default` is a command, never a `Set` value. `EncoderBar` is a profile-owned pool object (`Store UserProfile EncoderBar "Busking"`, a screen's `"EncoderBar"` On / Off / Toggle).

Scope options are the owner keywords: `/ShowData` (the shared layer), `/UserProfile ["x"]` (bare = yours), later `/Station ["x"]` — `Set Screen 1 EncoderBar /ShowData | /UserProfile`. Rejected: `/Global` (taken three ways), `/Local`, `/Own`, `/Mine`, `/Profile`, `Self`.

## Consequences
Every flag is declared (`/New /Unused /Hint /FixtureType /WithSubs /Only`; E-CMD-018 otherwise). The same scope switch serves variables, readouts, store filters, keys, WindowPreset and Usage when they come; `/Station` is not built. The show's definitions need Setup but no Patch claim.
