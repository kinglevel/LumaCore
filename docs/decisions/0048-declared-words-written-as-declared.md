---
id: 0048
title: Declared values and words run in their declared form
area: syntax
status: decided
date: 2026-10-16
built_in: —
supersedes: [archive §34 L9145–9160 "lowercase protocol names"]
superseded_by: —
confirmed_by: [E-CMD-008, E-SYN-014]
source: [archive §34 L9145–9170, archive §34 L9218–9222]
---
# Declared values and words run in their declared form

## Context
34f7b first named the protocol rows lowercase (`sacn`, `artnet`, `osc`) and required that exact spelling. Asked further as a syntax decision (lum.md updated), the rule turned around: a user types what is convenient, the station writes out what is declared.

## Decision
A declared value — a choice, a protocol — is typed bare in any case; lowercase always works; the line that runs, the journal, history, stored lines and the screens show its declared form (`htp` → `HTP`); any other spelling is refused with a hint (E-CMD-008 / E-SYN-014). Quoted is always text and keeps its case: a quoted choice (`"high"`) is refused ("type it without quotes"). Two-word choices keep their names (`UseMine`, `LowTP`). Protocols and properties are declared in CamelCase — `ArtNet`, `Sacn`, `Osc`, `MergeMode` — with no aliases (`Art-Net` gets a hint). Every declared word is written out as declared: verbs, objects, options, property names inside their quotes and declared values, in the line that runs, is forwarded, journalled, kept in history and stored (`set universe 1 "mergemode" htp /noconfirmation` → `Set Universe 1 "MergeMode" HTP /NoConfirmation`); free text and tree-path node names keep what was typed.

## Consequences
The values half is built with 34f7b: every choice comes from the rows, quoted is E-SYN-014. The words half is decided, not built: it comes with the lum syntax's printer (archive §33 part 3); until then a line is journalled as typed, the `Mode` words are spelled `output` / `input` in the vocabulary and the `Art-Net` hint is open. Never add a per-handler case table or alias list; the vocabulary is built from the registries (see lum-syntax-is-the-language).
