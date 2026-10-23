# Layout description format: research (2026-10-12)

Question: what should LumaCore's declarative UI layout file look like? It must be a declarative file that describes dialogs and windows: layout (a grid with Fixed / Stretch / Content tracks), widgets, bindings to tree properties and actions (command lines). It must also:

- be easy for users to read,
- be easy to produce from Lua plugins later (`plugin.toml`, roadmap "Plugin manifest"),
- be validated by the server with Rust/serde,
- be rendered by the plain-JS custom-element client (`luma-button`, `luma-cell`, `luma-grid`, …),
- diff cleanly in git,
- be **one** format for built-in windows and for plugins.

This is research only. Nothing is decided and no code was written.

---

## 1. Survey

| Project | Format | Layout | Bindings / events | Scripting | Verdict for LumaCore |
|---|---|---|---|---|---|
| **A reference console's layout XML** (baseline) | XML, one root class per file | A layout grid. Rows and columns are Fixed / Stretch / Content sized items. Children are placed with column/row anchors. Also auto layout and margins | A property name on a target object. The cell type follows the property type. Signals name Lua functions | Lua builds the same tree imperatively | Its ideas are excellent: the grid model, cells typed by property, one generic settings layout. Its files are verbose (attribute soup, PascalCase class names, 1000+ classes), and the signals are Lua callbacks, not commands. **Keep the ideas, not the syntax.** |
| **WoW FrameXML** — [wiki](https://warcraft.wiki.gg/wiki/XML_user_interface), [source mirror](https://github.com/Gethe/wow-ui-source) | XML with an XSD (`UI.xsd`), plus `.lua` files listed in a `.toc` | Anchors (`<Anchor point="TOPLEFT" relativeTo=…>`) and sizes. No grid | `<Scripts><OnClick>` holds inline Lua or `function="Name"`. Templates use `inherits="…"` and `mixin` | Addons mix XML templates with Lua `CreateFrame(type, name, parent, template)` | This is the classic Lua + XML split. It shows the danger: over time the community moved almost all UI into Lua code, because the XML only did half the job (the behaviour lived in Lua). Templates and inheritance are worth copying. An XSD is a good way to validate. |
| **Roblox** — [UI docs](https://create.roblox.com/docs/ui), [react-lua](https://github.com/jsdotlua/react-lua), [Fusion](https://github.com/dphfox/Fusion) | No file. Instances are built from Luau tables: `React.createElement("TextButton", {props}, {children})` | `UIGridLayout`, `UIListLayout`, scale + offset `UDim2` | Props tables, `[React.Event.Activated] = fn`, state hooks | All Lua | Shows that **a Lua table is a natural UI tree** (type, props, children). The behaviour lives in closures, which we cannot serialize or validate. Take the table shape only. |
| **Defold GUI** — [manual](https://defold.com/manuals/gui/) | `.gui` files in protobuf text format, written by the editor | Nodes with pivot, anchors, adjust mode. Box, text, pie, template nodes | A `.gui_script` (Lua) handles `on_input` and messages | Lua manipulates nodes by id | Editor-generated text that diffs OK but isn't meant to be hand-written. Templates (sub-scenes) are good. Too low-level (pixels and pivots). |
| **LÖVE UI libs** — [SUIT](https://github.com/vrld/SUIT), [Slab](https://github.com/flamendless/Slab) | No format. Immediate-mode Lua | Row/column layouter (SUIT `layout:row`) | `if suit.Button("x").hit then` | All Lua | Nothing declarative to copy. Useful only as a contrast (see egui). |
| **GMod Derma** — [guide](https://wiki.facepunch.com/gmod/Derma_Basic_Guide) | No format. Imperative Lua `vgui.Create("DButton", parent)` | `Dock(FILL/TOP)`, `DockMargin` | `function btn:DoClick() … end` | All Lua | Docking is a simple, readable layout model (a Lua user knows it). Being imperative-only makes it hard to diff or validate. |
| **REAPER** — [WALTER](https://www.reaper.fm/sdk/walter/walter), [ReaImGui](https://github.com/cfillion/reaimgui) | WALTER: a line-based expression language in `rtconfig.txt`. ReaImGui: Dear ImGui from Lua | WALTER: `set tcp.mute [x y w h ls ts rs bs]` rects with attach scalars | WALTER has no events (it only skins fixed controls). ImGui: `if ImGui.Button(ctx,"x")` | ReaImGui scripts | WALTER is famously cryptic. It is a warning case for user readability: expressive but unreadable. ReaImGui shows that Lua users accept immediate mode for tools, but that gives no stored layout. |
| **Open Stage Control** — [docs](https://openstagecontrol.ammd.net/docs/) | JSON session: `{session:{type:"root", widgets:[…], tabs:[…]}}` | Absolute top/left/width/height or panel `layout: vertical/horizontal/grid` + `gridTemplate` | `address`, `target`, `#{…}` templated props, `onValue` JS scripts | JS snippets inside props | A close cousin: a control surface, with every widget sending a message (our "command line"). The JSON is machine-written by the editor. Its huge default-property lists make diffs noisy, so **we should store only the non-default props**. The `#{$+1}` templating is a good fit for our `{field}` substitution. |
| **TouchOSC** — [manual](https://hexler.net/touchosc/manual), [tosclib](https://github.com/AlbertoV5/tosclib) | `.tosc`: compressed XML. Each node has properties, values and messages | Absolute frames, grids (`GRID` control) | Per-control OSC/MIDI messages. Lua `onValueChanged` per control | Lua 5.1 per control | A compressed binary file defeats git. Its per-control **message list** (what the control sends) is the same thing as our action. The Lua attached to each control is too fine-grained for us. |
| **Bitfocus Companion** — [repo](https://github.com/bitfocus/companion) | Internal JSON / `.companionconfig` export. Buttons on pages | A fixed grid of keys (pages × rows × cols) | Each button has `steps` → `actions` (press/release), `feedbacks` bound to variables | Modules in JS | It proves that "a grid of cells, each with press/release actions and feedback bindings" is enough for live operators. This matches our keys (§17). Its config is not meant for hand editing. |
| **Qt QML** — [reference](https://doc.qt.io/qt-6/qmlreference.html) | Its own declarative language: `Item { property: value; Child {} }` | `GridLayout`, `Layout.fillWidth`, anchors | Property bindings are live JS expressions; `onClicked: {…}` | JS | The nicest syntax for humans: nested blocks and `name: value`. But it embeds a full JS engine (bindings are code). We should copy its *look* (nested blocks), not its expression semantics. |
| **GTK Builder / Blueprint** — [GtkBuilder](https://docs.gtk.org/gtk4/class.Builder.html), [Blueprint](https://gnome.pages.gitlab.gnome.org/blueprint-compiler/) | Builder: XML. Blueprint: a terse block language compiled to that XML | `Gtk.Grid` with `layout { column: 1; row: 0; }` | `clicked => $on_click();` signals name handlers; `bind` property bindings | Any language via GObject | **Important lesson:** GNOME wrote Blueprint because users hated hand-writing Builder XML. That is a direct data point against XML for user readability. Blueprint's `layout {column,row}` per child is a good grid syntax. |
| **Avalonia AXAML** — [docs](https://docs.avaloniaui.net/docs/basics/user-interface/introduction-to-xaml) | XAML | `<Grid ColumnDefinitions="Auto,*,120">` | `{Binding Path}`, commands (`Command="{Binding Save}"`) | C# | **The best grid shorthand anywhere:** `"Auto,*,120"`, which equals Content, Stretch, Fixed. Its `Command` binding equals our command line. XAML namespaces and markup extensions are heavy. |
| **Flutter / Jetpack Compose** | Code (Dart/Kotlin) building a widget tree | `Row`, `Column`, `Expanded`, `Flexible` | Callbacks | Host language | This is code, not a file format. It can't be validated or diffed as data. It is only relevant as the model behind react-lua. |
| **JSON Forms** — [UI schema](https://jsonforms.io/docs/uischema/) / **RJSF** — [docs](https://rjsf-team.github.io/react-jsonschema-form/docs/) | JSON. The **data schema** (types, ranges) is kept apart from the **UI schema** (`VerticalLayout`, `Control` with `scope: "#/properties/x"`, `Categorization` = tabs) | Vertical/Horizontal/Group/Categorization | `scope` = a JSON pointer binding. The widget comes from the type. `rule` = show/hide/enable conditions | None (host renders) | **Very relevant.** This is "cell type follows property type" in an open format. Our data schema already exists: `TreeDef::write` / `core/props.rs`. So the layout only needs to name a path, and the server fills in the widget, range and rights. Rules (`SHOW` when …) are worth adopting later. |
| **Adaptive Cards** — [explorer](https://adaptivecards.io/explorer/), [Universal Actions](https://learn.microsoft.com/en-us/adaptive-cards/authoring-cards/universal-action-model) | JSON with `"type"` on every element and a top-level `"version": "1.6"`, published JSON Schema | `ColumnSet`/`Column width:"auto"\|"stretch"\|"50px"`, `Container`, `Table` | `Input.*` with `id`. `Action.Execute`/`Submit` collect inputs and send them to the host. `${…}` templating (separate data). `fallback` per element for older hosts | None. The **host renders natively** with its own host config (theme) | **The closest architectural match:** host-rendered, a fixed catalog, a versioned schema, inputs gathered into an action, and `auto/stretch/px` columns (the same as Fixed/Stretch/Content). Copy: the `type` + `version` + catalog + fallback discipline, and inputs referenced by id in actions. |
| **Google A2UI** — [repo](https://github.com/google/A2UI), [a2ui.org](https://a2ui.org/) | JSON (v0.8/0.9). A **flat list of components referencing children by id**, plus a separate data model and bindings by path | Row/Column/List/Card from a client **catalog** | Bindings by data path. Actions are sent back to the agent | Agents (LLMs) generate it | It confirms the catalog + path-binding + host-render pattern. The flat id-list exists for incremental streaming by LLMs. It is worse for human reading than a nested tree. |
| **Slint** — [docs](https://docs.slint.dev/) | `.slint` markup compiled by a Rust crate | `GridLayout { Row { … } }`, `HorizontalLayout`, `min-width`, `horizontal-stretch` | `<=>` two-way bindings, `callback clicked` | Rust/C++/JS/Python | It shows that a Rust project can own a clean UI DSL. It is compiled (not runtime data) and has its own expression language, which is too big for user/plugin files. `Row {}` inside `GridLayout` is a readable grid syntax. |
| **egui** — [repo](https://github.com/emilk/egui) | None (immediate mode, Rust) | `ui.horizontal`, `Grid::new` | `if ui.button().clicked()` | Rust | A contrast. Immediate mode gives no file, no diff, no server validation, and doesn't fit "server owns state". Rejected. |
| **KDL** — [kdl.dev](https://kdl.dev), [spec v2](https://github.com/kdl-org/kdl/blob/kdl-v2/SPEC.md), [kdl-rs](https://github.com/kdl-org/kdl-rs) | A node document: `name arg key=value { children }`, comments, raw strings `#"…"#` | n/a (a data language) | n/a | Rust `kdl` 6.x (v2 + v1, format-preserving). Derive crates: [knus](https://docs.rs/knus) (KDL v1-era), [facet-kdl](https://docs.rs/facet-kdl) (wip), [knurdy](https://lib.rs/crates/knurdy) (serde, small) | **Used for UI layouts in practice:** [Zellij layouts](https://zellij.dev/documentation/creating-a-layout.html) (`pane split_direction="vertical" size="50%" { … }`) and niri's config. Its node = type + args + props + children shape is exactly a UI tree. **Risk:** serde support for v2 is immature. Plan a small generic mapper `KdlNode → UiNode` instead of derive. |
| **Lua-table DSLs** — [nui.nvim](https://github.com/MunifTanjim/nui.nvim) (Neovim `Layout.Box({...}, {dir="row", size="50%"})`), [lualine config](https://github.com/nvim-lualine/lualine.nvim) | Lua tables: the array part = children, the hash part = props | Box with `dir` + `size` (absolute or %) | Callbacks in tables | Lua | This shows that the Lua community writes layout as nested tables happily. Mapping rule: **hash part → props, array part → children, `[1]`-string → shorthand arg**. With functions banned, a table converts 1:1 to JSON. |

### Cross-cutting lessons
1. **Grid tracks as a short string** (Avalonia `"Auto,*,120"`, Adaptive Cards `auto/stretch/px`, the console's Content/Stretch/Fixed) are the most readable part of every format. Auto-flow placement plus an optional `at` / `span` is easier than mandatory anchors.
2. **Typed by the bound property** (the console's property inputs, JSON Forms `scope`): the layout names a tree path, and the server supplies the type, range, enum, rights and read-only state from `TreeDef::write`. The file stays small and can't lie about types.
3. **Actions as data, never code.** Code in layouts (WoW `OnClick`, QML JS, TouchOSC Lua, OSC JS) is what makes formats unvalidatable. Ours already has the answer: a command line with `{input}` substitution, run through dispatch (rights gate, undo, replication) like any gesture.
4. **Versioned schema + catalog + fallback** (Adaptive Cards, A2UI) let the server refuse unknown widgets with a coded error and let old clients degrade.
5. **XML loses on human writing** (GNOME → Blueprint, WoW addons → Lua). **JSON loses on human reading** (quotes, commas, no comments). Editors store huge default lists (Open Stage Control, TouchOSC), so **store only non-defaults.**

---

## 2. Options (one example dialog)

The dialog: title "Station Settings", tabs General and Network, a grid with a swipe cell bound to `Station.OutputWhenAlone`, a number cell with keypad (`Station.Priority`), and a button that runs `Store Network "{name}"` using a text input `name`.

### A. Console-style XML (`*.ui.xml`)
```xml
<dialog version="1" title="Station Settings">
  <tab title="General">
    <grid columns="* *" rows="auto">
      <swipe bind="Station.OutputWhenAlone"/>
      <number bind="Station.Priority" keypad="yes"/>
    </grid>
  </tab>
  <tab title="Network">
    <grid columns="* 120">
      <text id="name" label="Network name"/>
      <button label="Store" run='Store Network "{name}"'/>
    </grid>
  </tab>
</dialog>
```
Pros: familiar to console users, an XSD is possible, and attributes with children are natural. Cons: `quick-xml`+serde handles mixed attribute/child models clumsily; Lua tables don't map naturally (attributes vs children); users dislike writing it (the Blueprint lesson); the quoting of `run` gets ugly.

### B. KDL (`*.ui.kdl`)
```kdl
dialog "Station Settings" version=1 {
    tab "General" {
        grid columns="* *" {
            swipe bind="Station.OutputWhenAlone"
            number bind="Station.Priority" keypad=#true
        }
    }
    tab "Network" {
        grid columns="* 120" {
            text id="name" label="Network name"
            button "Store" run=#"Store Network "{name}""#   // raw string: no escaping
        }
    }
}
```
Pros: the most readable (it reads like QML/Blueprint without the code); comments; raw strings for command lines; one node per line gives clean diffs; Zellij proves it works for layouts; a Lua table maps 1:1 with "array part = children, `[1]` = arg". Cons: less known; Rust serde for v2 is immature (use `kdl` 6 + a ~200-line generic mapper); the browser needs a parser, or the server sends JSON (recommended: the server always sends JSON).

### C. JSON (Adaptive-Cards style, `*.ui.json`)
```json
{ "type": "dialog", "version": 1, "title": "Station Settings", "children": [
  { "type": "tab", "title": "General", "children": [
    { "type": "grid", "columns": "* *", "children": [
      { "type": "swipe",  "bind": "Station.OutputWhenAlone" },
      { "type": "number", "bind": "Station.Priority", "keypad": true } ] } ] },
  { "type": "tab", "title": "Network", "children": [
    { "type": "grid", "columns": "* 120", "children": [
      { "type": "text",   "id": "name", "label": "Network name" },
      { "type": "button", "label": "Store", "run": "Store Network \"{name}\"" } ] } ] } ] }
```
Pros: serde `#[serde(tag="type", deny_unknown_fields)]` is trivial; `schemars` gives a JSON Schema for editor autocompletion; the browser parses it natively; a Lua table → JSON with no rules. Cons: the worst for users to read or edit (quotes, commas, no comments, escaped command lines); diffs are noisy unless canonically formatted.

### D. Lua table as source (`*.ui.lua`, run sandboxed and returning data)
```lua
return ui.dialog { "Station Settings", version = 1,
  ui.tab { "General",
    ui.grid { columns = "* *",
      ui.swipe  { bind = "Station.OutputWhenAlone" },
      ui.number { bind = "Station.Priority", keypad = true } } },
  ui.tab { "Network",
    ui.grid { columns = "* 120",
      ui.text   { id = "name", label = "Network name" },
      ui.button { "Store", run = [[Store Network "{name}"]] } } } }
```
Pros: it is the plugin language itself (loops and conditionals for generated layouts), and `[[…]]` long strings are clean. Cons: built-in windows would need a Lua VM on the server today; a file that is code can't be validated without running it; table key order is lost (diffs of the exported form are unstable); users must read Lua.

---

## 3. Recommendation

**One data model, written as KDL on disk, sent as JSON on the wire, and built from plain Lua tables in plugins (option B, with C as its wire form).**

- **The model is the format.** A Rust `UiNode { kind, args, props, children }` (serde, a fixed catalog, `version`) is the one thing the server validates. KDL files, Lua tables and the JSON to the client are three spellings of the same tree, joined by one fixed mapping: node name = `kind`, positional arg = the widget's shorthand (title/label), `k=v` = props, block = children. Lua: `[1]` = arg, hash = props, the rest of the array part = children, and **functions are refused** (actions are command strings).
- **One format for built-in and plugins.** The built-in windows ship as `.ui.kdl` (generated reference docs list the catalog from the registry, like `gen-docs`). Plugins ship `.ui.kdl` files named in `plugin.toml`, or return the same tree from Lua when they need loops. Both go through the same validator.
- **The validator (server)** checks:
  - the catalog `kind` (unknown → coded error),
  - `bind` paths against `TreeDef::write` / `core/props.rs`, which also infer the cell type, range and rights (JSON Forms style),
  - `run` templates: `{id}` must name an input in scope; after substitution, values are quoted and escaped by the server and the line goes through normal dispatch, so the rights gate, undo and replication apply,
  - unique ids,
  - grid tracks (`*`, `auto`, `120`, `2*`).
- **Readability:** KDL reads like QML/Blueprint, and comments and raw strings `#"…"#` keep command lines unescaped. Store only non-default props, and write canonical formatting (`kdl` 6 preserves formatting), so diffs show only what the user changed.
- **Risks:**
  - KDL serde for v2 is immature. Mitigation: parse with `kdl` 6 into a `KdlDocument`, then walk it with a small generic mapper into `UiNode`; serde stays on the JSON side.
  - KDL is unfamiliar. Mitigation: it's tiny, and there is a one-page guide.
- **Fallback if KDL is rejected:** C (JSON only) is the safest engineering choice, with a schemars JSON Schema for editors. It costs user readability, and the format can be revisited later because the model doesn't change.
- **Avoid:** A, because XML is the readability lesson GNOME and WoW already learned. Avoid D as the stored format; keep Lua as a builder only.

Take from the baselines: the console's grid + property-typed cells; Avalonia's track string; Adaptive Cards' `version` + catalog + input-ids-into-action + `fallback`; JSON Forms' `rule` (show/enable when a bound value…) for later; Open Stage Control's "only non-defaults".

### Sources
- WoW: https://warcraft.wiki.gg/wiki/XML_user_interface · https://github.com/Gethe/wow-ui-source
- Roblox: https://create.roblox.com/docs/ui · https://github.com/jsdotlua/react-lua · https://github.com/dphfox/Fusion
- Defold: https://defold.com/manuals/gui/
- LÖVE: https://github.com/vrld/SUIT · https://github.com/flamendless/Slab
- Derma: https://wiki.facepunch.com/gmod/Derma_Basic_Guide
- REAPER: https://www.reaper.fm/sdk/walter/walter · https://github.com/cfillion/reaimgui
- Open Stage Control: https://openstagecontrol.ammd.net/docs/
- TouchOSC: https://hexler.net/touchosc/manual · https://github.com/AlbertoV5/tosclib
- Companion: https://github.com/bitfocus/companion
- QML: https://doc.qt.io/qt-6/qmlreference.html
- GTK: https://docs.gtk.org/gtk4/class.Builder.html · https://gnome.pages.gitlab.gnome.org/blueprint-compiler/
- Avalonia: https://docs.avaloniaui.net/docs/basics/user-interface/introduction-to-xaml
- JSON Forms: https://jsonforms.io/docs/uischema/
- RJSF: https://rjsf-team.github.io/react-jsonschema-form/docs/
- Adaptive Cards: https://adaptivecards.io/explorer/ · https://learn.microsoft.com/en-us/adaptive-cards/authoring-cards/universal-action-model
- A2UI: https://github.com/google/A2UI · https://a2ui.org/
- Slint: https://docs.slint.dev/
- egui: https://github.com/emilk/egui
- KDL: https://kdl.dev · https://github.com/kdl-org/kdl/blob/kdl-v2/SPEC.md · https://github.com/kdl-org/kdl-rs · https://docs.rs/knus · https://docs.rs/facet-kdl · https://lib.rs/crates/knurdy
- Zellij layouts: https://zellij.dev/documentation/creating-a-layout.html
- Lua-table DSLs: https://github.com/MunifTanjim/nui.nvim
