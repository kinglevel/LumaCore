# The Tree window (2026-10-23)

The terminal's NAV mode ([0291](../../../../decisions/0291-nav-mode-ls-rows-and-an-inspector.md))
as a window of the web and the app: the same tree and inspector, for pointing at a path, a GUID or
a part of the tree while discussing a bug, a design or a show.

| File | What it asks |
|---|---|
| `01-tree-window.html` | Round 1: a working mock of the window on the demo show (the path bar, the tree, the inspector with copy buttons and settings cells, Used by links, the row menu, Follow, a value changed by another station flashing) and two phone frames; six questions and how it is built. |

Decided in the round's questions (all as recommended):

- editing: the inspector's values are settings cells; every change is one `Set` line, ↺ its `Reset`;
- Follow is a window setting, None / Selection / My NAV, Selection by default (My NAV is planned:
  the terminal's NAV cursor is not station state yet);
- the path bar: crumbs to go up, a field to type or paste a path or a GUID (a prefix of 8 digits
  is enough); a GUID that is gone says so in a toast;
- copy: a copy icon on Target, GUID and Path, and the row menu's Copy GUID / path / as target /
  Set line, all through the one clipboard door;
- shared with NAV: bookmarks (badges) and the ● changed markers; the row menu has "Open in the
  terminal's NAV" (`cd` + `Shortcuts Navigator`); Bookmark… waits for its word;
- phone: the tree fills the pane, a tap opens the inspector as the one sheet.

Approved 2026-10-23 ("looks good, implement"; record
[0292](../../../../decisions/0292-tree-window-nav-in-the-web-and-app.md)).
