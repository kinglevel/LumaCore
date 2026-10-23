---
id: 0252
title: One roadmap, archive never delete, generated reference
area: docs
status: built
date: 2026-10-12
built_in: 11a
supersedes: []
superseded_by: —
confirmed_by: [catalog test `the_reference_is_current`, `docs/archive/README.md`]
source: [archive §29 L3086–3103, archive §29 L3158–3163, archive §29 L3180–3189]
---
# One roadmap, archive never delete, generated reference

## Context
By step 11 the plan was spread over `TODO.md`, `requirements.md`, `project.md` and the open items of the handoffs, syntax tables were hand-kept in several docs, and the networking design sat in three overlapping files. The per-file plan (keep / fix / merge / archive) was reviewed with the user before 11a.

## Decision
`docs/meta/roadmap.md` is the one living plan: every open item is placed there and ticked when built. Superseded documents move unchanged to `docs/archive/` with an entry in `docs/archive/README.md` (file, why, replaced by, date); finished handoffs to `handoffs/done/`; vision-only notes to `visions/notes/`. Archive, never delete. The command reference is generated from the registries (`cargo run -p lumacore-station --bin gen-docs` → `docs/user/reference/commands.md` + `objects.md`: syntax, examples, aliases, rights, status) and a test fails when it is stale; hand-written docs keep concepts and guides and link to it — no hand-kept syntax or alias tables. The networking design is one current `docs/dev/networking/lumasession.md`. Decisions were kept as one file with a table of contents and status per §, stale lines annotated in place.

## Consequences
A registry change is followed by `gen-docs` or the catalog test fails. A new open item goes to the roadmap, never a new TODO file. The 2026-10-20 md upgrade (`docs/plans/active/md-upgrade.md`) replaced the one decisions file with one record per decision under `docs/decisions/` and froze the log unchanged as `docs/archive/decisions-2026-10.md`, its § numbers kept so references in code stay meaningful; the roadmap's shape (caps, Done to `CHANGELOG.md`, handoffs to `docs/plans/`) follows that plan. "Archive, never delete" stands.
