# lumacore-lumasession

LumaSession, the station ⇄ station side: the link between a master and its members, the
election, the merge on rejoin, the stream they talk over and replication.

## Purpose

How several stations run one show (decisions §10, §21 "6b"–"6d"). A member forwards every line
whose scope is Master over the stream; the master runs it as that user and sends the result —
`(epoch, seq)` subtree snapshots, undo steps, the session runtime — to every member, applied
strictly in order (`link`, `replication`). When the master is lost, members elect one (priority,
results applied, online time, station id) and follow it; two masters fence the lower epoch; a
station back from alone is asked Merge / Use Master / Use Mine / Cancel, and a merge is three-way
by GUID (`election`, `link`). The stream is zstd JSON frames on the stream port (a constant in lumacore-wire), with heartbeats,
loss after 5 s and the files a member lacks (`stream`). Invites and dismissals answer at once and
run after their job (`session`). Per-NIC services decide which interfaces serve it (`access`).
It runs on a station through a narrow `SessionHost` (decisions §34 19.3): the server implements
it and keeps one `LumaSession` with the session's own fields.

## Owns / Doesn't own

- **Owns:** `LumaSession`, `Deferred`, invite / dismiss (`session`); `SessionHost` (`host`); the
  link (`link`: admit, forward, flush, resync, merge, loss); the election, fencing, re-invites,
  `SessionWatch` (`election`); the stream, its frames and threads (`stream`); applying an op
  (`replication`); `ServiceAccess`, `Services` (`access`).
- **Doesn't own:** the command port and its handler, the state thread's loop, the network view
  (the server); the change path, the outbox core writes to, the session's status words
  (`lumacore-state`'s `session_link`); the clock sync (`lumacore-clocksync`); the merge algorithm
  (`lumacore_tree::merge`); the terminals (never — a DMX-only node stays possible).

## Public surface

- `link::{after_work, flush}`, `election::tick`, `run_deferred` — what the state loop calls.
- `handle_invite`, `handle_dismiss`, `election::*` — the command port's LumaSession requests.
- `stream::{spawn_stream_server, spawn_link, read_frame, write_frame}` — the stream.
- `SessionHost`, `LumaSession` — what a station provides and keeps.

## Rules

- Only the master changes the show; a member's lines go to it and results come back in order. [unchecked]
- A forwarded command never runs twice (resent after a reconnect: its reply again). [unchecked]
- It never names the terminals or the server: everything it needs from the station is `SessionHost`. [compiler + tidy crate_edges_point_down]
- No network I/O inside a state job: joins and invites run after it (`Deferred`). [unchecked]

## Edges

- **Depends on:** lumacore-registry, lumacore-base, lumacore-wire, lumacore-lanes, lumacore-tree, lumacore-objects, lumacore-net, lumacore-sysmon, lumacore-luma, lumacore-cmd-lumasession.
- **Used by:** lumacore-server, lumacore-sweep, lumacore-station.

## Decisions

- [§10](../../../docs/archive/decisions-2026-10.md) — state ownership, replication
- [§21 "6b", "6c", "6d"](../../../docs/archive/decisions-2026-10.md) — the link, master loss, merge on rejoin
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (19.3, 24.2), its README (18)

## Testing

- `cargo test -p lumacore-lumasession` — the election's order, the stream's frames, the services.
- On a whole server: the station's `lumasession` tests (link, election, remote edits).
