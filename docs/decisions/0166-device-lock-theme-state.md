---
id: 0166
title: A device is one app install or browser tab
area: devices
status: built
date: 2026-10-12
built_in: 12a3
supersedes: [archive §30 L3653–3657 "this station's screens", archive §30 L3658–3666 "every screen of the station"]
superseded_by: —
confirmed_by: [`Hello.device`, `Status.device`, `lumacore-cmd-devices` tests]
source: [archive §30 L3653–3657, archive §30 L3893–3897, archive §30 L3920–3922, archive §30 L3986–3999, archive §30 L4005–4006]
---
# A device is one app install or browser tab

## Context
12a hardening scoped the lock and the normal-theme override to "this station's screens". 12a3 found that a station serves many clients at once, each needing its own lock and look, and that a reload or app restart had to come back locked; so the unit became the device.

## Decision
A device is one app install (its id kept in `client.json`, all its windows one device) or one browser tab (`sessionStorage`); the CLI is not a device. Every connection names its device: the id rides on the shell hello (`Hello.device`, `?device=`). The station keeps per device `Locked` and `NormalTheme` (`AppState::devices`, station-local, never saved, never replicated) and tells each client its own state in `Status.device`. `NormalTheme` is a mode-style word (`On` / `Off` / bare toggle; a held key = while held) that shows the factory preset and nothing of the profile; `Set Station ThemeOverride Off|Dark|Daylight|Night` is the station-local default for every device of the station and never writes a user's theme (see theme-on-profile). The page actions a device does itself (`Sidebars`, `ViewBar`, `FullScreen`) are Client-scope objects sent as a `UiEvent::Page` to that device. The lock on a device is desklock's rule.

## Consequences
A new per-client state goes beside `locked` and `normal_theme` on the device, never on the user, the profile or the show; a new client transport must carry the device id in its hello. A change here is a wire change (bump `CLIENT_PROTOCOL`, no shims). The lock words and the unlock rules are in desklock; the surface a locked device shows is in one-surface-login-lock-connect.
