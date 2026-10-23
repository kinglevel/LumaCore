---
id: 0193
title: One hold, one owner, the profile's HoldTime
area: ui-kit
status: built
date: 2026-10-14
built_in: 12e part 1
supersedes: [archive §30 L5438–5441 "hairline hit band", archive §30 L3553–3557 "hold show me"]
superseded_by: —
confirmed_by: [tidy frontend_gestures_only_in_the_kit, e2e kit.spec.ts (split), e2e kit-12e.spec.ts]
source: [archive §30 L3550–3563, archive §30 L3647–3648, archive §30 L5412–5420, archive §30 L5438–5441, archive §30 L5469–5471, archive §30 L5514–5515, archive §30 L5593–5605, archive §30 L5684–5686]
---
# One hold, one owner, the profile's HoldTime

## Context
Hold has one meaning everywhere since the touch rules: a long press with a progress ring and small slop, right-click staying the mouse's. Then a tab inside a miniature pane opened both the full screen and the window menu, and minipane, the grid's lasso, the UI keys and the pools each ran their own timer.

## Decision
Every hold — pane, tab, header, pool cell, key, grid lasso, row — runs through `kit/pointer.js` with the profile's `HoldTime` (slop and ring delay are the pointer core's). `claim(e)` marks a press owned: the innermost target big enough to tap owns it (12c2's size rule), a press arms one hold, never two; inside a miniature only a `.lc-tap-ok` target keeps its own hold; `hold()` also ends on `lostpointercapture`; a hold swallows the next click, a right-click does not. The tab dot is the window's header button: tap = the window menu, hold on a tab or header the same. The split handle is a kit element (`kit/split.js`): a mouse grabs a thin band along the line, a finger only the grip pill — the hairline takes no touch; double-tap = 50 / 50, `role="separator"` with arrow keys; a drag, reset or key is one layout step; panes keep the kit's minimum.

## Consequences
No hand-made hold timer, no HTML5 drag, no window- or document-level pointer listener outside `kit/` — tidy `frontend_gestures_only_in_the_kit` (`tidy: hold-ok (<reason>)` for the UI keys' own Press / Release / Hold gesture). The pool's hold is the context menu now (see one-context-menu-from-the-registry); faders take no hold or double-tap. Drags are pointer-core-one-lift-gesture; the miniature's hold is phone-desk-miniature.
