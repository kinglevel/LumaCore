---
id: 0183
title: The look: palette as tokens, Lucide icons, vendored fonts
area: ui-kit
status: built
date: 2026-10-12
built_in: 12a1
supersedes: []
superseded_by: —
confirmed_by: [tidy frontend_colours_only_in_the_theme, tidy frontend_icons_come_from_the_kit]
source: [archive §30 L3534–3549, archive §30 L3643–3646, archive §30 L3682–3687, archive §30 L3701–3717, archive §30 L3727–3732]
---
# The look: palette as tokens, Lucide icons, vendored fonts

## Context
The user found the first restyle "a little bit too much vibecoded" and asked for the original design from `inspiration/` (analysed in the UI-kit research, `inspiration-style.md`): matte, flat, neutral greys, olive gold as the one accent, proven console sizes kept. "Dont change the design too much."

## Decision
Colours exist only as tokens: CSS `--lc-bg`, `--lc-surface`, `--lc-raised`, `--lc-line`, `--lc-text`, `--lc-text-dim`, `--lc-accent`, `--lc-info`, `--lc-ok`, `--lc-warn`, `--lc-alert`, `--lc-programmer`, `--lc-blind`, `--lc-park`, with the derived `--lc-hover`, `--lc-pressed`, `--lc-alt-row`, `--lc-focus`, `--lc-on-accent`, `--lc-disabled`, `--lc-gap`; in the show `Theme.Palette.*` carries the same names. `kit/tokens.css` holds the presets (Dark, Daylight, Night; roles derived by `color-mix()`, sizes larger under a coarse pointer), `theme.js` applies them and the xterm theme. The inspiration palette is the Dark preset: no shadows, no gradients, small radii, large regular type, sentence-case labels; status colours only as small marks; every tab carries the gold dot (bright on the active tab) instead of a colour per window type — the dot is the window's menu button; the window bar looks like the inspiration's option row. Icons: one Lucide sprite of only the names in `crates/ui/lumacore-webui/web/vendor/lucide/icons.txt` plus own glyphs, built by `scripts/unix/lucide-sprite.mjs`, drawn by `<luma-icon>`; no emoji in chrome. Fonts: vendored Source Sans 3 for the UI and JetBrains Mono for the command line and terminals, so every platform and screenshot looks the same. Both pages carry `<meta name="darkreader-lock">`.

## Consequences
No raw hex outside the theme module, no icon from elsewhere, no raw font size (see multiplatform-contracts): tidy fails. The TUI names colours by `tui_look::role` from the same theme, never a raw colour. The background hue / saturation adjusters asked as 12g stay decided, not built.
