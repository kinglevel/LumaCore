---
title: LumaSession Runtime
category: docs
---
# LumaSession Runtime

Canonical language source: `docs/user/syntax/lum.md`
Authoritative decisions: `docs/meta/decisions.md` §10, §11 (4d), §18, §21 "Step 6
hardening", §26.

This document describes the code boundary that owns LumaSession semantics (who is in a
LumaSession, how stations join and leave, how shared state stays coherent). The concepts
and status are in [network-sessions.md](./network-sessions.md); replication is in
[replication.md](./replication.md).

## Principle

LumaSession handling is **not a registry engine**. The registry engines are only
`PlaybackEngine`, `PerfettoEngine` and `HttpUiEngine` (decisions §26/§27). LumaSession
logic runs as:

- **state-thread jobs** — `Join`, `Leave`, `Invite`, `Dismiss` and the command-port
  requests run on the one state thread that owns `ServerContext` (§26); join/leave I/O is
  deferred until after the job (known exception: `Invite` / `Dismiss` still do
  UDP I/O inside the command)
- **listener threads** — the UDP command port (:47653), discovery (:47655) and the TCP
  stream (:47658: a reader + writer thread per member, `spawn_link` on a member) do their
  own I/O and send jobs to the state thread through `StateHandle`

It stays separate from:

- network transport and NIC services (`remote::lumasession::ServiceAccess`, `Interface`)
- protocol lines (`crate::protocols`: sACN, Art-Net, OSC)
- UI (the LumaSessions menu renders `GET /api/lumasessions`; it never authorizes)
- saved show objects

## What It Owns

- LumaSession identity (name, location — station settings) and membership
- `Join LumaSession`, `Leave LumaSession`, `Invite Station`, `Dismiss Station`
- station status: `Standalone`, `Master`, `Member`, `IdleMaster` (only when alone) (built)
- who sends DMX (members don't; leaving/dismissal holds output)
- the link and replication (built, 6b): `ReplicationJournal`, the TCP :47658 stream
  (`remote/stream.rs`), forwarding, pending outbox, master flush and member apply
  (`remote/link.rs`)
- heartbeat, election, epoch fencing, known stations, decision prompt (built, 6c,
  `remote/election.rs`); split base, merge on rejoin, the merge prompt's default, timeout
  and `Pause LumaSession` (built, 6d, `remote/election.rs` + `remote/link.rs`)

It does not own:

- raw socket binding or per-NIC service filtering
- DMX transport
- shell rendering (Shell sessions live in `session_manager.rs`)
- show data

## Lum Alignment

Objects: `LumaSession`, `Station`, `Shell`, `Client`, `Server`. No aliases for
`LumaSession`; "Session" alone is not a keyword.

```text
List LumaSession
Info LumaSession
Join LumaSession "Festival"
Join LumaSession 10.0.0.5 /Merge
Claim LumaSession
Pause LumaSession
Leave LumaSession
Invite Station "FOH-2"
Dismiss Station "FOH-2"
Info Station
List Shell
Info Shell 1
Info Client 4
```

Station properties (`Set Station …`): `Name`, `LumaSession`, `Location`, `Priority`,
`Invite`, `Online`, `HoldOutput`, `OutputWhenAlone` — station-local, saved in the station
file.

## Tree

```text
Root.Runtime
├── LumaSessions
│   ├── Stations        discovery results
│   ├── Members         stations known to the master
│   ├── Listeners
│   └── Connections
├── Users.{n}           per-user runtime: Modes, Undo, ViewSessions (replicated)
└── Claims              claims by GUID (replicated)
```

`Root.Runtime.SessionEngines` is still a placeholder node in the tree; nothing registers
there.

## Behavioral Rules

### 1. One Master

One master at a time executes show-changing commands as the sending user and pushes the
results (§10). Priority (`Never` … `High`) only influences election (6c, built); it never creates a
second master.

### 2. Shells Outlive Clients

A client disconnect does not end its `Shell`; clients attach, detach and reattach.

### 3. Reconnect Is Native (6c, built)

Known stations (in the show) are re-invited with back-off; not diverged → silent rejoin;
diverged → the prompt Merge / Use Master / Use Mine / Cancel (6d, built). The station peeks
the master's show and lists what differs; Merge is greyed out with the reason when blocked
(other show, no common base, patch changed on both sides). The timeout
(`ShowSettings.merge_timeout`, 60 s) runs `merge_default`; `Pause LumaSession` stops it.

### 4. Sync Is Result-Based

Stations never re-run commands. The change path (`terminal/change.rs`) fingerprints every
ShowData subtree around a mutating command and files exactly the changed ones as the undo
frame, the replication op (post-change snapshots) and the journal entry. Built (6b): members
forward Master-scope lines over the TCP stream, the master commits and pushes ops
`(journal id, epoch, seq)` with undo steps and the `SessionRuntime`, members apply in
order and `Resync` on a gap; join is atomic and checks protocol and format. Built (6c): each
new master bumps the epoch; the lower epoch is fenced. Built (6d): `Join LumaSession … /Merge`
sends the joiner's show, split base and journal; the master runs the 3-way merge by GUID
and installs the result (stream protocol 2).

### 5. Access

Stations trust each other by source IP; the command port stays open and requests name their
user (§19 step 5). One rights gate in dispatch covers every transport. Invite on/off is the
station's join policy.

### 6. Ownership Is Session-Scoped

- `Claim` / `Unclaim` take and release editable ownership; claimed objects stay visible.
- Claims are keyed by GUID in `Runtime` and replicate with the session runtime (6d). Only
  existing objects can be claimed; areas have name-derived GUIDs.
- `Edit <obj> <n>` claims the object until the editor is left; `cd` never claims. Changing
  commands on an object another user holds are refused (E-CLAIM-001).
- `Claim <obj>` on someone's object asks them; no answer in 10 s hands it over (decided on
  the master); Admin `/Force` takes it at once.
- Selecting does not claim (§4); `Claim Fixture <n>` locks a fixture.
- The programmer belongs to its `User` (`ShowData.Users.{n}.Programmer`) and is shared by
  all of that user's stations and clients.
- Claiming `Patch` edits a draft; `Unclaim Patch` asks Keep / Discard / Cancel (4d).

### 7. Timing Must Be Shared (planned)

The master owns the session clock; members follow it. Playback is not yet driven from one
authoritative clock across stations.

## LumaCore Mapping

- [remote/lumasession.rs](../../../crates/lumacore/src/remote/lumasession.rs)
- [remote/handler.rs](../../../crates/lumacore/src/remote/handler.rs)
- [remote/stream.rs](../../../crates/lumacore/src/remote/stream.rs) — TCP stream
- [remote/link.rs](../../../crates/lumacore/src/remote/link.rs) — forwarding, flush, apply,
  merge
- [remote/election.rs](../../../crates/lumacore/src/remote/election.rs) — election, split
  base, decision prompt
- [terminal/claims.rs](../../../crates/lumacore/src/terminal/claims.rs) — claims by GUID
- [terminal/commands/session.rs](../../../crates/lumacore/src/terminal/commands/session.rs)
- [server/state.rs](../../../crates/lumacore/src/server/state.rs) — the state thread
- [session_manager.rs](../../../crates/lumacore/src/session_manager.rs) — Shell sessions
