# lumacore-cmd-lumasession

The LumaSession words: joining, leaving and leading a session of stations.

## Purpose

`Join LumaSession "name" | <host>` (a name found through discovery, `/Merge` to merge this
station's show into the session's), `Leave LumaSession` (asks: leaving holds this station's DMX
output), `Claim LumaSession [Station "X"]` (take the master role, or hand it to a member; asks),
`Pause LumaSession` (stop a decision's countdown) and the LumaSession object (List / Info)
(decisions §11, §21, §30 "12d"). Every word only asks the kernel's outbox — a link request or a
session request on `AppState` — and never does network I/O; the stream, the election and the
replication are `lumacore-lumasession`'s, which registers this crate. Invite and Dismiss are verbs
of the Station object: their rows are here, their handlers stay with the Station (the kernel's
`session` door, until `cmd-station` is cut), like `join_at` / `leave_lumasession`, which the
Station's Connect, `Set Station "Online" Off` and the stream also call.

## Owns / Doesn't own

- **Owns:** the rows (`rows`), Join's parsing (`join`), Leave's ask (`leave`), Claim / Pause
  (`claim`), the LumaSession object (`lumasession`).
- **Doesn't own:** the outbox and the session's state (the kernel's `session_link`, `session`),
  the stream, election and replication (`lumacore-lumasession`).

## Public surface

- `register` — the rows and the object, after the kernel's.
- `ROWS`, `OBJECTS`, `HOOKS` (none).

## Rules

- No network I/O in a command: a word queues its request. [unchecked]
- Leaving and claiming the master are station asks. [test the_master_hands_its_role_to_a_member_after_an_ask]

## Edges

- **Depends on:** lumacore-registry, lumacore-wire, lumacore-tree, lumacore-objects,
  lumacore-cmd-station.
- **Used by:** lumacore-lumasession, lumacore-tidy.

## Decisions

- [§11](../../../../docs/archive/decisions-2026-10.md) — LumaSession, Network, Shell
- [§21 "6b"–"6d"](../../../../docs/archive/decisions-2026-10.md) — replication, master loss, merge on rejoin
- [§34 25.7](../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate

## Testing

- `cargo test -p lumacore-cmd-lumasession` — the words' replies and asks.
