---
id: 0284
title: The Device Sheet window
area: ui-kit
status: built
date: 2026-10-22
built_in: device sheet, round 1 (2026-10-22)
supersedes: []
superseded_by: —
confirmed_by: [lumacore-views devices::tests, tests/e2e/tests/kit-devices.spec.ts]
source: [session 2026-10-22, device sheet round 1 — docs/dev/ui-kit/mockups/2026-10-22-device-sheet/03-device-sheet.html, approved]
---
# The Device Sheet window

## Context
The programmer's values have to be seen before the programmer is built (ideas "Programmer and device sheet"). A reference console's fixture sheet was studied (settings, four modes, the layer toolbar). "Fixture Sheet", a flat list without folders, a follow-the-selection scope, one block per fixture type and a tap that opens the calculator were drawn and not taken.

## Decision
A window type "Device Sheet" (category Sheets) draws `<luma-device-grid>` (see device-grid-and-value-marks). Its settings, each a `Set Self "<Setting>"` line: Mode Fixture or Channel (Dimmer+ waits for a feature-group pick on the encoder bar), Layer Absolute, Fade, Delay, Output or DMX (Output and DMX read the rendered frame), Programmer only, Selected only, Selection first (on), Folders (on), Merge cells.

It opens on every device. A folder row shows the value every device below shares, a faint dot where they differ, and a bar when anything below is in your programmer. Columns are the union of the attributes in scope; the column sets are Condensed (each group's main attributes), Full and Extended.

A tap on a name selects (`Fixture 785`, again: `- Fixture 785`; a folder: `Fixture Media 7003`). A tap on a value selects its device; a second tap on the same cell opens the calculator and Enter runs `Fixture <n> Attribute "<a>" At <value>`. Hold is the one context menu. The miniature is the hybrid (12e round 5); a phone draws Channel mode.

## Consequences
Auto (follow the encoder bar's layer) and Dimmer+ are open. A new layer or mode is a setting choice on the window type and a field the feed carries.
