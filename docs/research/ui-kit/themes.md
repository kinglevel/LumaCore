# Theme editing research for LumaCore

Date: 2026-10-12. Read-only research; no project files changed and no live console contacted.

The user's brief: *"Other consoles have too many color items per theme and it is very confusing for the user. I like how REAPER DAW makes theme edits. Themes are stored on the user."*

---

## 0. Where LumaCore stands today

- `apps/lumacore-app/frontend/index.html` `:root` has **9 tokens**: `--bg #191a1e`, `--chrome #212228`, `--chrome2 #292932`, `--border #2d2d3a`, `--dim #606070`, `--text #c8c8cc`, `--amber #e8a000`, `--green #3ec83e`, `--red #cc3333`, plus `color-scheme: dark`.
- The same file has about **97 raw `#rrggbb` literals** and 216 `var(--…)` uses. The raw literals include the xterm `TERM_OPTS.theme` (line ~1026, its own 18 colours), `BAR_TERM` (`#26262c`) and a comment tying the status bar colours to an outside console's theme group.
- Pool window settings (`crates/show/lumacore-tree/src/window_types.rs`, `pool_settings`) already have colour settings with the hint `"#rrggbb; empty = theme"` (`empty_color`, `for_all_color #3c8c3c`, `for_some_color #c8a02a`, `for_none_color`).
- `docs/meta/roadmap.md:330` lists the menu tile "Desk Lights & Color Theme" as still grey. `docs/archive/decisions-2026-10.md:152` lists an old `ColorTheme` node under Root in the original tree sketch.
- The app shell is **wry**: WKWebView on macOS, WebView2 on Windows, WebKitGTK on Linux. Newer CSS colour features (relative colour syntax, some OKLCH behaviour) vary by engine, so derived colours are safest **computed in JS** and written as plain hex onto `:root`.
- `User` (`ShowData.Users.{n}`, `crates/show/lumacore-tree/src/programmer.rs:605`) owns the user's own work (programmer, selection). `UserProfile` (line 978) holds "how they work" (views, screens, keys, preferences).

---

## 1. REAPER: how theme editing works

REAPER has **three layers** of theme editing. Each goes deeper than the one before, and most users never leave the first.

### 1a. Theme Adjuster (Options > Themes > Theme adjuster) — the approachable layer

