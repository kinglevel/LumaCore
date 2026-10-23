---
title: LumaSessions
category: docs
---
# LumaSessions

Canonical language source: `docs/user/syntax/lum.md`
Authoritative decisions: `docs/meta/decisions.md` §10 (multi-station sync), §11 "Menus,
LumaSession and staged patch" (4d), §18 (naming), §19 "Step 5 hardening" (security), §21
"Step 6 hardening".

This document describes the server-to-server side of LumaCore: stations joined together in
a **LumaSession**. It is concise and conceptual; status is marked **built** or **planned**
throughout (6a–6d built).

## Principle

A LumaSession is live runtime, not saved show data and not a protocol output.

- `LumaSession`
  - the server-to-server mesh: stations, join / invite / dismiss / leave, discovery,
    replication. Always this word — never "Session" or "network session", and no aliases.
- `Station`
  - a running server that can join a LumaSession
- master
  - the one station that executes show-changing commands and pushes the results
- `Client`
  - one attached device or window (CLI, app window, browser tab); never owns authority
- `Shell`
  - the terminal-session object (logged-in users and their shell clients) — not a LumaSession
- `Network`
  - only a **data network** (§11: "Rig", "Guest" mapped to a NIC). Not the mesh.
- `sACN`, `Art-Net`, OSC
  - protocol lines (`crate::protocols`), not LumaSession concerns

Established consoles are the main reference (a Network menu, host status, Invite/Dismiss, merge prompt).

## Lum Alignment

Objects: `LumaSession`, `Station`, `Shell`, `Client`, `Interface`, `Server`.

```text
Join LumaSession "Festival"        # by name (found by discovery)
Join LumaSession 10.0.0.5          # by address (routed networks)
Join LumaSession 10.0.0.5 /Merge   # rejoin and merge (6d)
Claim LumaSession                  # take the master role
Pause LumaSession                  # stop the merge prompt's countdown
Leave LumaSession
Invite Station "FOH-2"
Dismiss Station "FOH-2"
List LumaSession                   # every station discovery sees
Info LumaSession
List Station / Info Station
Set Station Name|LumaSession|Location|Priority|Invite|Online|HoldOutput|OutputWhenAlone …
List Interface
Set Interface en1 LumaSession|Shell|HTTP Yes|No
List Shell / Info Shell 1          # terminal sessions
List Client / Info Client 3
```

`JoinSession` / `LeaveSession` and the old `Session` object are gone.

## Tree

```text
Root
├── Runtime                       session-wide runtime: replicated, never saved
│   ├── LumaSessions
│   │   ├── Stations              discovered stations (name, IP, LumaSession, location,
│   │   │                          status, priority, invite, version)
│   │   ├── Members               stations known to this master
│   │   ├── Listeners             this station's network listeners
│   │   └── Connections
│   ├── Network                   data networks: Interfaces, Lines, ArtNetNodes, Sources
│   ├── Users.{n}                 numbered like ShowData.Users: Modes, Undo, ViewSessions
│   ├── Claims                    claims by GUID (`List Claims`)
│   ├── Journal
│   └── Health
└── StationSettings               station-local: never replicated (name, LumaSession name,
                                  location, priority, invite, online, output when alone,
                                  NIC services, network → NIC map)
```

## Where State Lives (§10)

| Kind | Where | Replicated |
|---|---|---|
| Show, incl. `ShowData.Users.{n}.Programmer` (programmer + selection) | `ShowData` | yes |
| Per-user modes, undo history | `Runtime.Users.{n}` (`Modes`, `Undo`) | yes |
| Session runtime: membership, claims, global variables (clock: planned) | `Runtime` | yes |
| Station-local: settings, interfaces, keystore, UI, command lines, screens, clients | station | never |

- The split is enforced by type (which subtree a field lives in), not by a flag.
- Programmer and selection belong to the **User**, are saved with the show, and are shared
  by every station and client the user is logged in on.
- Running playback state is `ShowData.Playbacks` (saved, replicated, never undone).

## Canonical Distinctions

### LumaSession Vs Show

A LumaSession never lives in `Root.ShowData`. Membership, discovery and reconnect are
runtime; the same show may be loaded into different topologies. The LumaSession
**replicates** the show; it is not part of it.

### LumaSession Vs Protocol Output

- LumaSession runtime lives in `Root.Runtime.LumaSessions`
- protocol lines live in `Root.ShowData.Protocols` and are started by `crate::protocols`
- in a LumaSession **only the master sends DMX** (and listens to DMX input); see "Who Sends"

