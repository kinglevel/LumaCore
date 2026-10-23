---
id: 0031
title: Replies typed inside, rendered to text at the edge
area: dispatch
status: decided
date: 2026-10-13
built_in: —
supersedes: [archive §23 L2012–2013 "Command output stays text" (amended)]
superseded_by: —
confirmed_by: none
source: [archive §31 L6776–6782]
---
# Replies typed inside, rendered to text at the edge

## Context
Step 8 decided command output stays text, the same for every client, like command-line feedback on established consoles. The code review found the station parsing its own text (`journal_result` looking for `Error:` and `[E-…]`) and amended the rule: the text is what clients see, not what the station reads.

## Decision
What reaches every client stays the same text. Inside, a handler's reply carries ok / error and its code; the text is rendered at the transport. Dispatch, the journal, `set_rows`, sysmon and the TUI read the type, never the text (`journal_result` stops parsing). Built step by step: first a tidy check that every code literal is in a catalog, then the reply type — `DispatchResult::Output` carrying `Reply { text, error }`. Issue 03's code placement applies to the rendered text. Until then every error reply is rendered through the one door (`errors::render` / `render_code` / `reply`; never `text::render_error` outside `core::errors`).

## Consequences
**Not built.** It is ordered after the snapshot work (see show-snapshot-arc-parts). A new handler must not parse reply text to learn success; it returns the error through `errors` so the switch to the typed reply is mechanical. Today's checks: tidy `error_replies_carry_a_code` and the sweep's report of an uncoded `Error:`.
