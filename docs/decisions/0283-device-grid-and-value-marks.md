---
id: 0283
title: The device grid and one set of value marks
area: ui-kit
status: built
date: 2026-10-22
built_in: device sheet, round 1 (2026-10-22)
supersedes: []
superseded_by: —
confirmed_by: [lumacore-views devices::tests, tests/e2e/tests/kit-devices.spec.ts]
source: [session 2026-10-22, device sheet round 1 — docs/dev/ui-kit/mockups/2026-10-22-device-sheet/02-device-grid.html, approved]
---
# The device grid and one set of value marks

## Context
The device sheet is the first window that shows values; the programmer and the encoder bar show the same ones. Established consoles paint them one way everywhere: text colour = the source, background = what happens to it, thin bars = which layers hold data. A value column in the plain grid and a separate element were drawn and not taken.

## Decision
`<luma-device-grid>` is a kit element that extends `<luma-grid>` (`kit/devicegrid.js`, the grid's class exported with a cell hook and a header hook): rows are devices with the patch tree's folders as tree rows, columns are the attributes of the types in scope under feature-group headers, a cell the type lacks is black. It keeps the grid's selection, keys, context menu, virtual scroll, column sets and miniature; every gesture is a line (`Fixture 785`, `- Fixture 785`, `Fixture 785 Attribute "Pan" At -45`).

The value marks are one kit module (`kit/marks.js`) and one set of theme tokens (`--lc-v-*`), with the meanings in the kit's paint: the programmer's rose for an active value (white text on it) or an inactive one (rose text), cyan new, green lower, magenta tracked, white blocked, yellow / dim yellow another sequence, a dark-blue fading ground, the kit's teal for park; bars for absolute, relative, fade, delay, phaser, preset and park, white left for deactivated data. The grid, the encoder bar and the programmer draw them from the module.

The values reach a page on the `devices` feed, built by `lumacore-views`, sampled per subscription for the rows a window shows, read off the render tick.

## Consequences
A new value state is a token and a field in the feed, never a colour in a window. States the engine does not hold yet (relative, phaser, recipe, DMX tester, parts, new / tracked / blocked) keep their marks reserved.
