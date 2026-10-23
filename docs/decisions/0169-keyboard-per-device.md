---
id: 0169
title: Built-in keyboard; the Keyboard mode lives on the device
area: devices
status: built
date: 2026-10-12
built_in: 12a3 (the keyboard, moved up from 12b); 12b (the setting and the layouts)
supersedes: [archive §30 L3916–3919 "Kiosk opens it at once"]
superseded_by: —
confirmed_by: [tidy `keyboard_words_are_registry_words`]
source: [archive §30 L3566–3571, archive §30 L3916–3919, archive §30 L4096–4102, archive §30 L4183–4190]
---
# Built-in keyboard; the Keyboard mode lives on the device

## Context
Console and kiosk screens have no OS keyboard ("we also need a vk for consoles"), phones and tablets have one that must not cover the command line, and every number wants the kit's keypad. The surface needed the keyboard on consoles, so it moved from 12b into 12a3; the setting and the layout files followed in 12b.

## Decision
The kit's `<luma-keyboard>` is the built-in on-screen keyboard (text and password modes; the password mode hides the word row and masks); it types into the xterm command line too, its word row coming from the registry. A per-screen setting `Keyboard: Auto | OS | Built-in | Off` is kept by the device (the app per screen window in `client.json`, a browser tab in `localStorage`), default Auto (built-in on console / kiosk stations and on touch screens without an OS keyboard, OS on phones and tablets); it stays with the device whoever logs in. The app's Kiosk toggle means "Keyboard: Built-in on every screen". The VK button on the command line bar: a tap shows / hides the chosen keyboard (focus for the OS one), a hold picks the mode. On phones the command line bar stays above the OS keyboard through `visualViewport`. Layouts are data files, `web/kit/layouts/*.json` (value, shifted and AltGr per key); English (US) is the one shipped.

## Consequences
A new layout is a JSON file, never code; a word on the keyboard must be a registry word (tidy `keyboard_words_are_registry_words`). The keyboard mode never goes on the user, the profile or the show. National layouts beyond English are open.
