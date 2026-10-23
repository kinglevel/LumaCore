---
id: 0277
title: No sub-fixtures: cells, then parts and the pixelmapper
area: patch
status: decided
date: 2026-10-18
built_in: —
supersedes: [archive §28 L2977–2989 "cell layout is defined per mode"]
superseded_by: —
confirmed_by: none
source: [archive §28 L2977–2989, archive §28 L3294–3296, archive §34 L9837–9908, archive §34 L10185–10188]
---
# No sub-fixtures: cells, then parts and the pixelmapper

## Context
§28 (2026-10-11) decided one object everywhere, no sub-fixtures, the cell layout per mode on the FixtureType from GDTF's `GeometryReference`s, expanded only by the effect generator. Round 3 reopened it: a mode with the same attribute on two geometries can't be programmed apart, and "we don't want to follow the subfixture system". A parts-pool proposal was judged too close to sub-fixtures; three research agents ran first.

## Decision
A patched fixture is one object everywhere; sub-fixtures never exist — a fixture is a fixture (folders are fixtures, see nested-fixtures-folders). §28's cells-per-mode model is superseded as the model for parts; `lumacore_gdtf`'s reading of a type's geometry stays. The direction (2026-10-18): a built-in media server (the §36 renderer) renders onto canvases; a canvas is a device of a new kind, Pixelmapper, outside the DMX patch with a FID in the one number space, controlled like a fixture; mappings attach fixture parts (a Parts pool's kinds — ring, cells, heads; a type's geometries link to them by GUID) to canvas regions. The operator never programs a part directly: the pixelmapper controls a mapped part completely and the main fixture's values never leak in, while the output stage, grand master and Park still apply. A canvas holds layers — `Fixture 900 CanvasLayer 2 At 50`, owner first; `CanvasLayer` is reserved, a bare `Layer` is never a word, `Part` stays reserved for cue parts.

## Consequences
Not built; hardened on its own first (the parts model and where parts are used — the clone window among them, see map-and-clone-attribute-maps); the part word waits. Nothing programs a geometry apart today; nobody adds a sub-fixture id to lum.md.
