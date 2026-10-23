# lumacore-catalog-tests

The tests of the whole catalog: what holds for every row, whatever crate adds it.

## Purpose

Once the command crates are cut out of the kernel (decisions §34 25.13), the kernel's own tests
see only the kernel's rows. The checks whose subject is every row — one namespace, help examples
that parse, reading is anyone's, planned words, objects' verbs and properties, tree defs, the shim
ratchet — the lum.md examples and the Lum words golden record against every word, and the
generated reference live here, on the station's catalog (`lumacore_station::register`), so a cut
crate's rows are checked like the kernel's (§34 25.15). A crate's tests of its own logic stay
beside its code.

## Owns / Doesn't own

- **Owns:** the row rules (`rows/tests.rs`), the lum.md checks and `PENDING` (`lum/tests.rs`),
  the Lum words golden record (`testdata/lum_words.golden.txt`), the golden record of every
  reachable tree path (`paths/tests.rs`, `testdata/tree_paths.golden.txt`), the reference check,
  the rights table (`rights/tests.rs`), every menu's JSON (`menus/tests.rs`) and the behaviour
  that needs words from several command crates (`words/tests.rs`, decision 25.16).
- **Doesn't own:** the rows (each crate's `register`), the catalog's own checks at startup
  (`lumacore-registry` `freeze`), the reference generator (`gen-docs`).

## Public surface

None: a test crate (`publish = false`, dev-dependencies only).

## Rules

- The committed reference is what the catalog makes. [test the_reference_is_current]
- Every lum.md example parses with the catalog's words. [test every_lum_example_parses_with_the_registries]
- The handlers taking raw words only go down. [test the_shim_count_only_goes_down]
- A tree path change shows in the golden record. [test tree_paths_are_unchanged]

## Edges

- **Depends on:** nothing.
- **Dev-dependencies:** the station (its catalog), the kernel, the registry, the syntax crate,
  the tree, the views, the suggestion engine.
- **Used by:** nothing.

## Decisions

- [§33 3](../../docs/archive/decisions-2026-10.md) — lum.md is the language
- [§34 25.15](../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate

## Testing

- `cargo test -p lumacore-catalog-tests` — the golden record rewrites with `LUMACORE_BLESS=1`;
  read the diff.
