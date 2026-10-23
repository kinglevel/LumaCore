---
id: 0047
title: Every OSC line names its user explicitly
area: protocols
status: built
date: 2026-10-14
built_in: code review hardening
supersedes: [archive §11 L549–550 "names a user"]
superseded_by: —
confirmed_by: [E-RIGHTS-003, `RowDelete::affects`, `List Reference /Broken`]
source: [archive §31 L6884–6897]
---
# Every OSC line names its user explicitly

## Context
An input line that runs commands needs a rights level; the first rule only said "an Input line names a user". The code review asked what happens with no user, with Admin, with a user who is logged out or deleted. Rigs are isolated networks, so the rule stays light but explicit: never Admin as a fallback.

## Decision
Every OSC line (`Osc 1`) names the user that receives its commands: `/User` is required when it is stored (E-RIGHTS-003), and a line with no user or a gone user doesn't start — state "no user". Any user may be a line's user, Admin too, chosen explicitly within the storer's own rights level (only an Admin can name Admin); "never Admin" means never as a fallback. The line always runs: its user needn't be logged in anywhere. Its commands run as that user, through the one rights gate (see rights-one-gate), with that user's own programmer and selection, shared with anyone logged in as that user. Deleting a user a line names asks first and names the lines ("used by Osc 1"); going on stops the line ("no user") and the broken link shows in `List Reference /Broken`. Deleting several users names it per user (`RowDelete::affects`); the reply says which lines stopped.

## Consequences
No input path ever runs a line without a user, and no transport-specific rights check exists beside dispatch. A future Data protocol (MIDI, timecode) carrying commands inherits the rule through the Data kind's declared `User` property (see lines-one-type-runs); the `User` is a GUID link checked by the reference check. Nothing here is planned or open.
