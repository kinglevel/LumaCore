# 12d research — `<luma-grid>` + function block: lessons from open-source grids and apps

Date: 2026-10-13. Scope: cell editing, multi-select (with a phone section), bulk edit, action bars, columns, live data.
Evidence comes from the docs/issues fetched below. Items marked *(recollection)* come from memory and were not re-verified in this run.

## Sources (fetched or found this run)

- AG Grid touch: https://www.ag-grid.com/javascript-data-grid/touch/
- AG Grid edit start/stop: https://www.ag-grid.com/javascript-data-grid/cell-editing-start-stop/
- AG Grid fill handle: https://www.ag-grid.com/javascript-data-grid/cell-selection-fill-handle/
- AG Grid undo/redo: https://www.ag-grid.com/javascript-data-grid/undo-redo-edits/
- AG Grid batch editing: https://www.ag-grid.com/javascript-data-grid/cell-editing-batch/
- Glide Data Grid selection API: https://docs.grid.glideapps.com/api/dataeditor/selection-handling · repo https://github.com/glideapps/glide-data-grid
- Home Assistant tables: https://www.home-assistant.io/docs/organizing/tables/
- PatternFly bulk selection: https://www.patternfly.org/patterns/bulk-selection/
- Elastic EUI table selection: https://eui.elastic.co/docs/patterns/tables/table-selection/
- Material selection (M2/M1): https://m2.material.io/design/interaction/selection · https://m1.material.io/patterns/selection.html
- Apple two-finger pan multi-select: https://developer.apple.com/documentation/uikit/selecting-multiple-items-with-a-two-finger-pan-gesture
- Google Photos drag-to-select breakage (Aug 2026): https://www.androidauthority.com/google-photos-drag-to-select-missing-3696148 · fix https://www.androidauthority.com/google-photos-drag-to-select-fixed-3697467/
- Baserow paste / limits: https://baserow.io/user-docs/paste-data-into-baserow-table · https://community.baserow.io/t/copying-the-same-value-into-multiple-cells/1584 · https://gitlab.com/bramw/baserow/-/issues/1424
- Airtable bulk editing threads: https://community.airtable.com/base-design-9/bulk-editing-records-31349 · https://support.airtable.com/docs/batch-update-extension.md
- NocoDB bulk update: https://nocodb.com/docs/product-docs/extensions/bulk-update · https://nocodb.com/docs/product-docs/records/actions-on-record
- Grist mobile: https://grist-docs.beta.numerique.gouv.fr/browser-support/
- Google Sheets mobile (third-party guides): https://www.thebricks.com/resources/guide-how-to-select-multiple-cells-in-google-sheets-mobile
- Tabulator responsive layout: https://tabulator.info/docs/4.5/layout
- Syncfusion tablet scroll-vs-select: https://www.syncfusion.com/forums/113802/datagrid-scrolling-on-tablet
- Telerik MAUI double-tap default: https://docs.telerik.com/devtools/maui/knowledge-base/datagrid-maui-single-tap-editing
- Jira silent bulk failures: https://jira.atlassian.com/browse/JRACLOUD-67741
- Excel Ctrl+Enter: https://www.excelcampus.com/tips/ctrl-enter-shortcut/
- Tap delay / double-tap zoom: https://developer.chrome.com/blog/300ms-tap-delay-gone-away/
- Keyboard covering: https://htmhell.dev/adventcalendar/2024/4/ · https://bugs.webkit.org/show_bug.cgi?id=230225

---

## 1. Cell editing on phones and desktop

**What the projects do**
- AG Grid (desktop-first): Enter / F2 / a printable key start editing ("The default editor places this character into the edit field"); double-click is the mouse default; `singleClickEdit` opt-in; Esc "will discard changes"; Tab commits and moves. Clicking elsewhere does **not** stop editing by default (`stopEditingWhenCellsLoseFocus` opt-in), which people often trip over.
- AG Grid touch page gives **no touch edit gesture** and notes that double tap zooms the page unless viewport zoom is disabled. Touch was an afterthought.
- Grist: "to edit a cell, simply double-tap it"; their 2021 mobile work added custom dblclick handling and **Save/Cancel buttons next to the cell editor on mobile**.
- Google Sheets mobile: tap selects, double-tap edits in place.
- Telerik / Syncfusion MAUI: double-tap default; users repeatedly ask for single-tap; Syncfusion: double-tap fires "if we tap quickly" even in single-tap mode — the two conflict.
- Glide Data Grid: editors are **overlays** above the canvas *(recollection)*, which is the right model for a kit element (cell stays a display, editor is a separate floating element).

