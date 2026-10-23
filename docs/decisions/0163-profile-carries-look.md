---
id: 0163
title: The profile carries the look and travels as a file
area: users
status: built
date: 2026-10-12
built_in: 12a2 (the ask for a same-GUID import in 12b)
supersedes: [archive §30 L3667–3681 "LockScreen on the User", archive §30 L3805–3809 "needs /Overwrite", archive §30 L3847–3849 "needs /Overwrite"]
superseded_by: —
confirmed_by: [E-SHOW-003, E-MEDIA-001, E-MEDIA-002, E-MEDIA-003, E-MEDIA-004]
source: [archive §30 L3789–3800, archive §30 L3805–3809, archive §30 L3834–3849, archive §30 L4103–4107, archive §30 L4151–4160]
---
# The profile carries the look and travels as a file

## Context
The user wanted to "import the userprofile into another show and it should look the same without the need to import any more objects". 12a had put the theme and lock screen on the User; 12a2 moved them to the profile, since a user who wants their own look has their own profile. The first build made a same-GUID import need `/Overwrite`; 12b made it an ask.

## Decision
A `UserProfile` carries the look: `Theme` (see theme-on-profile), `LockScreen` (image, clock, text, auto-lock, default off) and two media pools, `Icons` and `Images`; profile UI links only to its own pools. The User keeps identity: name, password, rights, programmer, selection, which profile. Users of that profile and Admin may change it, one Oops step (UI). The show has no icons: a standard icon set ships with LumaCore, the same on every station, its names never removed (a name is a stable identity); custom icons and imported sets live in a profile's Icon pool, custom pictures for show data in the show's `Image` pool (`ShowData.images`, `IconRef::Image`). Copying works both ways with a new GUID (`Copy UserProfile Icon 3 At Image 10`). Media files sit in the store `library/media/.store/<sha256>.<ext>`, imported from `library/media/`, within the code's size limits (E-MEDIA-001…004). `Export UserProfile ["name"]` writes `library/userprofiles/<name>.lumaprofile`, a zip in the show file's layout; `Import UserProfile "name"` keeps the GUIDs, the same GUID asks Overwrite / Cancel, a GUID clash refuses (E-SHOW-003). Words go owner first through the `UserProfile` object (see owner-first-syntax).

## Consequences
Never add an icon pool to the show or a look field on the User. A new look setting is a declared profile property (see users-profiles-menu). Importing a profile into another show brings everything it looks like along.
