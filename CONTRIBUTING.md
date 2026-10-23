# Contributing

How a change is built, tested, committed and written down. The always-loaded rules are
[CLAUDE.md](CLAUDE.md); the shape of the code is [ARCHITECTURE.md](ARCHITECTURE.md); every test
suite is in [docs/dev/testing.md](docs/dev/testing.md); the source rules that fail a build are in
[docs/meta/tidy.md](docs/meta/tidy.md).

## Build

```bash
cargo build                                        # every crate but the desktop app (default-members; no HTTP, no perfetto)
cargo build -p lumacore-station --features perfetto,http   # full server build
cargo build -p lumacore-http                       # HTTP engine crate only
cargo build -p lumacore-cli -p lumacore-app         # clients (wire only; the app starts target/<profile>/lumacore-server, built with --features http)
./scripts/unix/build bench [local|release|dev]     # fresh timed build of every crate → a row in docs/dev/build-times.md
./scripts/unix/build bench --remote                # the same on every build server (buildservers.local.env); their rows land here
```

The scripts build `[profile.local]` (release + incremental, `target/local/`; `--ship` = full
release, `--debug` = dev). The git commit is stamped only by the station
(`apps/lumacore-station/build.rs`); never add a `build.rs` or `env!` stamp lower in the graph.

## Test

```bash
cargo test                                         # every crate but the desktop app; `--workspace` (or ./scripts/unix/test check) for all
cargo test -p lumacore-engines                     # engine registry only
cargo test -p lumacore-http                        # HTTP engine
cargo test -p lumacore-catalog-tests               # every row's rules, lum.md, the Lum words, the reference
cd tests/e2e && npx playwright test                # UI end-to-end: spec files in parallel, every file its own server
```

Where a test goes: a line needing words from more than one command crate → `tests/catalog`; the
kernel's machinery → its stand-in words; else beside its code, in `foo/tests.rs`. Every test
runs on the demo rig (`demo::fresh_state()`), never on a small show of its own.

**While building a step**, run only what the step touches: `cargo build` plus the touched
crates' tests, and the e2e files that drive the feature (one spec file or `-g "<title part>"`,
one `--project` while iterating; a kit file on all four projects before the step is called
done; `tests/e2e/README.md` says which file drives what). A change to shared ground (a kit
element, `tests/e2e/support`, `index.html`, the registry, the `Set` route) widens that to the
full suite.

**The gate**, once per step before the docs, the handoff and the commit: the full e2e,
`cargo test -p lumacore-tidy` (the rules and, through `the_docs_pass_docs_check`, the docs),
`cargo test -p lumacore-catalog-tests` (the reference, the golden files), the demo show test
when the show changed. A load flake in the gate is rerun by
file before it counts (`E2E_RETRIES=1` does that for a whole run).

Fuzzing and the command sweep are a long run only, started by hand
(`./scripts/unix/fuzz_long`), never a per-step test.

## Commits

One step per commit: the step's code, its tests, its docs and its decision record land
together, and the message says what changed and why — the rule or the bug, not the file list.
Nothing is committed while the gate is red. A commit never includes a hand-edited generated file
(the reference, a golden): regenerate instead.

## Where text goes — the placement rule

- A fact every change needs → `CLAUDE.md`, if it is non-obvious from the code, has been hit
  repeatedly, is specific enough to act on, and names its door or the tidy rule / test that
  confirms it. Otherwise:
- A rule for one area → its `.claude/rules/<area>.md`.
- A decision and its reasons → a record in `docs/decisions/`; a reversal is a new record that
  supersedes the old one, never an edit of it.
- A procedure → a skill under `.claude/skills/`.
- Status, progress → the plan in `docs/plans/active/` while open, `CHANGELOG.md` when built.
- A measurement → `docs/dev/benchmarks.md` or the plan; never a rule file.
- Research → `docs/research/<topic>/`.
- A value the code owns (a port, a format or protocol version, a rate, a count) → not in prose;
  name where the code holds it.
- Design for something not built → `visions/`.

## Writing a decision record

One file per binding decision, `docs/decisions/<NNNN>-<slug>.md`, numbered in order within its area
(the index shows the next free number); an id is never reused. The body stays under 300 words
with the three sections; longer design text links to `visions/` or the frozen log
`docs/archive/decisions-2026-10.md`. A reversal is a **new** record whose `supersedes` names the
old one; the old one's `status` becomes `superseded` and its `superseded_by` names the new one —
both files, so the pair can be checked.

```markdown
---
id: NNNN
title: <one line, the rule itself>
area: <tree | commands | render | protocols | patch | showfile | ui | lumasession | …>
status: decided | built | superseded | rejected
date: YYYY-MM-DD
built_in: <step or commit, or —>
supersedes: [<ids>]
superseded_by: <id or —>
confirmed_by: [<tidy rule, error code, test, or none>]
source: [archive §N L<from>–<to>, …]
---
# <title>

## Context
Why the question came up and what was true before.

## Decision
The rule, in the present tense, with the door or check that holds it.

## Consequences
What it costs, what it forbids, what follows for other records.
```

A record that only restates a `CLAUDE.md` invariant cites it instead of repeating it.

## `docs_check`

```bash
scripts/unix/docs_check            # every check, print findings; exit 1 when one fails
scripts/unix/docs_check --index    # rewrite docs/decisions/README.md from the records' frontmatter, then check
scripts/unix/docs_check --quiet    # only the summary line
```

Docs only — it never reads or changes code. It checks that every record has its frontmatter,
its archive source, the three sections and a body under the limit; that a superseded record and
its successor name each other; that `CLAUDE.md` stays under its line cap and carries no `§`
references, dates or history words; that every rules file has `paths:` whose globs match
something; that every repo path named in a doc exists and every local Markdown link resolves;
that no prose carries a code-owned number; and that the roadmap keeps its shape (no ticked item,
Now and Next within their caps). Run it before committing a doc; it is the Stop hook of the
docs-only agents.

## The reference is generated

`docs/user/reference/commands.md` and `docs/user/reference/objects.md` come from the
registries: `cargo run -p lumacore-station --bin gen-docs`. The test `the_reference_is_current`
(`tests/catalog`) fails until they are regenerated after a registry change. Never hand-write a
syntax table or an alias list anywhere — link the reference. The Lum grammar itself is
`docs/user/syntax/lum.md`, and any change to it is a discussion first, then the file, then the
code.
