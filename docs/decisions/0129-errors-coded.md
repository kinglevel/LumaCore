---
id: 0129
title: Every error reply carries a code
area: commands
status: built
date: 2026-10-11
built_in: step 2 (codes everywhere in step 11)
supersedes: [archive §26 L2514–2522 "handler string errors"]
superseded_by: —
confirmed_by: [tidy `error_replies_carry_a_code`, the sweep's `uncoded-error` finding, the error catalog's format and uniqueness test]
source: [archive §26 L2514–2522, archive §29 L3112–3116, archive §29 L3262–3265]
---
# Every error reply carries a code

## Context
Handlers returned free strings, so a refusal could not be explained, searched or tested for. Step 2 brought typed errors per crate (`thiserror`) with a stable code and the source location; step 11's cleanup made the code mandatory on every reply and made unbuilt words say so instead of pretending (`isn't built yet (planned)`, E-CMD-003).

## Decision
Every refusal is `E-AREA-NNN` — the area is the crate or subsystem, numbers are never reused — and the terminal shows message plus code; `Info Error <code>` and `List Error` read the catalog. Each crate owns an `ERRORS` catalog that its `register` adds to the one catalog in `lumacore_registry`; a new code goes into its crate's catalog first (a test checks format and uniqueness). Every error reply goes through `lumacore_state::errors`: `errors::render(&e)` for a typed error (`Coded`), `errors::render_code(msg, code)` for a message with a code, `errors::reply(msg, fallback)` for a message passed on from elsewhere (claims, a show load, the master — it keeps the code it has, never "Error: Error:"). `text::render_error` is called nowhere outside `core::errors`. The general codes cover the shared refusals (usage E-CMD-006, no such object 007, value 008, nothing to act on 009, not possible now 010, no user 011, file / disk 012, unknown option 018, lines nested too deep 019); rights are E-RIGHTS-001, claims E-CLAIM-001; an internal panic is E-INTERNAL-001.

## Consequences
A string `Error:` without a code fails tidy `error_replies_carry_a_code`, and the long run's sweep reports one as an `uncoded-error` finding. A new area adds its catalog and registers it; a message copied from another station or crate is re-rendered with `errors::reply`, never wrapped. Planned words share one reply (see the planned-words rule in one-registry-one-namespace).
