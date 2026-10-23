# Issues

Open problems found by the command sweep (`./scripts/unix/fuzz_long`, run of 2026-10-13,
`target/fuzz-report/`) and the review of its findings, one file per issue. Each file says how
to reproduce it, where the cause is, what the decision log (`docs/archive/decisions-2026-10.md`, frozen; the records in `docs/decisions/`) says, the fix options and
what is decided. Every fix is asked before it is built; fixed ones are marked below and their files moved to `done/`.

When an issue is fixed, tick it in the table and move its file to `issues/done/`. The roadmap
(`docs/meta/roadmap.md`, Next → "Sweep findings") points here.

Replays: `./scripts/unix/test replay <file>` (or `target/sweep/lumacore-sweep replay <file>`;
it makes a `sandbox/` next to the file, so copy the file somewhere first). The replay numbers
below refer to `target/fuzz-report/replays/` of that run, which is not in git — each issue
lists its minimal repro lines instead.

## Real bugs

| # | Issue | Severity | Decision |
|---|---|---|---|
| 01 | [Guests read host files through `Drives`](01-drives-guest-file-read.md) | security | **decided**: as today for now (all users need disk access) — roadmap with 08; secrets/rights with 10 |
| 02 | [A bare number skips the change path](done/02-bare-number-change-path.md) | high | **fixed** (§33 3b2, `dispatch::write_out_bare`: a bare number / name is written out as its default object first — `3 At 50` = `Group 3 At 50` — and takes the change path; the default keyword per user is `User.default_object`) — design below |
| 03 | [Error code not at the end of the first line](03-error-code-placement.md) | medium | **decided**: code at the end of the first line — not built (checked 2026-10-20: `with_code` lives in `lumacore-base`, but dispatch still appends `— nothing in … ran` after the `;` refusal's code) |
| 04 | [Admin refusals carry the wrong code](done/04-refusal-codes.md) | low | **fixed** (2026-10-13); at the merge with 12d an ownership refusal (Unclaim of another's claim) goes through `rights::require` and carries E-RIGHTS-001 |
| 05 | [`Show` flags run as ReadOnly](done/05-show-flags-readonly.md) | medium | **fixed** (Untracked); roadmap: `Set Show` props and save validation by hash across stations |
| 06 | [Restart / Shutdown save outside the change path](done/06-restart-shutdown-save.md) | low | **fixed** (Untracked) |
| 07 | [Quoted arguments: options vs paths and strings](done/07-quoted-arguments.md) | medium | **fixed** (2026-10-13): quoted words keep their quotes to the handler, library-relative filenames; variables on the roadmap |
| 08 | [Show words walk into `Drives` through the context](08-enter-into-drives.md) | medium | **roadmap** (users need disk access) |
| 09 | [Show files lose float precision on load](done/09-float-roundtrip.md) | low | **fixed** (`float_roundtrip`; the save→load→save property test runs again) |
| 10 | [Rights groups](done/10-rights-groups.md) | design | **built**: the ladder `Guest < Playback < Presets < Program < Setup < Admin` on `User.rights`, one gate in dispatch, declared per command and per object verb (§19 "Step 5 hardening", §30 "12d"); the target check for `Drives` stays with 01 |
| 11 | [Routes and merge modes are drafted while Patch is claimed](done/11-patch-draft-routes.md) | question | **fixed** (§34 "34g4 patch hardened" part 2b): fixtures + universes (MergeMode with them) are the draft; the wiring is the protocol lines (`Sacn 1` = one row, slots by number), live for anyone (§11 "Issue 11 hardening", §34 "34f7c hardened") |

## Sweep harness (false positives)

| # | Issue | Decision |
|---|---|---|
| 20 | [Sweep checks that report false findings](20-sweep-harness.md) | **fixed** a–h (2026-10-13); i, j open — rerun the sweep `--fresh` |

## Other open threads

| # | Issue | Decision |
|---|---|---|
| 30 | [Build and tooling leftovers](30-build-leftovers.md) | open (checked 2026-10-20: `tabled` is still 0.17; `scripts/win/` has the dev / server twins, `build` / `test` / `app` stay on the roadmap) |

## Bug reports

Larger read-only analyses, one file per round (`docs/research/r1-bugreport/luma-bugreport-R<n>.md`, moved
there 2026-10-20); items carry ids (`R1-01` …, `Q<n>`) and checkboxes, and become numbered issues here when they are confirmed.

| Round | Report | Date | Status |
|---|---|---|---|
| R1 | [Published console bug fixes vs LumaCore](../docs/research/r1-bugreport/luma-bugreport-R1.md) — 72 risks, 10 doc drifts, spec gaps (the set is in `docs/research/r1-bugreport/`) | 2026-10-13 | fixes built on branch `bugfixes`, merged into main after 12d (decisions §30 "Merge of bugfixes": backups → Versions, one Delete ask in dispatch, `.starting` + `running` woven, busy on 12d's paths) — see the report's Status section |

## Issue 02 — decided design (2026-10-13)

A **default keyword** (`Fixture Please`): a bare number uses it (`101 At 50` = `Fixture 101 At 50`,
with `Group` default `3 At 50` = `Group 3 At 50`), going through the normal change path.
- `<Keyword> Please` (nothing after the keyword) sets it; bare `Please` keeps today's meaning.
- Saved **per user** (`ShowData.Users`, format 11 batch); the prompt shows it (`Fixture>`).
- A profile's **start place**: a keyword (default `Fixture`), a tree path or a disk root path; `cd` alone goes there.
- The same in KBD / PRG / NAV (modes change single keys only). Universe> is just the default keyword Universe (`3` = `Universe 3`, no longer Info).
