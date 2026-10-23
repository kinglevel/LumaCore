---
id: 0110
title: Self reads fill a value slot; Not in conditions
area: syntax
status: decided
date: 2026-10-14
built_in: —
supersedes: [archive §33 L7222–7223 "!Self.X conditions"]
superseded_by: —
confirmed_by: none
source: [archive §33 L7216–7225 (item 32)]
---
# Self reads fill a value slot; Not in conditions

## Context
Window-bar items and title templates already read window state through `Self.<Property>` (12c: `Self` resolved on the station where the line starts), and the registry's declared conditions wrote "not" as `!Self.X`. With `!` taken by aliases, reads needed one spelling inside Lum.

## Decision
A read is one dotted word starting with `Self.` and fills a value slot: `Set Group 3 "Name" Self.Name`, `Set Self "Title" Self.DataPool.Name` — resolved when the line runs, on the station where it starts. With `Root.` paths it is the only place a dot nests (lum.md's `SELF-READ` token; the property names after the dot are bare). Reads start from `Self` only; there is no `Root.…` read and no arithmetic on a read. Title templates (`"U {Self.Universe}"`) use the same read.

A declared condition in the registry — a window-bar item's `state` / `shown_when` — says "not" with the word `Not`: `shown_when: "Not Self.Filter"` shows the chip only while no filter is set. `!` means only an alias (see user-aliases-bang), nowhere else; the old `!Self.X` conditions are rewritten.

## Consequences
Not built: lum.md's grammar names the token and the lexer reads a `Self.…` word as one dotted word, so the parser accepts it; what resolves the read when the line runs is open. A read in a stored line is exact like every stored line. Adding a second read root (`Root.`, another object) is a lum.md hard discussion.
