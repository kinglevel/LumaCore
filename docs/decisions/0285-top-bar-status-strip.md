---
id: 0285
title: The top bar is a status strip with the modes
area: ui-kit
status: built
date: 2026-10-22
built_in: top bar, round 1 (2026-10-22)
supersedes: []
superseded_by: —
confirmed_by: [tests/e2e/tests/kit-topbar.spec.ts]
source: [session 2026-10-22, top bar round 1 — docs/dev/ui-kit/mockups/2026-10-22-top-bar (01-top-bar, 02-status-mark), approved]
---
# The top bar is a status strip with the modes

## Context
The top bar held a green dot, the word LumaCore and the user; the dot only said the page's socket was open, as the command line's ONLINE key did. The modes were 50 px keys in the left sidebar. The user's reference (`inspiration/A1 S1.png`) is one thin strip of facts. Modes first, a strip quiet until a mode is on, pills or dots, a chip or a signal meter were drawn and not taken.

## Decision
The top bar is `<luma-status-strip>`: the status mark, a Show tab (gold dot, the show's name, its unsaved dot), the facts as `Key: Value` (User, Profile, DataPool, Sequence, LumaSession), the indicators, the clock. The indicators are words, always there, lit in their mode's colour (`--lc-mode-*`): Blind, Highlight, Freeze, Solo, Oops with its step count, Meow. They are the UI keys themselves (`data-key`: a tap, a hold and the lit state as before), and leave the left sidebar. On a touch screen the strip is the touch height.

The mark is a ring and the station's name: full green standalone or master, half blue a member, spinning amber busy, blinking amber reconnecting, red lost, hollow with output off. A tap opens the station's details; the command line's ONLINE key and its mode and LumaSession icons go.

## Consequences
The strip's facts ride on `Status.strip` (a wire change). A screen showing the command line in its strip (A1's second screen) is open.
