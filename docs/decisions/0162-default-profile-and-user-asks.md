---
id: 0162
title: Default profile is a locked template; user asks are station asks
area: users
status: built
date: 2026-10-13
built_in: 12c2
supersedes: []
superseded_by: —
confirmed_by: [E-USER-002, E-USER-003, E-USER-004, E-USER-005, E-USER-006, E-USER-007, E-USER-008, E-USER-009]
source: [archive §30 L4578–4600, archive §30 L4648–4658]
---
# Default profile is a locked template; user asks are station asks

## Context
The Users & Profiles window needed a proper New user flow, and a blank profile had to be impossible ("a blank copy should never be possible"). Established consoles lock their default profile the same way. The role on the profile went with 12d's rights (see rights-one-gate).

## Decision
UserProfile 1 Default ships in every show as a locked template (`UserProfile.locked`, `ensure_default_profile`) that nobody holds; a dispatch backstop refuses any line that changes, renames or deletes it (E-USER-002), except one that replaces the whole show (`AppState::show_loads`: LoadShow, NewShow, CleanStart bring their own Default). A new user gets their own profile, a copy of Default named after them, or shares a picked profile; a new profile comes from Default, a copy of another or an imported file, never blank (`Store UserProfile "name" [/CopyOf n]`, `Copy UserProfile n At m` with new GUIDs; a duplicate name takes the next free). Every prompt is a station ask, never a page dialog: `Store User` (name, rights level, profile pick list, optional password, colour), `Store UserProfile`, `Import UserProfile … /As`, `Label UserProfile`, `Set User "x" Name "y"` (the GUID keeps every link), `Delete User … [/WithProfile]` saying what goes. Deleting a user logs them out on every station, revokes their tokens, releases their claims; one Oops step brings user, programmer and profile back. Refused with a code: deleting yourself, the last Admin, Admin itself (E-USER-003…009); a profile in use can't be deleted and the refusal names its users. Rights come only from the User.

## Consequences
A test or tool that edits a profile picks one past Default. Loading a show without a logged-in user logs that user out. The delete question is the one Delete ask (see oops-categories-statements).
