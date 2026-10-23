# Meta — the plan and the rules

What we build next and why things are the way they are.

- [roadmap.md](roadmap.md) — **the one plan**: Now / Next / Later, one line per item with a link
  to its plan, record, issue or vision. Every open item lives here and nowhere else.
- [`docs/decisions/`](../decisions/) — **the rules**: one record per binding decision
  (frontmatter `id, title, area, status, date, …`; body Context / Decision / Consequences). Its
  `README.md` is the index, built by `scripts/unix/docs_check`. A reversal is a new record that
  supersedes the old one, never an edit.
- [`docs/archive/decisions-2026-10.md`](../archive/decisions-2026-10.md) — **the frozen log** the
  records came from, unchanged: a `§N` anywhere in the code or the docs names its section there.
- [tidy.md](tidy.md) — **the rulebook**: every source rule `lumacore-tidy` checks, why, its escape
  and its ratchet; the page and the code are checked against each other.

Older plans (the March phases, `TODO.md`, `requirements.md`, `project.md`) are merged into the
roadmap and kept unchanged in [docs/archive/](../archive/README.md). Work in progress is a plan
in [docs/plans/active/](../plans/README.md) (finished ones in `docs/plans/done/`); research under
`docs/research/`.
