---
id: 0034
title: Another station's identity edit goes to its command port
area: station
status: built
date: 2026-10-13
built_in: 12d part 4
supersedes: [archive §30 L4859 "edited remotely through the master"]
superseded_by: —
confirmed_by: [E-CMD-010, E-RIGHTS-001, remote_edit_tests, kit-12d "pending until it reports it"]
source: [archive §30 L5161–5167, archive §30 L5180–5236]
---
# Another station's identity edit goes to its command port

## Context
12d first said another station's identity is edited "through the master". Research over twelve projects decided direct: the master stores nothing of it, so it can't validate, undo or queue, and nothing would be editable without a master — which matters most for Priority, fixed exactly when a session is leaderless. Identity in the show was rejected too (a load could rename stations or change election priority).

## Decision
`Set Station "Tower" "Priority" High` for another station is sent absolute, with `/NoConfirmation`, as `SessionRequest::EditStation` straight to that station's command port (`settings_props::remote_station_set`), sent off the state thread (`election::edit_station`, retried), the answer coming back as a job. The target runs it as the sending user (last write wins), checks rights only there with the level `Station` declares (`objects::system::station_rights`; `Set Station <rows>` passes the sender's gate) and journals it with the sender's name. The reply is `old → new` plus a `Put back:` line; Oops never undoes it. A refusal comes back as it came (`[E-RIGHTS-001]`); an offline or silent station is E-CMD-010; a busy one E-CMD-017. Asks happen on the sender (`station_ask`: Online Off, HoldOutput On, Priority Never on the master; several rows ask once). `Toggle` and relative values need a reported value (else E-CMD-008). `AppState::remote_edits` (station-local, never saved) keeps a sent edit `pending` until answered and, for what a beacon carries, until discovery reports it; the `lumasessions` feed's rows carry `pending` / `refused`, drawn by `<luma-grid>` as a generic row state. A Priority change takes effect at the next election and the reply says so.

## Consequences
A new identity setting declares its level on `Station` and, if a beacon carries it, settles on the report. Wiring never takes this path (see station-table).
