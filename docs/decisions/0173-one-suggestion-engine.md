---
id: 0173
title: One suggestion engine; Enter runs the line as typed
area: suggest
status: built
date: 2026-10-12
built_in: 12b2
supersedes: [archive §30 L4254 "completer core in terminal/"]
superseded_by: —
confirmed_by: [`the_terminal_and_the_bar_show_the_same_suggestions`, e2e `suggest-parity.spec.ts`, e2e `kit-suggest.spec.ts`, `ShowSubtree::Usage`]
source: [archive §30 L4239–4282, archive §30 L4354–4368]
---
# One suggestion engine; Enter runs the line as typed

## Context
The F12 floating command line was to grow into a Raycast-like palette, and the user wanted the same suggestions at the bottom bar; 12b2 laid the foundations. The TUI's completer core moved down out of `terminal/`; usage data were to be "per user and go with the showfile so we can learn what users do".

## Decision
One engine, `crates/commands/lumacore-suggest`, serves the bar, the palette, the TUI, the sweep and HTTP: `suggest(line, cursor, surface)` returns the span to replace, the hint and typed items (`lumacore_wire::suggest`: kind, title, subtitle, icon, insert text, GUID, actions as command lines, Planned / no-rights state with its code, score). Ranking happens on the station: grammar first (only what is valid at the cursor), then the nucleo fuzzy score, then decayed usage, then a fixed tie-break; the client never ranks. Picking never runs code in the client: it writes the line or runs an item's command-line template. On the shell socket: `Suggest` / `Suggestions` with a sequence number dropping stale replies, `LineEdit` for a taken item; never a per-kind endpoint. The kit's `<luma-suggest>` opens above the bar (in full screen above the floating line); Tab or a tap takes the item, Right the ghost text, Esc closes, Enter runs the line as typed. The profile setting is `Suggestions Auto | TabOnly | Off`. Usage lives on `ShowData.Users.{n}.Usage` (per GUID or command template, decayed), saved, replicated, never in undo frames (`ShowSubtree::Usage`), recorded from dispatch on the master. Admin gets no history items.

## Consequences
A new surface asks the engine, never ranks or completes itself; a new item kind is a wire change. Open: global palette search, item actions on Enter / Meow+Enter, Tab-into pages, the preview pane.
