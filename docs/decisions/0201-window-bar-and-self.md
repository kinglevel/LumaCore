---
id: 0201
title: Window bar items are lines; Self is the line's owner
area: windows
status: built
date: 2026-10-13
built_in: 12c
supersedes: [archive §30 L3586 "its window or the key"]
superseded_by: —
confirmed_by: [E-CMD-016, window_bar tests]
source: [archive §30 L3575–3590, archive §30 L4461–4479, archive §30 L4492–4498, archive §30 L4532–4542, archive §30 L4551–4559]
---
# Window bar items are lines; Self is the line's owner

## Context
Before 12c the sysmon and DMX windows had hand-built toolbars writing window settings through the layout `PUT`. The 12c hardening (2026-10-13) replaced them with one window bar and one owner word, `Self`; an earlier sketch's "its window or the key" for a UI key was dropped.

## Decision
A window's bar sits inside the window below its tabs, drawn by the kit's `<luma-window-bar>` from `/api/window-types` and the `windows` State feed. Its items are pinned typed settings and the type's declared actions; `WindowBarItems` is their ordered id list (one id space; an unknown id is refused on `Set`, dropped on load). Every item is a `Set` or command line — DMX universe ± is `Set Self Universe + 1`; `Set Window` / `Set Self` take relative numbers, `Default`, `Toggle`, `Now`. Overflow goes into a `…` menu; below the type's minimum height the bar hides and the window menu lists its items.

`Self` is one registry object: the line's owner. Typed, it is the focused window; a window-bar, window-menu or settings-window line names that window; a UI key names the window it sits in, else the focused window, never the key. No owner is E-CMD-016, never a guess. It is resolved on the station where the line starts, before a forward, so journal, Oops, sysmon and the master see `Window <screen>.<n>`; keys, macros and Usage keep the `Self` text. Bar states (`state: "Self.<Property>"`), `shown_when` chips and live titles are `Self` reads the station resolves per window.

## Consequences
A new window action is a declared line, never a page handler; a window setting is written only by a line (the layout `PUT` carries splits, resizes and tabs only; see window-types-rust-json). Macro and plugin owners answer E-CMD-016 until their runtimes exist.
