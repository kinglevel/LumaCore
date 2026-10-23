---
title: LumaSessions And MultiStation
category: docs
---
# LumaSessions And MultiStation

## Goal

Keep the single-server model strong while several stations work on one show in a
**LumaSession**: the master executes every show-changing command and pushes the results
(`docs/meta/decisions.md` §10). Step 6 is split into **6a** groundwork, **6b** the link,
**6c** heartbeat / IdleMaster / election / epoch, **6d** identity, merge and ownership (all
built; §21 "Step 6 hardening").
Concepts: [network-sessions.md](./network-sessions.md); replication detail:
[replication.md](./replication.md).

## What Is Implemented (steps 6a to 6d, 2026-10-11)

### LumaSession (4d)

- Objects `LumaSession`, `Station`, `Shell`, `Interface`; verbs `Join`, `Leave`, `Invite`,
  `Dismiss`; runtime at `Root.Runtime.LumaSessions` (Stations, Members, Listeners,
  Connections)
- Status `Standalone` / `Master` / `Member` / `IdleMaster` (`remote::lumasession::status`)
- Discovery per NIC (station, LumaSession name, location, status, priority, version);
  join by name or address; Invite (on by default; autosave copy first) and Dismiss
- A member does not send DMX (`is_authority` false); leaving or dismissal leaves the
  station standalone with output held until `Set Station HoldOutput Off`
- `Set Station Name|LumaSession|Location|Priority|Invite|Online|HoldOutput|OutputWhenAlone`
  (station file); `Set Interface <nic> LumaSession|Shell|HTTP Yes|No` enforced live
- LumaSessions and Network menus over `/api/lumasessions` and `/api/network`

### Change Path And Coverage (6a)

- `terminal/change.rs::subtrees` is an exhaustive destructure of `ShowData`: one subtree per
  field, pools per entry (`Devices`, `DataPool <guid>`, `DatapoolPriorityOrders`, `ShowSettings`,
  `Spatial3D`, `Playback`, `Playbacks`, `Profile <guid>`, `User <guid>`, `Programmer <guid>`, `Protocols`,
  `EffectEngines`, `Variables`, `IconPool`, `AttributeDefs`, `FeatureGroups`,
  `ActivationGroups`). A new field fails to compile until listed.
- Each mutating command files exactly the changed subtrees as the undo frame, the
  replication op (post snapshots, into `replication_outbox`) and the journal entry.
- Everything is undoable except `Playbacks` (Oops never undoes a Go); creating and deleting
  users and profiles is undoable and replicated (a missing id = absent).
- Programmer and selection live on the user (`ShowData.Users.{n}.Programmer`), keyed by user
  number; profiles keep views, screens, keys and settings.

### Undo

- Each user's history is `Runtime.Users.{n}.Undo` — runtime, never saved in the show;
  members mirror the master's stack by replaying undo steps.
- `Oops` / `Redo` are replicated changes and file no undo frame of their own.
- `Clear Programmer User …` files the undo step on the **affected** user's stack.
- `Unclaim Patch` no longer clears undo stacks; Keep is one undo step (4d).

### The Link (6b)

- TCP :47658 stream (u32 length + zstd JSON `StreamMsg`; `Hello` → `Welcome` / `Refused`
  with E-LUMA-001 protocol, 002 format); UDP keeps discovery, Health, Login, Invite / Dismiss
- Members forward Master-scope lines (`registry::line_scope`); the master executes them as
  the sending user and, after every job (`flush`), commits ops `(journal id, epoch 1, seq)` to
  the `ReplicationJournal` and sends them with undo steps and the `SessionRuntime` (modes,
  claims, global variables, members with users)
- Members apply in order; a gap → `Resync`; a show load restarts the journal and sends a
  `Snapshot`; join is atomic (parsed and format-checked before install)
- Pending outbox (`AppState::forwards`): resent in order after a reconnect (back-off 0.5–5 s)
- Everything waits for the master; local programmer feedback is postponed
- Removed: UDP `ShowDataSnapshot`, `SubmitOperation`, `RequestOperations`, `Member`,
  `ReplicationPacket`; Proxmox testing
- Several servers on one machine: `--station <name>` (own files in
  `library/stations/<name>/`) and `--port-base N` (cmd N+3, shell N+4, HTTP N+6, stream N+8)

### Station Priority

- `StationPriority` (`Never|VeryLow|Low|Normal|High`), `Set Station Priority`, carried in
  discovery and Health
- `StationStatus` is `Standalone|Master|Member|IdleMaster` (IdleMaster is used since 6c, only
  when alone). `JoinPolicy` and `RemoteRequest::JoinNegotiation` are removed (6d)

### Health, Election and Epochs (6c)

- Heartbeat (`Ping` / `Pong`) every 500 ms on the stream, warning after 1.5 s, lost after
  5 s; link down on the NIC counts at once; latency, resends and outbox are shown in
  `Info LumaSession` and the status icon
