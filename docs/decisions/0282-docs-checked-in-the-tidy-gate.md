---
id: 0282
title: The docs are checked in the tidy gate
area: docs
status: built
date: 2026-10-20
built_in: md upgrade close
supersedes: []
superseded_by: —
confirmed_by: [tidy `claude_md_stays_short`, tidy `the_docs_pass_docs_check`]
source: [docs/plans/active/md-upgrade.md decision 10 and its findings, 2026-10-20]
---
# The docs are checked in the tidy gate

## Context
The md upgrade (docs/plans/active/md-upgrade.md) cut `CLAUDE.md` to invariants, made one record per decision and wrote `scripts/unix/docs_check` for the gardening checks: records, paths, links, code-owned numbers in prose, the roadmap's caps. The plan ran docs-only, so the script stood beside the gate as a fourth command anyone could skip, and `CLAUDE.md`'s cap was a sentence in the file itself. The docs drifted that way once before: format numbers, protocol numbers and most sampled paths were wrong when the upgrade measured them.

## Decision
Two tidy rules in `crates/dev/lumacore-tidy/src/docs.rs` put the docs inside `cargo test -p lumacore-tidy`. `claude_md_stays_short` fails when `CLAUDE.md` has more lines than `CLAUDE_MD_LINES` (the script holds the same number as `CLAUDE_LINES`). `the_docs_pass_docs_check` runs the script from the workspace root and fails with its findings as the message. The script stays runnable on its own, and `--index` is still the one way to rebuild the decisions index.

## Consequences
A step whose docs rot fails the gate with the script's finding, not a later gardening pass. A rule that outgrows `CLAUDE.md` moves to `.claude/rules/<area>.md` or a record. The tidy crate needs `python3` on the path, as the scripts already do. The README ratchet `READMELESS_AT_MOST` was lowered with the upgrade's READMEs (the ratchet check catches a count that moved); `HEADERLESS_AT_MOST` goes down as files are touched, per the ratchet rule. Open: the other findings of the upgrade (docs/plans/active/findings.md) are decisions or code of their own.
