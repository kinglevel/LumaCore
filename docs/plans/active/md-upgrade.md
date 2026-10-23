# The md upgrade — plan

Status: built 2026-10-20 (the docs: `967dcc99`; the code side — the tidy rules `claude_md_stays_short` and `the_docs_pass_docs_check`, the README ratchet — the same day, [0282](../../decisions/0282-docs-checked-in-the-tidy-gate.md)); what the docs work could not do is in [findings.md](findings.md). Decided 2026-10-20 in one session (the research and the hardening questions are in that session; this file is the contract the workers run on). **Rule of the job: no code is touched.** Workers may write only Markdown, text, `.claude/rules/**`, `.claude/skills/**`, the one script `scripts/unix/docs_check`, and the loop's own hook settings file; the generated reference and the goldens come only from the existing commands.

## 1. Why

Measured 2026-10-20: 200 tracked documentation files, 522k words. `CLAUDE.md` is 10.6k words (~18k tokens) and loads into every session; the median always-loaded agent file across 43 large repositories is 157 lines. `docs/meta/decisions.md` is 147k words (~198k tokens), an append-only log whose sub-entries are bold paragraphs no link can reach; §30 and §34 are 56 % of it; 6 of 8 sampled status rows are wrong; earlier text that was later reversed stays unmarked. `docs/meta/roadmap.md` is 22k words with 215 ticked items under Next and a Now section a week behind. The three grew from 3.6k to 180k words in two weeks (~12k words a day). The docs drift from the code: the show format is 31 in code and 10 / 15 / 25 in three docs; the session protocol is 21 in code and 13 / 20 in two docs; 16 of 20 sampled paths named in docs do not exist; 50 files have no inbound link; three handoffs say "nothing built" for committed work. Missing: a root ARCHITECTURE.md, CONTRIBUTING, CHANGELOG, AGENTS.md, LICENSE files, `.claude/rules`, `.claude/skills`.

What already cannot rot and is kept as the model: the 39 crate READMEs in the enforced seven-section shape, the generated reference with `the_reference_is_current`, the goldens, `lum.md` as the spec whose examples are tests, `tidy.md`.

## 2. Decisions taken (the hardening pass)

| # | Decision |
|---|---|
| 1 | Records: one file per binding rule cluster, 100–150 files, each loadable alone. |
| 2 | The old log is frozen unchanged as `docs/archive/decisions-2026-10.md`; section numbers kept, so the 3,400 `§N` references in code stay meaningful; every record cites its archive lines. |
| 3 | The in-flight patch-tree step's (40d) edits to `decisions.md`, `roadmap.md` and `lum.md` are folded in from the working copy; `lum.md` edits are carried verbatim, never reworded. |
| 4 | `CLAUDE.md` is the written file; `AGENTS.md` is a two-line pointer to it. |
| 5 | Research (UI-kit 72k words, patch, the R1 bug report set) moves to `docs/research/<topic>/` with one index, never loaded by default. |
| 6 | User docs for unbuilt features (presets, macros, timelines, the effects design) move under `visions/`, linked from the roadmap; a built doc's Planned lines shrink to one sentence pointing there. |
| 7 | One script is allowed, `scripts/unix/docs_check` (Python): builds the decisions index from frontmatter and runs the gardening checks. The stated exception to no-code. |
| 8 | Everything runs unattended after this plan is approved; the user reads this file and says go. |
| 9 | Record shape: frontmatter `id, title, area, status (decided | built | superseded | rejected), date, built_in, supersedes, superseded_by, confirmed_by, source`; body `Context / Decision / Consequences`, under 300 words; longer design text links to visions or the archive. |
| 10 | `CLAUDE.md`: hard cap 150 lines / 2,500 words. A rule enters only if it is non-obvious from the code, has been hit repeatedly, is specific enough to act on, and names its door or the tidy rule / test that confirms it. |
| 11 | Branches: workers commit to per-area branches off `md-upgrade`; a read-only reviewer pass gates each branch; passing branches merge into `md-upgrade` automatically; the user merges `md-upgrade` into main once. Main never moves unattended. |
| 12 | Guardrails are hooks: writes only to the allowed paths; Bash denied for `cargo build` / `cargo test` and for `LUMACORE_BLESS`; `gen-docs` allowed; a `STOP` file halts every tool; a budget and a turn cap per task; the Stop hook runs `docs_check`. |
| 13 | Roadmap: Now ≤ 10, Next ≤ 25, Later one line each; every item links to a plan, record, issue or vision; nothing ticked stays; Done moves to `CHANGELOG.md`; Ideas move to `visions/notes/`; `handoffs/` becomes `docs/plans/{active,done}/`, each plan with its own progress section. |
| 14 | Fact ownership: a doc never states a value the code owns (format version, protocol numbers, ports, crate counts, test durations, ratchet counts); it names where the code holds it. `docs_check` flags such numbers in prose. The generated reference and the goldens are the only docs carrying code-derived values. |
| 15 | `docs/dev`: root `ARCHITECTURE.md` replaces `docs/dev/architecture/architecture.md`; the March conceptual docs (`engines/runtime.md`, `networking/network-engine.md`) are archived; `engines.md`, `networking/lumasession.md`, `replication.md`, `api/http-api.md`, `testing.md` re-aligned and deduplicated; the 15 crates without a README get one in the seven-section shape. |
| 16 | After the upgrade: a nightly scheduled docs-only agent runs `docs_check` and fixes what it can on a branch. |
| 17 | Budget: the user is on Claude Max, so the ceiling is the plan's usage windows. `--max-budget-usd 10` per task as a runaway guard, `--max-turns`, at most 2 workers, the loop checks usage before each task and pauses under a floor. |

