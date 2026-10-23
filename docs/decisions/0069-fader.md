---
id: 0069
title: The fader drags relative only, no tap jump, no hold
area: ui-kit
status: built
date: 2026-10-14
built_in: 12f part 3
supersedes: [archive §30 L3559–3561 "double-tap or hold keypad", archive §30 L5747–5748 "hold = master's sheet"]
superseded_by: —
confirmed_by: [e2e `kit-12f.spec.ts`]
source: [archive §30 L3557–3561, archive §30 L5747–5748, archive §30 L5766, archive §30 L5783–5784, archive §30 L5798–5799, archive §30 L6519–6528]
---
# The fader drags relative only, no tap jump, no hold

## Context
The first kit rules took the established ones: a tap never jumps, a drag is relative, double-tap or hold opens the keypad. The 12f artifact rounds dropped the hold (first "hold = the master's sheet", then none) and picked look B, the locked Elements board's filled bar, over the concept's groove and cap.

## Decision
`<luma-fader>` (`kit/fader.js`; the Showroom shows the grand group and your Highlight live) is look B: the filled bar, the cap line in the derived `--lc-cap` role, the value on top, name and address below; its travel is the `--lc-fader-height` token. It only drags, relative: a tap never jumps, no hold, no double-tap; each finger owns one fader; a second finger on it is fine control; it snaps close to the ends (the margin in fader.js); the bubble shows the line while dragging; moves go out at a throttled interval (in fader.js). Every move emits `gesture` (`id, line, phase, label, from, value`) sent by the page's `wsGesture` document listener (a window control that needs `Self` sends its own); see gesture-channel. A fader's settings come from its window (bar / settings), never from the fader. The encoder bar's Masters mode is a `<luma-fader>` bank.

## Consequences
A hand-made fader or a tap-to-jump control is refused by the kit rule; a fader that needs a keypad opens the slider panel from its window, not from a hold. The e2e checks that a real drag moves `Master 2.1` and files one "Faders" entry while a tap sends nothing, on desktop, tablet, phone and WebKit.
