---
id: 0003
title: Programmers are LTP; selecting claims nothing
area: programmer
status: built
date: 2026-10-10
built_in: step 4a
supersedes: [archive §4 L268–269 "derived fixture-selection claim"]
superseded_by: —
confirmed_by: none
source: [archive §4 L261–279, archive §27 L2578–2581, archive §27 L2590–2593, archive §27 L2620–2623, archive §27 L2633–2636]
---
# Programmers are LTP; selecting claims nothing

## Context
Established consoles have no attribute locks, and the earlier code derived a "fixture-selection claim" (`ensure_fixture_selection_available`) that stopped two operators from selecting the same fixture. Step 4 hardening (2026-10-10) chose LumaCore's own model so several operators can follow actors and next through fixtures without collisions.

## Decision
Programmers are LTP with each other and with playbacks: any user may set any attribute, the latest touch wins the output, and the value moves into the toucher's programmer and out of every other programmer. Every write goes through `programmers::program` (`core/programmers.rs`) — the one place that moves values, refuses claimed fixtures and applies the Blind rule: Blind values touch nothing and take nothing; leaving Blind counts as a fresh touch. Inactive values output too; only active values store. Selecting claims nothing — two users can select the same fixture. `Claim Fixture 3` locks the whole fixture: others can neither select it nor set its values; claiming a fixture whose attribute another user holds live is refused naming the holder. Explicit claims stay in the claims table (see claims-by-tree-path); who holds what live is derived from the programmers, there is no lock table.

## Consequences
Never set a programmer value outside `programmers::program`, and never reintroduce a selection-derived claim. Where the programmer sits in the merge is freeze-lifts-programmer; who owns it is programmer-owned-by-user. Decided, not built: a Preview programmer with no output, one per user (not one shared programmer), and Program Time fading programmer changes.
