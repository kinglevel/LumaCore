---
id: 0021
title: The master executes as the sender; results flow in order
area: lumasession
status: built
date: 2026-10-11
built_in: step 6b
supersedes: [archive §10 L470–481 "programmer instant, corrected", archive §21 L1426–1428 "Old UDP pieces"]
superseded_by: —
confirmed_by: [E-LUMA-001, E-LUMA-002, E-LUMA-003]
source: [archive §10 L438–449, archive §10 L470–481, archive §21 L1394–1476, archive §21 L1500–1510]
---
# The master executes as the sender; results flow in order

## Context
§10 was rewritten after research: stations never re-run commands, replacing the March "remote stations replay commands". Step 6b built the link as one TCP stream per member and removed the UDP `SubmitOperation` / `ShowDataSnapshot` pieces. §10's "programmer changes show instantly, corrected later" was postponed: everything waits for the master.

## Decision
Every show-changing line from a member goes to the master, which runs it **as the sending user**, never as its own, and pushes the result. On a member, `dispatch` forwards every line whose `registry::line_scope` is Master (a `CommandDef` declares `scope: Client | Station | Master`, unset = Master; an object declares `ObjectDef::scope`) through `core::session_link::forward`, which puts it on the link outbox (`session_link::LinkOut` on `AppState::link`): pending until the reply, resent in order. After every job the master's `link::flush` sends `(epoch, seq)` subtree snapshots plus the sending user's undo steps and the session runtime to every member; members apply strictly in order, a gap or another journal asks for a `Resync`, a lower epoch is refused. Command text travels only as a label. A member's own show writes (screen write-through, station-local commands) go to the master as `Changes`. A join checks the stream protocol (E-LUMA-001), the show format (E-LUMA-002) and that the peer is a master (E-LUMA-003); the show arrives into a staging copy and replaces the station's only when complete and format-checked. Core never calls `lumasession/`; it asks through the outbox.

## Consequences
Never add a second sync path; no network I/O inside a job. A stream change bumps `SESSION_PROTOCOL` in `lumacore-wire` (see wire-crate-clients). Election: see master-loss-election; merge: see merge-on-rejoin; encoding: see session-stream-json-zstd-journal-cap. The sweep's replication property (ops applied in order give the master's show, twice = once) and the E-LUMA codes catch a drift.