## 3. The target tree

| Path | Contents | Size |
|---|---|---|
| `CLAUDE.md` | Build and test commands, feature flags, the 15 invariants of §4, the placement rule of §5, a routing table to rules, skills and docs. No history, no architecture prose, no code-owned numbers. | ≤ 150 lines |
| `AGENTS.md` | Two lines pointing at `CLAUDE.md` and the rules folder. | 2 lines |
| `ARCHITECTURE.md` | matklad shape: bird's-eye view, code map by crate group (one paragraph per group, naming crates, not files), invariants stated as absences, cross-cutting concerns (change path, catalog, rights, render). Links `docs/dev/architecture/crate-map.md` and `code-structure.md`. Revisited, not synced line by line. | ~200 lines |
| `CONTRIBUTING.md` | Build, the tests to run per step and the gate, commit shape, where each kind of text goes (the placement rule), how to write a record, how to run `docs_check`. | ~100 lines |
| `CHANGELOG.md` | Keep-a-Changelog shape, newest first: one entry per step with its commit, from the roadmap's Done and Next ticks and the archive's "built" entries. | grows |
| `LICENSE-MIT`, `LICENSE-APACHE` | The standard texts for the licence the README names. | fixed |
| `.claude/rules/<area>.md` | Path-scoped rules, frontmatter `paths:`; §6 lists the eight files, their globs and the `CLAUDE.md` bullets that move into them. Each rule cites its records. | 150–1,400 words each |
| `.claude/skills/<name>/SKILL.md` | Procedures: `step-tests`, `headers`, `regen-reference`, `build-bench`, `decision-record`, `doc-garden`. | 100–250 words each |
| `docs/decisions/NNNN-slug.md` | One record per binding decision (§7); `docs/decisions/README.md` is the index table built by `docs_check`. | 100–150 files |
| `docs/archive/decisions-2026-10.md` | Today's log, frozen, 40d content included. | frozen |
| `docs/meta/roadmap.md` | Now / Next / Later, one line per item with a link. | ~150 lines |
| `docs/plans/active/`, `docs/plans/done/` | The handoffs, triaged; this plan; each plan has `## Progress`. | per plan |
| `docs/research/<topic>/` + `docs/research/README.md` | The UI-kit research, the patch research, the R1 report set. | moved as is |
| `docs/user/` | Describes only what the station does today; stale facts fixed; code-owned numbers replaced by pointers. | trimmed |
| `docs/dev/` | Re-aligned per decision 15. | trimmed |
| `visions/` | Receives presets, macros, timelines, effects prose and the roadmap's Ideas; the two byte-identical concept files deduplicated. | grows |
| `scripts/unix/docs_check` | Python: index build + gardening checks (§9). | one file |

## 4. The CLAUDE.md invariants (the always-loaded rules)

Each line names its door and its check. Everything else from today's `CLAUDE.md` goes to a rules file (§6), a record (§7), `ARCHITECTURE.md`, `CONTRIBUTING.md` or the archive.

