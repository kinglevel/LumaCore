---
id: 0091
title: The .gdtf is the source; the reader is the vendored fork
area: patch
status: built
date: 2026-10-20
built_in: 11b, 40a
supersedes: []
superseded_by: —
confirmed_by: [refresh_revision, Info FixtureType, vendor/README.md, the "// LumaCore:" marks]
source: [archive §29 L3234–3243, archive §29 L3294–3300, archive §40 L11551–11560, archive §40 L11588]
---
# The .gdtf is the source; the reader is the vendored fork

## Context
11b made the GDTF file itself the truth so a type could be re-derived and later updated. In §40 the crates.io `gdtf` reader refused most of the GDTFs embedded in Capture and Depence exports — attributes the spec gives defaults for were left out — while it read the GDTF-DB fine; the fork fixed every file the same day.

## Decision
The original `.gdtf` is the source of truth and travels in the show (its place: see luma-show-file-container; the station's cache `library/fixturetypes/<sha256>.gdtf`). `FixtureType` is a derived view rebuilt from the file by `lumacore_gdtf`: the geometry tree with cells (GeometryReferences at their break offsets or sibling geometries), every ChannelFunction with DMX / physical ranges and ChannelSets, `DMXChannel@Highlight`, `LogicalChannel@Snap`, relations (Multiply = master dimmer), wheels, emitters, filters; footprint = the highest slot of break 1; the resolver keys by (geometry, attribute); `refresh_revision` keeps GUIDs; the render plan places a channel in its GDTF break. Import is warn only — it refuses nothing it can read, the report is in `Info FixtureType`.

The reader is the vendored fork `vendor/gdtf/`, pinned over its crates.io version with `[patch.crates-io]` in the root `Cargo.toml`, a path dependency and never a workspace member (tidy doesn't scan it), with the spec's defaults and lenient value readers (PhysicalUnit, Axis Position, Highlight, Pretty, Description, NaN colours, malformed gamut points, a negative DMXFrom, an oversized CRI). Every change is marked `// LumaCore:` and listed in `vendor/README.md`.

## Consequences
A reader fix goes into the fork, marked and listed — never a workaround in lumacore-gdtf. Editing a fixture type (writing a new `.gdtf` revision) is not decided — roadmap. Importing files and revisions: see gdtf-import-revisions-and-file-index.
