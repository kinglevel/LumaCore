---
id: 0271
title: The patch is keyed by a Slot; FID, IDType, CID
area: ids
status: built
date: 2026-10-20
built_in: 40b
supersedes: [archive §34 L9414–9417 "selectable numbers are MVR's", archive §34 L10388–10401 "No CIDs: tags instead", archive §34 L10682–10686 "several second ids", archive §34 L10368–10373 "the second id dropped"]
superseded_by: —
confirmed_by: [`Slot` has no `Display`, `FidIndex`, `PatchData`'s door, the tidy ratchet on `patch.by_id`, E-CMD-006]
source: [archive §34 L9414–9417, archive §34 L10158–10165, archive §34 L10388–10409, archive §40 L11597–11627, archive §40 L11672–11681]
---
# The patch is keyed by a Slot; FID, IDType, CID

## Context
The key went from the FID (a fixture without a number could not exist) through "No CIDs: tags instead" (second ids read-only, `MoreIds` for several) to §40's reading of a real rig, whose tree folders have no FID and whose second ids are used everywhere. The user chose that way and an internal key.

## Decision
The patch is `BTreeMap<Slot, Fixture>`: the key is a slot given at insert, never shown, never on a line (`Slot` has no `Display`); nothing stored points at it. Every read goes through `PatchData`'s one door (`fixture`, `fixture_by_fid`, `fixture_by_id`, `fixture_by_guid`, `insert_fid`, `remove_fid`, `fids`, `by_number`); `fixture_by_fid` is a checked number → slot lookup (`FidIndex`), never a scan. A fixture carries the three ids on `Fixture.Mvr`: `fid: Option<u32>` (`Fixture 849`; none = a folder), `id_type: IdType` (MVR's list, Generic in its reserved slot — the default type of a CID) and `cid: Option<u32>`; FIDs unique in the show, CIDs within their type; the properties `"FID"` (renumbers, the slot stays), `"IDType"`, `"CID"`. A line names a second id inside the fixture's id: `Fixture Media 7003`, `Fixture Media 1 Thru 9` (`LumShape::id_kinds`; the one resolver walks the ids that exist); with Fixture as the default object a bare `Media 7003` is written out; a bare `Media 7003` is never read as a second id on its own. MVR's `CustomIdType` + `CustomId` come in on import and go out on export; a fixture holds one second id — `MoreIds` is gone. The tree path is `Fixtures.<fid>`, a folder's its GUID.

## Consequences
Never show or type a slot; every new read goes through the door (the ratchet only goes down); never a FID namespace or a LumaCore-only id type. Folders: see nested-fixtures-folders.
