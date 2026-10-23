---
id: 0292
title: The Tree window is NAV's tree and inspector
area: ui-kit
status: built
date: 2026-10-23
built_in: Tree window (2026-10-23)
supersedes: []
superseded_by: —
confirmed_by: [the_tree_opens_down_to_the_place, a_place_is_a_path_or_a_guid, group_1s_inspector_has_settings_cells, follow_goes_to_the_newest_pick, the_tree_window_follows_the_selection, kit-tree-window.spec.ts]
source: [session 2026-10-23, "looks good, implement" — docs/dev/ui-kit/mockups/2026-10-23-tree-window/01-tree-window.html (round 1, every question as recommended)]
---
# The Tree window is NAV's tree and inspector

## Context
NAV ([0291](0291-nav-mode-ls-rows-and-an-inspector.md)) lives only in a terminal. Pointing at a path or a GUID while discussing a bug, a design or a show needs it in the web and the app too, without a second tree. Rejected: a read-only inspector, Follow without a None, a client-built tree.

## Decision
Window type `tree` (Rust registry; settings Follow None / Selection, Show All / Changed only, Split, and its view state Path, Open, Search). Two counter-driven feeds: `tree` (the open branches' rows from `tree_rows`, NAV's filter, bookmark badges, the Follow target) and `inspector` (`lumacore_views::inspector` as JSON, each property its `EditSetting` cell), both from `lumacore_views::tree_window`, built on the state thread as the user because the tree reads the runtime and the drives. A place is a typed path or a GUID (whole, or a unique prefix of `GUID_PREFIX` digits); a gone one is a toast and the window stays. Every gesture is a line: `Set Self "Path" | "Open" | "Search"`, a cell's `Set`, ↺'s `Reset`, "Open in the terminal's NAV" = `cd` + `Shortcuts Navigator`. Follow = Selected jumps to the object the user's newest line named (`Usage`), a fixture meaning the selection's last. Copies go through `kit/clipboard.js`; the row menu is the registry's menu plus the copies (`openContext` `extra`). `<luma-pathbar>` joined the kit; `<luma-tree>` gained a host mode (rows from the station, virtual, fold and search as events). A phone draws the tree and the inspector as the one sheet.

## Consequences
A new node or property shows here, in `ls` and in NAV alike. Open: Follow = My NAV (the terminal's NAV cursor is not station state; planned in the setting's hint); a bookmark word ([0291](0291-nav-mode-ls-rows-and-an-inspector.md)) — the row menu's Bookmark… is greyed; Runtime rows refresh when the show moves, not live.
