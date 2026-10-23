---
id: 0143
title: Every object declares its properties once on ObjectDef::props
area: properties
status: built
date: 2026-10-15
built_in: 3c1
supersedes: [archive §33 L7451–7458 "three property systems", archive §30 L4719–4721 "TreeDef::write kinds rights"]
superseded_by: —
confirmed_by: [E-SET-001, E-SET-002, the_shim_count_only_goes_down]
source: [archive §33 L7451–7472, archive §33 L7864–7909]
---
# Every object declares its properties once on ObjectDef::props

## Context
Three property systems stood behind `Set`: `TreeDef::write` (name, JSON field, kind, rights), the keyed settings of `core::settings_props` (get / set, home, section, unit) and many objects with a hand-written `Set` parser each. Info, EditSetting, the grid columns and the rights gate each read a different one.

## Decision
`core::props` is the one property registry. A `Property` is name, kind, named and display words, rights, home, section, unit, doc and its access: `Access::Field` (a field of the object's JSON, written through `Props::apply`), `Access::Keyed` (get / set by row: this station, an interface, the show) or `Access::Named` (one get / set for a list declared as data). `Props` sits on `ObjectDef::props`, beside the object's row: the lists, keyed rows, the implicit row of a bare owner, labels, apply / may, after, read-only, the station's own write, asks and wiring scope, `Props::dynamic` (every window type's settings, built once), `Props::scoped` (properties that exist only under an owner), `Props::within` (owner-first paths), `Props::hint` (the At form for `"Master"`). An object's Name / Note / Paint come from its meta level (`Props.meta: MetaLevel`), never a per-object list.

`Set` (see set-one-typed-handler), `Info`, `EditSetting`, the settings cells, the menus' grid columns, suggestions, the rights gate, help, the Lum vocabulary and the reference read only this registry. Everything undeclared is read-only and `Set` says why.

## Consequences
A new property is a `Property` on its object, nowhere else; `TreeDef::write` and `settings_props`' `KeyedDef` / `Setting` are gone and never return. Never a handler-local parser, on / off list or second property table. A form that isn't property plus value gets its own verb. The homes a property declares and how a cell shows them: see settings-home-icons.
