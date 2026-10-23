---
id: 0244
title: Rows whose type lives above the registry are handed back by type
area: catalog
status: built
date: 2026-10-16
built_in: 34f7a
supersedes: []
superseded_by: —
confirmed_by: [`Registry::check` at freeze]
source: [archive §34 L8253–8259 (item 22.7), archive §34 L8768–8770]
---
# Rows whose type lives above the registry are handed back by type

## Context
Building the protocol crates (34f7a) found a row the registry can't type: a `ProtocolDef` holds its opener, a function over protocol types that sit above `lumacore-registry`.

## Decision
The registry stores such rows untyped and hands them back by type: a crate adds them with `Registry::rows(&'static [T])` and reads them with `Catalog::rows::<T>()` (`lumacore_registry::get().rows::<ProtocolDef>()`, `protocol::find`). The crate that owns the type registers its own freeze check with `Registry::check(fn(&Catalog) -> Vec<String>)`, run when the catalog freezes (protocols: one name per protocol, the opener fits the kind). It is still the one catalog, pushed at startup (see catalog-one-registry); the behaviour sits beside its row like an `ObjectDef`'s. The command families, hooks and the station's rows go in the same way.

## Consequences
A new row type above the registry needs no registry change: add it with `r.rows(..)` and give it a `check`. Never keep a side table of such rows in the crate that reads them; read the catalog.