- REAPER 6 shipped a Default 6.0 Theme Adjuster (a Lua script by White Tie). It shows or hides buttons, picks layouts A/B/C, and sets name widths, folder indents and the "overall contrast and brightness" for the track panel (TCP), mixer (MCP) and transport. Settings go to **`reaper-themeconfig.ini`**, separate from the theme itself ([reaper.blog: Customizing the REAPER 6 theme](https://reaper.blog/2020/01/custom-reaper-6-theme/), [reapertips: a tool for customizing REAPER 6 theme](https://www.reapertips.com/post/a-tool-for-customizing-reaper-6-theme)).
- REAPER 7.17 (June 2024) shipped the **Default 7 Theme Adjuster** ([reaper.blog: Ultimate REAPER UI control with the new Default 7 Theme Adjuster](https://reaper.blog/2024/06/theme-adjuster/)). It is a window with a **sidebar of sections and a resizable main area** ([reapertips: The Theme Adjuster for REAPER 7 — a comprehensive guide](https://reapertips.com/post/the-theme-adjuster-for-reaper-7-a-comprehensive-guide)):
  - **Global Adjustments**: track colour strength (how strongly the custom track colour tints panels), background shade, label colouring, global text brightness, Track Selection appearance, Custom Colour Palettes, and **Color Processing**, which adjusts "the gamma, brightness, and contrast of the entire REAPER interface".
  - **Track Panel (TCP)**: element order, Visibility Programmer (which control shows in which track state), section assignment left/right with min/max widths, Layouts A/B/C.
  - **Envelope Controls**: background colour, label/fader length, divider opacity.
  - **Mixer (MCP)**: background, divider opacity, folder indent, three track states (unselected/selected/armed), width-based layouts (standard/reduced/strip).
  - **Reset everything to default**, plus **export/import** of the settings.
  - Changes are **live** in both the adjuster and REAPER.
- The global colour sliders are **Shadows, Midtones, Highlights, Saturation and Tint** (with gamma/brightness/contrast). A v7.17rc5 bug thread names all five and notes their neutral values are 0/100, stored in `reaper-themeconfig.ini` ([Cockos forum v7.17rc5 thread](https://forum.cockos.com/showthread.php?t=292327); the forum blocks automated fetch, so this is from search excerpts).
- The adjuster's knobs are **declared by the theme** (WALTER `rtconfig` parameters with name, description, default, min, max; "the default theme adjuster script will allow tweaking the first 10 parameters"). The theme author chooses the few things that are worth adjusting ([WALTER reference](https://www.reaper.fm/sdk/walter/walter)).

### 1b. Theme development / tweak window (Actions > "Theme development: Show theme tweak/configuration window")

- This is the **full list of colour items** (hundreds; e.g. "Toolbar button text", "Toolbar button armed color", "…armed fill mode", with a blend mode and alpha on some). You click a swatch to get a colour picker, and the change shows live. The REAPER SDK itself recommends it: "for many edits it is easier to do on the fly, by using the action 'Theme Development: Show theme tweak/configuration window'" ([reaper.fm theme development reference](https://www.reaper.fm/sdk/walter/images.php)).
- **Save theme** writes a `.ReaperTheme` text file in `ColorThemes/`. Distributed themes are `.ReaperThemeZip` (the `.ReaperTheme` plus images and `rtconfig.txt`) ([reapertips: how to tweak any REAPER theme](https://reapertips.com/post/how-to-tweak-any-reaper-theme)).
- A forum member draws the line clearly: "Theme development/tweaker is for adjusting color settings of the theme. Those changes are saved into xxx.ReaperTheme… if theme has an attached theme adjuster script, more adjustments will become available" ([Cockos forum: Theme development/tweaker vs Theme Adjuster/Color Control](https://forums.cockos.com/showthread.php?t=265643), search excerpt).
- **Theme element finder** (Actions, filter "Theme development"): "The first drop down in the Theme element finder will highlight the areas set by each color assignment." It goes both ways: you pick a colour item and REAPER flashes the screen areas that use it (same thread, search excerpt). This is the **"which colour is this?"** answer that the console theme editors lack.

### 1c. Hand-editing / community tools

- `.ReaperTheme` and `rtconfig.txt` are plain text (ABGR hex, e.g. `tcp_vol_zeroline FFFF0000`).
- Amagalma's **Adjust Theme Colors** script ([reaper.blog 2020](https://reaper.blog/2020/04/amagalma-theme-colors/)) shows the pattern from the brief almost exactly:
  - brightness/contrast/gamma sliders applied to **selected colour groups** only;
  - **Take Snapshot** makes the current state the new base;
  - snapshots in a list (Alt-click deletes);
  - undo/redo;
  - an **A/B button** that toggles between the current settings and the original colours;
  - a prompt to save on exit.
- It has a [YouTube demo](https://www.youtube.com/watch?v=Bnyx9ojkyJA). The page has no screenshot alt text; the video shows a small window with three sliders, a group checklist and a snapshot list.

### What makes REAPER approachable

1. **A few global controls first.** Brightness/contrast/gamma, tint, saturation, track-colour strength and selection colour change the whole look in seconds.
2. **Live preview everywhere.** Nothing needs an Apply step in the adjuster.
3. **Detail only on demand.** The hundreds of colour items sit behind a separate developer action, not in the main settings.
4. **The theme declares what can be tweaked**, so the adjuster stays short.
5. **Tweaks are kept apart from the theme** (`reaper-themeconfig.ini` vs `.ReaperTheme`). Updating or switching the base theme keeps the tweaks, and "reset" means "drop my tweaks".
6. **Find by pointing** (element finder).
7. **Safety nets**: reset all, export/import, A/B, snapshots.

Limits of these sources: screenshots on the reapertips and reaper.blog pages have no alt text, and the Cockos forum and wiki block automated fetch. The control names above come from guide text and forum excerpts. The exact slider list of Default 7's Color Processing panel should be checked in REAPER itself before quoting it in docs.

---

## 3. A per-use colour list — what to avoid

A reference console's theme holds over a thousand colour entries at two levels: a palette of a few hundred colour definitions named per use (`Text`, `TextDark`, `SelectedCellBackground`, …), then one colour group per UI class, and about a quarter of the entries carry a raw colour with no palette reference.

Why it confuses:
1. About 1,400 editable items at two levels with no hierarchy shown in the editor.
2. Palette names describe *where* a colour is used, not *what it means*, so one "concept" (selected) is spread over many defs.
3. Raw colours bypass the palette, so changing the palette never fully re-themes the UI (Daylight is a whole second 156 KB file, not a transform).
4. Nothing on screen says which item an element uses.
5. No global adjusters: making everything 10 % brighter means editing hundreds of values.

**Rule for LumaCore: no raw colours outside the theme module, every role derived from the palette, and an editor that never shows more than the palette unless asked.**

---

## 4. Other approachable models (brief)

- **VS Code** — `workbench.colorCustomizations` in settings.json overrides only the keys you name on top of the active theme, optionally scoped per theme (`"[Theme Name]": {…}`). The theme has 600+ colour ids ([theme colour reference](https://code.visualstudio.com/api/references/theme-color)), but users touch three or four. Borrow: **store overrides, not the whole theme**. Its weak spot is that there is no visual editor and no picker for the ids.
- **Ableton Live 12** — Settings > Theme & Colors: a theme dropdown plus **Brightness, Color Intensity, Color Hue** and **Grid Line Intensity** sliders (a community theme lists its settings as "Grid Line Intensity 15 %, Brightness 200 %, Color Intensity 0 %, Color Hue 205°", [gregsilver](https://gregsilver.gumroad.com/l/Ableton12DarkTheme)). Full themes are `.ask` XML files dropped into a folder. This is the cleanest example of "a few global sliders over a fixed theme". Third-party editors (e.g. [LiveThemes](https://livethemes.featurebase.app)) exist because the XML is long.
- **Bitwig** — a fixed dark UI with few user colour controls (mostly track/clip colours). It is not a model for editing.
- **Material Theme Builder / Material 3 dynamic colour** — one **seed colour** → 5 key colours (primary, secondary, tertiary, neutral, neutral-variant, plus error) → **tonal palettes** (tones 0–100 in the perceptual HCT space) → about 30 **roles** (`primary`, `onPrimary`, `primaryContainer`, `surface`, `surfaceContainerLow…Highest`, `outline`, …). Light and dark come from the *same* palettes at different tones, with standard/medium/high contrast levels ([M3: how the colour system works](https://m3.material.io/styles/color/system/how-the-system-works), [material-color-utilities](https://github.com/material-foundation/material-color-utilities)). Borrow: **roles are computed from a small seed set, and light/dark are two mappings of the same palette.**
- **Radix Colors** — every hue is a **12-step scale with fixed jobs**: 1–2 app/subtle backgrounds, 3–5 component bg (default / hover / pressed-selected), 6–8 borders (subtle / interactive / strong + focus), 9–10 solid fills (9 = purest, 10 = its hover), 11–12 low/high-contrast text, tuned to APCA contrast on step 2 ([Radix: understanding the scale](https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale)). Borrow: **the step-to-job mapping for the neutral ladder**, so a role like "button hover" is "neutral step 4", never a free colour.

---

## 5. Proposed LumaCore theme model

### 5.1 Three tiers, only the first is user-facing

```
Preset (Dark | Daylight | Night, built in, read-only)
  + user overrides (palette tokens the user changed)        ← stored on the user
  + adjusters (brightness, contrast, saturation, tint, accent) ← stored on the user
        │  derive()  (pure JS function, OKLCH maths, deterministic)
        ▼
Palette (14 resolved tokens)
        │  roles()   (fixed table: role = token + step/alpha)
        ▼
Semantic roles (~50 CSS custom properties on :root, e.g. --btn-bg-hover)
        ▼
Components use only roles (never palette tokens, never raw hex)
```

### 5.2 Palette — 14 user-facing tokens (two groups)

**Base (8)**

| Token | Job | Dark default (today's value) |
|---|---|---|
| `background` | app backdrop, gaps | `#191a1e` (`--bg`) |
| `surface` | window/panel body, chrome | `#212228` (`--chrome`) |
| `raised` | buttons, rows, cells | `#292932` (`--chrome2`) |
| `line` | borders, dividers, grid lines | `#2d2d3a` (`--border`) |
| `text` | primary text | `#c8c8cc` |
| `text-dim` | labels, secondary text, disabled | `#606070` (`--dim`) |
| `accent` | selection, focus, active tab, "the console colour" | `#e8a000` (`--amber`) |
| `info` | links, neutral highlights, terminal blue | `#4499ff` (from TERM_OPTS) |

**Status and console states (6)**

| Token | Job |
|---|---|
| `ok` | online, running, success (`#3ec83e`) |
| `warn` | warnings, pending, claims |
| `alert` | errors, Clear/Delete, offline (`#cc3333`) |
| `programmer` | active programmer values (consoles commonly show these red) |
| `blind` | Blind mode / blind values |
| `park` | parked / frozen / solo-locked output |

The step tokens (`hover`, `pressed`, `selected-bg`, `alt-row`, `focus-ring`, `on-accent`, `scrollbar`, …) are **not** in the palette. They are derived as Radix-style steps from `background → raised` (neutral) and from `accent` (accent scale). The editor can offer "show derived roles" read-only.

### 5.3 Semantic roles — derived automatically

- A fixed table in one JS module (`theme.js`, or a `<script>` section). Examples:
  - `--btn-bg = raised`
  - `--btn-bg-hover = step(raised, +1)`
  - `--btn-bg-active = mix(raised, accent, 25 %)`
  - `--btn-text-active = accent`
  - `--cell-alt = step(surface, +0.5)`
  - `--focus = accent`
  - `--on-accent = auto black/white by contrast`
  - `--exec-running = ok`
  - `--sheet-value-prog = programmer`
  - `--pool-for-some = warn`
  - `--term-*` (xterm theme built from the palette, applied with `term.options.theme = …`)
- Steps are taken in **OKLCH lightness**, not HSV. HSV round-trips drift and step unevenly, and OKLCH keeps the steps visually equal for any tint.
- Computed in JS and written as hex with `document.documentElement.style.setProperty`. This avoids relying on CSS relative-colour support across WKWebView, WebView2 and WebKitGTK, and the same function can run in a test.
- **Contrast guard**: after derivation, check `text`/`text-dim`/`on-accent` against their backgrounds (WCAG ratio, or APCA as in Radix). If below the minimum, show a warning on the token and the adjuster ("Text on Raised: 3.1:1"). It does not auto-clamp, so the user stays in control, but the Daylight preset passes.
- **Tidy rule** (in the house style): no `#rrggbb`/`rgb(` in frontend CSS/JS outside the theme module and the vendored libs, and no palette token used directly by a component, only roles. The ~97 literals today are the migration list.

### 5.4 Global adjusters (the REAPER/Ableton first screen)

Five sliders, all **neutral = no change**, each with a reset dot:

| Adjuster | Range | Effect (OKLCH) |
|---|---|---|
| Brightness | −50…+50 | shifts L of the neutral ladder (bg/surface/raised/line); text follows to keep contrast. Low values make a dark-booth "night" UI. |
| Contrast | 50…150 % | spreads the ladder's L steps apart and pushes text away from bg |
| Saturation | 0…200 % | scales C of accent and status tokens (0 = monochrome status, useful under coloured stage light) |
| Tint | hue 0–360° + amount 0–100 % | gives the neutrals a hue cast (cold blue greys, warm greys) |
| Accent | colour pick (or hue wheel) | sets `accent`; the accent steps follow |

Maybe later: a **UI dimmer** shortcut (brightness only), because consoles need quick "darker for the show" changes. It could be a key/command.

### 5.5 The editor window (one window type, every gesture a command line)

Layout, following REAPER's "few controls first":

1. **Top**: preset selector (Dark / Daylight / Night), **A | B** toggle, Import, Export, Reset all.
2. **Adjusters**: the 5 sliders.
3. **Palette**: 14 swatches in two rows (Base, States). A dot marks a token overridden from the preset, and each has a ↺ per-item reset. Clicking a swatch opens a picker (OKLCH L/C/H sliders + hex field + "from preset" value).
4. **Pick on screen** (eyedropper button, REAPER's element finder):
   - Click any element in any LumaCore window.
   - The finder walks the element's matched CSS rules (`el.matches(rule.selectorText)`), collects the `var(--role)` names used for background/color/border, and maps each role back to its palette token through the role table.
   - It then shows "This: Button (active) → background **Raised** + accent 25 %, text **Accent**" with the tokens highlighted for editing.
   - The reverse also works: hovering a token flashes every element using it, the way REAPER's finder highlights areas per colour item.
5. **Details (collapsed)**: the derived role list, read-only with values, for the curious. It is never editable; the per-use console model's mistake is making this level the editor.

Live preview while dragging stays on the client: CSS vars update every frame. **One command on release** files exactly one Oops step and one replication op per gesture.

### 5.6 A/B compare

Two slots, A = "the theme as it was when the editor opened", B = the working state. The toggle swaps them live. "Keep A" or "Keep B" commits. It is per editor session and not stored. Named **snapshots** (as in Amagalma's script) are left out: import/export plus A/B cover them with less UI. Add them only if users ask.

### 5.7 Storage — on the user

- Field: `User.theme: UserTheme` on `ShowData.Users.{n}`. It is **replicated and saved with the show**, follows the user to every station and client, and is readable in the tree (`Root.ShowData.Users.<n>.Theme`).
- Shape (store **overrides**, as VS Code does, so preset improvements reach the user):
  ```json
  { "preset": "Dark",
    "adjust": { "brightness": 0, "contrast": 100, "saturation": 100, "tint_hue": 220, "tint_amount": 0 },
    "palette": { "accent": "#e8a000" } }
  ```
  `palette` holds only changed tokens; the accent adjuster writes `palette.accent`. Unknown token names are refused on load. Under the house rules there are no migrations, and the forward-only format rule applies.
- **Presets are built in** (code, named GUIDs via `Guid::named`), not show data. Dark = today's `:root`. Daylight = high-L ladder with dark text; its accent/status hues are re-tuned for light backgrounds, not inverted (see Material: same hues, other tones). Night = a very dark, low-saturation, red-shifted variant for dark booths.
- **Undo / rights / change path**: writes go through `Set` on tree properties (`TreeDef::write` declares `Theme.*` with type/range), so the automatic change path covers undo, replication and journal. The Oops category is **UI** (refine in `dispatch::step_category`). A user may edit their own theme; Admin may edit anyone's. These are ordinary rights through the one gate.
- **Import/Export**: `Export Theme "name"` writes the JSON above to `library/themes/<name>.json` through `core::drive`; `Import Theme "name"` reads it. Both use the existing file reach rules. A theme file carries no GUIDs except the preset name, so it can move between shows.
- Pool window colour settings (`"#rrggbb; empty = theme"`) should accept **token names** (`accent`, `ok`, …) as well as hex, so a user's theme change also reaches them.

### 5.8 Open questions for the user

1. **User vs UserProfile.** Decisions §10 put "how they work" (views, screens, keys, preferences) on the *profile*, and own work on the *user*. The brief says "stored on the user". Should two users sharing one profile have different themes? This proposal says yes (on `User`).
2. **Station override?** A FOH station in daylight and a dark booth station may want different presets for the same user. Possible answer: a station-local "force Daylight / Night" setting that only swaps the preset and keeps the user's overrides. Never replicated.
3. **Command words**: `Theme` as an object (`Set Theme Accent "#e8a000"`, `Reset Theme Accent`, `Theme Daylight`) vs plain `Set User … Theme.…` paths. Check the registry for `Reset`/`Import`/`Export` before naming; every word must resolve uniquely (§15).
4. Should the status/state colours (programmer, blind, park) be per user, or show-wide so that every operator reads "red = programmer" the same way? This proposal allows per user, but Daylight could tune them.
