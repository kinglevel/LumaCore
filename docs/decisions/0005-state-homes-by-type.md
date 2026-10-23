---
id: 0005
title: Every field has one home: show, runtime, station-local
area: tree
status: built
date: 2026-10-11
built_in: step 6b
supersedes: [object-tree-shape]
superseded_by: —
confirmed_by: none
source: [archive §10 L450–469, archive §21 L1411–1418]
---
# Every field has one home: show, runtime, station-local

## Context
Replication needs to know what travels; the first tree mixed saved, session-wide and per-machine state in one drawing. The LumaSession rewrite (§10, 2026-10-10) split state into three kinds, and step 6b (2026-10-11) placed each in its own subtree so the split is enforced by where a type lives, not by a per-node flag.

## Decision
Three homes, one per field. `ShowData` is replicated to every station and saved; only the session master changes it, by running commands as the sending user (see the replication record). `Runtime` holds session-wide runtime state — logged-in users with their mode flags and undo, membership, clock, claims, global variables — replicated, never saved. Station-local state — station identity and settings, keystore, UI, command lines, screens, clients, watches, scroll positions — lives in a `Station` subtree and is never replicated. A station's rig wiring is show data: its row of `ShowData.Stations`, applied by that station only. Screen configurations are show data on the profile, and live window changes write through to them (`screens::sync`) so a user on two stations sees the same layout.

## Consequences
When adding a field, decide which of the three it is and put it in that subtree; the type's place is the rule. A `ShowData` field also needs its subtree arms and file entry (see change-path-undo-subtrees); a `Runtime` field travels in the session result as a small snapshot; a station field never leaves the machine. Never flag a node as "local" inside a replicated type, and never copy station state into the show. What belongs to the user rather than the profile is programmer-owned-by-user.
