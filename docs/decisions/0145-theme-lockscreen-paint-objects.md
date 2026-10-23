---
id: 0145
title: Theme, Paint, LockScreen are one-of-a-kind profile children
area: properties
status: built
date: 2026-10-15
built_in: 3c2b
supersedes: [archive §33 L7473–7478 "dotted Theme.AccentHue property"]
superseded_by: —
confirmed_by: [Verb::EditSetting, E-SYN-007]
source: [archive §33 L7479–7487, archive §33 L7505–7509, archive §33 L7846–7863]
---
# Theme, Paint, LockScreen are one-of-a-kind profile children

## Context
Theme and lock-screen settings were dotted property names (`"Theme.AccentHue"`, `"LockScreen.AutoLock"`), which read a following `-30` as a glued list operator (E-SYN-007) and gave no place to group settings before more arrived.

## Decision
A property is never nested inside its quotes — no dotted names. A group of settings is a one-of-a-kind child object, written owner first with no grammar change: `Theme` (`"Base"` Dark / Daylight / Night, brightness, contrast, saturation, tint, accent hue), its child `Paint` (the screen colours, see color-light-paint-screen; none = the base's) and `LockScreen` (`"Image"` — a number or name of the profile's own Images stored as its GUID —, `"ShowClock"`, `"Text"`, `"AutoLock"`). `Set UserProfile Theme "AccentHue" -30`, `Set UserProfile "LD" LockScreen "AutoLock" 10`, `Set UserProfile Theme Paint "Accent" "#9a7a08"`. The row is the `UserProfile <x>` in front (`Props::within`), else your own; the fields live in the profile's JSON (`Props::view`). Typed alone each runs its default verb `EditSetting` (`Theme` = `EditSetting UserProfile Theme`; typed `EditTheme` / `EditPaint` / `EditLockScreen`); `EditSetting UserProfile [x]` shows the children's cells, each line owner first; `Reset [UserProfile x] Theme [Paint "Accent"]` puts the declared defaults back. The property is `"Base"`, never `"Preset"` (the lighting word); the bases keep their names (cat-coat names were declined). Theme, LockScreen and Paint are words of the one namespace.

## Consequences
Never add a dotted property. A new group of settings is a new one-of-a-kind child object, hardened on its own (verbs, default verb, rights, what it holds). The `"Theme"` / `"LockScreen"` reset words are gone. The Users & Profiles window leaves Paint to the theme editor.
