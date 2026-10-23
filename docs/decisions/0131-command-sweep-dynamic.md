---
id: 0131
title: The command sweep is generated from the registries
area: tests
status: built
date: 2026-10-11
built_in: step 10
supersedes: [archive §25 L2842–2846 "sweep in the station"]
superseded_by: —
confirmed_by: [`lumacore-sweep`, the `run.json` resume, the sweep report's `unreachable` list]
source: [archive §25 L2781–2833, archive §25 L2842–2846, archive §29 L3122–3130, archive §29 L3272–3276]
---
# The command sweep is generated from the registries

## Context
The user asked for every command, keyword and combination to be tested, "dynamic, so if we add new features down the line we dont need to update it", and for the Unix-like words and the disk to be left alone. A hand-written list would rot with the first new verb.

## Decision
The sweep (`crates/dev/lumacore-sweep`, its binary a thin main in the station) reads everything at run time from the catalog: `CommandDef` names, aliases, flags and `args`, `ObjectDef` keywords, verbs and pools, the writable properties, the completion engine, the error catalogs. Phase 1 is exhaustive — every verb × keyword × flag × writable property, ids and values in and out of range; phase 2 is random stateful sequences grown from completion. Every line runs once per rights level (users generated from `UserRights::ALL`, plus one holding claims). `Show` lines run in an isolated in-process `AppState` (no server, protocols, engine threads or sockets), `Drive` lines in the drive door's sandbox, a throwaway copy of `library/` (see reach-drive-door); the tree shell stays out. After every line: no panic, no E-INTERNAL-001, no hang, no duplicate GUIDs or new broken links, a refused line changes nothing, Oops restores the pre-image, `Set` reads back or refuses with a code; periodically save → load, a member equals the master, the plan compiles and a tick runs. What it can't generate is reported as `unreachable`; a finding is logged with a replay line and the run goes on; `target/fuzz-report/run.json` makes it resumable.

## Consequences
Never add a hand-written command list; a new `Args` shape or `PropKind` gets a generator when the report names it unreachable. It is a long run only, started by `./scripts/unix/fuzz_long` (see fuzz-long-run-only); an uncoded `Error:` is a finding (see errors-coded).
