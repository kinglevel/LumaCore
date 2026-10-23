---
id: 0106
title: Tree paths are typed; code names a Node, never a string
area: commands
status: built
date: 2026-10-18
built_in: code review steps 0–7 (2026-10-14), lum part 3a, decisions 53–54 (2026-10-18)
supersedes: [archive §31 L6815–6817 "string compares ~10 files", archive §31 L6827–6828 "cd enter lenient", archive §33 L7307–7325 "numeric positions refused"]
superseded_by: —
confirmed_by: [tidy `tree_paths_are_typed`, golden `tests/catalog/testdata/tree_paths.golden.txt`, `Node::no` / `nth_child`, E-SYN-003]
source: [archive §31 L6815–6837, archive §33 L7210–7215 (item 31), archive §33 L7307–7336 (items 49, 50), archive §33 L7388–7419 (items 53, 54, 55, 56)]
---
# Tree paths are typed; code names a Node, never a string

## Context
The code review found `"root.showdata…"` strings compared in about ten files. A reference console, probed live, accepts dotted tree node names (`ShowData.DataPools.1.Sequences.1`), refuses a dotted keyword path, numbers every node and warns the numbers change between releases.

## Decision
One `Node` enum in `core::tree_path` declares every node once with its parent and `Kind`; children derive from the parent links, so two tables can't disagree. `TreePath` is steps with one parser. Code names a `Node`, never a `"Root.…"` string; the VFS, claims, List, selections, scope and completion match on it; `cd` / `enter` / `ls` share `AppState::resolve_target`; `ShowSubtree::node()` is exhaustive.

In Lum, a word starting with `Root.` is the tree form — with `Self.` the only place a dot nests: `Select Root.ShowData.DataPools.2`. A part is a node name as stored (plural collections), a number or quoted text; the object keyword is not a part (`DataPool.2.Group.3` → E-SYN-003). Inside the fence a path is relative to where you are (`cd DataPools.1.Groups.1`, `cd ..`, `cd 3`); `Root.` makes it absolute; a slash path is refused naming the dotted form.

Every node has a number from its declaration order, none written in code; a collection's children keep their member numbers. A number in a path is the child's No: `Root 3`, `Root.3` and `Root.Resources` name one node; `Root` is a counted object whose default verb is List (`List Root 3`, `Root 1.5`). `print` keeps a path's numbers or names as typed — the echo confirms what ran; a stored line may hold numbers and may point elsewhere after an update (accepted).

## Consequences
A path change shows in the golden (`LUMACORE_BLESS=1`, read the diff). Fuzzy matching of node names belongs to typing only (see one-registry-one-namespace); `resolve_target` still prefix-matches in a line (left for 3f, open).
