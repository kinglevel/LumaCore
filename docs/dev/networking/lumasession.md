---
title: LumaSession
category: docs
---
# LumaSession

The one developer doc for the server-to-server side of LumaCore: stations joined together in
a **LumaSession**. It replaces `session-engine.md`, `multi-station.md` and
`network-sessions.md` (now in `docs/archive/dev/networking/`).

- Language: [lum.md](../../user/syntax/lum.md); generated command reference:
  [commands.md](../../user/reference/commands.md), [objects.md](../../user/reference/objects.md)
- Decisions: the archived log [decisions-2026-10.md](../../archive/decisions-2026-10.md) §10
  (multi-station sync), §18 (naming), §19 "Step 5 hardening" (security), §21 "Step 6
  hardening" (6a–6d), §23 "Step 8" (layers, wire), §24 "Session clock (researched)", §26 (state
  thread), §29 (step 11), §30 "Remote station edits"; the records in `docs/decisions/`
- The change path and the ops: [replication.md](./replication.md)

## Overview And Naming

A LumaSession is live runtime, not saved show data and not a protocol output. One station,
the **master**, executes every show-changing command as the sending user and pushes the
results; every other station holds a full copy of the show and applies them.

| Word | Means |
|---|---|
| `LumaSession` | the server-to-server mesh: stations, join / invite / dismiss / leave, discovery, replication. Always this word: never "Session" or "network session", no aliases. |
| `Station` | one running server (`lumacore-server`) that can join a LumaSession |
| master | the one station that executes show changes and pushes results |
| `Client` | one attached device or window (CLI, app window, browser tab); never owns authority |
| `Shell` | a terminal session on a station (a logged-in user and their clients). Not a LumaSession. |
| `Network` | only a **data network** (§11: "Rig", "Guest" mapped to a NIC). Not the mesh. |
| sACN, Art-Net, OSC | protocol lines (`lumacore_lines::runtimes`), not LumaSession concerns |

LumaSession handling is **not a registry engine** (the engines are `RenderEngine`,
`PerfettoEngine`, `HttpUiEngine`; §26/§27). It runs as state-thread jobs plus listener
threads that only do I/O (see "Code Map").

Established consoles are the main reference: a network menu, host status, Invite / Dismiss, the data
negotiation prompt and a system monitor.

### Commands

```text
Join LumaSession "Festival"        # by name (found by discovery)
Join LumaSession 10.0.0.5          # by address (routed networks), host[:port]
Join LumaSession 10.0.0.5 /Merge   # rejoin and merge
Leave LumaSession
Claim LumaSession                  # take the master role (handover / Use Mine)
Pause LumaSession                  # stop the merge prompt's countdown
Invite Station "FOH-2"
Dismiss Station "FOH-2"
Delete Station "FOH-2"             # forget a known station (master, Setup rights)
List LumaSession / Info LumaSession
List Station / Info Station
Set Station Name|LumaSession|Location|Priority|Invite|Online|HoldOutput|OutputWhenAlone|MulticastBase …
List Interface
Set Interface en1 LumaSession|Shell|HTTP Yes|No
List Shell / Info Shell 1          # terminal sessions
List Client / Info Client 3
List Claims / cd Claims
Info Clock / Info Health Clock
```

Station properties are station-local and saved in the station file
(`library/station_settings.json`, or `library/stations/<name>/` with `--station`).

### Where State Lives (§10)

| Kind | Where | Replicated |
|---|---|---|
| Show, incl. `ShowData.Users.{n}` programmer and selection | `ShowData` | yes (results from the master) |
| Known stations, patch version | `ShowData.session` (`ShowSubtree::Session`) | yes, never undone |
| Per-user modes, undo history | `Runtime.Users.{n}` (`Modes`, `Undo`) | yes, never saved |
| Session runtime: members, claims, global variables, epoch | `Runtime` | yes (`SessionRuntime`), never saved |
| Station-local: settings, interfaces, keystore, UI, command lines, screens, clients | `StationSettings`, station | never |

The split is enforced by type (which subtree a field lives in), not by a flag. The same
show can be loaded into different topologies; the LumaSession replicates the show, it is
not part of it.