1. **The catalog.** Every declaration is a row a crate adds in its `register`; never a static list of another crate's rows. Door `lumacore_registry`; check `freeze()`.
2. **One state thread, two lanes.** Never `StateHandle::call` inside a job, never network I/O inside a job, never compile or render on the state thread. Door `StateHandle::call_within` / the outbox; check E-INTERNAL-001.
3. **The change path is automatic.** A new `ShowData` field does not compile until it has a `ShowSubtree` variant, its `oops.rs` arms and a `show_file` entry. Check: the compiler.
4. **Strict identity.** Every stored link is a GUID, never a slot number or a name; every show type implements `Walk` exhaustively. Check E-SHOW-003, the compiler.
5. **Errors have codes.** Every refusal carries `E-AREA-NNN` from its crate's catalog; never `text::render_error` outside `errors`. Check tidy `error_replies_carry_a_code`, the catalog test.
6. **One resolver, one property registry, one namespace.** Targets only through `core::resolve`, properties only through `ObjectDef::props`, never a keyword match outside the registry, new handlers typed. Check tidy `old_resolvers_only_go_down`, the keyword rule, `the_shim_count_only_goes_down`.
7. **Tree paths are typed.** Code names a `Node`, never a `"Root.…"` string. Check tidy `tree_paths_are_typed`, `tree_paths.golden.txt`.
8. **Show files only move forward.** Bump `FORMAT_VERSION` on any change, no migrations, no compat code. Check E-SHOW-001 / 002.
9. **Every test runs on the rig.** `demo::fresh_state()`, never a small show of your own; never a second walk of the show per line. Check `the_demo_show_builds_loads_and_matches_its_golden`.
10. **The render tick allocates, locks, logs and looks up nothing.** Check `assert_no_alloc`, the tidy render regions, the budget test.
11. **One rights gate.** Rights are declared on the row; dispatch checks them; never a per-transport or `is_admin()` check in a handler. Check E-RIGHTS-001.
12. **Code structure.** Headers only from `scripts/unix/headers`, Parts in the one order, one file per verb, no `mod.rs`, files under the limit, shared things through their door, crate edges only down, ratchets only go down. Check `cargo test -p lumacore-tidy`.
13. **The station only assembles; clients compile only the wire.** Any wire change bumps the protocol numbers, no shims. Check tidy `the_station_only_assembles`, `crate_edges_point_down`.
14. **The disk has one door.** `core` touches files only through `drive`; every `CommandDef` declares its reach. Check tidy `core_reaches_the_disk_only_through_drive`.
15. **The language is `lum.md`.** Any change to the syntax is a hard discussion first; every example is a test; a word that isn't built is `Planned`, never a stub that acknowledges. Check the catalog's lum tests, `planned_words_reply_planned_and_change_nothing`.

## 5. The placement rule (goes into CLAUDE.md and CONTRIBUTING.md)

- A fact every change needs → `CLAUDE.md`, if it passes decision 10's four tests. Otherwise:
- A rule for one area → its `.claude/rules/<area>.md`.
- A decision and its reasons → a record in `docs/decisions/`; a reversal is a new record that supersedes the old one, never an edit of it.
- A procedure → a skill.
- Status, progress → the plan in `docs/plans/active/` while open, `CHANGELOG.md` when built.
- A measurement → `docs/dev/benchmarks.md` or the plan; never a rule file.
- Research → `docs/research/<topic>/`.
- A value the code owns → not in prose; name where the code holds it.
- Design for something not built → `visions/`.

## 6. The rules files (path-scoped)

