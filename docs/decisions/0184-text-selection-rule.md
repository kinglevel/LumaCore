---
id: 0184
title: Text is selectable only where you read or type
area: ui-kit
status: built
date: 2026-10-13
built_in: 12b2
supersedes: []
superseded_by: —
confirmed_by: [tidy text_selection_only_in_the_kit_rule]
source: [archive §30 L4308–4311, archive §30 L4381–4384]
---
# Text is selectable only where you read or type

## Context
A hold on iOS brought up the callout and magnifier, and a mouse drag selected chrome text. Partial rules were scattered over the page; 12b2 replaced them with one kit rule and 12c's hardening restated it as the contract.

## Decision
Nothing is selectable app-wide: `html` / `body` are `user-select: none`, with the `-webkit-` prefixes (the app's WebKit needs them) and no touch callout. Text is selectable only in text inputs, `[contenteditable]`, the command line and terminals (`.xterm`), and on elements marked `.lc-selectable` — dialog text, System Monitor rows, help / info text and a settings window's last line. The rule lives once, in `kit/kit.css`.

## Consequences
A new surface whose text must be copyable gets `.lc-selectable`, never its own `user-select`; `user-select` anywhere outside `kit.css` fails tidy `text_selection_only_in_the_kit_rule`. The virtual list keeps its visible rows selectable while the rest of the page stays inert.
