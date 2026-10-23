---
id: 0101
title: lum.md is the binding spec, one total parser
area: syntax
status: built
date: 2026-10-15
built_in: lum part 1 (parser and grammar tests), parts 2–3 (binding, handlers)
supersedes: [archive §14 L1128–1131 "Abbreviation System", archive §15 L1159–1166 "prefix tie rules", archive §30 L4268 "Enter runs typed", archive §33 L7952 "issue 02 Please"]
superseded_by: —
confirmed_by: [the grammar-listing test against lum.md's `grammar` block, every ```lum / ```lum-error example a test (tests/catalog lum/tests.rs, `PENDING`), the E-SYN catalog, fuzz target `syntax_parse`, tidy `the_shim_count_only_goes_down`]
source: [archive §33 L6976–7007 (items 1, 2, 3, 5), archive §33 L7226–7239 (items 33, 34), archive §33 L7343–7346, archive §33 L7943–7953]
---
# lum.md is the binding spec, one total parser

## Context
The 2026-10-14 code review found the command language read many ways: five range parsers, about seven `Set` routes, three relative-value parsers, about ten bool parsers, handlers matching raw tokens. Established consoles are loose inspiration, not the rule: their irregular parts are not copied. A parser-generator dependency was declined.

## Decision
`docs/user/syntax/lum.md` is the language: its fenced `grammar` block is the spec; the generated reference lists the words, lum.md says how they combine. Any change to the syntax or to lum.md is a hard discussion of its own, decided first, then lum.md, then the code — never on the side of another task.

One hand-written crate, `lumacore-syntax` (its one dependency `lumacore-base`, for `ErrorInfo`), holds a lexer and a recursive-descent parser driven by a vocabulary built from the registries at run time. It is total: never fails, never panics, records coded diagnostics with spans and what it would accept at each point (completion reads that; no second grammar). Entry points: `parse`, `at_cursor`, `highlight`, `print` (the canonical line for forwarding, keys, macros, the journal), `grammar_listing`. Handlers never see the raw line (see typed-handlers-derive-args).

The parser owns `E-SYN-…`: everything refused before a handler runs carries one, caret under the word (lum.md's Errors section is the list). Errors that need the show keep their area codes.

## Consequences
A test fails when the printed grammar differs from lum.md's block; every example must parse or fail with its code; lines whose words are not built sit in `PENDING`, which only shrinks. Main never has two parsers: the branch merges when `the_shim_count_only_goes_down` reaches zero. Not built: parts 4–5 (every stored line rewritten, a forward-only format bump; the old lexer, `parse_selection`, `full_words` and the `tokens::unquote` copies gone). E-CMD-005 / 006 / 018 retire into E-SYN; E-CMD-006 is still in use.
