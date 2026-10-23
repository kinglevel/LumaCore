---
id: 0246
title: sysmon, the luma disk side and fixtures are crates of their own
area: crates
status: built
date: 2026-10-16
built_in: 34g2a, 34g2b, 34g2d
supersedes: []
superseded_by: —
confirmed_by: [tidy `crate_edges_point_down`, tidy `crates_sit_in_their_group`]
source: [archive §34 L8438–8451 (item 25.10), archive §34 L8917–8935, archive §34 L8947–8954]
---
# sysmon, the luma disk side and fixtures are crates of their own

## Context
Before the objects crate could be cut (see objects-split-kernel-and-domain-crates), three things below the commands sat in the wrong crate: the System Monitor in state, state's whole disk side, and the fixture data in the tree. LLVM's `Support` and Chromium's `//base/logging` were the model for a logging system that knows no domain.

## Decision
`crates/foundation/lumacore-sysmon` holds the System Monitor, feedback and `panic_report`, on no workspace crate; every crate logs to it directly (lanes' watchdog and clocksync lost their `warn` callbacks), the render tick still never logs; a panic report's E-code text comes from the catalog, not state. `crates/formats/lumacore-luma` is state's disk side: `show_file` (with `parse_show`), `drive`, `background`, `backups`, `media`, `station_config`, `build_info` and the E-SHOW / E-MEDIA codes; what reads `AppState` (the recovery asks) stays in state as `recovery`; tidy reads the crate as `core/`, so the disk door rule holds there. `crates/model/lumacore-fixtures` holds `fixture_type`, `fixture`, `attribute_def` and `universe` with their `Identified` / `Walk` impls, on base and the protocols; the identity macros are base's, one copy for every crate with show types; `lumacore-gdtf` sits on base and fixtures, not the tree. The protocol contract's move below the tree (34g2c) is its own record.

## Consequences
State keeps `AppState`, the change path and Oops only. A new logging call names `lumacore_sysmon`; a new file read or write goes in `lumacore-luma` behind `drive`; new fixture data goes in `lumacore-fixtures`. Crate edges are checked by tidy; a crate out of its group fails `crates_sit_in_their_group`.
