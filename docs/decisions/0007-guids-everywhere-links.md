---
id: 0007
title: Every object a GUID, every stored link a GUID
area: tree
status: built
date: 2026-10-11
built_in: step 6d
supersedes: [archive §21 L1619–1623 "numbers names as links", archive §21 L1690–1691 "tombstones of deleted"]
superseded_by: —
confirmed_by: [E-SHOW-003, ShowData::identity(), List Reference /Broken]
source: [archive §21 L1493–1496, archive §21 L1613–1628, archive §21 L1650–1653, archive §21 L1679–1711, archive §29 L3101–3103]
---
# Every object a GUID, every stored link a GUID

## Context
Merge on rejoin (step 6d, 2026-10-11) must tell a moved object from a new one. A reference console was studied: GUIDs not on every object, links as name paths, a merge that duplicates on a move. LumaCore went further on purpose and uses no tombstones: the 3-way split base tells "deleted" from "never existed".

## Decision
Every object has a GUID through its meta (see node-meta-levels); `Guid` is `lumacore_core::identity` (`Default` = fresh v4, `Guid::named` = v5 of a standard name). Every show type implements `Walk` with an exhaustive destructure (pool / link / data), so a new field does not compile until classified; `ShowData::identity()` is the GUID index and the reference check. Load refuses a missing or duplicate GUID with E-SHOW-003. Every stored link is a GUID — executor → sequence, line → network, user → profile, cue value → preset, group member → fixture, the one-only selections — never a slot number or a name, so a move or renumber breaks nothing. Command text stays text. Copies get new GUIDs, moves and overwrites keep them. Replication, undo and merge name pool entries by GUID. The reference check (`core::references`) runs after load, join, merge, Oops and Redo: a link to a missing GUID is kept (a later merge or undo can heal it), counted in sysmon and `Info Show`, listed by `List Reference /Broken`; nothing may contain itself. Only the same `ShowSettings.show_id` can Merge.

## Consequences
A new relation is a `Uuid` field classified in `Walk`; never a slot-keyed path, never a dropped broken link. A test over the demo show checks every link resolves. Decided, not built: a references view per missing object with Resurrect / Delete links, and the check after every Store (a total reference update).