### Station Vs Client Vs Shell

- `Station`: a network-visible server with a stable id, priority, online time, invite policy
- `Client`: an attached surface; many clients attach without creating stations
- `Shell`: one logged-in user's terminal session on a station; it outlives client
  disconnects and can be reattached

### Claims

- `Claim` / `Unclaim` take and release editable ownership; claims block mutation, not
  `List`, `Info` or `Watch`.
- Claims are keyed by GUID (`Runtime.claims: BTreeMap<Uuid, Claim>`, `claim_order`,
  `claim_asks`); areas (Patch, ShowData, Devices, DataPools, Protocols, UserProfiles,
  ShowSettings, StationSettings) have name-derived GUIDs. Only existing objects can be
  claimed. `List Claims` (`Root.Runtime.Claims`) shows them (6d).
- `Edit <obj> <n>` claims the object (editor claim); leaving the editor (`Exit` / `..`)
  releases it. `Enter` still claims areas; `cd` never claims.
- A changing command naming an object another user holds, or anything inside it, is refused
  (E-CLAIM-001).
- `Claim <obj>` on someone's object asks them: the owner answers `Unclaim <obj>` (hand over)
  or `Claim <obj> /Keep`; no answer in 10 s → handed over (decided on the master); owner
  logged out → handed over at once; Admin `Claim <obj> /Force` takes it. A Patch draft is
  never taken by timeout. The owner's UI shows a Keep / Hand over pop-up from the status
  packet's `claim_asks`.
- **Selecting does not claim** (§4); `Claim Fixture <n>` locks a fixture.
- Claiming `Patch` takes a **draft** (staged patch, 4d); others' Live Patch edits to
  fixtures are refused while it is held (E-PATCH-008). Networks, lines, routes and merge
  modes need no claim.
- Claims are session runtime (`RuntimeState::claims`) and replicate with `Runtime`.

## Behavioral Rules

### 1. Master Executes, Results Flow Out (§10)

- Every show-changing command from any station goes to the **master**, which executes it
  **as the sending user** and pushes the **result** (post-change subtree snapshots) to every
  station. Stations never re-run commands; the command text is only a label.
- Results carry `(epoch, seq)` (a new master bumps the epoch); stations apply in order and ask
  for missing ranges on a gap (`Resync`, full snapshot if trimmed). A lower epoch is rejected
  and its master fenced (6c).
- Each `CommandDef` / `ObjectDef` declares `scope: Client | Station | Master` (unset =
  Master); members forward Master lines and run the others locally.
- In 6b everything waits for the master, programmer and selection too; "instant, corrected
  later" is postponed until waiting is noticeable.
- Undo is replicated as steps with the user's runtime state; `Oops` from any station goes to
  the master. Undo is never saved.
- The master is authoritative for save timing.

### 2. Station States

`Standalone`, `Master`, `Member`, `IdleMaster` (a station that lost its master and is alone) — the
`lumacore_session::StationStatus` variants.

### 3. Joining, Invite, Dismiss, Leave (4d)

- LumaSessions are **named and found by discovery** (UDP :47655 beacons per NIC that serves
  LumaSession: station, LumaSession name, location, status, priority, version). Join by
  address stays for routed networks.
- **Invite**: a station with Invite on (default) joins without asking and takes the
  session's show (atomic download: on failure it keeps its old show); unsaved changes are kept as an autosave copy first. Invite off → the
  master is told. A joiner with higher priority than the master is told it outranks the master and
  that `Claim LumaSession` makes it master (6c).
- **Dismissed or leaving** stations become standalone with a copy of the show and DMX output
  **held** until their operator resumes it (`Set Station HoldOutput Off`) or output when
  alone is on. "Standalone sends" applies only to stations that never joined.
- **LumaSession off** (`Set Station Online Off`) stops everything networked but localhost.
- Per-NIC services (`Set Interface en1 LumaSession|Shell|HTTP Yes|No`, station-local);
  loopback always serves Shell and HTTP.

### 4. Who Sends

A station that never joined sends; in a LumaSession only the master; an IdleMaster only with
"output when alone" (off by default). Members and a fenced master (lower epoch) do not send.

### 5. Losing The Master (6c, built)

- Heartbeat on the stream every 500 ms (`Ping` / `Pong`); warning after 1.5 s, lost after
  5 s; link down on the NIC counts at once. Under the timeout: keep working; show-changing
  commands go into a reliable outbox, marked *pending*, resent until the master confirms.
