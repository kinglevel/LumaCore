---
id: 0112
title: Handlers are derived Args structs bound from the right
area: commands
status: built
date: 2026-10-15
built_in: lum part 2 (the boundary), parts 3a–3d (families moved)
supersedes: [archive §31 L6803–6808 "no signature change"]
superseded_by: —
confirmed_by: [`compile_fail` tests of `lumacore-syntax-derive`, `Signature::check`, E-SYN-001, E-SYN-014, the ratchet `the_shim_count_only_goes_down`]
source: [archive §33 L6996–6999 (item 4), archive §33 L7116–7123 (item 17), archive §33 L7362–7376]
---
# Handlers are derived Args structs bound from the right

## Context
Handlers took `&[&str]` and re-read flags, ranges and values each their own way; the code review kept that signature, the lum discussion reversed it: one typed boundary, migrated family by family.

## Decision
A handler is a `#[derive(Args)]` struct (`lumacore-syntax-derive`, a proc-macro so `lumacore-syntax` keeps one dependency) implementing `commands::typed::Command` (`run(self, &mut AppState)`), registered as `run: typed::<C>` or `(Verb::X, typed::<C>)`. The struct is the signature: parse, completion, hints, help, the generated reference, rights and the sweep read it; a duplicate option or helper, or two optional plain slots that can't be told apart, is a compile error (`Signature::check`, tested as `compile_fail`). Static `CommandDef` / `ObjectDef` rows stay.

Binding (`lumacore_syntax::bind`): a `Signature` has a target (never / optional / required), plain slots (Property, Name, Value), helpers with `At` as Value or Destination — never both —, options, a phase and an If. Trailing plain slots bind from the right — the value last, the quoted property before it — and the ids take what is left, so a quoted id never collides with a quoted property (`Set UserProfile "LD" "ClearBuffer" On`); a verb without a target reads bare ids as its slots (`Redo 3`). A value is one expression (`21 Thru 30`, `* 2`, `"Rig"`). Every `Set` has a value (`Set User "LD" "Password" Ask`). Unknown helpers or options, a value `At` where a destination is declared, a bare word where a property is declared are E-SYN; `OnOff` / `Switch` / `u32` refuse with E-SYN-014.

## Consequences
New handlers are typed, always. Handlers still taking `&[&str]` are counted by `the_shim_count_only_goes_down` (the branch merges at zero); until dispatch hands over the parsed statement, `typed` writes the words back into a line and parses it. Handlers never see the raw line (see lum-grammar-one-parser).
