---
id: 0192
title: The prompt shrinks on a narrow bar, fixed on the station
area: ui-kit
status: built
date: 2026-10-13
built_in: 12e part 1
supersedes: []
superseded_by: —
confirmed_by: [repl test the_prompt_shrinks_on_a_narrow_line]
source: [archive §30 L5406–5411, archive §30 L5588–5592]
---
# The prompt shrinks on a narrow bar, fixed on the station

## Context
On a phone the one-row command line showed nothing while the prompt was longer than the bar (found at 12a1). The TUI had two prompt builders and a "Terminal too small" guard that blanked an input-only view.

## Decision
One fix on the station serves the web bar and the TUI: `terminal/repl`'s one prompt builder (`fit_prompt` / `prompt_for`) shrinks the prompt in steps — `[user@host] scope> ` → `scope> ` → `‹end> ` → `> ` — the `>` never cut and a share of the line always left for typing; `input_offset` scrolls the line to keep the cursor a few columns from the right edge (readline's horizontal-scroll-mode); `window_spans` draws dim `‹` / `›` on clipped edges; a view of only the command line is never "Terminal too small".

## Consequences
Never a second prompt builder or a client-side prompt; a change to the prompt is tested with TestBackend at narrow widths in `crates/ui/lumacore-terminal/src/repl/tests.rs`. The phone bar's layout is multiplatform-contracts.
