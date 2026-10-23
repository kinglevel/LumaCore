---
id: 0182
title: Typed descriptors from the registries; every gesture a command line
area: ui-kit
status: built
date: 2026-10-13
built_in: 12b2
supersedes: []
superseded_by: —
confirmed_by: [CLIENT_PROTOCOL bump on every wire change, e2e kit-12b2.spec.ts, e2e kit-12c.spec.ts]
source: [archive §30 L3527–3533, archive §30 L3564–3565, archive §30 L4085–4095, archive §30 L4165–4169, archive §30 L4312–4327, archive §30 L4385–4395, archive §30 L4514–4515, archive §30 L4553, archive §30 L5452–5455, archive §30 L5606–5611]
---
# Typed descriptors from the registries; every gesture a command line

## Context
The station already declares what a UI shows — an object's props, a window type's settings, a pool's definition, a dialog — so the page must not keep a second description. 12b built the editors, 12b2 the typed wire shape, 12c the window settings.

## Decision
The server sends typed descriptors built from what declares them: kind, id (GUID or path), label, value, range / steps / choices, unit, rights (disabled, never hidden, below the level) and gestures as command lines. The editor follows the kind: choice → pick list, bool → toggle, number → keypad with the old value preselected and a unit swipe only when the descriptor has units, link → the pick-list pop-up (filter, None for optional links, Add New stores at the next free number, a broken GUID shown as missing), text → text input, colour → the colour dialog. Settings apply at once: each change is one `Set` line and one Oops step, no OK / Cancel. The wire shape is `lumacore_wire::FieldInput` with `SettingField.section` / `hint` / `pick`; links are `PropKind::Link` / `OwnLink` naming their pool; named choices replace magic numbers (DataPool `"selected"` or a GUID, Fit). Every `EditSetting` draws in `<luma-settings>`. Plugins publish descriptors, not DOM code. No client-only state.

The one recorded exception: a window's scroll position goes as the quiet `scroll` packet on `/ws/shell` (`ShellClientPacket::Scroll`) — station-local, no echo, no Oops step, not replicated.

## Consequences
A new setting is declared once on its object; never a handler-local editor or a page-side kind table. A wire change bumps `CLIENT_PROTOCOL`. Bare `Select DataPool|Sequence|Timecode` and `Store User "x" /Profile` ask through the one dialog with a link input (see one-server-held-dialog).
