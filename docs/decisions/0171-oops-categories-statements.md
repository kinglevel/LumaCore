---
id: 0171
title: Oops: one list, category a label, statement a step, Delete asks once
area: undo
status: built
date: 2026-10-13
built_in: 11c (Oops); 12c2 and 12d part 5 (the one Delete ask)
supersedes: [archive §29 L3142–3146 "Programmer Selection ShowData Create", archive §29 L3357–3359 "Reset and Meow ReadOnly", archive §29 L3388–3392 "target-less Set CreateOops", archive §30 L4103–4107 "Delete without use stays", archive §30 L4589–4596 "handler asks for Delete User"]
superseded_by: —
confirmed_by: [`oops_tests.rs`, `a_members_station_step_is_undone_there_by_an_oops_on_the_master`, `a_grid_delete_asks_once_naming_its_rows`, E-CLAIM-001]
source: [archive §29 L3138–3157, archive §29 L3319–3406, archive §30 L4147–4150, archive §30 L4589–4596, archive §30 L5347–5360]
---
# Oops: one list, category a label, statement a step, Delete asks once

## Context
Step 11 studied a reference console's Oops; the user said "we only want categories so the user can have it coloured and see them in the lists". R1-62 / Q10 made every statement its own step; R1-61 / Q35 and 12d part 5 moved every Delete question into dispatch.

## Decision
One Oops list per user (`Runtime.Users.{n}.Undo`, replicated, never saved); Oops always undoes the newest step. The category (Programmer, Selection, Show, Patch, UI, Users, Settings, Keys, Station) is only a label (`List Undo /<cat>`; `OopsKind`, refined by `dispatch::step_category`). Every statement of a line is its own step: rights checked for all first, then each runs, is confirmed and forwarded alone; one that asks stops the line, its answer runs the rest (`Ask::then_run`). Station settings ride in a step as `UndoFrame.local`, put back only on their station (`oops::restore_local`). `/NoOops` files no step; the profile's Oops filters and `OopsConfirmation` (Always / Main / Never) apply (`oops::confirm_first`, before the line runs or is forwarded; set through `Set UserProfile`). A show load clears every list; running playback and the modes are never undone; a step touching another user's claim is refused (E-CLAIM-001). Delete asks once: `dispatch::delete_first` is the only Delete question (`/NoConfirmation` answers it; never in a stored key or macro line; Show, ViewButton, planned Deletes exempt); an object shapes it with `ObjectDef::delete_ask` (`set_rows::RowDelete` names the rows, refuses first those that can't go, runs the rest as one statement and one Oops step; a single User / UserProfile keeps its handler's question); handlers never ask again.

## Consequences
A new verb declares its `OopsKind`; a new Delete shapes `delete_ask` instead of asking; never a page confirm or a second ask path. The window is oops-window.
