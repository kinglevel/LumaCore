---
id: 0203
title: WindowPreset on the profile; layered resets; UI JSON export
area: windows
status: built
date: 2026-10-13
built_in: 12c
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §30 L3572–3574, archive §30 L4480–4491, archive §30 L4510–4513, archive §30 L4543–4550]
---
# WindowPreset on the profile; layered resets; UI JSON export

## Context
12a asked for a window-settings shell that can load and save a window's settings for other windows of the same type. The 12c hardening (2026-10-13) named and placed it, and added the user's demands: "a careful system to reset to defaults if a user deletes or modifies too much" and "anything should be able to be exported and imported as json".

## Decision
A saved window setup is a `WindowPreset` — never "Preset", the lighting word — named and numbered per window type on the `UserProfile` (`UserProfile.window_presets`). `Store WindowPreset "Big DMX"` saves `Self`'s window; `WindowPreset 2` loads into `Self`; the settings window's Load / Save buttons run these lines, a bare Store or call asks through the one dialog. A preset is a copy, not a live link: the window keeps `Window.preset` only to preselect it. It holds everything — settings, bar, label, colour, scroll. It travels with `Export UserProfile`.

Resets are layered: one setting (↺ on the cell, `Set Self <setting> Default`), one window (`Reset Self` / `Reset Window`: type defaults, bar items included), the bar (Load defaults on the Window bar tab), a profile's windows of one type (`Reset UserProfile "x" Window /Type t`). Each is one Oops step and asks first when it touches more than one window. `Verb::Reset` is an object verb; a bare `Reset` stays the programmer reset.

View, ScreenConfig and WindowPreset export and import as JSON into `library/ui/<kind>/` through `core::drive`; an import lands in your own profile with new GUIDs on a clash.

## Consequences
A preset or reset is a line on the station, never a layout-side write (see window-bar-and-self). A generic export for every other object kind is roadmap. The profile field is show data, so it moved the show format forward.
