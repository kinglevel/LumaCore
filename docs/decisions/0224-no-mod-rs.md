---
id: 0224
title: Modules are foo.rs plus foo/, never mod.rs
area: structure
status: built
date: 2026-10-15
built_in: 34f1 (ratchet)
supersedes: []
superseded_by: —
confirmed_by: [modules_are_foo_rs_not_mod_rs]
source: [archive §34 L8006–8007 (item 10)]
---
# Modules are foo.rs plus foo/, never mod.rs

## Context
The workspace mixed both module styles, so a search for a module's name found a folder in one crate and a file in the next, and every `mod.rs` tab read the same.

## Decision
A module with children is `foo.rs` beside a folder `foo/`; `mod.rs` is never written. `modules_are_foo_rs_not_mod_rs` counts the `mod.rs` files left (the constant lives in tidy) and the count only goes down; when it reaches zero the clippy lint `mod_module_files` takes over and the ratchet goes.

## Consequences
A moved or new module uses the one style; a crate cut out of the station renames its `mod.rs` as it moves (see station-code-becomes-crates). Open: the last `mod.rs` files, then the lint.
