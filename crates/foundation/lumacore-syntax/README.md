# lumacore-syntax

Lum, the command language: one lexer, one total parser, the cursor, the printer and the binder,
all driven by a vocabulary the station builds from its registries.

## Purpose

The grammar is `docs/user/syntax/lum.md` (decisions §33): `GRAMMAR` is its `grammar` block; a
test fails when they differ. The words are not fixed here — the station fills a `Vocabulary`
from its registries and every function reads it, so there is no second word list and no second
grammar for completion. `parse` is total: what it can't accept becomes a coded `Diag` with a span,
so highlighting and completion work on half-typed lines. `print` writes the one canonical spelling that is forwarded, stored in keys and macros and
journalled; `bind` fills a verb's typed arguments from a statement.

## Owns / Doesn't own

- **Owns:** `lex` (`Token`, `TokenKind`), `parse` (`Line`, `Statement`, `Origin`), the AST
  (`ast`), `at_cursor` (`Cursor`, `Expect`), `highlight` (`Class`), `print`, `bind` (`Args`,
  `Signature`, `Bound`, `CommandLine`, `FenceLine`, `OnOff`, `Switch`), the vocabulary types
  (`Vocabulary`, `VerbDef`, `ObjectDef`, `OptionDef`, `FenceDef`, `WordDef`), `GRAMMAR`,
  `KEYWORDS`, `Span`, the `E-SYN` codes (`ERRORS`, `codes`).
- **Doesn't own:** the words themselves (`lumacore_objects::lum` builds them), what a statement
  does (dispatch), the `#[derive(Args)]` macro (`lumacore-syntax-derive`), the examples (lum.md).

## Public surface

- `parse(line, &vocab) -> Line` — statements, diags, what each token expects.
- `at_cursor(line, pos, &vocab) -> Cursor` — completion's one source.
- `highlight`, `print` — classes per token; the canonical line.
- `bind::<A: Args>(&Statement)` and `Signature::check` — the typed boundary.
- `Vocabulary` and its `*Def` rows, `WordId`, `is_word`.
- `register(&mut Registry)` — this crate's error catalog.

## Rules

- `GRAMMAR` is lum.md's grammar block, word for word. [test the_grammar_is_lum_md]
- Every ` ```lum ` example in lum.md parses and prints back to one spelling, and every error
  example fails with its code. [test every_lum_example_parses_and_prints_back + test every_error_example_fails_with_its_code]
- The parser is total: any text parses without a panic and the tokens cover the line in order.
  [test any_text_parses_without_panicking + test tokens_cover_the_line_in_order]
- Completion reads what the parser expects; there is no second grammar.
  [test the_cursor_expects_what_comes_next]
- Ids follow the one range rule; a glued sign is the number, a spaced one is relative.
  [test ids_follow_the_one_range_rule + test a_glued_sign_is_the_number_a_spaced_one_is_relative]
- Owner-first paths nest; a child-first one is refused.
  [test an_owner_first_path_nests_and_a_child_first_one_is_refused]
- A quoted word is text, never a keyword. [test a_quoted_word_is_text_never_a_keyword]
- The station's vocabulary is the golden record of its words. [test the_words_are_the_golden_record]
- No parser library; it depends on the registry and base only. [tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-registry, lumacore-base.
- **Used by:** lumacore-state, lumacore-objects, lumacore-terminal, the `cmd-` crates, catalog, fuzz.

## Decisions

- `docs/decisions/0101-lum-grammar-one-parser.md` — lum.md binds; one hand-written total parser
- `docs/decisions/0104-ids-operators-case-lexing.md`, `docs/decisions/0103-owner-first-syntax.md` — tokens, nesting
- `docs/decisions/0112-typed-handlers-derive-args.md` — the binder
- `docs/decisions/0010-selection-ordered-one-range-rule.md` — the one range rule
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md) §33, §15

## Testing

- `cargo test -p lumacore-syntax` — `tests/lum_md.rs` (the grammar and every example),
  `tests/parser.rs`, `tests/properties.rs` (proptest), the module tests in `src/`.
- With the station's words: `cargo test -p lumacore-catalog-tests` (`tests/catalog/src/lum`).
- Fuzzing (long run only): `tests/fuzz/fuzz_targets/syntax_parse.rs`.