```text
Root
├── Runtime                       session-wide runtime: replicated, never saved
│   ├── LumaSessions
│   │   ├── Stations              what discovery sees
│   │   ├── Members               stations known to this master
│   │   ├── Listeners
│   │   └── Connections
│   ├── Network                   data networks: Interfaces, Lines, Nodes, Sources
│   ├── Users.{n}                 Modes, Undo, Screens
│   ├── Claims                    claims by GUID
│   ├── Journal
│   └── Health                    Render, Lines, Clock, Threads
└── StationSettings               station-local, never replicated
```

`Root.Runtime.SessionEngines` and `NetworkEngines` are empty placeholder nodes; nothing
registers there.

## Ports

The ports are constants in `crates/foundation/lumacore-wire/src/lib.rs`
(`REMOTE_CMD_PORT` … `REMOTE_STREAM_PORT`, the discovery groups `SESSION_DISCOVERY_GROUP[_ALT]`):

| Constant | Use |
|---|---|
| command port (UDP) | Health, Login, Command, Shutdown, Invite, Dismiss, Yield, and the session clock's `Time` pings (answered at the socket) |
| shell port (TCP) | interactive shells (REPL sessions) |
| discovery port (UDP, multicast) | the beacon, sent out of every NIC that serves LumaSession; the Default or the Alternative group (`Set Station "MulticastBase"`) |
| HTTP port | REST / WebSocket (`http` feature) |
| Perfetto port | the trace (`perfetto` feature) |
| stream port (TCP) | the LumaSession stream: member ↔ master |

Two protocol numbers, bumped on any wire change, no shims — the current values are the
constants in `lumacore-wire`:

- `SESSION_PROTOCOL` (station ↔ station; must match to join, E-LUMA-001)
- `CLIENT_PROTOCOL` (a client's hello and the page's `/ws/feeds` hello; E-WIRE-001)

The show format must match too (`FORMAT_VERSION` in `lumacore-tree`, E-LUMA-002), and every
station of a LumaSession runs the same build, version and commit (E-LUMA-006).

The beacon carries station name and id, LumaSession name, location, status, priority,
invite, group, epoch, results applied, online time, show status, `StationKind`, ports and
both protocol numbers.

## Roles And States

`Info LumaSession` and the beacon say one of `Standalone`, `Master`, `Member`, `IdleMaster`.
Inside, `lumacore_state::session_link::Role` is `Normal`, `IdleMaster { diverged, … }` or
`Fenced { by }`:

- **Standalone**: never joined, or left. Sends DMX.
- **Master**: others joined it. Sends DMX.
- **Member**: has a link to a master. Does not send DMX.
- **IdleMaster**: lost its master and nobody else was reachable. Its own master; sends only
  with `OutputWhenAlone` (off by default: receivers flap between sources).
  Diverged once it changes the show.
- **Fenced**: a master that met a master with a higher epoch. No DMX until its operator
  decides.

Who sends is `lumacore_objects::protocol_plan::may_send`: online, output not held, not a
member, not fenced, and not an IdleMaster without output when alone.

Show status is one of: NoShow, Loaded, Downloading, Downloaded,
Uploading, Saving, Negotiating (master / member), Waiting for new master.

## Joining, Leaving, Invite, Dismiss

- **Discovery**: LumaSessions are named and found by beacon. Stations on different multicast
  groups (`Set Station "MulticastBase"`) don't see each other; join by address still works.
  Two servers on one machine both bind the discovery port with `SO_REUSEPORT`.
- **Join**: the member connects to the master's stream and sends `Hello` (protocol, format,
  commit, station, station id, priority, user, ports, link, resume). The master answers `Welcome`
  (or `Refused`: E-LUMA-001 protocol, 002 format, 003 not a master — or another station
  with this name is in the session, connected or reconnecting, or no station id: members
  are one per name). The show is parsed and
  format-checked off the state thread, then installed at once (atomic: on failure
  the station keeps its old show and output). A joiner that outranks the master is told
  that `Claim LumaSession` makes it master.
- **Invite**: a station with Invite on (default) joins without asking and takes the
  session's show; unsaved changes are autosaved first (`invite`). Refused when LumaSession
  is off, Invite is off or it is already a member. A station that leads that LumaSession (a
  master with members, a diverged IdleMaster, a fenced master) asks its operator instead: Use
  Mine (default) / Use Master / Cancel, Merge greyed (`election::invited_back`, R1-22).
