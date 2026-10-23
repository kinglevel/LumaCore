---
id: 0229
title: Every crate has a README naming its checks
area: crates
status: built
date: 2026-10-15
built_in: 34f1 onwards (ratchet, crate by crate)
supersedes: []
superseded_by: —
confirmed_by: [crates_have_readmes]
source: [archive §34 L8048–8063 (item 18)]
---
# Every crate has a README naming its checks

## Context
Asked 2026-10-15 with the structure round: a crate's purpose, surface and rules were spread over its `lib.rs` doc, CLAUDE.md and the decisions log, and nothing said which rule had a check. The README is the crate's specification page, quick to read; decisions stay where they are decided.

## Decision
Every workspace crate (under `crates/`, `apps/`, and the test crates) has a `README.md` in one shape, checked by `crates_have_readmes`: `# <package>` plus one sentence, then Purpose · Owns / Doesn't own · Public surface · Rules · Edges · Decisions · Testing, in that order. The README never restates a decision: its Decisions section lists what it follows, one line each, linked. Every rule names its check in brackets — a test, a tidy rule, the compiler, or `unchecked` written out — and tidy verifies a named test or rule exists. Public surface is hand-written (what other crates should use); Edges are checked against the crate's `EDGES` row and its Cargo dependencies. `lib.rs` keeps its banner and one `//!` line pointing at the README (see file-headers-and-parts). A crate gets its README, headers, `foo/tests.rs` and module renames in one pass, smallest crates first; a new crate is born with all of them.

## Consequences
A new dependency edits the README's Edges with the `EDGES` row, or `crates_have_readmes` fails; a rule without a check says `unchecked`. The crates still without one are the ratchet `README_LEGACY` / the readmeless count in tidy, which only go down. Open: the last legacy READMEs.
