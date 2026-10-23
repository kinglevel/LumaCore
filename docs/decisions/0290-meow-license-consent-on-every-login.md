---
id: 0290
title: Meow License consent on every login
area: ui-kit
status: built
date: 2026-10-23
built_in: the Meow consent (2026-10-23)
supersedes: []
superseded_by: —
confirmed_by: [E-USER-010, a_login_waits_for_the_meow_license_and_consent_finishes_it, each_holder_waits_on_its_own_and_its_token_with_it, login-sysmon.spec.ts]
source: [session 2026-10-23, "Meow consent A is good" — docs/dev/ui-kit/mockups/2026-10-23-meow-consent/01-consent.html, approved]
---
# Meow License consent on every login

## Context
Before the repo goes public, every login asks the user to accept the Meow License, and says plainly that LumaCore is vibe-coded software with no warranty. A client-only confirm was rejected (one server-held dialog), and so was a Decline button beside Consent (variant B): one gold button and a quiet "Not me". The repo's code license is unchanged; the Meow License is the in-app consent only.

## Decision
Every login asks before it is finished: `POST /api/login` (the page, the app, autologin, `?autologin=now`), a shell's user + password hello (lumacore-cli, the shell port), `Login` on any terminal and a user switch. Unlocking a DeskLock is not a login and never asks. The question is one dialog from `consent::ask` in `lumacore-state` (the text's one door) opened on the board for the login's holder (its device, its shell client, or the station's console); its buttons are the lines `Consent` and `Logout`. Until `Consent` runs, the users family's gate refuses every line but Consent, Login and Logout (E-USER-010), and HTTP takes the login's token as no login (`/api/me` says `consent: true`). The pending logins are station-local (`AppState::consent`), never replicated or saved. Each consent writes "<user> consented to the Meow License" to the station log. The dialog reuses the wire's `Dialog` (text, an alert detail, note, topic `consent`): no protocol bump.

## Consequences
A new login path opens the dialog through `consent::begin` (or `await_token` for a token); a client that waits for the login keys on the topic. The show file is untouched. A script that logs in over HTTP consents through a shell (`tests/e2e/support/server.ts` does).
