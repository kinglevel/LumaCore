---
id: 0261
title: The patch is faithful to MVR and to GUIDs
area: patch
status: built
date: 2026-10-18
built_in: 34g4 part 1
supersedes: [archive §34 L9499–9508 "separate Luma layer tree"]
superseded_by: —
confirmed_by: [the exhaustive `Walk` destructure, E-SHOW-003, `tests/catalog/testdata/tree_paths.golden.txt`]
source: [archive §34 L9395–9434, archive §34 L9522–9541]
---
# The patch is faithful to MVR and to GUIDs

## Context
The patch overhaul was hardened on 2026-10-17 after reading MVR 1.6, GDTF and a reference console's patch. A separate Luma layer tree, an "Automatic" layer placement and a By-Luma-layer export were decided first and dropped once tags became the private sorting method.

## Decision
The patch is `Devices.Dmx.Patch` (FixtureTypes, Fixtures, Universes) and holds only fixtures; MVR's 3D objects belong to `Root.ShowData.3DSpatial`, not hardened; a fixture's links to them stay GUIDs, shown as outside the patch, never dropped. A fixture is its meta (the MVR uuid is its GUID, MVR's Name its name) + `Fixture.Mvr` (every MVR 1.6 `<Fixture>` field, typed, links as GUIDs; `lumacore_fixtures::fixture_mvr`) + `Fixture.Luma` (what MVR lacks; MasterReact today). One home per field: a field MVR defines lives in `Fixture.Mvr`; properties stay flat to the operator (`Set Fixture 101 "Name" "x"`), each property's home naming its part; the parts are tree nodes, never words (no `"Mvr.Name"`). Position and rotation are one stored world matrix; `"PosX"`…`"RotZ"` read from and write into it; export writes local = parent⁻¹ × world. Type, revision and mode are GUID links; a fixture always links a type in the show; an embedded GDTF with a known FixtureTypeID and new content becomes a new revision, kept by SHA-256; a mode is found by name inside its revision. The show file keeps `Devices/Devices.json` and `Devices/Dmx/Patch/FixtureTypes/<n>/Revisions/<i>/source.gdtf`. Later device kinds get their own `Devices.<Kind>`.

## Consequences
A new MVR field goes into `Fixture.Mvr` with its `Walk` arm and declared property, never a second home. Point 5's selectable numbers are replaced by the slot key (see patch-slot-key-and-three-ids); the pools are mvr-metadata-pools-by-uuid; the claim is staged-patch-draft. Open: 3DSpatial, export-mvr-and-xchange, fixture-luma-extras-live-columns.
