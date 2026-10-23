# Object paint (2026-10-23)

Every show object already carried a Paint in its meta ([0006](../../../../decisions/0006-node-meta-levels.md)),
typed as hex and drawn only in pool cells. These rounds proposed a picker, words for colours,
painting many objects, where paint shows, colour from tags and theme paint slots. Every mock is
drawn with the kit's own `tokens.css` and `kit.css`.

| File | What it asks |
|---|---|
| `01-paint.html` | Round 1: a named-swatch picker (Stage and Desk palettes), painting many, where paint shows (pools, the Tree window and NAV, three ways to mark a list row, the Device Sheet name), which objects, tags. |
| `02-paint.html` | Round 2: one Palette ⇄ Free picker (12 hues × 4 tones + greys, the base hues named; a saturation / value square and a hue strip). |
| `03-paint.html` | Round 3: a collapsible Examples section in the picker, tinted list rows (chosen), colour from tags with a Priority, "Colour from" in the inspector and the lists. |
| `04-paint.html` | Round 4 (approved): the Theme mode — links to theme paint slots (`Set Group 3 "Paint" Theme "Position"`), the theme editor's Paint tab, feature groups on their slots, the Rules window. |

Decided (all as recommended):

- a Paint is `#rrggbb`, a palette word (the 12 base hues, Black, White — stored as hex), a theme
  slot link `Theme "<slot>"`, or `none` (its tags', then the pool's colour); no alpha;
- the slots are LumaCore's set (Dimmer, Position, Gobo, Color, Beam, Focus, Shutter, FX, Control,
  Shapers, Video, Special 1–4) plus the ones a theme adds; one colour per slot, the light base draws
  it like any painted object; each viewer sees their own theme's;
- feature groups default to the slot of their name; a lost slot falls back to the tags and the pool;
- a tag has a Paint and a Priority (higher wins, ties by tag-tree order; a nested tag without a
  Paint takes its parent's); "Colour from" in the inspector and as a list column;
- where it shows: pools, list rows tinted, the Tree window and NAV, the Device Sheet name, view bar
  buttons, cue rows; every object with a meta can be painted;
- painting many is one line (the range rule); a gradient and "by fixture type" are later;
- the Rules window is a new window type in this step;
- clean-ups: one colour parser, the settings dialog's Theme bug, feature groups off the old `Color`
  word.

Approved 2026-10-23 ("looks good, implement"; record
[0294](../../../../decisions/0294-object-paint-palette-free-theme-links.md)).
