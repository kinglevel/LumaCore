---
id: 0022
title: Heartbeat, election, IdleMaster only when alone, fencing
area: lumasession
status: built
date: 2026-10-11
built_in: step 6c
supersedes: [archive §10 L482–502 "IdleMaster after timeout", archive §10 L497–498 "Standalone Connected IdleMaster GlobalMaster"]
superseded_by: —
confirmed_by: [every_order_elects_the_same_winner]
source: [archive §10 L482–513, archive §21 L1477–1492, archive §21 L1511–1606, archive §29 L3108–3109, archive §29 L3259–3260]
---
# Heartbeat, election, IdleMaster only when alone, fencing

## Context
§10 let a cut-off station become IdleMaster after the timeout while the rest elected; 6c resolved this to election first, after a live probe of a reference console; `Connected` / `GlobalMaster` gave way to Standalone, Master, Member, IdleMaster, Fenced.

## Decision
Master and members beat on the stream (`lumacore_lumasession::election`: `BEAT`, `WARN`, `lost_after()`; debug builds take `--session-timeout`). A link down on the stream's NIC counts at once; the timeout still decides. After a loss every member computes the **same winner** from the replicated member list, counting only stations still in discovery: highest priority → most results applied → online time (whole buckets) → lowest stable station id; `Never` can't win. The winner becomes Master with a **new epoch** before any output and keeps the show; the others connect to it and resend their pending lines there. A station is **IdleMaster only when no winner is reachable**: its outbox runs locally, everything since the split is journaled, output only with `OutputWhenAlone`. Two masters meeting: the **lower epoch is fenced at once** (DMX stops) and its operator decides Use Master / Use Mine / Cancel, the buttons running `Join LumaSession "<name>"` / `Claim LumaSession` / `Leave LumaSession` (see merge-on-rejoin). `Claim LumaSession` hands the role over cleanly with a new epoch (an Admin on any member; a higher-priority joiner is told it may). `ShowData.session` (`SessionData`: `patch_version`, every station that ever joined) is replicated, never undone; known stations back in discovery are re-invited with back-off; `Delete Station "X"` forgets one. `Set Station MulticastBase Default|Alternative` picks the discovery group.

## Consequences
Election inputs come only from the replicated runtime and discovery; a change must keep every order electing the same winner (the named test). Undo frames carry station and time (`List Undo /ThisStation`).
