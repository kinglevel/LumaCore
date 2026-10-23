---
id: 0149
title: Notify reaches the LumaSession; profile category grid; Team object
area: other
status: decided
date: 2026-10-15
built_in: —
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §33 L7803–7845]
---
# Notify reaches the LumaSession; profile category grid; Team object

## Context
`Set Notifications <category> <level> <surface>` went with the target-less `Set` forms (see set-one-typed-handler) and had no home. It settled where the settings live and how one reaches everyone.

## Decision
A notification sent globally reaches the whole LumaSession — every station and every logged-in user — replicated through Runtime, never saved. `Notify "<text>"` sends one; no audience = the LumaSession; `At User "LD"`, `At Team "Crew"`, `At Station "FOH"` narrow it. `/Level Info|Warn|Error|Alert` (default Info) and `/Importance Normal|Important|Critical` (default Normal): Normal follows each profile's grid, Important cannot be muted and needs Setup, Critical pops up to everyone and needs Admin — the rights gate reads the declared option. A person's notifications are category Show; the system keeps its own categories.

How messages show is a category × level grid (None / Card / Popup) on the UserProfile — a profile is how the system looks and behaves, a User is the account. The grid object is `Notification`, one row per category, its properties the levels: `Set UserProfile Notification "Network" "Info" Card`. Sent notifications are not objects: Notify only sends, the System Monitor keeps the log.

`Team` is a named set of users by GUID in ShowData, replicated: `Store Team "Crew" At User "LD" + "Video"`, `Assign User "Bob" At Team "Crew"`, `Unassign User "Video" At Team "Crew"`, `Label` / `Delete` (asks); changing Teams needs Setup, anyone reads them. A `Notify` line runs on the session master like any Master-scope line and reaches members in `link::flush` — never a second sync path; not an Oops step, never saved.

## Consequences
Not built; the e2e "important events can not be muted" stays `fixme` until then. Chat comes later and hooks into the notification audience. The new words are registry vocabulary; lum.md's grammar is unchanged. Users known beyond one show are a note in `visions/notes/global-users.md`.
