# Assigning tags (2026-10-24)

Tags worked as a model ([0207](../../../../decisions/0207-tags-pool-and-links.md),
[0208](../../../../decisions/0208-tags-ui.md)), but the only way to give one was EditSetting's
Tags tab. The Tags window only picked a tag. These rounds propose a pool square on every tag row
that feeds the command line, Alt+A's fourth press `Tag`, and a tag picker that `Tag Group 3`
opens, as `Paint Group 3` opens the colour picker. Every mock is drawn with the kit's own
`tokens.css` and `kit.css`.

| File | What it asks |
|---|---|
| `01-tag-assign.html` | Round 1: a live desk (the Tags window with squares, Groups and Sequences pools, keys, the command line). The square tap with an empty line runs the tag's action; with `Assign` it arms `Assign Tag n At`. Armed, the cells that carry the tag show it, and a cell tap tags it ("Stays after a tap"). The tag picker on one object and on several (half boxes, New tag from the search). The line `Tag <objects>`. Four questions. |

Round 1 approved and built ([0297](../../../../decisions/0297-tag-squares-and-picker.md)) with
the recommended answers: the armed line stays (now until you clear it — Esc never clears the line),
a tap on a carrier changes nothing, New tag goes to the top level, no drop on a tag row yet.

Decided before round 1 (by the user):

- `Tag <objects>` (a Tag with no number, objects following) opens the picker; `Tag 1.1` still
  selects; Alt+A cycles Assign › Label › Paint › Tag;
- the square lives on each tree row; the rest of the row picks the tag as before;
- a square tap with `Assign` on the line gives `Assign Tag n At`; with an empty line it runs the
  tag's default action;
- in the picker each tap applies at once (one line, one Oops step).