- **Split base on disk** (R1-22): the split base's autosave path, journal and sequence are noted
  in `<station dir>/split_base.json`; at start it is restored when the loaded show has the same
  show id, so Merge still works after a restart; a merge forgets it.
- **Dismiss / Leave**: the station becomes standalone with a copy of the show and its DMX
  output **held** until `Set Station "HoldOutput" Off` (or output when alone). "Standalone
  sends" applies only to stations that never joined. `Leave` sends `Bye`; pending commands
  are dropped and named.
- **LumaSession off** (`Set Station "Online" Off`) stops everything networked but localhost.

**No network I/O inside a job** (§23 "Step 8"). Commands only queue work in the kernel's
outbox (`lumacore_state::session_link`):

- `Join LumaSession` pushes a `LinkRequest` (`AppState::link_requests`)
- `Invite` / `Dismiss Station` push `SessionRequest::Send`; `lumasession` sends it on its own
  thread after the job and the other station's answer goes to the line's client (`ReplyTo`)
- `Claim LumaSession` pushes `SessionRequest::HandOver` (member) or `UseMine`
- a forwarded line goes on `LinkOut` (`AppState::link`)

On the invited station, `Invite` and `Dismiss` answer at once and run the join / leave after
the job (`lumasession::Deferred`), because the join talks to the sender that is still
waiting for the answer.

## Link And Replication

The change path and the ops in detail: [replication.md](./replication.md). In short:

- The **stream** (the TCP stream port): one connection per member. Frames are a u32
  big-endian length plus zstd-compressed JSON `StreamMsg`: `Hello`, `Welcome`, `Snapshot`,
  `Preview`, `Refused`, `Command` / `Reply`, `Op`, `Logins`, `Changes`, `Resync`, `Bye`,
  `Ping` / `Pong`, `Claim`, `NewMaster`, `FileWant` / `File` (files a member lacks, by
  SHA-256), `Upload`. Stream frames carry show types, so they live in
  `crates/server/lumacore-lumasession/src/stream.rs`, not in `lumacore-wire`.
- **Where a line runs**: each `CommandDef` / `ObjectDef` declares `scope: Client | Station |
  Master` (unset = Master). On a member, `dispatch` forwards every line whose
  `registry::line_scope` is Master (`session_link::forward`), before anything runs. Client
  and Station lines run locally. Another station's own settings (`Set Station "Tower" …`) go
  straight to its command port as `SessionRequest::EditStation`, never through the master or
  the show (§30 "Remote station edits").
- **Master executes, results flow out**: the master runs the line as the sending user. The
  change path (`lumacore_state::change`) files exactly the changed subtrees as undo frame,
  replication op and journal entry. After every job `link::flush` commits the ops to the
  `ReplicationJournal` with `(journal, epoch, seq)` and sends them, with undo steps and the
  `SessionRuntime` when they changed, to every member. Stations never re-run commands; the
  text is only a label.
- **Strict order**: `link::op_order` lets only `seq + 1` of the same journal apply; a repeat
  is ignored; a gap or another journal sends `Resync` (missed ops, or a full snapshot if
  trimmed). A new master and a show load start a new journal, so members take a snapshot.
- **Pending outbox**: forwarded lines wait in `AppState::forwards`, shown *pending* after
  1 s, resent in order after a reconnect (back-off from 0.5 s) until the master confirms.
- **Never twice**: a forwarded line is named by (station id, link, command id); the link
  (`Hello.link`, `LinkHandle::link`) is fresh per join and kept across reconnects and a new
  master. The master records the newest id it ran per member in the session runtime
  (`Runtime.LumaSessions.commands_done`, replicated with the result, never saved) and keeps
  recent answers by station id (`ServerContext::replies`; a Bye or a new link drops them).
  A resent line that already ran gets its answer again, or "Already ran on the master (its
  answer was lost)" — on the same master after a reconnect and on a newly elected one. A
  member marks the forwards a result says ran (`session_link::mark_ran`); a new master or
  an IdleMaster runs only lines that never ran.
  Everything waits for the master, programmer and selection too (an isolated LAN answers in
  about 1 ms); "instant, corrected later" is postponed until waiting is noticeable.
- A member's own show changes (screen write-through, `Store View` …) go as `Changes`; the
  master files them as that user.
- **Undo** lives on the master and is replicated as steps; every station's
  `Runtime.Users.{n}.Undo` matches, so a new master can still undo. Never saved.
