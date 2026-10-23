---
id: 0141
title: The property's kind parses the value once; Reset restores defaults
area: properties
status: built
date: 2026-10-15
built_in: 3c2
supersedes: [archive §33 L7098–7104 "handler-local bool lists"]
superseded_by: —
confirmed_by: [E-SYN-014, E-CMD-008, E-SET-003, Property::default]
source: [archive §33 L7098–7104, archive §33 L7488–7496, archive §33 L7510–7518]
---
# The property's kind parses the value once; Reset restores defaults

## Context
About ten handlers kept their own on / off word lists (one turned any unknown word into false), links were looked up by hand (`network_link`, `user_link`), and Theme / LockScreen took a value word `Default` to reset. On established consoles `Default` is only the fixture-value word and `Reset` clears an object's settings.

## Decision
A property's `PropKind` (`core::props`, declared on `ObjectDef::props`, see one-property-registry) decides what its value slot takes, and the value is parsed once in `commands::set::route`. On / off is `On` / `Off` / `Yes` / `No`; `Toggle` only where the property is a `Switch`. A Choice takes its declared words; a bare object keyword without ids is the value word when the property declares it (`Set Fixture 1 "MasterReact" Group`). A Link takes a target of the linked type through the one resolver (`core::resolve`) and is stored as the object's GUID, so it follows a move. Numbers take relative input and a `Thru` value spread over the rows. A word the kind doesn't take is refused naming what it takes (E-SYN-014 from the binder, E-CMD-008 for an unparseable value); never 0, never false. `On` / `Off` at the head of a statement stay verbs; position tells verb from value.

`Default` is never a `Set` value (refused with the Reset hint). `Reset <target>` puts every property of an object back, `Reset <target> "<Property>"` one, for every object of the registry; every property declares `Property::default`, which the settings cells' reset reads too.

## Consequences
A new value shape is a new `PropKind` (or `PropKind::Declared`, the object's type checking it), never a parser or word list in a handler. The value-`Default` words for `"Theme"` / `"LockScreen"` are gone. The light-colour value is open, see color-light-paint-screen.
