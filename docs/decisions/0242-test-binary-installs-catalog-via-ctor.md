---
id: 0242
title: A test binary installs its catalog before its first test
area: catalog
status: built
date: 2026-10-16
built_in: 34f3b
supersedes: []
superseded_by: —
confirmed_by: [tidy `no_unsafe_code`, tidy `crates_on_the_registry_register_and_install`]
source: [archive §34 L8162–8172 (item 21.8), archive §34 L8652–8660]
---
# A test binary installs its catalog before its first test

## Context
Rust's test harness has no "before all tests" hook. A per-test install call that one test forgets passes or panics depending on test order. rustc and Bevy build a context per test, the logging crates use an idempotent `try_init`; `ctor` runs once per binary before `main`.

## Decision
Every crate on the registry has, in its `lib.rs`, a test-only hook: `#[cfg(test)] #[ctor::ctor] fn test_catalog() { lumacore_registry::install_with(crate::register) }`. It installs that crate's rows and everything below them, so the binary's tests read `lumacore_registry::get()` like the product. `ctor` is a dev-dependency only; the product keeps its explicit `install_catalog()` (see catalog-one-registry). Because ctor's expansion allows `unsafe`, the workspace lint `unsafe_code` is `deny`, not `forbid` (`Cargo.toml` `[workspace.lints]`); tidy `no_unsafe_code` refuses any `unsafe` or `allow(unsafe_code)` written in the source instead, and `crates_on_the_registry_register_and_install` checks that a crate on the registry has both its `register` and its hook, and that a crate on state or objects is on the registry.

## Consequences
A new crate that declares rows adds `register` and the hook or fails tidy. Nobody writes `unsafe` in the source; a test never installs the catalog by hand. A test binary whose subject is the whole catalog lives in `tests/catalog` instead (see catalog-tests-crate).
