---
id: 0008
title: The change path diffs subtrees; undo per ShowData field
area: undo
status: built
date: 2026-10-11
built_in: step 6a
supersedes: [archive §26 L2470–2471 "hand-written affected_subtrees list", archive §26 L2469 "root.change door"]
superseded_by: —
confirmed_by: [change::subtrees]
source: [archive §21 L1331–1393, archive §26 L2466–2479, archive §26 L2480–2495]
---
# The change path diffs subtrees; undo per ShowData field

## Context
Handlers once named the subtrees they touched in a hand-written `affected_subtrees` list, and a `root.change(…)` door was planned. Step 2 hardening (2026-10-10) replaced both with an automatic diff, and step 6a (2026-10-11) generated the subtree list from `ShowData`'s fields so undo and replication can never miss a field.

## Decision
Everything in the tree is readable; every write goes through one change path (`core/change.rs`). Dispatch fingerprints every `ShowData` subtree before and after a mutating command; exactly the changed subtrees get the undo frame, the replication op and the `Runtime.Journal` entry. A command declares only `undo: ReadOnly | Untracked | Tracked(OopsKind)`. `ShowSubtree` has one variant per `ShowData` field — pools per entry, per user `User`, `Programmer`, `Selection` — produced by an exhaustive destructure in `change::subtrees`, with write and restore arms in `oops.rs`; a missing pool entry is `null`, users restore before programmers, a restore refuses an occupied slot. Everything is undoable except `Playbacks` (Oops never undoes a Go). Undo lives on the master and is replicated: frames are filed on the sending user's stack in `Runtime.Users.{User}.Undo`, never saved; `Oops` / `Redo` are `Untracked`. Writes outside commands (load, join, merge, restore) call `ShowData::after_load()` and bump `AppState::show_generation`.

## Consequences
A new `ShowData` field does not compile until it has its `ShowSubtree` variant and its `oops.rs` arms, and does not save until listed in `show_file` (see the show-file record). Never write a subtree list by hand, never bypass dispatch for a show write, and never add a second walk of the show per line — the two fingerprints are the line's cost on a rig. How a line splits into steps, categories and station-local frames is the Oops record; where undo frames travel is the replication record.