- **Ownership / claims** (§4): claims are keyed by GUID in `Runtime` (`claims`,
  `claim_order`, `claim_asks`) and replicate with the session runtime. `Edit` claims until
  the editor is left; `cd` never claims; selecting never claims (`Claim Fixture <n>` locks a
  fixture). A changing command on an object another user holds, or anything inside it, is
  refused (E-CLAIM-001). `Claim <obj>` on someone's object asks the owner (`Unclaim` /
  `Claim … /Keep`; 10 s, or at once when the owner is logged out → handed over; Admin
  `/Force`). A Patch draft is never taken by timeout.

## Master Loss And Election

Constants: `election::BEAT` 500 ms, `session_link::WARN` 1.5 s, `lost_after()` 5 s (debug
builds take `--session-timeout <s>`).

- **Heartbeat**: `Ping` / `Pong` both ways on the stream every 500 ms (also measures
  latency). After 1.5 s silence: a warning, commands wait. After 5 s: lost. Silence, round
  trips and the decision countdown run on the station's monotonic clock
  (`session_link::link_ms`), never the wall clock; claim takeovers on the session clock. A
  running link reads frames in pieces of 64 KiB and counts each as heard, so a big
  snapshot or file on a slow link isn't silence; writes time out after the loss timeout. A link down on
  the stream's NIC cuts the station off at once ("no cable"); the 5 s still decides.
