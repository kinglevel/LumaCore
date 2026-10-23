---
name: decision-record
description: How to write or supersede a decision record in docs/decisions. Use when a design decision is made, changed or reversed, or when asked "where is this decided".
---

# Decision records

One file per binding decision under `docs/decisions/`, named with the next free number and a slug (0204-pool-window-unified.md is one; `ls docs/decisions | tail -1` shows the last). A record is immutable once accepted; a reversal is a **new** record that names the old one in `supersedes`, and the old one gets `superseded_by` and `status: superseded`. Never edit a decision in place beyond those two fields and typos.

Frontmatter, every field present:

```yaml
---
id: 0191
title: <≤ 10 words>
area: <patch | protocols | commands | ui-kit | …>
status: decided | built | superseded | rejected
date: 2026-10-21
built_in: <step or part name, or —>
supersedes: [<slug or archive citation>]
superseded_by: —
confirmed_by: [<tidy rule, test, golden or error code>]   # none is allowed and visible
source: [<where it was decided: a plan, a session, archive §N L…>]
---
```

Body under 300 words, three sections: **Context** (the situation and why, what was rejected), **Decision** (the rule today, present tense, the door and the refusal code), **Consequences** (what a change must do, what is forbidden, which check catches it, what is open).

Never write a value the code owns (format version, protocol numbers, ports, counts, durations); name the constant. Then rebuild the index and check:

```bash
scripts/unix/docs_check --index
```

The history before 2026-10-20 is the frozen log `docs/archive/decisions-2026-10.md`; cite it by section and lines. Where a rule lives: `CLAUDE.md` only if every change needs it; an area rule → `.claude/rules/<area>.md` citing the record; status → the plan or `CHANGELOG.md`, never the record.
