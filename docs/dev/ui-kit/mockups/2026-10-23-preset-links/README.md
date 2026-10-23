# Preset links: Update, Recast, Extract (2026-10-23)

Three rounds, asked by the user: a cue that uses a preset should follow it, and an update should put a programmer's changes back where they came from. Design first; the build followed the approval (record 0293).

| File | Round | What it shows |
|---|---|---|
| `01-preset-links.html` | 1 | The idea end to end: the link from `At Preset` through the programmer into a cue (the link only, no copy), the Update ask as a reference tree with value diffs, per-fixture scope, a freeze per cue and "update as a new preset", Recast, Extract, the delete ask, the Device Sheet's Show. Rejected in its look: hand-made styling, and the user didn't want individual values listed. |
| `02-preset-links.html` | 2 | Drawn with the kit's own CSS (`tokens.css` + `kit.css` in each frame): the Update ask as a grid of presets and cues, no values; the delete ask, Recast, the Device Sheet's Show. |
| `03-preset-links.html` | 3 | **Approved** 2026-10-23 ("approved"): the Update ask as the kit's tree in pick mode with guides — each preset a branch (S / G / U, the cues that follow as leaves; untick a cue to keep its old look; a circular reference red and not tickable, ready for embedded presets), cues with plain values in their own branch; Content (Original only / Add new / Overwrite), the Recast and Protect switches, the lines it will run, Cancel · Preview in Blind · Update. |

Decided (the recommended answers, every question):

- a cue stores the link only, one per fixture and attribute; Store Preset keeps flattening until embedded presets come;
- presets and cues the programmer changed start ticked; an unticked cue gets an Extract first;
- the whole Update is one Oops step;
- a plain cue value lands where it was stored, and Protect the next cues starts on;
- Preview in Blind shows the result on the Blind output (built as planned: no Blind output exists yet);
- the ask remembers Content and its switches on the profile;
- the Device Sheet's Show starts at Both;
- the lines: `Update`, `Update Preset 3 + 1 /Merge | /Add | /Overwrite`, `Update Sequence 1 Cue 2 /Original | /Active [/Shield]`, `Recast Preset 3 [Sequence 1]`, `Extract Preset 3 [Sequence 4 [Cue 9]]`, `Extract Sequence 1 Cue 2`.
