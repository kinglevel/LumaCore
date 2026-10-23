---
id: 0105
title: Each object declares how it reads a dotted id
area: syntax
status: built
date: 2026-10-17
built_in: Tags part 2
supersedes: [archive §12 L866–868 "path by shape"]
superseded_by: —
confirmed_by: [catalog test `an_object_that_nests_itself_reads_a_path`, `IdReading` on `LumShape`, the reference's `ids decimal` / `ids 2 parts` lines, the golden record]
source: [archive §12 L862–874, archive §12 L910–918]
---
# Each object declares how it reads a dotted id

## Context
The tag tree (any depth, `Tag 1.2.1`) met the one-number rule for ids: `Cue 1.10` is `Cue 1.1`, `Master 2.1` is two fixed parts, but a tag path must keep every part. "A path for every object that nests itself, by shape alone" was rejected: it leaves DMX, Cue and Window implicit, a structural change would silently re-read old lines, and the reference can't show it.

## Decision
Every object declares how it reads a dotted id, on its `LumShape` as `lumacore_registry::object::IdReading`: `None`, `Whole`, `Decimal` (Cue: `1.10` = `1.1`), `Parts(n)` (Master, Preset, ViewButton, Window) or `Path` (one part per tree level, any depth). Constructors: `LumShape::ids(n)`, `decimal()`, `path()`. An object that nests itself defaults to Path (the catalog test holds it); Tag declares it. `Fixture` declares none: a fixture is a fixture. `DMX` alone reads one part as a universe (`DMX 1`), two as an address (`DMX 1.001`).

The one resolver reads ids only through the declaration, never as a float. A dotted id on an object that declares none, or a wrong number of parts, is E-SYN (`Fixture 1.2.3` → E-SYN-012). On a Path, `Thru` walks the children of one parent (`Tag 1.2 Thru` = 1.2, 1.3, 1.4, …); ends under different parents are refused — join them with `+`.

## Consequences
Changing a declaration is a visible registry change: the reference shows it and the golden record moves. A range of tags reaches each listed tag's members for verbs that reach members; the Tag object's own verbs touch only the listed tags. lum.md's rule text still carries the discussion's "not built yet" marker; the archive's Tags part 2 entry is the later state. A number in a `Root.` path follows another rule, the child's No — see tree-paths-typed-node.