- Election (`remote/election.rs`): each member computes the same winner (priority → results
  applied → online time ±5 s → station id; `Never` can't win); the winner becomes master with
  a new epoch, the others follow it and resend pending commands. IdleMaster only when alone
- Two masters: the lower epoch is fenced (no DMX) and its operator gets the prompt Merge /
  Use Master / Use Mine / Cancel (6d)
- `Claim LumaSession`: handover (new epoch) or Use Mine
- Known stations in the show (`ShowData.session.stations`, re-invited when back,
  `Delete Station`), patch version (`ShowData.session.patch_version`), the fuller show status set,
  output-distribution log, discovery Multicast Base Default / Alternative, undo frames with
  station and time

### Identity (6d)

- Every show object has a GUID (`NodeMeta.guid` or a `guid: Guid` field;
  `lumacore_base::Guid` defaults to a fresh v4). UI keys and known stations derive theirs from
  their id.
- `Identified` + `Walk` (`lumacore-tree/src/identity.rs`): every show type walks its fields
  with an exhaustive destructure, so a new field doesn't compile until it is classified as
  object pool, GUID link or data. `ShowData::identity()` reports duplicates, nil GUIDs,
  broken links, wrong-kind links and containment loops.
- Every stored link is a GUID (executor → sequence, line → network / user, route → line,
  user → profile, fixture → revision / mode, `Value.preset`, running playbacks keyed by
  sequence GUID, …). Command text stays text.
- Copies get new GUIDs, moves keep them; `Store` over an existing group keeps its GUID.
- `FORMAT_VERSION` 3: older (E-SHOW-002) and newer (E-SHOW-001) formats refused, missing or
  duplicate GUIDs refused (E-SHOW-003); `ShowSettings.show_id` (new on New Show).
- Undo / replication subtrees for pool entries are keyed by GUID (`DataPool(Uuid)`,
  `Profile(Uuid)`, `User(Uuid)`, `Programmer(Uuid)`); restore removes by GUID first, places
  at the slot the data names and refuses (never overwrites) a slot held by another object.
- Reference check after load, join, merge, Oops and Redo: broken links are kept and listed
  (sysmon "References", `Info Show`, `List Reference [/Broken]`).

### Merge On Rejoin (6d)

- Split base: an IdleMaster keeps its show at the split, a new master keeps the show it
  followed (memory + autosave `split-base`)
- Prompt Merge / Use Master / Use Mine / Cancel with a preview of what differs (`Hello.peek`
  → `Preview`); Merge greyed out when blocked (other `show_id`, no common base, patch changed
  on both sides); default and timeout from `ShowSettings` (`Show /MergeDefault`,
  `Show /MergeTimeout`, Admin); `Pause LumaSession` stops the countdown
- `Join LumaSession … /Merge`: the master picks the common base, runs
  `lumacore_tree::merge::three_way`, autosaves `before-merge`, installs the merged show and
  logs each item in sysmon "Merge"; refused with E-LUMA-005
- Stream protocol 2 (`STREAM_PROTOCOL`)
- Details: [network-sessions.md](./network-sessions.md) "Merge On Rejoin"

### Ownership (6d)

- Claims keyed by GUID in `Runtime` (`claims`, `claim_order`, `claim_asks`), replicated with
  the session runtime; areas have name-derived GUIDs; only existing objects can be claimed
- `Edit <obj> <n>` claims the object until the editor is left; changing commands on an
  object another user holds are refused (E-CLAIM-001)
- `Claim <obj>` on someone's object asks them (`Unclaim` / `Claim … /Keep`; 10 s → handed
  over; owner logged out → at once; Admin `/Force`); a Patch draft is never taken by timeout
- `List Claims` replaces `ListClaimed`

## Remaining Work

- Local programmer feedback ("instant, corrected later") is postponed
- `Invite` / `Dismiss` still do UDP I/O inside the command
- One authoritative session clock for playback

## LumaCore Mapping

- [remote/](../../../crates/lumacore/src/remote/) — command port, shell transport,
  discovery, `lumasession.rs`, `stream.rs` (TCP stream), `link.rs` (forwarding, flush,
  merge), `election.rs` (election, split base, decision prompt)
- [lumacore-tree/src/identity.rs](../../../crates/lumacore-tree/src/identity.rs),
  [lumacore-tree/src/merge.rs](../../../crates/lumacore-tree/src/merge.rs) — GUID walk,
  3-way merge
- [session_manager.rs](../../../crates/lumacore/src/session_manager.rs) — Shell sessions
- [lumacore-base/src/replication.rs](../../../crates/lumacore-base/src/replication.rs) —
  `PendingReplicationOp`, `ReplicationOp`, `ReplicationJournal`
- [lumacore-base/src/undo.rs](../../../crates/lumacore-base/src/undo.rs) — `ShowSubtree`
- [terminal/change.rs](../../../crates/lumacore/src/terminal/change.rs) — the change path
- [remote/replication.rs](../../../crates/lumacore/src/remote/replication.rs) — op apply
