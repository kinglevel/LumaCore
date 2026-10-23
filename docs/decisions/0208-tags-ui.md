---
id: 0208
title: Tags in EditSetting, the pool window and the sidebar
area: ui
status: built
date: 2026-10-17
built_in: tags parts 4a–4b
supersedes: []
superseded_by: —
confirmed_by: [E-TAG-003]
source: [archive §12 L838–861, archive §12 L959–999]
---
# Tags in EditSetting, the pool window and the sidebar

## Context
The tags hardening (2026-10-17, two mockup rounds, round 2 locked "Go") decided there is no Tags section under Show: tags are given where every show object is edited, and overseen in one pool window. Parts 4a and 4b built the two surfaces.

## Decision
`EditSetting` of any show object whose row has a `meta.guid` gets a Meta tab after its own tabs (the NodeMeta fields plus read-only GUID, Path, Used by) and a Tags tab from a `SettingsTab` row of `lumacore-cmd-tags`: one `FieldInput::Tags` field drawn by the kit's `<luma-tree>` in pick mode, tri-state over several objects. A tick is `Assign Tag <path> At <object>`, an untick `Unassign …`, New tag here `Store Tag …` under the highlighted tag or at the top.

The Tags pool window (window type `tags`, `<luma-tags-pool>`, the `tags` feed) has two panes: the tree (`<luma-tree>` browse) with colours, counts and the Action badge, and the picked tag's members in a read-only `<luma-grid>`. Its settings are `Tag` (a Link by GUID), `Members` Direct | Deep, `Default action`, `Counts`; a tap is `Set Self "Tag" 1.1`, Direct | Deep `Set Self "Members" …`, Run action runs the tag's own `Action` or the window's default (`Tag 1.1 [/R]`); the ≡ menu's actions are station lines. A tag's own settings (`EditSetting Tag 1.1`) are keyed properties on its GUID; a sibling's name is refused (E-TAG-003), Label's check. A tag pick list answers with the GUID (`Pick.by_guid`) because a tag's number is a path.

## Consequences
Never a tag editor outside these surfaces or a client-side tag tree; the sidebar's Tags mode and the Tags cell in `<luma-grid>` went to the patch work (see patch-dialog-one-tile-claim-button). Not built: a kind filter on the members pane, dragging a tag to re-parent it, Expand all, Go+ / Off actions.
