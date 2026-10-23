# Replication — the change path and the ops

The mechanism under [lumasession.md](./lumasession.md) "Link And Replication": how a mutating
line becomes a replication op, how the master commits it and how a member applies it. Roles,
join, election, merge, the clock, ports and the code map are in that page; this one holds only
the types and the steps. Decisions: the archive's §10 and §21 "Step 6 hardening"
(`docs/archive/decisions-2026-10.md`).

Replication sends post-change **subtree snapshots**, never command replay: idempotent,
convergent, and the same snapshots undo uses.

## The change path

`lumacore_state::change` + `commit` in the kernel's dispatch, for every mutating statement of a
line (each `;`-separated statement is its own step):

1. Fingerprint every `ShowData` subtree before and after (`lumacore_base::fingerprint`).
2. Exactly the changed subtrees get: a `Runtime.Journal` entry; an undo frame of their
   pre-images on the user's stack (except `Playbacks` and `Session`, and only for
   `undo: Tracked`); a `PendingReplicationOp` of their post-images.
3. While `replaying_remote_op` is set only the journal entry is written.

Commands never list subtrees by hand; they declare `undo: ReadOnly | Untracked |
Tracked(kind)`. Station settings and the user's watches are not ShowData: dispatch diffs them
around a tracked line and the step carries them as `UndoFrame.local`; a member sends its own
station step to the master as `Changes` with that `local` part, and Oops / Redo put it back
only on that station (`oops::restore_local`).

## Subtrees

`ShowSubtree` (`crates/foundation/lumacore-base/src/undo.rs`) has one variant per `ShowData`
field, pools per entry by GUID (`DataPool(guid)`, `Profile(guid)`, `User(guid)`,
`Programmer(guid)`, `Selection(guid)`, `Usage(guid)`); the full list is the enum.

- The list comes from an **exhaustive destructure** of `ShowData` in `change::subtrees`: a
  new field fails to compile until it is listed.
- A missing pool GUID means absent, so creating and deleting users, profiles and datapools
  replicate (and undo) like any change.
- Restore removes by GUID first, then places each entry at the slot its data names; it
  refuses (never overwrites) a slot held by another object.
- `Programmer(guid)` is a user's programmer values, `Selection(guid)` their fixture selection
  and one-only selections, `User(guid)` the rest of that user, `Usage(guid)` their command
  usage (saved, never undone).
- `Playbacks` and `Session` replicate but are never undone (`ShowSubtree::is_undoable`).

## The types

| Type | Where | What |
|---|---|---|
| `PendingReplicationOp` | `lumacore-base` | Made by the change path after a mutating statement; waits in `AppState.replication_outbox`: `user`, `command` (masked), `affected_subtrees`, `post_snapshots` |
| `ReplicationOp` | `lumacore-base` | A master-committed op: `journal` (id), `epoch`, `seq` (monotonic within the journal), `station`, `authority_timestamp_ms`; carries the undo steps and, when it changed, the `SessionRuntime` |
| `ReplicationJournal` | `lumacore-base` | The master's ring of committed ops: `commit`, `ops_from(seq)` (`None` when trimmed), `ack(station, seq)`; its size is the constant in that file |
| `StreamMsg` | `lumacore-lumasession` `stream.rs` | Every frame on the TCP stream: ops, snapshots, gap fill, forwarded commands and replies, changes, logins |

A new journal starts with every master and every show load. Each new master (election
winner, `Claim LumaSession`, Use Mine) bumps the epoch: stations reject results from an older
master and a master with a lower epoch is fenced.

## On the master

`ServerContext` on the master owns the `ReplicationJournal` and one peer per connected member.
After every job (`lumacore_lumasession::link::flush`):

- forwarded commands run as the sending user; the outbox is drained, committed to the journal
  and sent — with the `UndoStep`s and the `SessionRuntime` when it changed — to every member;
- members' own show changes (`Changes`, e.g. screen write-through) are filed as that user;
- a show load restarts the journal and sends a `Snapshot`.

## On a member

`link::apply_op` checks the order with `link::op_order`: only `seq + 1` of the same journal
applies, a repeat is ignored, a gap or another journal id sends `Resync`. Then
`replication::apply_replicated_op` sets `replaying_remote_op`, restores each post snapshot
(`oops::restore_subtrees`), replays the undo steps, clears the flag, updates `subtree_owners`,
writes the `Runtime.Journal` entry and sets `last_applied`. The `SessionRuntime`, if the op
carries one, is applied last. After every show replace (load, join, merge) and after Oops /
Redo the reference check runs; broken links are kept and listed (`List Reference /Broken`).

## Undo across stations

- Each user's history is `Runtime.Users.{n}.Undo` — runtime, never saved. Each result carries
  what happened to the sending user's stack (frame filed / Oops / Redo); members replay it
  (`UndoStack::apply_step`), so every stack matches the master's and a newly elected master
  can still undo. `Oops` from any station goes to the master.
- `Oops` / `Redo` are replicated changes and file no undo frame of their own.
- `Clear Programmer User …` files the step on the **affected** user's stack.
- `subtree_owners` records which user last changed each subtree (the conflict check for Oops).

## AppState fields

`crates/model/lumacore-state/src/app_state.rs`:

| Field | Purpose |
|---|---|
| `replication_outbox` | `PendingReplicationOp`s awaiting the master's sequencing |
| `forwards` | A member's forwarded lines, resent in order until the master confirms |
| `replaying_remote_op` | Suppresses undo and the outbox while a remote op is applied |
| `last_applied` | The last journal id + sequence applied |
| `is_authority` | Standalone or master: this station sends DMX |
| `subtree_owners` | Last user + undo seq per subtree |
| `link`, `link_requests`, `session_requests` | The outbox the kernel asks LumaSession work through (`session_link`) |

Claims live in `Runtime` (`claims`, `claim_order`, `claim_asks`) and travel with the
`SessionRuntime`.

## Tests

The property tests `ops_apply_strictly_in_order` and `a_member_ends_with_the_masters_show`
(`crates/dev/lumacore-sweep/src/tests.rs`), the stream and election tests beside their code in
`lumacore-lumasession`, and the sweep's in-process member check every 25 lines
([testing.md](../testing.md)).
