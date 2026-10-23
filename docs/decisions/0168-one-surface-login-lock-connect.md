---
id: 0168
title: One surface for connect, login and lock
area: devices
status: built
date: 2026-10-12
built_in: 12a3
supersedes: [archive §30 L3674–3681 "lock-screen image on the User", archive §30 L3688–3689 "old home.html", archive §30 L3901–3903 "open event stream", archive §30 L4010–4011 "SSE station events"]
superseded_by: —
confirmed_by: [e2e `kit-surface.spec.ts`, e2e `connect.spec.ts`]
source: [archive §30 L3667–3681, archive §30 L3688–3689, archive §30 L3810–3824, archive §30 L3901–3910, archive §30 L3949–3955, archive §30 L4005–4017]
---
# One surface for connect, login and lock

## Context
The app's home page, the web login and the lock were three flows; the user asked for one Mac-style full-screen surface with switch user, designed as an artifact first. The lock image moved from the User to the profile with the rest of the look (see profile-carries-look).

## Decision
The kit's `<luma-surface>` is the one surface: connect, login, locked and the blurred secondary screen; the old `home.html` and login markup are gone. It draws the user's lock-screen image, clock and date, station and LumaSession / show, the current user large with a password field or "tap to unlock", the show's other users as `<luma-avatar>` (initials, the user's own colour, a rights badge), `Servers…` and a status line. Pre-login data come from the open `GET /api/station` and the open `station` feed on `/ws/feeds` (users, presence on every station via the master, lock pictures); the old SSE stream is gone. Start: the surface for the last server with the last user picked; a last user without a password logs in after the surface's countdown (a tap logs in at once, Esc stops it; tests pass `?autologin=now`); never an automatic Guest; Admin is always asked for the password (see rights-one-gate). Switch user switches the whole device: the previous user's token on this device is revoked, their other logins and their work stay. The lowest-numbered open screen gets the full surface, the others the blurred picture; a tap moves it. The desktop app shows the WebUI's connect page (`web/connect.html`) at `lumacore://localhost/`, then one OS window per screen on the station's page.

## Consequences
Never a second login page, a client-only lock or a poll for presence; a new pre-login fact goes into the station view and its feed.
