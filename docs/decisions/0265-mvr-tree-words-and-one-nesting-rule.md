---
id: 0265
title: MVR tree words; Assign puts in, Move At a node nests
area: patch
status: built
date: 2026-10-20
built_in: 34g4 part 3c, round 3 1c, 40d
supersedes: [archive §34 L9483–9484 "Delete deletes the node's fixtures", archive §34 L9634–9636 "then unpatches them", archive §34 L9597–9599 "read only: they come by import"]
superseded_by: —
confirmed_by: [E-PATCH-011, `commands::door::nest_ask`, lum.md's MVR examples, `tests/e2e/tests/kit-34g4.spec.ts`, the 40d e2e on the rig]
source: [archive §34 L9473–9484, archive §34 L9595–9649, archive §34 L9995–10003, archive §34 L10153–10157, archive §34 L10658–10671, archive §40 L11700–11744]
---
# MVR tree words; Assign puts in, Move At a node nests

## Context
Point 12 made the sidebar MVR's own Layers › Groups and private sorting tags. The words were a Lum vocabulary question (declined: tree paths only, `Layer` / `GroupObject`); Delete of a node first deleted its fixtures, then unpatched them, and now asks; 40d added folder fixtures to the nodes.

## Decision
`MvrLayer`, `MvrGroup` (numbered pools of `Devices.Metadata.MVR`; a group's place is its `"Parent"`), `MvrClass`, `MvrPosition` (by import only, labelled in the draft); a bare one selects the fixtures in and under it. Putting in is Assign: `Assign Fixture 101 Thru 104 At MvrGroup 4` (one place — it leaves the old), `Unassign Fixture … At MvrGroup` → No MVR group; `MvrClass` likewise. A node is `MvrLayer n`, `MvrGroup n` or a folder `Fixture Media n` (`devices_meta::nodes::node_target`). One nesting rule: `Move <node> At <node>` makes it a child after one ask ("… becomes a child of … (n groups, n fixtures)?" Nest / Cancel; `/NoConfirmation` for stored lines and the dialog's drag; `commands::door::nest_ask`; Tags too); `At <number>` renumbers; a layer nests nowhere; nothing goes under itself or a descendant (E-PATCH-011). Only tree kinds nest; pools stay flat; no sync between the tag tree and the MVR tree. Every tree write needs the Devices draft. Delete of a node asks once: `Delete MvrGroup 3 /KeepChildren` (children to the parent) · `/WithChildren` (the fixtures too, naming what uses them) · Cancel; a folder fixture asks Delete all | Keep them (`/KeepChildren`) | Cancel (`fixture::delete_ask`). The sidebar's ≡ menu and the lift run these lines (see nested-fixtures-folders).

## Consequences
Never a silent nest or a per-kind delete rule. Not built: Duplicate (its word first), New folder here on a folder, the Metadata tab (3d).
