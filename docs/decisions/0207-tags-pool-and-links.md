---
id: 0207
title: Tags are a tree pool with GUID links; verbs reach members
area: tags
status: built
date: 2026-10-17
built_in: tags parts 1–3b
supersedes: [archive §12 L808–829 "deep, Assign nests"]
superseded_by: —
confirmed_by: [E-TAG-001…007, an_object_that_nests_itself_reads_a_path]
source: [archive §12 L783–890, archive §12 L891–958]
---
# Tags are a tree pool with GUID links; verbs reach members

## Context
Tags were decided on 2026-10-17 from the 34g4 patch hardening. The hardening first made a tag reach its objects deep by default and let `Assign Tag At Tag` nest; the lum.md hard discussion revised both: depth comes from `/R`, nesting is a Move.

## Decision
Tags are both a pool and links. The definitions form a real tree, `Root.ShowData.Tags.<n>.<n>…` (`crates/model/lumacore-tags`: one parent, no cycles, names unique among siblings — E-TAG-003 / 006), numbered within the parent, so a tag carries its path and a move renumbers it with the GUID kept. Every object with a `NodeMeta` holds `meta.tags` as GUIDs, walked as links; runtime and station objects are never tagged. `ShowData.tags` is its own `ShowSubtree`.

Every object declares how it reads a dotted id (`IdReading` on `LumShape.reading`); one that nests itself defaults to Path, and the resolver walks a Path object only through its `PathDef` row (`Thru` walks one parent's children, E-TAG-005 across parents; its row is its GUID, never its last number).

The Tag object (`lumacore-cmd-tags`) has `Store Tag`, `Label`, `Delete Tag [/WithChildren | /KeepChildren]`, `Move Tag … At Tag | Root | <n>`, `Assign` / `Unassign Tag … At <objects>` (Assign always tags, never nests), `List`, `Info Tag [/R]`, and the bare `Tag 1.1 [/R]` selecting its fixtures. Member verbs reach the tag's own objects, `/R` adds every tag under it; each kind runs its own handler under its own rights, the rest are skipped with a summary, nothing taking it is E-CMD-009; one line, one Oops step.

## Consequences
Never a second parent, a slot-keyed tag link or a per-kind membership walk. Tags as a live column under the Devices claim belongs to the staged patch's live columns (see staged-patch-draft). Importing another show's data asks Tags: Bring | Leave — roadmap.