- **Election first**: every member computes the same winner (`election::elect`, `best`,
  `ranks_above`) from the replicated member list, counting only stations discovery still
  sees:
  1. highest priority (`Never` < `VeryLow` < `Low` < `Normal` < `High`; `Never` can't win)
  2. most results applied
  3. longest online time, compared in whole 5 s buckets (transitive, so every member picks
     the same winner whatever order it lists them in; §29)
  4. lowest stable station id

  The winner becomes master with a **new epoch** before any output (`become_master`) and
  runs the commands still pending there; the others follow it and resend their pending
  commands. A winner unreachable for 5 s is dropped and the next one computed. Every
  station already holds the show and every programmer, so no download is needed.
- **IdleMaster only when alone** (`become_idle`): the split is journaled, the split base
  kept, the pending outbox run locally; output only with output when alone.
- **Reconnect**: known stations (`ShowData.session.stations`, every station that ever
  joined) are re-invited when they reappear with Invite on (one per 10 s, at most 5 a
  minute). Leave and Dismiss keep them listed; a station that said `Bye` is not re-invited
  until it joins again. `Delete Station` forgets one (refused while connected). An
  IdleMaster that sees its session again rejoins silently when not diverged, or gets the
  prompt.
- **Two masters** (split healed): beacons carry the epoch. The higher epoch wins (equal:
  the ranking); the loser is fenced at once, sends its members to the winner (`NewMaster`)
  and its operator gets the prompt. The winner's operator gets a notice.
- **`Claim LumaSession`**: on a member (Setup rights) asks the
  master to hand over (`Claim` → `NewMaster`, new epoch, nothing lost). On a fenced master
  or IdleMaster it is **Use Mine**: become master with a new epoch and send `Yield` to the
  other master, which autosaves and joins.
- **Health**: `Info LumaSession` / `Info Station` and the status-bar icon show role, latency,
  last heard, resends, pending and the decision. Output distribution changes are logged
  in sysmon ("Output: distribution changed").

## Merge On Rejoin

Data negotiation, done by **GUID, not slot** (every show object has a GUID, every
stored link is a GUID; §21 "6d").

- **Split base**: an IdleMaster keeps its show at the split (`become_idle`); a station that
  takes over from the master it followed keeps that show (`become_master`). Kept in memory
  (`AppState::split_base`, with journal and seq) and autosaved as `split-base`.
- **The prompt**: **Merge / Use Master / Use Mine / Cancel**. The buttons run
  `Join LumaSession <ip:port> /Merge`, `Join LumaSession <ip:port>`, `Claim LumaSession`,
  `Leave LumaSession`. The station peeks the master's show (`Hello.peek` → `Preview`) and
  lists what differs; the list refreshes when it changes its own show while waiting. Merge
  is greyed out with the reason when blocked: another `show_id`, no common base, or the
  patch changed on both sides (`patch_version`).
- **Default and timeout**: `ShowSettings.merge_default` (Merge) and `merge_timeout` (60 s;
  0 = no prompt; Unlimited = wait), set with `Show /MergeDefault …` and
  `Show /MergeTimeout <s>|Unlimited` (Setup). The timeout runs the default (a blocked Merge
  → Cancel). `Pause LumaSession` (or clicking the pop-up) stops the countdown.
- **Join with `/Merge`**: `Hello.merge` carries the joiner's show, split base and journal.
  The master picks the common base (`link::common_base`), runs
  `lumacore_tree::merge::three_way`, autosaves `before-merge`, installs the merged show
  (members get a snapshot), logs each item in sysmon "Merge" and replies `Welcome` with the
  `merged` lines. Refused with E-LUMA-005. The joiner autosaves first too.
- **Rules** (the unit is the smallest GUID object):
  - added on one side → added
  - both added in the same slot → the master keeps the slot, mine moves to the next free one
  - deleted and untouched → deleted; deleted vs edited → the edit wins
  - changed on both sides → the master's (mine is in the autosave)
  - a move is a slot change of the same GUID: applied, no duplicate
  - running playbacks are always the master's; `patch_version` = the max
- **Autosaves**: `library/shows/.autosave/{show}_{reason}_{station}_{time}.luma`, reasons
  `invite`, `use-master`, `use-mine`, `before-merge`, `split-base`. Every losing side is
  saved first, and a show is never replaced when its copy can't be written: an Invite is
  declined (sysmon error, the inviter is told, E-CMD-012), Use Master / Merge reply
  E-CMD-012 "nothing was replaced", a Yield is refused (the claimant's higher epoch then
  fences this master), a member told to follow with an autosave leaves instead (output
  held), a merge on the master is refused (E-LUMA-005). A stop signal still stops and logs
  the error.
- **Reference check** (`lumacore_objects::references`) after load, join, merge, Oops and Redo: broken
  links are kept and listed (sysmon "References", `Info Show`, `List Reference /Broken`).

## Session Clock

§24 "Session clock (researched)"; code `lumacore_clocksync` + `lumacore_base::clock`.

- **Rate overlay, never backwards**: session time = base + (Instant − anchor) × (1 +
  frequency + correction), re-anchored on every change. Correction = offset / 2 s, at most
  0.5 % (2 % while more than 20 ms off); frequency clamped to ±500 ppm. A station ahead slows
  down. It steps only on first lock after joining, or beyond 1 s (panic, logged).
- **Pings on the UDP command port**: a member sends `Time { t1 }` to its master's command
  port (`Welcome.cmd_port`); the master's UDP thread stamps `t2` on read and `t3` before the
  reply, never through the state thread or behind a snapshot. The stream heartbeat is for
  liveness and election only.
- **Filter**: a burst of 16 pings 20 ms apart when it starts following a master (or after a
  step, a gap, 3 outliers), then one every 500 ms. 32 samples; RTT gate (min RTT + max(0.5
  ms, ½ min RTT)); a single 5σ RTT outlier dropped; offset = median; frequency =
  RTT-weighted regression (chrony).
- **Holdover**: 4 pings unanswered (about 3 s), or no master: the clock runs on at its last frequency.
  A new master keeps its time; members drop their samples, keep their frequency and burst
  again (linuxptp).
- **Stats** in PTP terms (Unlocked / Acquiring / Locked / Stable / Holdover, offset, path
  delay, jitter, frequency …) in `Runtime.Health.Clock`: `Info Clock`, `List Health`, the
  Realtime window. Members report theirs in each ping, so the master lists them all.
  linuxptp is read only (`lumacore_net::ptp`).
- Fades, group-master fades and effect anchors use `clock::session_ms()`, never wall clock.

## Services Per NIC And Security

- `Set Interface en1 LumaSession|Shell|HTTP Yes|No` (station-local).
  `lumacore_lumasession::ServiceAccess` (shared, ArcSwap) filters the command port, discovery and the
  stream by source subnet, and Shell / HTTP by the local address they arrived on. Loopback
  always serves. `Set Station "Online" Off` stops everything but loopback.
- **Security stays light** (§19 "Step 5 hardening"): lighting networks are mostly isolated
  and open, so security is about who did what with which rights.
  - The command port and HTTP reads stay open; requests name their user, no token.
  - Stations trust each other by source IP; members keep the show's password hashes to
    check logins themselves.
  - One rights gate in dispatch covers every transport (E-RIGHTS-001), including lines
    forwarded to the master.
  - Passwords are masked once in dispatch before journal, undo, replication ops and sysmon.

## Code Map

| Module | Responsibility |
|---|---|
| [session.rs](../../../crates/server/lumacore-lumasession/src/session.rs) | `ServiceAccess`, Invite / Dismiss on the invited station, `Deferred` join / leave |
| [stream.rs](../../../crates/server/lumacore-lumasession/src/stream.rs) | the TCP stream: `StreamMsg` frames, `Hello` / `Welcome` / `Preview`, stream server (reader + writer thread per member), member link thread (`spawn_link`, reconnect), heartbeat |
| [link.rs](../../../crates/server/lumacore-lumasession/src/link.rs) | state-thread side: `admit`, `run_forwarded`, `flush`, `apply_op` / `op_order`, `Resync`, `SessionRuntime`, `Changes`, merge (`preview`, `common_base`, `merge_in`, `merge_offer`) |
| [election.rs](../../../crates/server/lumacore-lumasession/src/election.rs) | `elect` / `best` / `ranks_above`, `become_master`, `become_idle`, fencing, two masters, the decision prompt, `tick`, re-invites, `SessionRequest` sending |
| [replication.rs](../../../crates/server/lumacore-lumasession/src/replication.rs) | `apply_replicated_op`: restore post snapshots, replay undo steps |
| [clock.rs](../../../crates/network/lumacore-clocksync/src/clock.rs) (lumacore-clocksync) | session clock: `Time` answers, filter, discipline, holdover |
| [session_link.rs](../../../crates/model/lumacore-state/src/session_link.rs) (lumacore-state) | what the kernel keeps: `Role`, `ShowStatus`, `SplitBase`, the `LinkOut` outbox (`LinkHandle`), `forward`, `submit_changes`, `leave`, `SessionRequest`, `LinkRequest`, `ReplyTo`, autosave copies, timeouts |
| [commands/session.rs](../../../crates/model/lumacore-objects/src/commands/session.rs) (lumacore-objects) | the kernel's side of `Join` / `Leave LumaSession`, `Invite` / `Dismiss Station`; the words themselves are `crates/commands/objects/lumacore-cmd-lumasession` |
| [change.rs](../../../crates/model/lumacore-state/src/change.rs) (lumacore-state) | the change path (undo frame, replication op, journal entry) |
| [claims.rs](../../../crates/model/lumacore-objects/src/claims.rs) (lumacore-objects) | claims by GUID, claim requests |
| [references.rs](../../../crates/model/lumacore-objects/src/references.rs) (lumacore-objects) | the reference check |
| [protocol_plan.rs](../../../crates/model/lumacore-objects/src/protocol_plan.rs) (lumacore-objects) | who sends (`may_send`) |
| [net/spawn.rs](../../../crates/server/lumacore-server/src/net/spawn.rs) (lumacore-server) | UDP command port (answers `Time` at the socket), shell listener, discovery |
| [net/handler.rs](../../../crates/server/lumacore-server/src/net/handler.rs) (lumacore-server) | command-port requests on the state thread |
| [discovery.rs](../../../crates/network/lumacore-net/src/discovery.rs) (lumacore-net) | station-side discovery state |
| [menu_rows.rs](../../../crates/ui/lumacore-views/src/menu_rows.rs) (lumacore-views) | the `lumasessions` and `network` feeds' rows |
| [ptp.rs](../../../crates/network/lumacore-net/src/ptp.rs) (lumacore-net) | linuxptp, read only |
| [server/state.rs](../../../crates/server/lumacore-server/src/state.rs) | the state thread (jobs, then flush / deferred work) |
| [terminal/session_manager.rs](../../../crates/ui/lumacore-terminal/src/session_manager.rs) | Shell sessions (outlive their clients) |
| [lumacore-wire](../../../crates/foundation/lumacore-wire/src/lib.rs) | ports, groups, protocol numbers, beacon, `RemoteRequest` / `RemoteReply`, `TimeReply` |
| [lumacore-base/src/replication.rs](../../../crates/foundation/lumacore-base/src/replication.rs) | `PendingReplicationOp`, `ReplicationOp`, `ReplicationJournal` |
| [lumacore-base/src/undo.rs](../../../crates/foundation/lumacore-base/src/undo.rs) | `ShowSubtree` |
| [lumacore-base/src/clock.rs](../../../crates/foundation/lumacore-base/src/clock.rs) | the overlay clock |
| [lumacore-tree/src/merge.rs](../../../crates/model/lumacore-tree/src/merge.rs), [identity.rs](../../../crates/model/lumacore-tree/src/identity.rs) | 3-way merge by GUID; `Walk`, `ShowData::identity()` |

Layering (§23, tidy `crate_edges_point_down`): the kernel (`lumacore-state`,
`lumacore-objects`) never calls `lumacore-lumasession`; it asks through its outbox and reads
status from `Runtime.LumaSessions`. `lumacore-lumasession` never imports a terminal crate, so a
DMX-only node build stays possible. The threads in `lumacore-server`'s `net/` and in
`stream.rs` only do I/O and reach state through `StateHandle`; never call `StateHandle::call`
from inside a job.

## Testing With Local Servers

Multi-station tests run as local servers, never Proxmox.

```bash
cargo build -p lumacore-station --features http --bin lumacore-server
target/debug/lumacore-server --station A --port-base 48650
target/debug/lumacore-server --station B --port-base 48660
```

- `--station <name>` keeps that station's files (settings, sACN CID, revoked tokens, logs) in
  `library/stations/<name>/`; shows and GDTF stay shared.
- `--port-base N`: command N+3, shell N+4, HTTP N+6, stream N+8. Discovery stays on the
  one discovery port for both (`SO_REUSEPORT`), so they hear each other.
- Join with `Join LumaSession "<name>"` or `Join LumaSession 127.0.0.1:<N+8>`.
- Debug builds: `--session-timeout <s>` for the loss timeout.
- A lost server without a goodbye: SIGKILL (SIGTERM is an announced shutdown).

Unit tests beside their code: `crates/server/lumacore-lumasession/src/election/tests.rs`,
`stream/tests.rs` and `access/tests.rs`; the words' tests in
`crates/commands/objects/lumacore-cmd-lumasession/src/tests.rs`; the property tests for
`op_order`, `election::best` and the member's show in `crates/dev/lumacore-sweep/src/tests.rs`.
See [testing.md](../testing.md).

## Background

Notes from the archived docs that still explain why things are the way they are.

- **Why results, not commands** (§10, rewritten 2026-10-10 after studying established consoles, game netcode
  and replicated systems such as Figma, Raft, Kafka): replaying commands on every station
  diverges as soon as one station's state differs. Subtree snapshots are idempotent and
  convergent, and undo uses the same snapshots.
- **Why one TCP stream per member** (6b, system monitor research): a reference console sends session
  data by multicast and needs IGMP snooping, falling back to "TCP only". A session has a few
  stations, so per-member TCP fan-out is cheap, reliable and ordered, with no router needs.
- **Why election first, IdleMaster only when alone** (6c): it resolves §10's earlier plan
  (every station becomes IdleMaster after the timeout). Electing among those who still see
  each other keeps one show; only a cut-off station diverges.
- **Why GUIDs, not slots** (6d, identity research): a reference console matches by slot / address, so
  a move duplicates. With a GUID on every object and link, a move applies cleanly, and the
  3-way split base tells "deleted" from "never existed" without tombstones.
- **Why the prompt words are the established ones** (6c research): data negotiation is
  Cancel / Use Master / Merge / Use Mine on established consoles; the same words appear in the pop-up, sysmon and
  `ShowSettings`.
- **Removed on the way**: UDP `ShowDataSnapshot`, `SubmitOperation`, `RequestOperations`,
  `Member` and `ReplicationPacket` (6b, replaced by the stream); `JoinPolicy` and
  `RemoteRequest::JoinNegotiation` (6d); the beacon clock sync `ClockSyncWire` (step 9);
  the old `remote/` module (step 8: split into what is now `lumacore-net`,
  `lumacore-lumasession` and `session_link`); `JoinSession` / `LeaveSession` and the old
  `Session` object (4d).
- **Still open**: local programmer feedback ("instant, corrected later") is postponed; owner
  claim notices reach only the master's own Shells (members see the pop-up from the
  replicated asks); objects without a tree definition (Timecode) can't be claimed.
