# Inspiration style — analysis (2026-10-12)

Source: the user's own console concept, `inspiration/` (gitignored): `A1 Whole Low Res.png`,
`ljusbord.png` (rendered desk), `A1 S1.png` / `A1 S2.png` / `A1 S3.png` (the three screens),
`A1 3DM.png` (desk detail). Colours sampled with Pillow (3×3 averages and histograms).
"I think the general ux concept and ui design is good. But it looks a little bit too much
vibecoded and is lacking some of the original design i have from inspiration / analyze this
style and colors. Dont change the design too much."

## Character

- **Matte, flat, neutral.** Pure greys with no blue tint, no gradients, no drop shadows, no
  glow. Depth only from a few grey steps.
- **One accent: olive gold.** Used sparingly: the dot on every tab, the screen frames, the
  highlight lane / column in sheets and timelines, encoder rings, fader cap lines. Everything
  else is grey. Status colours are small marks only (green online dot, red "changed" corner
  dot, blue / green markers on the timeline).
- **Large, light type.** Tabs and sheet text are big (≈ 18–24 px on the screen), regular
  weight, never bold; bright text only for the active tab and titles, mid-grey for content,
  dark grey for inactive.
- **Dense but calm.** Thin dark gaps (≈ #090909) separate windows instead of borders.

## Palette (sampled)

| Role | Hex | Where |
|---|---|---|
| Gap / deepest | `#090909` · `#0c0c0c` | between windows, scope / display background |
| Window background | `#121212` | pool area, option rows |
| Sheet / dark field | `#1c1c1c` · `#1e1e1e` | sheets, attribute boxes, tab strip under windows |
| Strip / header | `#242424` · `#262626` | title bar, tab strip, sheet rows |
| Cell / raised | `#303030` · `#353535` | pool cells, active tab, checkboxes |
| Line / key | `#424242` · `#484848` | cell borders, executor frames, hardware keys `#3e3e40` |
| Pressed / selected button | `#606060` · `#6c6c6c` | active transport button, Select |
| Selected row | `#9c9c9c` (dark text) | sheet row selection |
| Text dim | `#606060` · `#787878` | inactive tabs, numbers in cells |
| Text | `#9c9c9c` | sheet content |
| Text bright | `#d1d1d1` | active tab, titles, clock |
| Gold (dot) | `#7c6200` | tab dot, active; `#312600` inactive dot |
| Gold lane | `#1e1c06` · `#383309` | highlighted column / lane |
| Gold frame | `#645c49` · `#66604e` | screen frames |
| Gold light | `#ccc085` | encoder rings |
| Fader line | `#f9ed4f` | fader cap line |
| OK | `#009b09` | online dot |
| Alert / changed | `#e10000` | red corner dot on changed values |
| Marker blue | `#758ba0` · `#002a54` | timeline markers |

## Shapes and parts

- **Tabs:** text tabs with a round gold dot at the left (≈ 14 px); the active tab has a
  `#353535` rounded-top background and bright text, inactive tabs only dim text and a dark
  dot; a small square `+` after the tabs (add window).
- **Option row under the tabs** = our **window bar**: small square checkboxes
  (`#313131`, ≈ 16 px) with labels ("Preset Sort", "Direct Select", "Fixture Sort", "Mask",
  "Icons") and drop-downs ("Sort by: Truss ▾") on the window background — no button boxes.
- **Pool cells:** rounded squares (radius ≈ 8 px) `#353535`, flat, the number tiny and dim in
  the top-left; empty cells drawn the same.
- **Attribute / encoder boxes:** label on top, value large below, dark field, red corner dot
  when changed.
- **Executor cells:** thin `#454243` frames on the background, small "6.1" number and name.
- **Buttons (transport, tools):** square-ish, flat grey `#262626`, the active one lighter
  `#606060` with a light border; disabled = dim text, same box.
- **Desk:** dark matte body, hardware keys `#3e3e40`, yellow fader cap lines, gold-ringed
  encoders, an orange-brown jog ring; screens framed in gold.

## What the first mockups got wrong ("vibecoded")

Blue-tinted greys, bright saturated amber, coloured tab dots per window type, card shadows,
12–14 px radii, uppercase letter-spaced section labels, bold weights, buttons in boxes for the
window bar. Fix: the palette above as the Dark preset, gold only as an accent, flat surfaces,
radius 4–8 px, regular weights, sentence-case labels, checkbox / drop-down window bar, gold
dots on every tab.
