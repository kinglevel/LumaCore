---
id: 0243
title: Properties stay on the ObjectDef, not in catalog rows
area: catalog
status: built
date: 2026-10-16
built_in: 34f3g2
supersedes: [archive §34 L8134–8137 "menu column names"]
superseded_by: —
confirmed_by: [`lumacore_views::column_json`, catalog test `every_column_prop_resolves_on_its_object`]
source: [archive §34 L8177–8184 (item 21.10), archive §34 L8718–8720]
---
# Properties stay on the ObjectDef, not in catalog rows

## Context
When the objects became catalog rows (34f3g), the question was whether a `Property` is a row too. Its `Access` (get / set on `AppState`) and its `Props` (keyed rows, apply, ask, after, within, view) are behaviour, and every reader of them sits in the objects crate.

## Decision
An object's properties, the settings among them, stay beside its row on the `ObjectDef` (`ObjectDef::props`), not in `lumacore-registry`. The check that a menu column names a declared property is a test, not a freeze check: `lumacore_views::column_json` builds every column of every menu through `objects::def_of`, and `tests/catalog` runs it over every row. The one property registry's rules (declared once, `Set` typed by kind) are see one-property-registry.

## Consequences
A property is declared on its object and nowhere else; a menu column naming an undeclared one fails the views test, not `freeze()`. Properties become rows only when something outside the objects crate — a plugin, the views — must declare or read them without the objects crate; that needs a `Property` split into a row and its get / set first (roadmap, not built).
