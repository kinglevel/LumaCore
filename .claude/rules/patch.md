---
paths:
  - crates/model/lumacore-patch/**
  - crates/model/lumacore-fixtures/**
  - crates/model/lumacore-state/src/patch_draft*
  - crates/model/lumacore-objects/src/objects/patch/**
  - crates/formats/lumacore-mvr/**
  - crates/formats/lumacore-poolxml/**
  - crates/formats/lumacore-gdtf/**
  - crates/ui/lumacore-views/src/patch*
  - crates/ui/lumacore-webui/web/kit/insert.js
  - crates/ui/lumacore-webui/web/kit/mvrimport.js
---

# Patch, fixtures, ids, MVR

Records: `patch-slot-key-and-three-ids`, `nested-fixtures-folders`, `staged-patch-draft`, `universe-move-delete-ask`, `patch-faithful-to-mvr-and-guids`, `mvr-metadata-pools-by-uuid`, `mvr-tree-words-and-one-nesting-rule`, `import-mvr-reviews-against-the-draft`, `patch-words-store-assign-at-dmx`, `patch-dialog-one-tile-claim-button`, `insert-fixtures-dialog`, `dmx-map-two-zooms-park-on-conflict`, `identify-probe-find`, `renumber-is-move-skip-by`, `multipatch-inside-the-fixture`, `gdtf-native`, `cells-on-fixture-type`.

## The key and the ids

- The patch is `Devices.Dmx.Patch`, a `BTreeMap<Slot, Fixture>`: the key is a slot given at insert, never shown, never on a line (`Slot` has no `Display`). Every read goes through `PatchData`'s one door (`fixture`, `fixture_by_fid`, `fixture_by_id`, `fixture_by_guid`, `insert_fid`, `remove_fid`, `fids`, `by_number`); `fixture_by_fid` is a checked number → slot lookup (`FidIndex`), never a scan, never trusted without the slot's number agreeing. A line names a fixture by `Fixture::handle()`.
- A fixture carries three ids on `Fixture.Mvr`: `fid: Option<u32>` (`Fixture 849`; none for a folder), `id_type: IdType` (MVR's list with Generic in its reserved slot, the default type of a CID) and `cid: Option<u32>` (`Fixture Media 7003`); FIDs unique in the show, CIDs within their type; properties `"FID"` / `"IDType"` / `"CID"`. A plain `Fixture Media 7003` selects by GUID; the resolver's `Member.no` is the FID (0 for a folder, its GUID beside it, `resolve::fixture_guids`). The tree path is `Fixtures.<fid>` (a folder: its GUID).
- A fixture may hold fixtures (`Mvr.parent` a fixture); a folder is a channel-less fixture; nothing has sub-fixtures. Every fixture, MVR layer and group keeps `order` under its parent, spaced by `next_order`. Multipatch copies live inside their fixture, owner first.
- A fixture is its meta (the MVR uuid as GUID, name, note) + `Fixture.Mvr` (every MVR field: type / revision / mode / breaks, one world matrix behind `"PosX"`…`"RotZ"`) + `Fixture.Luma` (`MasterReact` and the like). `Devices.Metadata.MVR` holds layers, groups, classes, positions and files by UUID. The `.gdtf` travels in the show and the `FixtureType` is derived from it; cells per mode live on the type.

## The draft

- Claiming Devices (`Claim Root.ShowData.Devices`, an area named by its tree path, never a word, or `enter patch`) edits a draft of the whole `Devices` (`lumacore_state::patch_draft`); the published show keeps the live devices; the holder's edits skip undo and replication; others' are refused (E-PATCH-008) except the declared live columns (`Props::live`): such a line, and every Oops / Redo while a draft is open, runs against the live devices (`commands::live_draft`, swapped in for the statement) and the draft follows cell by cell (`patch_draft::rebase`; a cell the holder changed is refused naming them).
- `Unclaim Root.ShowData.Devices` asks Keep / Discard / Cancel (`/Keep`, `/Discard`); Keep files one undo step. A holder gone for the watch's timeout loses the draft. Networks, lines and routes are live and need no claim.
- The draft holds fixtures and universes (MergeMode with them); lines name slots by number and stay live for anyone; Move Universe over input lines asks; Delete Universe names its fixtures (they lose their address there) and lines; Keep names lines on empty slots.

## The words

- `Store Fixture` creates, `Assign Fixture … At DMX …` addresses, `Unassign` removes the address; `DMX` is the slots object (`Fixture n DMX`, `/FixtureType`, `/Every`, `/KeepSpacing`, `/Swap`). Renumber is `Move` (`/Skip`, `/By <column>`). An overlap on the map parks both. `Identify` is one object above everything but Park, per user, self-ending; leaving the patch runs `Off Identify`.
- The MVR tree words `MvrLayer` / `MvrGroup` / `MvrClass` / `MvrPosition` (`objects/patch/devices_meta`): `Assign Fixture … At MvrGroup n` puts in; `Move <node> At <node>` nests after the one nesting ask (`commands::door::nest_ask`, shared with Tags); Delete of a node asks Delete all | Keep them (`/KeepChildren`) | Cancel. The sidebar shows nodes only; a folder fixture is a node.
- `Import Mvr "file" [/Drive] [/Scope <target>] …` reads the file (`lumacore-mvr`) against the holder's draft and keeps the review on `PatchDraft.review` (nothing written; `List` / `Info Mvr`; the `patch` feed's `mode: review`); `Import Mvr {rows} /Apply` writes the ticked rows (a gone MVR group asks per group, `/Gone Recreate | Parent | None`); `/Cancel` drops it; Keep waits for it. The preset is the profile's "MVR import" properties (`Set UserProfile "MvrMatch" …`, `MvrIdType`). A file fixture inside a file fixture nests; an untyped CustomId gets the profile's `MvrIdType` or `/IdType`; a numberless fixture with a second id is a New row; a folder already in the draft is Same by uuid.
- `Import PoolXml "file" Groups [/Drive] [/DataPool n]` makes one group per exported group by fixture number (`lumacore-poolxml`, streaming). `Import FixtureType` reads `library/gdtf/`; revisions by SHA-256; a member's upload goes to the master first.

## The dialog

- The Menu's one Patch tile is `Menu "Patch"`, Live by default; `<luma-setup>` with an area: crumb, kind rail, tree sidebar (MVR tree / Class / Tag), the hold-to-claim button, the ✕ asks while claimed. The grid is the review when one is open, with its own slab cells (`Action.when`). Insert fixtures is Choose then Place, every field part of one `Store` line (`kit/insert.js`); the DMX map is one element with two zooms; Edit Patch has its declared ways in.
- Not built, named so: Duplicate (its word first), the Metadata tab, Export MVR, MVR-xchange per NIC, type / revision / mode change with native values (part 5), Map & clone (part 6), the `Fixture.Luma` extras (part 7), private sorting by tags.
