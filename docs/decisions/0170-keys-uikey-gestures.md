---
id: 0170
title: Keyboard to UiKey to command line; core keys; Meow is Shift
area: keys
status: built
date: 2026-10-12
built_in: step 3 (one key system); 12a3 (core keys); 12b2 (Meow = Shift, F12 full screen)
supersedes: [archive §17 L1242–1245 "KeyBinding key→command map", archive §17 L1247–1249 "Meow default Option/Alt", archive §30 L3923–3948 "F12 hides sidebars only", archive §30 L3945–3946 "F1–F5 F9 F10 defaults"]
superseded_by: —
confirmed_by: [E-KEY-005, `keybindings::CORE_KEYS`, `ui_keys::CORE_MEOW`, e2e `kit-12b2.spec.ts`]
source: [archive §17 L1230–1257, archive §29 L3110–3111, archive §30 L3923–3948, archive §30 L3973–3975, archive §30 L4002–4004, archive §30 L4230–4252, archive §30 L4297–4305]
---
# Keyboard to UiKey to command line; core keys; Meow is Shift

## Context
Established consoles layer keys the same way: a shortcut names a key, the key fires lines. Step 3 replaced the key→command map with key→UiKey. 12a3 researched F keys after the user asked for F keys only; 12b2 picked Shift as Meow on every platform because Cmd and Ctrl are taken on macOS.

## Decision
A keyboard binding names only a UiKey id; command lines live only in UiKeys (`lumacore_tree::ui_keys`), one full line per gesture (Press, Release, Hold, ReleaseAfterHold, DoublePress) plus the Meow lines; never a command line in a `KeyBinding`. `Bind F5 "List Fixture"` makes a user UiKey `user.<key>` (GUID `Guid::named` from profile and key id) and binds at once; `lumacore-cmd-keys` checks lines on save and on show load (`Hook::ShowReplaced`). Every surface fires the same UiKeys; a key's LED reads the user's own mode state. Meow is Shift by default (`Set MeowKey` per profile; the sidebar button lights while held, latches on a tap). Core keys (`keybindings::CORE_KEYS`) never resolve from a profile and `Bind` refuses them (E-KEY-005); letters are never bound, F1–F8 stay free: F9 / Pause `DeskLock` · Meow+F9 `NormalTheme` · F10 shortcut mode · Meow+F10 `ViewBar` · F11 / Print Screen `Screenshot` · Meow+F11 the Shutdown menu · F12 full screen of the windows (`Sidebars`: every bar hidden, the kit corner handle brings them back, typing floats the command line in) · Meow+F12 native `FullScreen` of that window. LumaCore touches no OS setting (macOS gets one hint per device).

## Consequences
A new default key is a UiKey row, never a core key; the TUI fires Press and DoublePress only. Tidy fails a core UI key or TUI hotkey that doesn't resolve.
