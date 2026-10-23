# The UI kit

Every window, menu, dialog, pop-up, bar and plugin UI of the WebUI is built only from the kit:
plain custom elements (ES modules, no build step) in `crates/ui/lumacore-webui/web/kit/`, drawn
from server descriptors; every gesture is a command line. A window that needs a missing element
waits: the element is designed as a mockup first ([mockups/](mockups/README.md)), approved, added
to the kit and the Showroom, then used. The rules are the archive's §30 (`docs/archive/decisions-2026-10.md`)
and the tidy `frontend` rules ([tidy.md](../../meta/tidy.md)).

The page waits for `window.LumaKit` (event `lumakit`) before it boots. Nothing loads from the
internet: the libraries are vendored in `crates/ui/lumacore-webui/web/vendor/` (its README).

## Elements

The custom elements the kit registers (`customElements.define` in each file; verify with
`grep -o "customElements.define('[a-z-]*'" crates/ui/lumacore-webui/web/kit/*.js`).

| Element | File | What |
|---|---|---|
| `<luma-avatar>` | `avatar.js` | A user's avatar (colour, initials) |
| `<luma-button>` | `button.js` | A real `<button>` inside; disabled stays visible |
| `<luma-cue-sheet>` | `cuesheet.js` | The sequence sheet, fed by the `sequence` feed |
| `<luma-detail>` | `detail.js` | A Users & Profiles detail (`detail` feed) |
| `<luma-dmxmap>` | `dmxmap.js` | The Universes map: blocks and runs, drag on the pointer core's `lift` |
| `<luma-encoder-bar>`, `<luma-encoder-bars>` | `encoderbar.js`, `encoderbars.js` | The encoder bar (`encoder` feed) and a profile's bars |
| `<luma-fader>` | `fader.js` | A fader; its moves go out as one gesture |
| `<luma-files>` | `files.js` | The one files element: drives rail, crumbs, a pick (`files` feed) |
| `<luma-graph>` | `graph.js` | A time graph (the Realtime window) |
| `<luma-grid>` | `grid.js` | Every list of objects: rows by GUID, columns from the registry, selection, the edit card |
| `<luma-icon>` | `icon.js` | An icon from the vendored Lucide sprite and the kit's own glyphs |
| `<luma-keyboard>`, `<luma-keypad>` | `keyboard.js`, `keypad.js` | The built-in keyboard (layouts in `layouts/*.json`) and the keypad |
| `<luma-maxbar>` | `maxbar.js` | The bar of a window held full screen |
| `<luma-overview>` | `overview.js` | The screens overview |
| `<luma-people>` | `people.js` | Users & Profiles |
| `<luma-pool>` | `pool.js` | The one pool window (`pools` feed) |
| `<luma-popup>` | `popup.js` | A pop-up over the page |
| `<luma-settings>` | `settings.js` | A settings window: tabs from sections, a cell per input, every change one `Set` line |
| `<luma-setup>` | `setup.js` | A setup menu (`MenuDef`): tabs, grid, slab, the Patch dialog's area |
| `<luma-slab>`, `<luma-power>` | `slab.js` | The action block (New / Copy / Delete …, fixed order) and the power toggle |
| `<luma-suggest>` | `suggest.js` | The suggestion list above the command line bar |
| `<luma-surface>` | `surface.js` | The lock / login surface |
| `<luma-tags-pool>` | `tagspool.js` | The Tags pool window (`tags` feed) |
| `<luma-tiles>` | `tiles.js` | A tile grid (the Menu, Shutdown, Backup tiles) |
| `<luma-tree>` | `tree.js` | A tree sidebar (the Patch dialog's MVR tree) |
| `<luma-vlist>` | `vlist.js` | A virtual list (the System Monitor) |
| `<luma-window-bar>` | `windowbar.js` | A window's bar from `/api/window-types` and the `windows` feed |

## Modules without an element

| File | What |
|---|---|
| `kit.js`, `kit.css`, `tokens.css` | The kit's entry, the elements' styles, the theme tokens (the only file with raw colours) |
| `theme.js` | Presets (`<html data-theme>`), tokens as colours, xterm's theme from the tokens |
| `pointer.js` | The pointer core: `hold()` and the one `lift` gesture (tap, drag, lasso, context menu, spring) |
| `feeds.js` | `FeedClient`: the one `/ws/feeds` socket ([http-api.md](../api/http-api.md)) |
| `dialog.js`, `toast.js`, `context.js`, `menu.js` | The station's dialogs, toasts, the context menu, the window menu |
| `editcard.js`, `editpatch.js`, `insert.js`, `mvrimport.js`, `groupeditor.js`, `attrboard.js`, `sheet.js`, `tools.js`, `value.js`, `sliderpanel.js` | The grid's edit card, Edit Patch, Insert fixtures, the MVR import dialog, the group editor, the attribute board, the object sheet, editor tools, the value dialog, the slider panel |
| `split.js`, `minipane.js`, `phone.js` | The split tree, miniature panes, the phone's desk |
| `oops.js` | The Oops window (`oops` feed) |
| `showroom.js`, `showroom12d.js` | The Showroom window type: every element in every state |
| `layouts/` | The built-in keyboard's layouts (value / shifted / AltGr per key) |

## Checks

- `cargo test -p lumacore-tidy frontend::` — no table markup outside `grid.js`, no hand-made
  action buttons outside `slab.js`, colours only in the theme, icons only from the kit, one
  socket for feeds, no event stream or poll, text selectable only where decided.
- `cd tests/e2e && npx playwright test tests/kit*.spec.ts` — the kit's specs on all four
  projects (desktop, tablet, phone, webkit); `multiplatform.spec.ts` is the fit guard.
- `./scripts/unix/screenshots` — every window type, menu and overlay into `target/screenshots/`.
