---
id: 0286
title: The theme editor; background hue and saturation replace Tint
area: ui-kit
status: built
date: 2026-10-22
built_in: the theme editor (2026-10-22)
supersedes: []
superseded_by: —
confirmed_by: [lumacore-state look_tests, tests/e2e/tests/kit-theme.spec.ts]
source: [archive §30 "12g" L4199–4212, session 2026-10-22 — docs/dev/ui-kit/mockups/2026-10-12-12g-test (ThemeBg, option A)]
---
# The theme editor; background hue and saturation replace Tint

## Context
theme-on-profile left the editor and two background adjusters decided, not built (12g). The user asked for a background hue and saturation; the ThemeBg test board compared them with Tint and the user chose "A tint replaced". The board was the starting point for the editor.

## Decision
`Theme` (`EditSetting [UserProfile n] Theme`) opens the theme editor (`kit/themeeditor.js`) on the profile's fields: the base, the six adjusters (Brightness, Contrast, Saturation, BackgroundHue 0–360°, BackgroundSaturation 0–100, AccentHue) with a contrast readout, the 14 palette colours, Pick on screen, A | B against the theme as opened, Import / Export as JSON, Reset all (`Reset … Theme`, `Reset … Theme Paint`), a preview and Hold: normal theme. A drag or a colour previews on the page; the release is one line, one Oops step. BackgroundSaturation blends Background, Surface, Raised and Line toward BackgroundHue at their own lightness (up to 55 %), before the other adjusters; text and status colours stay. `Theme.Tint` is gone with the show format bump, no migration.

## Consequences
The derivation stays twice, in `kit/theme.js` and `lumacore_state::look`, and `the_adjusters_match_theme_js` holds them equal; a new adjuster changes both and that test.
