---
id: 0248
title: The whole catalog's tests are the crate tests/catalog
area: tests
status: built
date: 2026-10-16
built_in: 34g4 cut 2
supersedes: []
superseded_by: —
confirmed_by: [`cargo test -p lumacore-catalog-tests`: `the_reference_is_current`, `settled_clashes`, `the_demo_show_builds_loads_and_matches_its_golden`]
source: [archive §34 L8504–8516 (item 25.15), archive §34 L8997–9016]
---
# The whole catalog's tests are the crate tests/catalog

## Context
The tree shell's cut left the kernel's test binary seeing only the kernel's words: the reference check, the lum.md examples and the Lum words golden failed, and every row check would have checked less with each further cut. Chosen over the station's own tests (the station only assembles) and over each crate running the row checks on its rows (the same checks copied per crate).

## Decision
`tests/catalog` is the package `lumacore-catalog-tests`, beside systemtest and the fuzz crate, dev-dependencies only, its test binary on the station's full catalog (`lumacore_station::register`). It holds every check whose subject is every row: one namespace, help examples that parse, reading is anyone's, planned words change nothing, objects' verbs / properties / tree defs, pool members' names, the shim ratchet, the rights table, every menu's JSON and columns; the lum.md examples with `PENDING` and the Lum words golden; the reference check; the golden record of every tree path and of the demo show. Modules by feature (`rows`, `lum`, `rights`, `menus`, `paths`, `words`, `wiring`, `lines`, `import`, `protocols`, `kernel`). A crate's tests of its own logic stay beside its code (see three-homes-for-a-test).

## Consequences
A row check or a golden over the whole vocabulary goes here, never in a crate that sees part of the catalog. Goldens are rewritten only with `LUMACORE_BLESS=1` after reading the diff. The crate is in the gate before every commit (`cargo test -p lumacore-catalog-tests`).
