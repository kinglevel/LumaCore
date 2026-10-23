---
id: 0024
title: Invite, Dismiss, Leave and per-NIC services
area: lumasession
status: built
date: 2026-10-11
built_in: step 4d, step 6b
supersedes: [archive §21 L1475–1476 "UDP I/O inside"]
superseded_by: —
confirmed_by: [ServiceAccess]
source: [archive §11 L700–721, archive §21 L1429–1433, archive §27 L2690–2718]
---
# Invite, Dismiss, Leave and per-NIC services

## Context
The "DMX Protocols window" hardening took a reference console's network menu as the model. Step 6b noted that `Invite` / `Dismiss` still did UDP I/O inside the command; today they ride the outbox.

## Decision
Sessions are found by discovery beacons (station, LumaSession name, location, status, priority, version); joining by address stays for routed networks. `Join LumaSession "Festival"` / `<host>`, `Leave LumaSession`, `Invite Station "X"`, `Dismiss Station "X"`, `List LumaSession`. A station with `Invite` on (default) joins an invite without asking and takes the session's show, unsaved changes kept as an autosave copy first; Invite off tells the master so. A station that **leads** a LumaSession (a master with members, or alone after changing the show) never joins silently: its operator gets Use Mine (default) / Use Master / Cancel, Merge greyed (R1-22, Q26). A joiner always joins as a member; one outranking the master is told `Claim LumaSession` makes it master. Dismissed or leaving stations become standalone with a copy of the show and their output **held** until the operator resumes it or `OutputWhenAlone` is on; only stations that never joined send as standalone. `Invite` / `Dismiss` / `Join` queue an outbox request sent after the job — never network I/O inside it. Per-NIC services are station-local: `Set Interface "en1" "LumaSession"|"Shell"|"HTTP" Yes|No`; `lumacore_lumasession::ServiceAccess` (shared ArcSwap) filters the command port and discovery by source subnet, Shell and HTTP by the local address they arrived on; loopback always serves; a change restarts that listener. `Set Station "Online" Off` stops everything networked but localhost (it leaves first).

## Consequences
A new networked service is declared per NIC through `ServiceAccess`, never opened unconditionally. Where wiring and identity are stored: see station-table. Security stays light by decision (rigs are isolated networks).
