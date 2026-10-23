# The preset pool (2026-10-23)

Round 1, asked by the user: one modern preset pool window. There are no preset pool types; the window follows the feature group the programmer has selected on the encoder bar, filters one flat pool, and narrows further by tags. Three reads came first: the old docs (§21, §22, §27, §30, §33, §35; records 0150, 0105, 0009, 0122), the code (`PresetGroup` / `Preset` / `PresetEntry` exist, and only `List Preset` is built), and how established consoles lay out their preset pools. Design only: nothing is built until the user decides.

| File | Artifact | What it asks |
|---|---|---|
| `01-presets.html` | https://claude.ai/artifact/GrbfJRfuFh8GEPGJz6GND1 | A working mock on the rig's groups: the encoder bar's feature groups, a selection, the one Presets window (Follow / pinned, the filter rule, Packed / Fixed, the tap action, tag chips), cells (how the preset fits the selection, a live preview, feature-group squares, use count, mode letter), Store + tap, hold for the context menu, Edit opens the editor. Five questions. |

Decided in the round's questions:

- one window type;
- one flat pool, with tags as well as feature groups;
- Store keeps every active programmer value (§30);
- tag chips on top;
- a window follows the programmer's feature group and can be pinned;
- Store + tap stores there, and Store alone takes the next free number;
- the cell shows all four indicators (fit, preview, feature-group squares, use count);
- the tap action is a window setting;
- modes are automatic, shown as a letter;
- hold opens the context menu, and Edit opens the editor.

Approved 2026-10-23 with the recommended answers (record 0288): touches the group, packed (fixed as a setting), All shows every preset, `Preset 12`, the first build as recommended.
