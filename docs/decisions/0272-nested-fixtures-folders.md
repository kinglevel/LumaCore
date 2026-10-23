---
id: 0272
title: A fixture may hold fixtures; a folder keeps an order
area: patch
status: built
date: 2026-10-20
built_in: 40b–40d
supersedes: [archive §28 L2982 "room for Fixture 3.4"]
superseded_by: —
confirmed_by: [`next_order`, `nodes::fixtures_under`, E-PATCH-011, `fixture::delete_ask`, the 40d e2e on the rig]
source: [archive §40 L11590–11596, archive §40 L11603–11607, archive §40 L11627–11635, archive §40 L11682–11744]
---
# A fixture may hold fixtures; a folder keeps an order

## Context
The demo rig's MVR and its console export nest fixtures (MVR's `ChildList`, tree folders). The user wanted that imported natively — and never sub-fixtures of a fixture ("a fixture is a fixture"; the part / element system comes later, see cells-on-fixture-type).

## Decision
A fixture may contain fixtures (`Mvr.parent` a fixture); a folder is a fixture of a channel-less type, usually without a FID, named by its second id (`Fixture Media 1`). Every fixture, MVR layer and group keeps `parent` + `order`, spaced by `next_order`; the MVR order round-trips and every walk is (parent, order). A folder on a line is the folder; a value at a folder fans out (`Fixture Media 1 At 50` writes every fixture below, one Oops step; the folder keeps nothing). `Assign Fixture <rows> At Fixture Media 9` places under a folder (never under itself or a descendant, E-PATCH-011); `Move Fixture … At <node>` nests through the nesting ask, `At <number>` renumbers, a folder At a number is refused; Delete of a node asks Delete all | Keep them: move up to <parent> | Cancel (`/NoConfirmation` = all, `/KeepChildren` = kept); `List` / `Info Fixture Media 1` take a folder; counts are fixtures only, folders beside. The sidebar shows nodes only, the Live pane fixtures only in tree order; sorting is the one lift gesture. Import nests file fixtures parents-first in the file's order; an id-less folder gets the next free second id of the preset's type.

## Consequences
Never add sub-fixture ids or a folder type word; sub-fixture group items like `849.1` stay out. Not built: the programmer's fan-out switch, New folder here on a folder, Export's `ChildList` (see export-mvr-and-xchange).
