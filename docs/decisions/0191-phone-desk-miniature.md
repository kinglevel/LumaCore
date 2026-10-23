---
id: 0191
title: Same view everywhere: miniature panes, hold full screen, the phone rule
area: ui-kit
status: built
date: 2026-10-19
built_in: 12e part 4, 37
supersedes: [archive §30 L4631 "hold 0.4 s", archive §30 L5448–5451 "at 600 px"]
superseded_by: —
confirmed_by: [e2e kit-12c2.spec.ts, e2e kit-12e.spec.ts, e2e phone.spec.ts, e2e audit-phone.spec.ts]
source: [archive §30 L4617–4639, archive §30 L4659–4663, archive §30 L5448–5451, archive §30 L5472–5475, archive §30 L5494–5500, archive §30 L5516–5518, archive §30 L5531–5549, archive §30 L5552, archive §30 L5690–5713, archive §37 L11186–11319]
---
# Same view everywhere: miniature panes, hold full screen, the phone rule

## Context
Options A–E for a desk view on a phone (stack, reflow, fold, per-device config, scale) were all not taken: the user wants the same view with hold-to-full-screen. The phone audit then found the setup frame starved on a phone and decided seven fixes.

## Decision
A phone or tablet shows the desk's screen with every pane in place and proportion; live screens stay shared, nothing is local. A pane under its type's `bar_min` draws in miniature (kit `miniature()`): anything under the touch floor is not tappable, a tap says "hold to open". A hold (the profile's HoldTime, see one-hold-one-owner) takes that window full screen with `<luma-maxbar>` ‹ › ×. The profile's `HoldFollows` (default on) mirrors it as `Set Self Maximized`. A miniature list is the whole list drawn small (`<luma-grid view="mini">`): the window type declares its mini columns and each column its renderer — never a scaled copy of every column; no window type declares one yet.

A phone is a screen whose short side is under the threshold in `kit/phone.js` (`phoneMode()` → `up` · `side` · none), never the width alone; a tablet is not a phone. On a phone `<luma-setup>` keeps two rows of chrome, the grid's selection bar is its No cell, `<luma-slab bar>` shows the actions that work now, Oops, ⋯ More and the lock cell; on its side the chrome is one row and the slab a right column. Every phone pop-up is the one sheet `kit/sheet.js`.

## Consequences
Never a per-device layout or local pane state; a new window type declares its mini view when built. `phone.spec.ts` (one test per phone decision) drives phone work; `audit-phone.spec.ts` measures it.
