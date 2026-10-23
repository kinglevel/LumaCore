---
id: 0266
title: Devices.Metadata.MVR holds the MVR pools by UUID
area: patch
status: built
date: 2026-10-18
built_in: 34g4 part 1
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §34 L9426–9434, archive §34 L10004–10008, archive §34 L10020–10022]
---
# Devices.Metadata.MVR holds the MVR pools by UUID

## Context
MVR carries more than fixtures: layers, classes, positions, mapping definitions and the file itself. Point 7 gave them one home beside the patch so both the Patch and the later 3DSpatial read the same records and a re-import matches what it brought before.

## Decision
`Root.ShowData.Devices.Metadata.MVR` (`lumacore_fixtures::MvrMetadata`, `devices_meta.rs`) holds Layers, Groups, Classes, Positions, MappingDefinitions and Files, each entry a NodeMeta keyed by its MVR UUID; the tree shows `Devices.Metadata.MVR.<pool>` as collections with counts. Classes, Positions, MappingDefinitions and Files arrive and update only through imports, matched by UUID (a rename in CAD renames here); they can be labelled in the claimed draft (`Label MvrClass` / `Label MvrPosition`; the name goes out with an export, a re-import shows a renamed one as a diff cell to take or leave), filtered by, and used as an import scope. Layers and Groups are the editable patch tree (see mvr-tree-words-and-one-nesting-rule). Files is one record per import: name, SHA-256, provider and version, MVR version, when, who, scope, preset, counts and UserData; the `.mvr` itself goes to `library/mvr/.store/<sha256>.mvr` and travels between stations by hash.

## Consequences
Never a second home for an MVR record and never a match by name. Not built: "Clean up unused" (a function-block button on the Metadata tab, claimed draft, naming what it removes — classes, positions, files nothing links to — and asking once, one Oops step) and the Metadata tab itself (3d).
