---
id: 0132
title: Fuzzing is a long run; property tests are fast tests
area: tests
status: built
date: 2026-10-11
built_in: step 10
supersedes: [archive §25 L2847–2854 "fuzz/ over lumacore_station::fuzzing"]
superseded_by: —
confirmed_by: [`scripts/unix/fuzz_long`, the proptest suite in `lumacore-sweep`]
source: [archive §25 L2731–2762, archive §25 L2847–2854]
---
# Fuzzing is a long run; property tests are fast tests

## Context
Step 10 turned the promised invariants (replication order, undo, identity, merge, election) into properties and put a fuzzer on every decoder on the open LAN. The user did not want slow tests in fast development, so the two kinds got two schedules.

## Decision
Fuzzing is a long run only — never a step's test, never in `cargo test`, no decoder smoke tests on stable. The targets live in `tests/fuzz/`, a cargo-fuzz crate outside the workspace (nightly only), reaching private decoders through `#[doc(hidden)]` entry points in `lumacore_sweep::fuzzing`; no `arbitrary` derives on wire or tree types, and a fuzz build never compiles the server. Each target checks no panic and no hang on any bytes; codecs that encode check decode(encode(x)) = x. The targets cover every decoder on the LAN (OSC, sACN, Art-Net, the command port, shell packets, the beacon, `Time`, the stream frame, tokens) plus the lexer and parser with dispatch on a fresh state. No input size caps (isolated networks): fuzzing looks for panics and hangs, not memory use. Property tests (proptest, fixed seeds, `lumacore-sweep`'s `src/tests.rs`) are ordinary fast `cargo test`s, run per step when their area changes; one the code breaks is `#[ignore = "finding: …"]`. The long run is `./scripts/unix/fuzz_long [minutes] [--fresh] [--no-fuzz]`, started by hand overnight, resumable: the sweep (see command-sweep-dynamic), every fuzz target, the property tests with a random seed; the report is `target/fuzz-report/summary.md`. No CI, no scheduler.

## Consequences
Findings are recorded and fixed later, crash inputs kept in the fuzz crate's artifacts; a new decoder on the LAN gets a target and a hidden entry point, never a derive on the type. The archive's older paths (`fuzz/`, `lumacore_station::fuzzing`) are superseded by the ones above.
