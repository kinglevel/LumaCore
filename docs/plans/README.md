# Plans

A plan is the working file of one piece of work between sessions: what was decided, what is built, what remains, where the research is. Moved here from `handoffs/` on 2026-10-20 (`active/moves-handoffs.md` lists every move).

- `active/` — plans with work still to do. Each carries a `## Progress` (or `## Status`) section at the top that says what is built (with where: commit, roadmap item, decision record) and what remains; the roadmap (`docs/meta/roadmap.md`) links the plan from its item.
- `done/` — plans whose work is built or whose leftovers moved to the roadmap. A plan closed after its work was already committed opens with a `Closed <date>:` note naming what was built and where the leftovers live; the body stays as written (history, with its old paths).

Rules: every plan carries a progress section (`## Progress`, or `## Status` in the plans moved today), dated; status never lives in `CLAUDE.md` (only rules do) and never in a decision record (a record says decided / built / superseded, the plan says how far); a finished plan is moved, never deleted; the reasoning trail of a review session is history and goes to `docs/archive/handoffs/`.
