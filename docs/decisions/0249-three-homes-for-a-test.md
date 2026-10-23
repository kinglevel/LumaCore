---
id: 0249
title: A test's home: catalog, stand-in words, or beside its code
area: tests
status: built
date: 2026-10-16
built_in: 34g4 cut 6
supersedes: [archive §34 L8010 "tests/it/ binary per crate"]
superseded_by: —
confirmed_by: [the stand-in words in `commands/test_words.rs` (TestAsk, TestEcho, TestInternal)]
source: [archive §34 L8517–8541 (item 25.16), archive §34 L9066–9083]
---
# A test's home: catalog, stand-in words, or beside its code

## Context
The station's cut broke the kernel tests that used Debug and Echo, and the user asked whether every test should move to `tests/catalog`. Research (cargo's one testsuite binary, rustc, rust-analyzer, LLVM lit vs unittests, VS Code's registry tests with throwaway commands, matklad's "Delete Cargo Integration Tests") found no project keeping every test at one level: behaviour tests central, unit tests beside the code, a registry's machinery tested with stand-in commands. §34 11's tests/it/ binary per crate is superseded.

## Decision
One question decides a test's home. A line that needs words from more than one command crate is tested in `tests/catalog`, in feature modules (see catalog-tests-crate); the kernel's dispatch tests move there feature by feature as cuts need it. The kernel's machinery — dialogs, the rights gate, flags, `;`, Oops — is tested with stand-in words only its test binary registers: `commands/test_words.rs` (TestAsk asks and needs Admin, TestEcho prints, TestInternal is the station's own level), never in the product's catalog. Everything else stays beside its code in `foo/tests.rs`: unit tests and a command crate's own words, on the kernel and that crate. Every test runs on the rig (see demo-show-test).

## Consequences
A kernel test never reaches for a real word another crate owns; add a stand-in word instead. A test that breaks when a crate is cut was in the wrong home. Later, from fish / Deno / LLVM: behaviour tests as plain files (a line and its reply) with a bless mode — decided direction, not built.
