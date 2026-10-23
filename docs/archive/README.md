# Archive

Superseded docs, kept for their data (decisions §29).

**Rules**

- Archived files are kept **unchanged**. Never edit them — links inside them may be dead
  and facts may be old. Read the replacement for the current state.
- A doc that is replaced moves here unchanged, under `docs/archive/<area>/` (the same
  path it had under `docs/`), and gets a row below. Docs are never deleted.
- Finished plans are not archived here; they live in `docs/plans/done/`.

| File | Archived | Why | Replaced by |
|---|---|---|---|
| [dev/networking/network-engine.md](dev/networking/network-engine.md) | 2026-10-20 (the md upgrade) | March conceptual networking doc, superseded by the built design | [dev/networking/lumasession.md](../dev/networking/lumasession.md) |
| [dev/engines/runtime.md](dev/engines/runtime.md) | 2026-10-20 (the md upgrade) | March conceptual runtime doc | [dev/engines/engines.md](../dev/engines/engines.md), [ARCHITECTURE.md](../../ARCHITECTURE.md) |
| [dev/ui-framework.md](dev/ui-framework.md) | 2026-10-20 (the md upgrade) | The panel-prototype decision log; its kit table became the kit index | [dev/ui-kit/README.md](../dev/ui-kit/README.md) |
| [handoffs/2026-10-10-decisions-qa.md](handoffs/2026-10-10-decisions-qa.md) | 2026-10-20 (the md upgrade) | A 15k-word Q&A log of one review session | [decisions/](../decisions/README.md) |
| [handoffs/effects.md](handoffs/effects.md) | 2026-10-20 (the md upgrade) | A verbatim copy of the effects design as of 2026-10-11 with dead paths | [visions/effects.md](../../visions/effects.md) |
| [decisions-2026-10.md](decisions-2026-10.md) | 2026-10-20 (the md upgrade) | The one decisions log, §1–§40, 147k words: an append-only record whose sub-entries no link could reach; frozen unchanged so every `§N` reference in code and docs stays valid | [decisions/](../decisions/README.md) (one record per decision; each cites its § and lines here) |
| [meta/TODO.md](meta/TODO.md) | 2026-10-11 (step 11a) | Second task list; open items merged into the one plan | [meta/roadmap.md](../meta/roadmap.md) |
| [meta/requirements.md](meta/requirements.md) | 2026-10-11 (step 11a) | Requirement list from 2026-03; done / obsolete / open items merged into the plan | [meta/roadmap.md](../meta/roadmap.md) |
| [meta/project.md](meta/project.md) | 2026-10-11 (step 11a) | Project overview; vision and constraints became the roadmap's preamble | [meta/roadmap.md](../meta/roadmap.md) |
| [user/terminal/command-reference.md](user/terminal/command-reference.md) | 2026-10-11 (step 11a) | Hand-kept command list, out of date (unbuilt words, wrong abbreviations) | [user/reference/commands.md](../user/reference/commands.md) (generated) |
| [user/syntax/targets.md](user/syntax/targets.md) | 2026-10-11 (step 11a) | Merged into the language source | [user/syntax/lum.md](../user/syntax/lum.md) |
| [user/syntax/values.md](user/syntax/values.md) | 2026-10-11 (step 11a) | Merged into the language source | [user/syntax/lum.md](../user/syntax/lum.md) |
| [user/syntax/functions.md](user/syntax/functions.md) | 2026-10-11 (step 11a) | Merged into the language source | [user/syntax/lum.md](../user/syntax/lum.md) |
| [user/syntax/lumacore-syntax.md](user/syntax/lumacore-syntax.md) | 2026-10-11 (step 11a) | Duplicate | [user/identity/lumacore.md](../user/identity/lumacore.md) |
| [user/object-tree/all-objects.md](user/object-tree/all-objects.md) | 2026-10-11 (step 11a) | Hand-kept object catalog with dead links and unbuilt objects | [user/reference/objects.md](../user/reference/objects.md) (generated) |
| [dev/networking/session-engine.md](dev/networking/session-engine.md) | 2026-10-11 (step 11a) | One of three overlapping LumaSession docs, pre-step 6 | [dev/networking/lumasession.md](../dev/networking/lumasession.md) |
| [dev/networking/multi-station.md](dev/networking/multi-station.md) | 2026-10-11 (step 11a) | One of three overlapping LumaSession docs, pre-step 6 | [dev/networking/lumasession.md](../dev/networking/lumasession.md) |
| [dev/networking/network-sessions.md](dev/networking/network-sessions.md) | 2026-10-11 (step 11a) | One of three overlapping LumaSession docs, old "network session" naming | [dev/networking/lumasession.md](../dev/networking/lumasession.md) |
| [dev/architecture/definitions-coverage.md](dev/architecture/definitions-coverage.md) | 2026-10-11 (step 11a) | Coverage policy; one line kept in the testing doc | [dev/testing.md](../dev/testing.md) |
| [dev/architecture/ecosystem-reference.md](dev/architecture/ecosystem-reference.md) | 2026-10-11 (step 11a) | Research, not current architecture: GDTF/MVR ecosystem survey (2026-03); its test priorities went to the roadmap | — (kept as research) |
| [dev/integrations/api-web-ui.md](dev/integrations/api-web-ui.md) | 2026-10-11 (step 11a) | Merged into the one API doc | [dev/api/http-api.md](../dev/api/http-api.md) |
| [dev/build/build-docs-contributor-flow.md](dev/build/build-docs-contributor-flow.md) | 2026-10-11 (step 11a) | Planning prose, covered elsewhere | [dev/build/vibecode-workflow.md](../dev/build/vibecode-workflow.md) + [getting-started.md](../getting-started.md) |
| [dev/firmware/banner.md](dev/firmware/banner.md) | 2026-10-11 (step 11a) | Banner and cat mark merged into the lore | [user/identity/lore.md](../user/identity/lore.md) |
| [code/language.rs](code/language.rs) | 2026-10-12 (step 11b) | Sentence preview / mode inference, never called; `CommandFamily` moved to `core/commands/registry.rs` | — (kept as code data, not compiled) |
| [code/tls.rs](code/tls.rs) | 2026-10-12 (step 11b) | rustls config behind the unused `--tls-cert` / `--tls-key` flags (dropped: isolated networks) | — (TLS is an idea in the roadmap) |