| File | `paths:` | Moves in (today's CLAUDE.md topics) |
|---|---|---|
| `render.md` | `crates/engine/**`, `crates/network/protocols/lumacore-lines/**` | Render engine owns DMX, render thread rules, render plan, engine panic / overload, session clock, health, merge / cue timing / Highlight / masters / echoes, gesture channel |
| `protocols.md` | `crates/network/protocols/**`, `crates/model/lumacore-protocols/**`, `crates/commands/objects/lumacore-cmd-protocols/**`, `crates/commands/objects/protocols/**` | Each protocol declares itself, a protocol is an object and a line one row, networks and NIC mapping, lines and DMX input, E-SHOW-007 / 008 |
| `patch.md` | `crates/model/lumacore-patch/**`, `crates/model/lumacore-fixtures/**`, `crates/model/lumacore-objects/src/objects/patch/**`, `crates/formats/lumacore-mvr/**`, `crates/formats/lumacore-poolxml/**`, `crates/formats/lumacore-gdtf/**`, `crates/ui/lumacore-views/src/patch*` | A fixture's ids and the patch's key, the staged patch and live columns, Import MVR, the MVR tree words, nesting, 40c / 40d |
| `commands.md` | `crates/model/lumacore-objects/**`, `crates/model/lumacore-state/**`, `crates/foundation/lumacore-syntax*/**`, `crates/commands/**`, `tests/catalog/**` | The catalog's row shapes, commands and objects publish themselves, owner-first addressing, quoting, the range rule, default object, bare verbs, typed handlers, Oops, selections, groups and pools, programmers, mode objects, reference check, reach, Lum vocabulary |
| `showfile.md` | `crates/formats/lumacore-luma/**`, `crates/formats/lumacore-drives/**`, `crates/commands/objects/lumacore-cmd-showfile/**` | The `.luma` zip, the stores, embedded size limits, backup and recovery, the `.starting` marker, Import Show, drives |
| `ui-kit.md` | `crates/ui/lumacore-webui/**`, `crates/ui/lumacore-views/**`, `crates/ui/lumacore-http/**`, `crates/ui/lumacore-terminal/**`, `tests/e2e/**` | UI kit strict, the pointer core, grids and slabs, dialogs, menus, window types declared in Rust, settings typed, keyboard, devices and lock, screenshots, feeds contract, phone rules, the TUI theme |
| `lumasession.md` | `crates/server/**`, `crates/commands/objects/lumacore-cmd-lumasession/**`, `crates/commands/objects/lumacore-cmd-station/**`, `crates/network/lumacore-clocksync/**`, `crates/foundation/lumacore-wire/**` | Ports and services, replication and the outbox, remote station edits, merge on rejoin, master loss and election, ownership and claims, shutdown order, app ↔ local server |
| `keys.md` | `crates/commands/objects/lumacore-cmd-keys/**`, `crates/model/lumacore-tree/src/ui_keys*` | Keys and gestures, core keys, never a command line in a `KeyBinding` |

Nested `crates/dev/lumacore-tidy/CLAUDE.md` (optional, ~60 words): one entry per rule in `tidy.md`; ratchets only go down.

## 7. The records

Filled from the extraction (four read-only passes over the archive, 2026-10-20). Numbering is by area in archive order; ids are stable once assigned. Status is the latest state the archive shows. `confirmed_by: none` is allowed and visible.

The candidate tables are in [md-upgrade.records.md](md-upgrade.records.md), one section per archive range, each with its archive-only ranges, its earlier-versus-later contradictions and its bold leads for citation mapping.

| Range | Candidates |
|---|---|
| §1–§29 | 62 |
| §30 | 58 |
| §31–§33 | 47 |
| §34–§40 (40d included) | 70 |
| **Total** | **237** |

That is above the 100–150 decided in §2. The ranges were read independently, so the same rule often appears twice; phase 0 merges these before the tasks are cut. Cross-range merges already visible (each pair becomes one record whose source cites both ranges):

- staged-patch-draft (§11) + devices-claim-edits-a-draft (§34)
- lines-one-type-runs (§11) + line-names-protocol-by-row (§34) + protocol-is-object-line-is-row (§34) + routing-universe-option (§33)
- networks-mapped-per-station (§11) + station-table (§30)
- rights-one-gate (§19) + rights-ladder (§30) + admin-password-fixed (§30)
- luma-show-file-container (§8) + luma-zip-format (§30)
- autosave-recovery-versions (§9) + backup-versions-recovery (§30)
- one-state-thread (§26) + state-lanes-thread-per-concern (§31) + busy-coverage (§30)
- tree-paths-typed-node (§31) + tree-paths-in-lum (§33)
- demo-show-test (§21) + demo-show-is-the-rig (§40)
- layers-point-down (§23) + state-objects-dispatch-crate-cut (§34) + crates-in-folders-by-domain (§34)
- wire-crate-clients (§23) + webui-crate-embedded-with-override (§36)
- keys-uikey-gestures (§17) + core-keys (§30) + meow-is-shift (§30)
- group-masters (§22) + masters-playbacks-live (§30)
- playback-state-saved (§13) + playbacks-registry-live-echoes (§32) + sequence-cuezero-offcue (§32) + cuezero-offcue-echoes (§30) + top-echoes-temp-flash (§32)
- tracking-cues-parts (§27) + cue-timing-layers (§30)
- pool-window-unified (§22) + luma-pool-object-sheet (§30)
- quoting-values-options (§15) + quoting-text-properties-backticks (§33)
- one-registry-one-namespace (§15) + one-name-per-word-fuzzy-typing (§33) + registry-rows-state-only-differences (§31)
- selections-one-only (§29) + select-vs-selectfixtures (§33)
- oops-categories-statements (§29) + delete-asks-once (§30) + oops-window (§30)
- render-tick-rules (§24) + realtime-thread-priority (§31) + render-budget-benchmarks-warn (§31) + playback-merge-order (§27)
- dmx-input-merge (§11) + line-rates-send-pacing (§24) + line-timing-lock-to-engine-frames (§31)
- cells-on-fixture-type (§28) + fixture-parts-and-pixelmapper (§34)
- one-object-door (§35) + next-free-is-first-unused (§34)
- effects-model-and-control + effects-deterministic-anchors (§28) + effect-targets-per-line (§32)

After these merges the count lands near 190. Two ways to reach 150: merge further by area (one record per area sub-topic, e.g. all four `links-*` rows as one "live links" record, the five `patch-*` dialog rows as two), or accept ~190 and keep each rule loadable alone. **Open for the user's go: which.** The index and the citation mapping work either way.

## 8. The unattended run

**Queue.** `docs/plans/active/md-upgrade.tasks.json`: `{id, phase, area, title, inputs, outputs, check, depends_on, status, claimed_by, result}`. Workers only move `status` forward and fill `result`. Phases:

| Phase | Tasks | Notes |
|---|---|---|
| 0 setup | archive the log (working-copy version); folders; `docs_check` skeleton; the hook settings file; the six skills; this plan's progress section | one task, run first, alone |
| 1 records | one task per record of §7 | depends on 0; any order; the index rebuilt by `docs_check` on each Stop |
| 2 always-loaded | `CLAUDE.md`, `AGENTS.md`, the eight rules files, the tidy nested file | depends on 1 (rules cite records) |
| 3 standard files | `ARCHITECTURE.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, the licence files, `README.md` fixes | depends on 1 |
| 4 re-align | one task per file in `docs/dev` and `docs/user`; one per missing crate README | depends on 1 |
| 5 plans, research, visions | triage `handoffs/` → `docs/plans`; moves to `docs/research`; unbuilt docs → `visions`; `issues/` closed where built; roadmap rewrite | depends on 3 (changelog takes Done) |
| 6 close | full `docs_check`, dangling-reference list for the user (code refs are listed, never edited), final index | last, alone |

**Worker.** `scripts/unix/agent_loop <worker>` (a docs-only variant of the loop designed on 2026-10-20): claim the next unblocked task by committing the claim on `md-upgrade`; worktree `.claude/worktrees/<worker>` on branch `md/<area>`; `claude -p` with the task, the record template, the placement rule and §4 in the prompt; `--settings .claude/md-upgrade.settings.json` (the fence below, so normal sessions are not fenced); `--permission-mode acceptEdits --permission-prompts none --max-budget-usd 10 --max-turns 60 --no-session-persistence --output-format json`; on exit, record cost and result; retry a failed task once; then leave it `blocked` with a `BLOCKED-<id>.md`. Two workers at most; no cargo runs, so no shared target directory is needed.

**The fence (`.claude/md-upgrade.settings.json`).** PreToolUse: Write / Edit allowed only when the path matches `*.md`, `*.txt`, `.claude/rules/**`, `.claude/skills/**`, `scripts/unix/docs_check`, `docs/plans/active/md-upgrade.tasks.json`; Bash denied when the line contains `cargo build`, `cargo test`, `cargo run` (except `gen-docs`), `LUMACORE_BLESS`, `git push --force`, `git checkout main`, `rm -rf`; every tool denied while `docs/plans/active/STOP` exists. Stop: runs `scripts/unix/docs_check --task <id>` and blocks the stop until it passes (cap 8). SessionEnd: nothing.

**Reviewer.** A read-only `claude -p` per finished branch, no Write / Edit / Bash write: checks the §9 criteria, writes `docs/plans/active/reviews/<task>.md` with PASS or NEEDS_WORK and the reasons; PASS merges the branch into `md-upgrade` (the loop does the merge, fast-forward or a plain merge, never a force); NEEDS_WORK re-queues the task with the review attached, once.

**Kill switch.** `touch docs/plans/active/STOP`. Resume: remove it.

## 9. Review criteria (what `docs_check` and the reviewer test)

1. A record has every frontmatter field, a `source` naming archive lines, a body under 300 words with the three sections, and no code-owned number.
2. Every record's `status` agrees with the latest entry in its source range; a `superseded` record names its successor and the successor names it.
3. Every bold sub-entry title in the archive maps to at least one record or is listed in `docs/decisions/README.md` under "archive only" — so a `§30 "12d part 5 built"`-style reference can be looked up.
4. No two records state the same rule; no record restates a `CLAUDE.md` invariant without citing it.
5. `CLAUDE.md` ≤ 150 lines; every rule names a door or a check; no `§` references, dates, "built", R1 / Q ids.
6. Every rules file has `paths:`; every glob matches at least one existing path.
7. Every path named in any doc exists (`docs_check` resolves backticked paths; a path in the archive is exempt).
8. No prose number from the owned-fact list (format version, protocol numbers, ports, crate counts, test durations, ratchet counts) outside the generated reference, the goldens and the archive.
9. The roadmap has no `[x]`, Now ≤ 10, Next ≤ 25, every item has a link.
10. `lum.md` differs from the working copy only where a record says so; 40d's edits are byte-identical.
11. No file outside the allowed paths changed on the branch (`git diff --name-only` against `md-upgrade`).
12. Every moved file leaves no dangling Markdown link (the archive README lists what moved where).

## 10. Not in this job (findings for the user, written to `docs/plans/active/findings.md`)

- A tidy rule that caps `CLAUDE.md` and runs `docs_check` in the gate (code).
- Code header `//! Decisions:` lines that cite a `§` sub-entry: listed with their record ids, never edited.
- `READMELESS_AT_MOST`, `HEADERLESS_AT_MOST` and other ratchet constants that could go down after the docs land (code).
- `SECURITY.md`: not written; rigs are isolated networks by decision; the user decides whether a one-paragraph file is wanted.
- The `scripts/unix/agent_loop` and `agent_claim` / `agent_record` helpers are scripts, not product code; they live under `scripts/unix/` with the same exception as `docs_check`.

## Progress

- 2026-10-20: plan written on branch `worktree-md-upgrade`; the record tables (§7) extracted in four passes; the user said go with ~190 records accepted.
- 2026-10-20, the run: driven from one session with parallel subagents instead of the loop script of §8 (same fences by prompt and by `git diff`, the reviewer role taken by the orchestrator and `docs_check`); 40d turned out to be committed on main before the run, so decision 3's fold was a no-op.
  - Phase 0: the log frozen at `docs/archive/decisions-2026-10.md`; folders; `scripts/unix/docs_check` (index + checks 1, 2, 5–9, 12, and a `--fence` mode for docs-only agents, used by `.claude/md-upgrade.settings.json`); the six skills; the archive index rows.
  - Phase 1: 203 records in `docs/decisions/` (fourteen batches on disjoint id blocks; the cross-range merges of §7 applied; gaps in the numbering are the unused ends of the blocks); the index built by the script.
  - Phase 2: `CLAUDE.md` rewritten (90 lines), `AGENTS.md`, the eight rules files, `crates/dev/lumacore-tidy/CLAUDE.md`.
  - Phase 3: `ARCHITECTURE.md`, `CONTRIBUTING.md`, `CHANGELOG.md` (332 ticked items under their dates), `LICENSE-MIT`, `LICENSE-APACHE`, `README.md` fixed.
  - Phase 4: `docs/dev` re-aligned (three March docs archived, the old architecture file deleted, the crate map rewritten, numbers → constants), `docs/user` re-aligned (presets, macros, timelines and the effects design → `visions/`), the 15 missing crate READMEs written, three existing READMEs' port numbers → constants.
  - Phase 5: `handoffs/` → `docs/plans/{active,done}` and `docs/archive/handoffs`; research → `docs/research/` with an index; the R1 report set with it; issues 02, 10, 11 closed; the roadmap rewritten to Now 10 / Next 25 / Later 17 with every item linked; the Ideas → `visions/notes/ideas-2026-10.md`.
  - Phase 6: merged-away slugs remapped, slug-only references numbered, old decisions links repointed; `docs_check` from 833 findings to 0; `findings.md` written for the code-side items.
- Next: the user reads the branch's diff and merges `worktree-md-upgrade` into main; then the nightly `doc-garden` run, and this plan moves to `docs/plans/done/`.
