---
id: 0291
title: NAV mode is ls's rows and an inspector
area: ui-kit
status: built
date: 2026-10-23
built_in: NAV mode and inspector (2026-10-23)
supersedes: []
superseded_by: —
confirmed_by: [nav_rows_are_ls_rows, every_object_opens, every_row_opens_by_its_path, fixture_101_has_identity_facts_properties_and_links, a_bookmark_survives_renumbering, ls_draws_the_last_glyph_on_the_last_row, copy.spec.ts]
source: [session 2026-10-23, "all looks good i think" — docs/dev/ui-kit/mockups/2026-10-23-nav-mode/01-nav.html (round 1, layout A) and 02-inspector.html (round 2); research into four families of tools: file managers, dashboards, data explorers, inspectors]
---
# NAV mode is ls's rows and an inspector

## Context
NAV drew its own tree from each child's label (`DataPool 1 "Default"`) and used that label as the path step, which the typed path ([0106](0106-tree-paths-typed-node.md)) refuses: objects didn't open and Enter found nothing. Its pop-up showed raw JSON beside a few hand-mapped fields. Rejected: three columns (layout B), keeping the full-width tree with a fixed pop-up (C), editing in place, one `y` menu, bookmarks on the device only, the log off by default.

## Decision
One row builder, `lumacore_objects::tree_rows`, makes the rows below a node; `ls`, NAV and the inspector read it, and a row's path step is a declared name or an object's number, never a label. NAV is the tree on the left and the inspector on the right, following the cursor (`p` pins it); the inspector is built by `lumacore_views::inspector` from the one property registry `EditSetting` reads: Slot, Kind, GUID, Path, facts, every property with its value, kind, `Set` line and ● when it differs from its declared default (`Reset` line), and the links (Used by, Links to; a gone GUID is red "missing"). Every gesture that changes something is a line: `Set` / `Reset` / range / `Delete` lines go on the command line, `cd`, `Shortcuts` and `Set UserProfile "NavLog"` run; copying goes through the one clipboard door; a bookmark is a letter → GUID on `ShowData.Users.{n}.Bookmarks` (its own `ShowSubtree`, never undone), filed through `commands::record_change`. The keys are `NAV_KEYS` in the TUI.

## Consequences
A new tree node or collection shows in `ls` and NAV alike; a test compares them. The format bumped for bookmarks and the profile's `NavLog` (empty step). Open: a member station keeps bookmarks for its session only; a word for bookmarks would be a syntax discussion.