- After the timeout: **election first** — each member computes the same winner from the
  replicated member list and discovery: highest priority (`Never`, `VeryLow`, `Low`, `Normal`,
  `High`), then most results applied, then longest online time (±5 s), then lowest stable
  station id. `Never` stations can't win. The winner becomes master with a **new epoch**
  before output; the others follow it and resend their pending commands there.
- **IdleMaster only when alone** (no winner reachable): outbox applied locally, everything
  since the split journaled; no network output unless output when alone. It counts as diverged
  once it changes the show.
- Reconnect: known stations (in the show) are re-invited with back-off; a not-diverged
  IdleMaster rejoins silently; a diverged one gets the prompt **Merge / Use Master / Use
  Mine / Cancel** (6d, see 6).
- Two masters (split healed): the lower epoch is fenced (no DMX) and its operator gets the
  same prompt. `Claim LumaSession` is also the handover (new epoch).
- Health: `Info Station` / `Info LumaSession` and a status icon show latency, resends and
  outbox depth.

### 6. Merge On Rejoin (6d, built)

Session data merge. Objects match by **GUID, not slot** (every show object has one;
every stored link is a GUID).

- **Split base**: an IdleMaster keeps its show at the split (`become_idle`); an elected or new
  master keeps its show when it takes over from the one it followed (`become_master`). Kept
  in memory and autosaved as `split-base`.
- **Prompt**: **Merge / Use Master / Use Mine / Cancel**; the buttons run
  `Join LumaSession <ip:port> /Merge`, `Join LumaSession <ip:port>`, `Claim LumaSession`,
  `Leave LumaSession`. The station peeks the master's show over the stream (`Hello.peek` →
  `Preview`) and lists what differs; the list is refreshed when it changes its own show while
  waiting. Merge is greyed out with the reason when blocked: another `show_id`, no common
  base, or the patch changed on both sides.
