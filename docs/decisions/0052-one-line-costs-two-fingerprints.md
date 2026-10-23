---
id: 0052
title: A line costs two fingerprints, never a second walk
area: engine
status: built
date: 2026-10-20
built_in: 40f
supersedes: [archive §40 L11790–11796 "sysmon's JSON walks"]
superseded_by: —
confirmed_by: [`lumacore_base::fingerprint`, `FidIndex`, `the_demo_show_builds_loads_and_matches_its_golden`]
source: [archive §40 L11773–11800]
---
# A line costs two fingerprints, never a second walk

## Context
When every test moved onto the rig (the demo show with over a thousand fixtures, see demo-show-test), a command line in debug took about a second: two JSON walks of the show for the System Monitor's "tree modified" line and two more for the change path's fingerprints, plus `Fixture n` scanning the slot map.

## Decision
A line's fixed cost on a big show is the change path's two fingerprints — before and after the statement — and nothing else. `lumacore_base::fingerprint` is a serde serializer into a byte buffer hashed once, with tags and lengths so shapes never collide, in the readable form JSON has; `finish` hashes once and re-hashes all but Devices after the attribute sync. The sysmon's walks are gone: `TreeCounts` is counts, the change path says whether anything moved, `sysmon::reported` keeps the old subtrees. `Fixture n` is a lookup through `PatchData`'s `FidIndex` (number → slot, never saved, every hit checked against the slot's number), never a scan (see fixture-ids-and-slot-key). The four model crates build at a raised opt-level in debug (`[profile.dev.package.…] opt-level` in the root `Cargo.toml`) because the walk is instantiated in the state crate. Never add a second walk of the show per line.

## Consequences
Anything that wants to know what a line changed reads the change path's result, not the tree again. A new per-line feature that walks `ShowData` is refused in review; measure it on the rig first. The measured numbers belong in `docs/dev/benchmarks.md` and the archive, not here. The next lever is structural and open on the roadmap: the fixture types are most of the Devices JSON and change only on Import.
