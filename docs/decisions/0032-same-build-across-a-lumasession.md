---
id: 0032
title: Every station of a LumaSession runs the same build
area: lumasession
status: built
date: 2026-10-16
built_in: 34f7b
supersedes: [archive §21 L1402–1404 "build commit only warns"]
superseded_by: —
confirmed_by: [E-LUMA-006, Hello.version + commit]
source: [archive §34 L9136–9140, archive §34 L9204–9206]
---
# Every station of a LumaSession runs the same build

## Context
6b matched the stream protocol and the show format on a join and only warned about a different build commit. The 34f7b hardening, asked with real-world scenarios (a show naming a protocol one build lacks), closed that: a mixed rig is refused, as established consoles require one software version.

## Decision
Every station of a LumaSession runs the exact same build — version and commit, the build id `lumacore-server --version` prints. `Hello` carries `version` and `commit`; a join from another version or commit is refused before the show is taken (E-LUMA-006, naming both builds). The protocol and format checks stay first (E-LUMA-001 / 002, see master-executes-results-flow). The commit is stamped only by the station (`apps/lumacore-station/build.rs` → `lumacore_station::stamp()` → `lumacore_luma::set_git_commit`; `unknown` in tests). A rebuilt station is rebuilt everywhere. A show naming a protocol this build lacks, or a setting its protocol doesn't declare, is refused at load (E-SHOW-007 / 008; see the protocols record).

## Consequences
Never add a `build.rs` or `env!` stamp lower in the crate graph — a commit would recompile everything above it. No "compatible enough" path: a different build never joins and never warns. A converter for old shows is deferred to major releases, not built.
