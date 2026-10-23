---
id: 0235
title: No product crate depends on the developer tools
area: crates
status: built
date: 2026-10-16
built_in: 34g1
supersedes: [archive §34 L8273 "tools/ tidy banner"]
superseded_by: —
confirmed_by: [dev_tools_never_ship]
source: [archive §34 L8117–8118 (20.10), archive §34 L8426–8432 (25.8), archive §34 L8878–8891]
---
# No product crate depends on the developer tools

## Context
The sweep, tidy and the header tool first sat in crates/tools/, and the sweep was in the station, so a fuzz build compiled the server. 25.8 took Chromium's `DEPS` as the model (`//tools` and `//testing` kept out of `//content`) and renamed the folder `crates/dev/`, chosen over rustc's `tools/` knowing "dev" is also Cargo's word — the rule, not the name, keeps the tools out of the product.

## Decision
Tidy, the header tool and the sweep live in `crates/dev/` and are never a dependency of anything that ships. `dev_tools_never_ship` holds it: no crate outside `crates/dev/` names one in it in `[dependencies]`, `[build-dependencies]` or a target table, except the station (the sweep's thin main), systemtest and the fuzz crate. The sweep sits on objects, LumaSession and the protocols, never the server (see station-code-becomes-crates); the fuzz crate depends on the sweep, not the station.

## Consequences
A new developer tool is born in `crates/dev/` with its own README; a product crate that wants a tool's function gets that function moved below the tools instead. Open: none.
