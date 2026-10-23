---
id: 0225
title: A module's tests live in foo/tests.rs; three homes for a test
area: structure
status: built
date: 2026-10-16
built_in: 34f1 (ratchet); 34g4 cut 3 (tests/catalog)
supersedes: [archive §34 L8008–8010 "tests/it/ binary"]
superseded_by: —
confirmed_by: [tests_live_in_foo_tests_rs]
source: [archive §34 L8008–8010 (item 11), archive §34 L8516–8541 (25.16)]
---
# A module's tests live in foo/tests.rs; three homes for a test

## Context
Item 11 ended three test layouts (sibling `*_tests.rs`, `#[path]`, inline modules) and asked for one tests/it/ binary per crate for behaviour tests. When the station's cut broke fourteen kernel tests that used Debug and Echo, 25.16 looked at how rustc, cargo, nushell, VS Code and others place tests and replaced the tests/it/ plan; code-structure.md's §4 still shows the older text.

## Decision
A module's unit tests are `#[cfg(test)] mod tests;` in `foo.rs` with the body in `foo/tests.rs`, one way — no sibling `*_tests.rs`, no `#[path]`, no inline module body. Which crate a test belongs to is the three-homes rule (see three-homes-for-a-test): beside its code unless it needs words from more than one command crate (then `tests/catalog`) or tests the kernel's machinery (then the stand-in words). Every test runs on the rig (see demo-show-test).

## Consequences
A new or moved module brings its tests into `foo/tests.rs` in the same pass (see crate-readme); a crate's `tests/` folder holds no `it/` binary. `tests_live_in_foo_tests_rs` counts the old test files (`*_tests.rs`, `#[path]`) and only goes down. Open: behaviour tests as plain files (a line and its reply) with a bless mode, after fish, Deno and LLVM.
