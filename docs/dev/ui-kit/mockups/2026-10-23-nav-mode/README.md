# NAV mode and the inspector (2026-10-23)

The terminal's NAV mode drew its own tree: it used a child's label (`DataPool 1 "Default"`) as its
path, so objects didn't open, and its pop-up showed raw JSON. Two rounds, design only until the
user decided; the research behind round 2 looked at four families of tools (file managers,
dashboards, data explorers, inspectors) and kept the ideas several of them share.

| File | What it asks |
|---|---|
| `01-nav.html` | Round 1: NAV today on the demo show beside `ls`, why they differ, and a working proposal in two layouts (A tree + inspector, B three columns); five questions (layout, the rows, big pools, editing, the four bugs found on the way). |
| `02-inspector.html` | Round 2: layout A with copy keys (`y`, `yg yp ya yn ys`), Used-by links, filters and `n` / `N`, changed values (●, `r` for Reset, a changed-only toggle), marked rows into one range, `?` help, the gesture log, bookmarks by GUID, a pinned inspector, values that flash when another station changes them, and how each client copies (the CLI, the page, the app). |

Decided in the rounds' questions (all as recommended):

- layout A: the tree on the left, the inspector on the right following the cursor; Tab between;
- the rows are exactly `ls`'s (slot, "name", kind, counts) from one builder, paths from numbers
  and declared names;
- a big pool is one list drawn only where it shows; digits jump to a slot, `/` filters;
- an edit is a command line: Enter on a property puts its `Set` line on the command line;
- the four bugs are fixed in the same step (`ls`'s `└──` order, `Info DataPool`, the
  `Info Fixture` channel table, `cat 1`);
- copy keys A (`y` in the inspector, `yg` / `yp` / `ya` / `yn` / `ys` on a tree row);
- the gesture log on by default, the profile keeps the choice (`NavLog`);
- bookmarks per user in the show, like Usage;
- following a link moves the cursor only; a gone GUID shows red "missing".

Approved 2026-10-23 ("all looks good i think"; record 0291).
