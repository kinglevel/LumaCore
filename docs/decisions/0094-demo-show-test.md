---
id: 0094
title: The demo is the rig; every test runs on it
area: demo
status: built
date: 2026-10-20
built_in: 40f
supersedes: [archive §23 L2169–2179 "committed demo.luma matches", archive §23 L2195–2203 "builds loads committed", archive §29 L3299–3300 "carries AuraXB file"]
superseded_by: —
confirmed_by: [the_demo_show_builds_loads_and_matches_its_golden, demo::ensure, demo::fresh_state, tests/catalog/testdata/demo.golden.txt]
source: [archive §23 L2169–2179, archive §23 L2195–2203, archive §40 L11589, archive §40 L11641–11663, archive §40 L11745–11836]
---
# The demo is the rig; every test runs on it

## Context
Step 9 moved the demo builder into the library and committed a `demo.luma` a test compared against. §40 replaced the six-AuraXB demo with the DemoData rig — "it is good to have a large dataset now when we start to test things" — and the user ruled that all tests must run equally well on it.

## Decision
`lumacore_objects::demo::build` runs the station's own words on the sources in git: `library/mvr/ti2026.mvr` (`Import Mvr … /IdType Media` into the claimed Devices, Apply, Keep), one sACN line per run of universes, the user LD, `library/poolxml/ti2026-groups.xml` (`Import PoolXml … Groups`), then every line of `library/demo/manual.lum` — the manual demo is lines in git, never a file, so a format bump never breaks it; the show is named `Demo`. `library/shows/demo.luma` is gitignored; `demo::ensure` is the one door that builds it when missing or stale (the station's start for that path, the e2e setup, `gen-demo`). The catalog test checks the counts and a SHA-256 of the normalized JSON against `tests/catalog/testdata/demo.golden.txt` (`LUMACORE_BLESS=1`).

Every test runs on the rig through `demo::fresh_state()`: a test names the rig's facts (its counts are in the demo golden) or steps past them, never a small show of its own; a golden that walks the tree caps what it lists and numbers every GUID. A line's cost on a big show is the change path's two fingerprints (`lumacore_base::fingerprint`) — never add a second walk of the show per line; `Fixture n` is an index lookup (`FidIndex`), never a scan.

## Consequences
A new source (another pool's extract) enters git when its word lands, never the full DemoData folder. A station word that runs a script file is its own discussion — roadmap.
