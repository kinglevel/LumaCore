---
id: 0190
title: The kit's multiplatform contracts
area: ui-kit
status: built
date: 2026-10-13
built_in: 12c2
supersedes: []
superseded_by: —
confirmed_by: [tidy frontend_font_sizes_are_tokens, tidy font_and_grid_scanners, e2e multiplatform.spec.ts, e2e kit-12c2.spec.ts]
source: [archive §30 L4601–4616, archive §30 L4664–4672]
---
# The kit's multiplatform contracts

## Context
12c2's audit of every window, menu, settings window and dialog on phone, tablet and WebKit found closes pushed off the screen, tiny targets, raw px fonts and layouts sized by the viewport. All findings became kit contracts in one part, with a stop-gap until 12d rebuilt the menus.

## Decision
Every overlay follows the kit's box contract `.lc-scrim` / `.lc-box` / `.lc-box-title` / `.lc-box-body` / `.lc-close`: a fixed title row whose × never leaves the screen (actions wrap or fold into `…` under the container width the kit sets), the body scrolls, the box never larger than the screen minus the kit's margin. Every interactive element meets the touch-target floor on coarse pointers (the minimum tidy and `support/fit.ts` check). Font sizes come only from `--lc-fs*` tokens, never a raw px. Safe-area insets go on every edge bar and fixed element. Layouts size by container queries, not the viewport. Tile grids are `<luma-tiles>` (Menu, Shutdown, Backup); toasts are `toast()`. Every hover or right-click action has a tap or hold equal (a DMX cell's info, a view button's menu); F11 / F12 are reachable by touch. The phone command line is PURR · line · VK · ⋯ with suggestions clamped to the screen.

## Consequences
A raw px font or grid size outside the kit fails tidy `frontend_font_sizes_are_tokens` (`tidy: px-ok` / `grid-ok` with a reason). `multiplatform.spec.ts` opens every window, menu, settings window and dialog on five screens and fails on small targets, off-screen content, sideways scroll or small text; new UI runs it before it is called done. The phone's own folds and the miniature desk are phone-desk-miniature.
