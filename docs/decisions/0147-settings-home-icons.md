---
id: 0147
title: Three homes, one icon per cell, gold where it differs
area: settings
status: built
date: 2026-10-20
built_in: —
supersedes: [archive §30 L4715–4718 "every cell says"]
superseded_by: —
confirmed_by: [kit-12d.spec.ts data-home]
source: [archive §39 L11491–11533]
---
# Three homes, one icon per cell, gold where it differs

## Context
The home mark repeated "In the show · every station" on every cell, cutting names short while the tab's header said it once. The user asked for the string to go; two mockup rounds (`docs/dev/ui-kit/mockups/2026-10-20-home-mark/`) picked icons on every cell with three homes.

## Decision
The property registry's `Home` (`lumacore_state::props::Home`) has three values: Show file — the show, every station (icon `save`); User — one user's, in the show file: a profile's preferences, a user's own password and rights, a window's settings, the screen props (`user`); Station — this station only, never in the show (`server`). `Props::home` / `home_of` give an object's home for all its properties; `SettingField.home`, `Tab.home`, a menu's `home` and a grid column's `home` carry the word; the rights gate and `Set` don't read it.

No words on cells. The kit's `homeMark(home, tabHome)` (`settings.js`, `editcard.js`) draws `<luma-icon class="lc-home" data-home>` grey at the right of every cell's label, gold (`.odd`) where the cell's home differs from its tab's (the Network menu's identity cells among the wiring, a mixed edit card); the tooltip (hover, hold on touch) spells it out — "Show file · every station", "Your profile · in the show", "This station · never in the show". The header keeps its one line, icon plus words (`homeLine`); the hosts pass the tab's home (`setup.js`, the edit card from its grid; the people element and the Import MVR dialog say `user`).

## Consequences
A new property declares its home; a new host of settings cells passes the tab's home. Never put the home words back on a cell. The e2e reads `data-home`; the Showroom's Kinds board shows the three. Not chosen: words gone with only the header, words kept on mixed tabs, the icon only on the odd ones out.
