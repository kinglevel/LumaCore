---
id: 0187
title: Menus are MenuDefs drawn by the setup frame
area: menus
status: built
date: 2026-10-13
built_in: 12d part 3
supersedes: [archive §30 L3591–3594 "over existing SSE"]
superseded_by: —
confirmed_by: [GET /api/menu-types, List Menu / Info Menu, e2e kit-12d.spec.ts]
source: [archive §30 L3591–3594, archive §30 L4337–4339, archive §30 L4406–4408, archive §30 L4684–4721, archive §30 L4904–4907, archive §30 L5047–5068, archive §30 L5130–5142]
---
# Menus are MenuDefs drawn by the setup frame

## Context
12d took a classic setup layout (title with status, tabs left, content, function block, power toggle) for Network, DMX Protocols, LumaSessions, Backup and a new Settings menu. Part 3 built the registry and the frame, part 4 moved every menu onto it; the old overlays, their polling and click-again confirms went.

## Decision
A menu is a `MenuDef` row in `core::menu_defs` (a catalog row): tabs, or `left` tabs from a feed with the tabs as a top strip (DMX Protocols' networks); grid columns from the declared props; slab actions as command-line templates (`{object}` / `{rows}` / `{at}` / `{value}`, `{value}` asking through the dialog); a few bound `Set` cells; rights; a power toggle `{feed, path, invert, off_values}` in look A (the state word, what a press does; failing shows the reason). `GET /api/menu-types` publishes them. `Menu` is an object: `Menu "<name>" [/Tab x]` opens one, the same line closes it (never a stacked copy), `/Tab` is kept on the profile (Untracked), `List Menu` / `Info Menu` read them. `<luma-setup>` draws any menu: title, tabs, content, the last line naming the rows the lines act on, the slab; never larger than the screen, full screen on a phone (see phone-desk-miniature). Rows come from the feeds (`network`, `lumasessions`, `station`, `shows`, built by `lumacore_views::menu_rows`); a tapped row only fills the templates — nothing hidden on the server. Every other setting lives on the menu's settings tab, one home per setting. Dangerous lines are station asks.

## Consequences
A new setup menu is a `MenuDef` plus feed rows, never a page overlay; the Menu tile runs `Menu "x"`. Every list in a menu is `<luma-grid>` with `<luma-slab>` (see luma-grid, slab); a settings cell shows its home icon (see the settings-home record).
