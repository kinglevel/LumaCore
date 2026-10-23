---
paths:
  - crates/commands/objects/lumacore-cmd-keys/**
  - crates/model/lumacore-tree/src/ui_keys*
  - crates/ui/lumacore-webui/web/kit/layouts/**
---

# Keys

Records: `keys-uikey-gestures`, `keyboard-per-device`, `desklock`.

- Keyboard binding → UiKey id → command line per gesture (Press, Release, Hold, ReleaseAfterHold, DoublePress) plus Meow lines. Command lines live only in UiKeys (`lumacore_tree::ui_keys`); never put a command line in a `KeyBinding`. `lumacore-cmd-keys` binds (`Bind` makes `user.<key>`, its GUID derived from the profile and the key id), checks lines on save and on show load (`Hook::ShowReplaced`); a stored key line with a bare number or a `/Watch` is refused (E-SYN-008).
- Core keys (`keybindings::CORE_KEYS`) can't be bound (E-KEY-005); never letters; F1–F8 stay free: F9 / Pause DeskLock · Meow+F9 NormalTheme · F10 shortcuts · Meow+F10 ViewBar · F11 / PrintScreen Screenshot · Meow+F11 Shutdown menu · F12 full screen of the windows · Meow+F12 native FullScreen. Meow = Shift by default (`Set MeowKey`; the sidebar button lights while held).
- The built-in keyboard's layouts are data files (`web/kit/layouts/*.json`: value, shifted, AltGr per key); the `Keyboard` mode lives on the device per screen. Key lines are checked, never trusted: a line that names a word the registry lacks fails the save with its code.
