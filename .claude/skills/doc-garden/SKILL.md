---
name: doc-garden
description: The docs gardening pass: run docs_check, fix what rots (dead paths, dead links, code-owned numbers, built items left open, missing index rows) on a branch. Use nightly or whenever docs_check fails.
---

# Doc gardening

Docs only. Never touch code; if a finding needs a code change (a stale test name, a tidy constant), write it to `docs/plans/active/findings.md` instead.

```bash
scripts/unix/docs_check --index        # rebuild docs/decisions/README.md, then every check
```

Work through the findings by check number:

- **7 paths, 12 links**: the file moved or was renamed; find it (`git log --follow`, `find . -name`) and point at the new place. A path that is runtime data under `library/` is fine.
- **8 code-owned numbers**: replace the number with where the code holds it ("the current `FORMAT_VERSION` in lumacore-tree", "the ports are constants in lumacore-wire"). History files (archive, CHANGELOG, research, plans) are exempt.
- **9 roadmap**: a ticked item moves to `CHANGELOG.md` under its date; Now over ten or Next over twenty-five means something is finished or belongs in Later.
- **5 CLAUDE.md**: over 150 lines, or a `§`, date, "built" or R1/Q id crept in: move the text to its rules file, record or plan.
- **1, 2, 6 records and rules**: a missing field, a superseded record without its successor, a rules glob matching nothing.

Also look for what the script cannot see: a doc describing behaviour the code no longer has (compare with `CLAUDE.md` and the generated reference), a plan under `docs/plans/active/` whose work is committed (move it to `done/` with a closing line), an open roadmap item that the changelog already lists as built.

Commit on a branch named `garden/<date>`, one commit, message "docs: garden <date> — <what changed>". Never push to main.