- **Default and timeout**: `ShowSettings.merge_default` (Merge) and `merge_timeout` (60 s;
  0 = no prompt; Unlimited = waits), set with `Show /MergeDefault Merge|UseMaster|UseMine|Cancel`
  and `Show /MergeTimeout <s>|Unlimited` (Admin). The timeout runs the default (a blocked
  Merge → Cancel). `Pause LumaSession` stops the countdown (the pop-up's text click).
  `Info LumaSession` shows the decision, its answers and the differences.
- **Join with `/Merge`**: `Hello.merge` carries the joiner's show, split base and journal.
  The master picks the common base (`common_base`: the joiner's base if its journal is the
  master's or the master's split base's lineage; else the master's own split base if the
  joiner is the old master), runs `lumacore_tree::merge::three_way`, autosaves its show as
  `before-merge`, installs the merged show (members get a snapshot), logs each item in sysmon
  "Merge" and replies `Welcome` with the `merged` lines. Refused with E-LUMA-005. The joiner
  autosaves `before-merge` first.
- **Rules**: the unit is the smallest GUID object. Added → added. Same slot on both sides →
  the master keeps the slot, mine goes to the next free slot (the object's id follows).
  Deleted and untouched → deleted; deleted vs edited → the edit wins. Changed on both sides →
  the master's. A move is a slot change of the same GUID (applied, no duplicate). Running
  playbacks are always the master's; `patch_version` = max.
- **Autosaves**: `library/shows/.autosave/{show}_{reason}_{station}_{time}.luma`, reasons
  `invite`, `use-master`, `use-mine`, `before-merge`, `split-base`.

### Reference Check (6d)

Runs after every show replace (load, join, merge) and after Oops / Redo. Broken links are
kept (a later merge or undo can heal them) and listed: sysmon "References", `Info Show`
(References), `List Reference [/Broken]`. It also finds duplicate and nil GUIDs, wrong-kind
links and containment loops (`ShowData::identity()`).

### 7. Security (§19 step 5)

Lighting networks are mostly isolated and open: security is about who did what with which
rights. The UDP command port and HTTP reads stay open (requests name their user, no token);
stations trust each other by source IP; members keep the show's password hashes to check
logins themselves; one rights gate in dispatch covers every transport; passwords are masked
before they reach the journal, sysmon, replication ops or history.

## Current Implementation Status (step 6d, 2026-10-11)

Built:

- LumaSession objects and verbs (`LumaSession`, `Station`, `Shell`, `Interface`; `Join`,
  `Leave`, `Invite`, `Dismiss`), `Root.Runtime.LumaSessions`, the LumaSessions / Network
  menus (`GET /api/lumasessions`, `/api/network`)
- `remote::lumasession`: status Standalone / Master / Member / IdleMaster, Invite / Dismiss requests,
  deferred join/leave, autosave copy before an invited join, `ServiceAccess` (per-NIC
  filtering of the command port, discovery, Shell and HTTP)
- **the link (6b)**: TCP :47658 stream (zstd JSON frames, `remote/stream.rs`); members
  forward Master-scope lines (`registry::line_scope`, `ObjectDef::scope`); the master
  executes them as the sending user, commits ops `(journal id, epoch, seq)` and pushes them
  with undo steps and the `SessionRuntime` (modes, claims, global variables, members with
  users); members apply in order, a gap → `Resync`; pending outbox (`AppState::forwards`)
  resent after reconnect; show load restarts the journal and sends a `Snapshot`
- atomic join: show parsed and format-checked before install; refused with E-LUMA-001
  (stream protocol) or E-LUMA-002 (format); a different commit only warns
- `Runtime.Users.{n}.Undo` mirrors the master's stack on every station
- `lumacore-server --station <name>` / `--port-base N` run several servers on one machine
- **health and election (6c)**: heartbeat 500 ms / warn 1.5 s / lost 5 s, election
  (`remote/election.rs`), new epoch per master and fencing, IdleMaster when alone, the
  Use Master / Use Mine / Cancel prompt, `Claim LumaSession`, known stations in the show
  (`ShowData.session.stations`, `Delete Station`), patch version, the fuller show status set,
  output-distribution log, link-down detection, discovery Multicast Base Default / Alternative
- **identity, merge and ownership (6d)**: a GUID on every show object and every stored link
  (`Identified` / `Walk`, `ShowData::identity()`); `FORMAT_VERSION` 3 (older and newer
  refused, missing or duplicate GUIDs refused, E-SHOW-003); `ShowSettings.show_id`; undo and
  replication subtrees keyed by GUID; the reference check (`List Reference`); split base,
  the Merge / Use Master / Use Mine / Cancel prompt with a preview of the differences,
  `Join LumaSession … /Merge` and the 3-way merge, merge default and timeout,
  `Pause LumaSession`; claims by GUID, editor claims, E-CLAIM-001, claim requests with
  `/Keep` and `/Force`, `List Claims`; stream protocol 2
- see [replication.md](./replication.md)

Not built:

- local programmer feedback ("instant, corrected later") is postponed: in 6b everything
  waits for the master
- `Invite` / `Dismiss` still do UDP I/O inside the command
- playback is not yet driven from one authoritative session clock across stations

## LumaCore Mapping

- [remote/lumasession.rs](../../../crates/lumacore/src/remote/lumasession.rs) — status,
  invite/dismiss/member, `ServiceAccess`
- [remote/handler.rs](../../../crates/lumacore/src/remote/handler.rs) — UDP command port
  requests (Health, Login, Invite, Dismiss)
- [remote/stream.rs](../../../crates/lumacore/src/remote/stream.rs) — TCP :47658 stream
- [remote/link.rs](../../../crates/lumacore/src/remote/link.rs) — forwarding, pending
  outbox, master flush, member apply, merge (`merge_in`, `preview`, `common_base`,
  `merge_offer`)
- [remote/election.rs](../../../crates/lumacore/src/remote/election.rs) — heartbeat,
  election, split base, the decision prompt, `Pause LumaSession`
- [lumacore-tree/src/merge.rs](../../../crates/lumacore-tree/src/merge.rs) — 3-way merge by
  GUID; [lumacore-tree/src/identity.rs](../../../crates/lumacore-tree/src/identity.rs) —
  `Identified`, `Walk`, `ShowData::identity()`
- [terminal/claims.rs](../../../crates/lumacore/src/terminal/claims.rs) — claims by GUID,
  claim requests; [terminal/references.rs](../../../crates/lumacore/src/terminal/references.rs)
  — the reference check
- [remote/replication.rs](../../../crates/lumacore/src/remote/replication.rs) — op apply
- [terminal/commands/session.rs](../../../crates/lumacore/src/terminal/commands/session.rs) —
  `Join` / `Leave` / `Invite` / `Dismiss`
- [session_manager.rs](../../../crates/lumacore/src/session_manager.rs) — server-owned Shell
  sessions that clients attach to
- [terminal/change.rs](../../../crates/lumacore/src/terminal/change.rs) — the change path
  (undo frame, replication op, journal entry)
- [multi-station.md](./multi-station.md), [replication.md](./replication.md),
  [session-engine.md](./session-engine.md), [network-engine.md](./network-engine.md)
