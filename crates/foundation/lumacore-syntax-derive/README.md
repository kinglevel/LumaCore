# lumacore-syntax-derive

`#[derive(Args)]`: a verb's typed arguments are its signature, written as a Rust struct.

## Purpose

A handler on the station takes a struct, never the raw words (decisions §33 decision 4). This
proc-macro crate reads that struct — `#[lum(target)]`, `#[lum(property)]`, `#[lum(value)]`,
`#[lum(helper = "Fade")]`, `#[lum(at = "destination")]`, `#[lum(option = "Overwrite")]`,
`#[lum(phase)]`, `#[lum(condition)]` — and writes the `lumacore_syntax::Args` impl: the
`SIGNATURE` const and `from_bound`. It also emits a `const` that runs `Signature::check` when
the crate compiles, so a helper or option declared twice, or two optional plain slots that can't
be told apart, is a compile error, not a runtime surprise. It is kept out of `lumacore-syntax`
so the language crate stays free of `syn`; `syn` here reads Rust, not Lum.

## Owns / Doesn't own

- **Owns:** the `Args` derive, the `lum` attribute's grammar, the expansion to `Signature` /
  `from_bound`, the compile-time check.
- **Doesn't own:** `Args`, `Signature`, `Bound` and the binder (`lumacore-syntax`), the
  handlers that derive it (`lumacore-objects`, the `cmd-` crates), the registry rows that name
  them (`run: typed::<C>`).

## Public surface

- `#[derive(Args)]` with `#[lum(verb = "…")]` on the struct and one `#[lum(…)]` per field.
- Field types: `Target`, `Value`, `String`, `Phase`, `Condition`, `bool` for a switch, and
  `Option<_>` where the slot may be left out.

## Rules

- The struct is the signature; what it declares is what binds, and a helper or option declared
  twice is a compile error.
  [test the_struct_is_the_signature + test values_options_and_helpers_are_checked]
- Trailing slots bind from the right. [test trailing_slots_bind_from_the_right]
- Bare ids fill a verb without a target. [test bare_ids_fill_a_verb_without_a_target]
- A hold verb binds its phase. [test a_hold_verb_binds_its_phase]
- Handlers still taking `&[&str]` only go down. [test the_shim_count_only_goes_down]
- It depends on no workspace crate; `lumacore-syntax` is a dev-dependency for its tests only.
  [tidy crate_edges_point_down]

## Edges

- **Depends on:** nothing.
- **Used by:** lumacore-objects, lumacore-cmd-station, lumacore-cmd-tags, lumacore-cmd-protocols,
  lumacore-cmd-sacn, lumacore-cmd-artnet, lumacore-cmd-osc.

## Decisions

- `docs/decisions/0112-typed-handlers-derive-args.md` — handlers are `#[derive(Args)]` structs; trailing
  slots bind right
- `docs/decisions/0101-lum-grammar-one-parser.md` — why the macro sits beside the language crate, not in it
- `docs/decisions/0122-at-verb-on-target.md` — `#[lum(at)]` as a value or a destination
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md) §33

## Testing

- `cargo test -p lumacore-syntax-derive` — `tests/derive.rs` derives structs against
  `lumacore-syntax`'s binder; the two `compile_fail` doctests in `src/lib.rs` show the refused
  shapes.
- The shim ratchet runs with the catalog: `cargo test -p lumacore-catalog-tests`.