**Pain points with evidence**
- Double-tap zoom / 300 ms delay: fixed by `width=device-width` viewport; `touch-action: manipulation` on the grid also kills double-tap-zoom (Chrome blog). Do not disable pinch zoom.
- Scroll vs. select/edit: Syncfusion tablet thread — "a tablet touch intended for scrolling instead selected the row immediately" (`AllowSelectionOnPointerPressed`). Rule: act on **pointerup without movement**, never on pointerdown.
- Keyboard covering the editor: iOS Safari keeps the layout viewport full height (`interactive-widget=resizes-content` is Chrome-only; WebKit bug 230225). A `position: fixed` bottom editor ends up under the keyboard unless positioned with `visualViewport`.
- Fat fingers: 44–56 px rows; editors with big targets (keypad, choice list).

**Recommendation**
- **Tap = select (focus cell). Tap on the already-focused cell = edit.** Double-tap also edits (as a shortcut, since tap-tap does the same). No long-press for editing (long-press is reserved for selection, see §2).
- Toggle cells (bool, power) flip on the **second** tap too — never on first tap, so a scroll-start never changes the show.
- Keyboard: arrows move, Enter/F2 edit, typing a printable char starts a text/number editor with that char, Esc cancels, Enter commits + moves down, Tab commits + moves right, Ctrl/Cmd+Enter commits to **all selected cells** (Excel convention).
- Editors by kind (all kit elements, the cell value never edited "inside" the canvas):
  - toggle → flips (no editor); choice/link → pick list; number → keypad (relative ops `+ − × ÷`, `Thru`); text → text field; colour → colour picker; IP / time → typed field with mask.
  - **Desktop/tablet: popover anchored to the cell. Phone (< 600 px): bottom sheet** with explicit OK/Cancel (Grist's lesson) and the cell's column title + the selection count in the sheet header ("Rate · 3 lines").
  - Text editors on phone sit in a sheet positioned with `visualViewport` so iOS keyboard never covers it.
- Each commit is exactly one command line shown to the user (`Set Output 1 Thru 3 Rate 44`) — the editor never writes state directly.

## 2. Multi-select (touch and desktop)

**What the projects do**
- Material: long press starts selection, tapping adds, "long press and drag gesture" selects many; "reserve the long press gesture for selection exclusively"; contextual bar replaces the app bar.
- iOS: two-finger pan selects rows and enters edit mode on its own (`shouldBeginMultipleSelectionInteractionAt`), dragging over selected rows deselects; plus Edit button → checkboxes.
- Google Photos: long-press then drag selects across tiles — and in Aug 2026 it **broke**: "dragging … would only cause the screen to scroll"; tablets unaffected. Evidence that drag-select fights native scroll and is fragile on phones.
- Home Assistant: explicit **"Enter selection mode" button** in the table toolbar, then tap items; "Select all" in a selection menu; bulk actions (labels, enable/disable) apply to the set.
- PatternFly / EUI: tri-state bulk selector first in toolbar, menu with Select none / Select page / Select all; "selected count will update whenever selection is changed"; EUI hides bulk actions until something is selected.
- AG Grid / Baserow / Sheets (desktop): click, shift-click range, ctrl-click toggle, drag rectangle, header click selects row/column.
- Glide: `rowSelect` / `columnSelect` / `rangeSelect` (`cell|rect|multi-cell|multi-rect`) with **selection blending "exclusive"** by default — selecting cells clears row selection and vice versa.
- Immich: shift-click range only works on rendered items — a virtualization pitfall; Baserow caps selection at 200 rows with no warning (GitLab #1424) — users read that as a bug.

### 2a. Phone multi-select (extra requirement)

| Pattern | Works | Fails / risk |
|---|---|---|
| Long-press row → selection mode, then tap to add | Universal Android/Photos/Material; one gesture to learn; no tiny targets | Hidden (needs discovery); long-press timing (≈500 ms) slow for first item; conflicts if long-press is also used for menus |
| Select-mode button (HA "Enter selection mode", iOS "Edit") | Discoverable, deterministic, no gesture conflict | One extra tap |
| Row checkboxes always visible | Clear state | Costs ~44 px of 390 px; tiny checkboxes = fat-finger misses; clutter |
| Drag-to-select after long-press (Photos) | Fast for runs | Fights scroll; broke in Google Photos 2026; needs edge auto-scroll |
| Two-finger pan (iOS) | Fast, no mode | Undiscoverable; conflicts with pinch-zoom; web has no native equivalent |
| Range by two taps ("Thru": tap first, tap **Thru**, tap last) | Exact, no drag, maps to console language (`1 Thru 12`) | Needs a visible Thru action |
| Select all / none / invert in the count bar | Trivial, essential | — |
| Sticky count bar with Done | Always shows what is selected and the way out | Takes one bar of height |

**Recommendation for 390 px that is the same rule set on tablet/desktop:**
1. **Select button** in the function block (and long-press on a row as the shortcut) enters selection mode. Desktop: ctrl/shift-click enter it implicitly.
2. In selection mode a **tap toggles** a row (the whole row is the target, the row-marker column shows a large check — the marker is the visual, not the hit area). Tap never edits in selection mode.
3. **Range = Thru**: tap row A, press `Thru` in the count bar, tap row B → A..B selected (desktop: shift-click does the same). Drag-select through the **row-marker column only** (a thin strip on the left where vertical drag is defined as select, not scroll), with edge auto-scroll; content area drag always scrolls. This avoids the Photos failure.
4. Sticky **count bar** (top of the grid, replaces header row's left part): "12 selected · All · None · Thru · Done". Done exits. Count includes rows hidden by filter, shown as "12 selected (3 hidden)".
5. Cell multi-select (one column, several rows): in selection mode, tapping a **column header** selects that column's cells for the selected rows; editing any of them edits all (§3). Rectangle cell ranges are desktop-only extras (shift-click/drag), not required on phones.

## 3. Multi-edit / bulk edit

**Evidence**
- Excel: Ctrl+Enter fills every selected cell (even non-contiguous) with one entry.
- Baserow: copy one cell, paste into a selection → fills all; paste past end creates rows.
- Airtable has **no native set-value-on-selection** — forum answer is "drag the lower right-corner"; scattered rows need scripts. Users complain → a real gap we can beat.
- NocoDB: select rows → Bulk update modal (pick fields, set values) → confirmation "displays the number of fields and records"; "Bulk updates are irreversible" (bad).
- AG Grid fill handle: single value copies, numeric range continues the series, Alt toggles increment; batch editing: pending values shown, "a committed batch counts as one undo action".
- Home Assistant: bulk enable/disable/labels from the selection bar.
- Jira: bulk edit "finishes without an error and the issues stay unchanged" when validators fail; generic failure messages that don't name the item — the worst outcome.

**Recommendation**
- Editing a cell while rows are selected applies to **that column on all selected rows**, sent as one line: `Set Output 1 Thru 3 Rate 44` (or `Set Output 1 + 4 + 7 Rate 44`). The editor header says "Rate · 3 lines".
- **Mixed values**: the cell shows its own value; the editor shows `Mixed` (placeholder, dimmed, with the min–max range for numbers: "Mixed 25–44"). A toggle on mixed shows a dash state; first tap sets all On.
- **Relative edits** on the keypad: `+ 10`, `- 10`, `* 2`, `/ 2` → `Set … Rate + 10` (already a LumaCore rule: "Numbers take relative input on the station").
- **Fill down / series**: function-block action "Fill" (copy first selected value down) and, for numbers, "Fill series" (1, 2, 3 …). Desktop gets the fill handle later; phones get the action.
- **Paste over a range**: TSV paste into a cell range is one line per column or one compound line — server decides; paste larger than the table offers "Create n new rows?" via a server dialog.
- **Undo = one step** per line (the change path already does that; AG Grid batch confirms the model).
- **Partial failures**: the server runs the line per object and returns a summary that names the refused objects and why ("2 of 12 lines not changed: Output 5 (claimed by Bob, E-CLAIM-001), Output 9 (E-CMD-008)"); the grid marks the refused cells and keeps the whole selection (decision Q5).
- Never apply bulk edits silently to hidden (filtered) rows without saying so: the editor header shows "(3 hidden)".

## 4. Action bars

**Evidence**: Material contextual bar overlays the app bar while selected; EUI hides bulk actions until selection, swaps the count into a primary button; PatternFly bulk selector first in toolbar, row-only actions in a row kebab; HA keeps selection-mode entry in a persistent toolbar; the reference console keeps a fixed function block (user's own requirement).

**Recommendation**
- Keep the **fixed function block** (console muscle memory, stable positions = no hunting) — but actions are **disabled, not hidden**, when they don't apply (positions never move; a touch console needs spatial memory). A disabled cell explains why on tap (toast: "Select a line first").
- Left half (grid edit, fixed order): Select · New · Copy · Cut · Paste · Delete · Undo · Import · Export (Undo = Oops). Right half: domain actions, bound setting cells (live value, tap = editor), power toggle at the far right.
- **Delete** last in the left group, not adjacent to New; red only when enabled; deleting > 1 row asks via server dialog with the count.
- Action labels carry the count when a selection exists: "Delete 3", "Copy 3".
- Phone: function block docks at the **bottom** (thumb reach) as two rows of 4–5 cells, overflow "More" sheet for the rest; selection count bar at the top of the grid. Tablet/desktop: one slab under the content.

## 5. Columns

**Evidence**: HA "Customize table" shows/hides and reorders columns, "Restore defaults", settings "saved in your browser" (so they don't follow the user — a known complaint); AG Grid column menu via 500 ms long-press on header on touch, touch-drag to move/resize; Tabulator `responsive` priorities (0 = never hide, higher = hides first) with `collapse` mode listing hidden columns under the row.

**Recommendation**
- Columns declared in the server registry with: id, title, kind, width (min / default), `priority` (0 = always visible, Tabulator semantics), sortable, filterable, editable (prop + rights).
- Per-user, per-grid **view state** (visible, order, widths, sort, filter) lives on the user profile (window settings), not the browser — follows the operator to any device; "Reset columns" is a command.
- Header tap = sort (asc → desc → none); header long-press / header ⋮ button = column menu (sort, filter, hide, freeze, width reset). Selection-mode header tap = select column (see §2).
- **Frozen first column** (name/number) always; horizontal scroll for the rest on tablet/desktop.
- Phone: **priority hiding + expand row** (Tabulator collapse): the row shows the frozen column + priority-0/1 columns that fit; tapping the row-expand chevron shows the remaining fields as a typed settings list (`<luma-settings>` style) — editing there is the same Set line. Avoid pure card view (loses the column-edit-across-rows model) and avoid only horizontal scroll (a 390 px phone with frozen 120 px leaves ~2 columns).
- Filter: one quick-filter text field + per-column filter chips; filter state shown as chips above the grid; "Select all" acts on filtered rows only and says so.

## 6. Large and live data

- Virtualize rows (DOM recycling) once > ~200 rows; Glide claims millions via lazy canvas cells, but a DOM kit grid with recycling is enough for setup lists (dozens–thousands). Keep selection as **a set of GUIDs, not row indexes** — Immich's "only rendered assets can be selected" and Baserow's 200-row cap are the failure modes of index/page-bound selection.
- Row height: 48 px default on touch, 40 px compact on desktop (Material/Apple 44–48 px targets).
- Live updates: rows keyed by GUID; value changes patch cells in place (brief flash), never re-sort while the user is interacting — **sort is applied on change only when no editor is open and the pointer is up**; new rows appended with a "n new" pill if they'd move the viewport; scroll anchored to the focused row; deleted selected rows drop out of the selection with a count change.
- Live row colours (state: sending / error / off) from the feed, separate from selection colour (selection = outline + tint, state = left bar or row tint) so they never conflict.
- Feed: the grid rides on `/ws/feeds` State (rows) — no per-grid endpoint.

---

## Concrete recommendation — `<luma-grid>` rule set

1. **Rows are objects, keyed by GUID**; columns come from the server registry (kind, priority, width, editable prop, rights). The grid never stores state; every gesture is one command line.
2. **Tap selects a cell; tap the focused cell (or double-tap, Enter, F2, typing) edits it.** Pointer acts on up-without-move only. Toggles flip on second tap.
3. **Editors are kit elements**: popover on ≥ 600 px, bottom sheet with OK/Cancel on phones, positioned by `visualViewport`.
4. **Selection mode**: Select cell in the function block or long-press a row (desktop: ctrl/shift-click). Tap toggles rows; `Thru` makes a range; drag-select only in the row-marker strip; sticky count bar "n selected · All · None · Thru · Done".
5. **Edit one = edit all selected in that column**, one line (`Set Output 1 Thru 3 Rate 44`); mixed shows "Mixed (min–max)"; relative `+ − × ÷`; Fill / Fill series actions; paste = one line per column.
6. **One undo step per line; partial failure names each refused object with its code** and marks those cells.
7. **New-object line** at the bottom of the grid (always last row, "+ New Output"): tapping a cell there runs `Store Output <next>` then edits that cell.
8. **Function block fixed**: left grid actions (Select · New · Copy · Cut · Paste · Delete · Undo · Import · Export), right domain actions + bound settings + power. Disabled not hidden; counts in labels; Delete confirmed via server dialog when > 1.
9. **Columns**: header tap sorts, long-press / ⋮ opens column menu; frozen first column; per-user view state on the profile; phones use priority hiding + row expand.
10. **Live**: in-place cell patch, no re-sort under the finger, scroll anchored, row state colour separate from selection.

## Decision questions (recommended first)

1. **Edit trigger on touch** — (a) tap selects, tap again edits *(rec: predictable, no accidental edits while scrolling, matches Sheets/Grist)*; (b) single tap edits (fast but scroll accidents; Syncfusion/Telerik pain); (c) long-press edits (clashes with selection gesture).
2. **Phone multi-select entry** — (a) Select button + long-press shortcut, tap toggles, Thru range *(rec: discoverable + fast + console vocabulary; same on all sizes)*; (b) always-visible checkbox column (costs width, tiny targets); (c) drag-select across rows (fights scroll; Photos 2026 bug).
3. **Phone layout for wide grids** — (a) priority hiding + row expand to settings list *(rec: keeps column edit model, Tabulator-proven)*; (b) horizontal scroll with frozen first column (2 columns visible at 390 px); (c) card view (loses multi-edit by column).
4. **Function block actions not applicable** — (a) disabled, explain on tap *(rec: spatial memory on a console)*; (b) hidden (EUI style, bar jumps); (c) contextual bar replaces block on selection (Material; loses fixed positions).
5. **Selection after a bulk edit with refusals** — (a) keep whole selection, mark refused cells *(rec: the operator can retry or adjust without reselecting)*; (b) reduce selection to refused rows (handy for retry but surprising); (c) clear selection.
6. **Where column view state lives** — (a) user profile window settings *(rec: follows the operator; HA's browser-only storage is a known gap)*; (b) device/browser; (c) show-wide.
7. **Bulk edit with filter active** — (a) apply to all selected incl. hidden, say "(3 hidden)" *(rec: selection is truth)*; (b) auto-drop hidden rows from selection when filtering; (c) refuse until filter cleared.
8. **Cell ranges (rectangles)** — (a) desktop extra only (shift/drag), phones use row selection + column *(rec: the bulk use case is "this column on these rows")*; (b) full rect range on all devices with handles (Sheets-style, fiddly on 390 px).
