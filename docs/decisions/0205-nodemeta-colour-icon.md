---
id: 0205
title: Colour and icon live only in NodeMeta
area: pools
status: built
date: 2026-10-14
built_in: 12e part 2
supersedes: []
superseded_by: object-paint-palette-free-theme-links
confirmed_by: [a_group_takes_a_colour_and_an_icon]
source: [archive §30 L5490–5496, archive §30 L5619–5625]
---
# Colour and icon live only in NodeMeta

## Context
The 12e round 2 (2026-10-13) settled where a pool object's look comes from. A reference console keeps an appearance pool that objects link to; LumaCore already carried colour and icon on every object's `NodeMeta` and drew them in the pool cell. The user decided against a second home.

## Decision
An object's colour and icon are fields of its `NodeMeta` only, forever — no Appearance pool. They are declared `META` props on every pool object through `core::props`, so `Set Group 3 Paint|Icon …` writes them with the palette and icon pick lists and `EditSetting Group 3` lists them; each write is one Oops step on the change path. The colour is written as `#rrggbb`, kept as RGBA bytes, and `none` means the pool type's theme colour: an object without a colour draws in `--lc-pool-<kind>` (`--lc-pool-groups`, `--lc-pool-default`), an object with one in its own `--lc-obj`. The icon is a standard icon's name (`PropKind::Icon`). `props::shown` reads them back for `Set` replies and `EditSetting`. A prop write carries the object's datapool (`props::Apply`), so a group in any pool takes it.

Today the field is `paint` on the `Annotation` and the property `Paint`; the archive's `Color` spelling is superseded by the paint record (see color-light-paint-screen). The meta levels themselves are node-meta-levels.

## Consequences
A kind that wants a colour or icon declares nothing: it gets them with its `NodeMeta`. Never a per-object colour field, a linked appearance object or a client-side palette; a mini pool cell draws colour alone, so the token fallback is what keeps an uncoloured object visible. The test `a_group_takes_a_colour_and_an_icon` holds the round trip.
