---
id: 0006
title: Identity and Annotation: meta in levels on every node
area: tree
status: built
date: 2026-10-17
built_in: Tags part 1b
supersedes: [archive §12 L1038–1040 "per-object META lists", archive §12 L1052–1054 "guid_identified identified macros", archive §12 L1033–1035 "NodeMeta.id number"]
superseded_by: —
confirmed_by: [every_object_with_a_node_meta_has_its_level, every_resource_has_a_derived_guid]
source: [archive §12 L1000–1098, archive §21 L1613–1618]
---
# Identity and Annotation: meta in levels on every node

## Context
Step 6d gave objects a GUID through `guid_identified!` / `identified!` macros and per-object `META` lists, with a number (`NodeMeta.id`) duplicated in the meta. Tags part 1b (2026-10-17) needed tags as saved links on every show object and became the meta model for the whole tree, so later areas add a level instead of another shape.

## Decision
Meta comes in levels. Every node under `Root` has an `Identity { guid, name }`; every `ShowData` object, its parts included, also has an `Annotation { note, paint, icon, locked, hidden, tags }`. A show object holds one flat `meta: NodeMeta`; Runtime, StationSettings and Resources nodes hold `meta: Identity`. The blocks and the `Meta` trait live in `lumacore_base`; `Identified` is implemented only through `meta()`, so a show type without a NodeMeta does not compile. The name is an optional display name; a name that is an identity stays its own required field (`User.login`, `UiKey.id`, an Attribute's GDTF name). No number lives in the meta: the pool's key is the number. Properties come from the level — `IDENTITY_PROPS` and `ANNOTATION_PROPS` declared once, an object only saying `Props.meta: MetaLevel`. Outside ShowData a GUID is derived where the node is stable (`Guid::named`: window types, core UI keys, stations), fresh where the station makes it (logins, claims), kept in the station files for screens.

## Consequences
A new show type gets `meta: NodeMeta` and no other guid, name, note or paint field; a new tree level is one more block in a new composite. Never add a per-object meta property list or an id in the meta. Two catalog tests catch a show object whose meta is only an Identity and a Resources node whose GUID is not derived. Links are guids-everywhere-links.
