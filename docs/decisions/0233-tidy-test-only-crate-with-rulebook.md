---
id: 0233
title: Tidy is a test-only crate with one rulebook
area: tidy
status: built
date: 2026-10-15
built_in: 34f1
supersedes: []
superseded_by: —
confirmed_by: [the_rulebook_matches_the_rules]
source: [archive §34 L8108–8112 (20.8), archive §34 L8621–8629]
---
# Tidy is a test-only crate with one rulebook

## Context
Tidy began as one long `tidy.rs` inside the station, and its rules were repeated in CLAUDE.md, code-structure.md and testing.md, each copy drifting from the code. Decision 20.8 made it a crate when the station's code moved out, and gave the rules one written home.

## Decision
`crates/dev/lumacore-tidy` is a test-only crate (paths from the workspace root, see lumacore-root-env), one file per area — words, layers, render, doors, frontend, structure, and `files` for the shared scanners and the ratchet check. Its rulebook is `docs/meta/tidy.md`: one entry per rule — the name (= the test), what it checks, why (the §), the escape comment, and for a ratchet its constant and value. `the_rulebook_matches_the_rules` checks both ways: a test without an entry, an entry without a test, or a ratchet number that differs from the code fails tidy. A new rule is written in the rulebook first, then as a test in its area's file. Escapes are a comment naming why, `// tidy: <tag> (<reason>)`, never a bare marker. Other docs link the rulebook instead of repeating a rule. The rules that read the station see the state and objects crates as `core/…` and a cut-out crate under its old folder name (`CUT_OUT`) until the station is empty.

## Consequences
`cargo test -p lumacore-tidy` is part of every step's gate; a ratchet that drops is lowered in the code and the rulebook together or the test fails. Tidy never ships (see dev-tools-never-ship). Open: the `core/…` naming goes as the last station-reading rules read the crates' own paths.
