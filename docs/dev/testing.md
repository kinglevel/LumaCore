# Testing

The suites, what to run per step and once at the gate, the property tests and the long run.
Every test runs on the rig: `demo::fresh_state()` (the demo show built from its sources, a
copy per test) — a test names the rig's facts or picks numbers past it, never a small show of
its own (archive §40).

| Suite | Where | Run | When |
|---|---|---|---|
| Unit tests | `#[cfg(test)] mod tests;` beside the code in every crate (`foo/tests.rs`) | `cargo test -p <crate> [filter]` | Every step, the touched crates |
| The catalog's tests | `tests/catalog`: every row's rules, lum.md's examples, the Lum words golden, the tree paths golden, the rights table, every menu's JSON, the reference (`the_reference_is_current`), the demo golden (`the_demo_show_builds_loads_and_matches_its_golden`), the shim ratchet | `cargo test -p lumacore-catalog-tests` | The gate; a registry change |
| Tidy | `crates/dev/lumacore-tidy`, every rule in [tidy.md](../meta/tidy.md) | `cargo test -p lumacore-tidy` | The gate; a structure change |
| Property tests | `crates/dev/lumacore-sweep/src/tests.rs` (proptest, fixed seeds) | `cargo test -p lumacore-sweep` | A step in their area |
| Render budget | `a_35k_parameter_frame_renders_under_3_ms` (`lumacore-render`, release) + `cargo bench -p lumacore-render --bench render` | `cargo test --release -p lumacore-render a_35k` | A render-path change |
| UI end-to-end (Playwright) | `tests/e2e/` | `cd tests/e2e && npx playwright test` | The spec files that drive the feature per step; the whole suite at the gate |
| systemtest | `tests/systemtest/` | `cargo test -p systemtest` | The gate (`--workspace`); a dispatch or registry change |
| Screenshots | `tests/e2e/tests/screenshots-all.spec.ts` | `./scripts/unix/screenshots` (→ `target/screenshots/`) | While building UI: look at every window type, menu and overlay |
| The long run (sweep + fuzz) | `crates/dev/lumacore-sweep`, `tests/fuzz/` | `./scripts/unix/fuzz_long` | By hand, overnight; never per step |

## What to run when

- **While building a step:** `cargo build` + the touched crates' tests, and the e2e files that
  drive the feature (one spec file or `-g "<title part>"`, one `--project` while iterating; a kit
  file on all four projects before the step is called done; [`tests/e2e/README.md`](../../tests/e2e/README.md)
  says which file drives what). A change to shared ground (a kit element, `tests/e2e/support`,
  `index.html`, the registry, the `Set` route) widens that to the full suite.
- **The gate, once per step before the docs, the handoff and the commit:** the full e2e,
  `cargo test -p lumacore-tidy`, `cargo test -p lumacore-catalog-tests` (the reference, the
  golden files), the demo show test when the show changed. A load flake in the gate is rerun by
  file before it counts (`E2E_RETRIES=1` does that for a whole run).
- Where a test goes (§34 25.16): a line needing words from more than one command crate →
  `tests/catalog`; the kernel's machinery → its stand-in words
  (`crates/model/lumacore-objects/src/commands/test_words.rs`); else beside its code.
- A golden rewrites with `LUMACORE_BLESS=1`; read the diff.

`cargo test` runs every crate but the desktop app (`default-members`); `--workspace` (or
`./scripts/unix/test check`) runs all.

## UI end-to-end (Playwright)

`tests/e2e/` drives the real interface in Chromium: the server's web UI, the same UI in app
mode (`window.LUMACORE_APP`, `window.LumaHost`), and the app's connect page. Each spec file
starts its own `lumacore-server` (built with the `http` feature by the global setup) in a temp
directory on a free block of ports and with `--RootMonitor`, so sysmon lines can be asserted.
Your running server and show are never touched. Spec files run in parallel, one worker each;
the tests inside a file share its server in order. The worker count and the measured run times
are documented in `tests/e2e/playwright.config.ts` (`E2E_WORKERS=n` overrides).

```bash
cd tests/e2e
npm install                                          # once
npx playwright test                                  # the whole suite
npx playwright test tests/login-sysmon.spec.ts -g "Debug"
npx playwright test tests/kit.spec.ts --project=phone
E2E_RETRIES=1 npx playwright test                    # rerun a failed file once; a pass then is "flaky"
```

Projects (archive §30): every spec runs on `desktop`; the kit's specs (`kit*.spec.ts`) also run
on `tablet`, `phone` and `webkit` (needs `npx playwright install webkit` once). Helpers `tap()`
(checks nothing covers the element) and `hold()` live in `support/server.ts`; kit elements
carry `data-kit`. Which file drives what, and what is not covered (the native app itself):
[`tests/e2e/README.md`](../../tests/e2e/README.md).

## systemtest

`tests/systemtest/` is a workspace member: curated state checks (load the rig, dispatch known
lines, verify the tree, programmer, watches and path state) and a registry syntax sweep (every
command row's examples against a fresh state), grouped by domain in `src/test_domains/`
(`curated_groups()` in its `mod.rs`); its `build.rs` rebuilds it when the registry, dispatch or
the demo's sources change and writes a manifest and the last run under `target/systemtest/`.
Policy: systemtest follows the definitions ([lum.md](../user/syntax/lum.md), the generated
reference, the records) — when a decision changes a command's meaning, ownership, LumaSession
behaviour, object identity or runtime structure, update it in the same work. How to add a
domain and run a group: [`tests/systemtest/README.md`](../../tests/systemtest/README.md).

## Stale object files (macOS, rust#161824)

Since Rust 1.88 each incremental build leaves its `.o` files in `target/<profile>/deps` under new
names; rustc lists that directory on every run, so builds slow down as it grows.
`./scripts/unix/prune_objects` deletes every `*.rcgu.o` no executable names in its debug map once
a `deps/` holds more than its threshold (`--force`: always); `build`, `test quick` / `test crate`
and the project's Claude Code hook run it after cargo. Cargo still sees everything fresh afterwards
and backtraces keep `file:line`. Plain `cargo` doesn't run it — run it by hand after a busy day.
Remove it once the rustc fix ([rust#162561](https://github.com/rust-lang/rust/pull/162561)) is stable.

---

# Property tests and the overnight long run (archive §25 "Step 10 hardening")

Two kinds of test, run differently:

- **Property tests** — ordinary fast `cargo test`s (proptest), run per step with their area:
  `cargo test -p lumacore-sweep`. Command-sequence properties use fixed seeds in `cargo test`
  (deterministic); the long run gives them a random seed.
- **The long run** — fuzzing and the command sweep, **never per step**. Started by hand,
  overnight: `./scripts/unix/fuzz_long [minutes] [--fresh] [--no-fuzz]`.

## Property tests (`crates/dev/lumacore-sweep/src/tests.rs`)

Random command sequences from the rig through real dispatch (half from the sweep's exhaustive
plan, half grown by completion), plus the pure rules:

| Property | Test |
|---|---|
| no panic, no duplicate GUID after any sequence | `no_line_panics_or_duplicates_a_guid` |
| `.luma` save → load: same content, clean identity | `a_saved_show_loads_back_the_same` |
| `.luma` save → load → save: same bytes | `a_saved_show_saves_the_same_bytes` |
| a refused line (E-RIGHTS-001 / E-CLAIM-001) changes nothing | `a_refusal_changes_nothing` |
| a member fed the master's ops has the master's show | `a_member_ends_with_the_masters_show` |
| Oops restores the pre-image, Redo repeats, ReadOnly changes nothing | `oops_restores_and_redo_repeats` |
| no line leaves a new broken link | `no_line_breaks_a_link` |
| merge(base, x, x) = x; merge(base, base, mine) = mine (bar Playbacks); unique GUIDs | `merging_a_show_with_itself_changes_nothing`, `an_untouched_master_takes_mine`, `a_merge_keeps_guids_unique` |
| ops apply strictly in order (repeat ignored, gap → resync) | `ops_apply_strictly_in_order` |
| every member elects the same winner | `every_order_elects_the_same_winner`, `distinct_stations_elect_the_same_winner` |
| the clock filter keeps its frequency bounds | `the_clock_filter_keeps_its_frequency_bounds` |
| masking never leaks a password and is idempotent | `masking_hides_the_password_and_is_idempotent` |

A property the code breaks today is `#[ignore = "finding: …"]` (run with `-- --ignored`);
fix the code, then drop the ignore.

## The long run

`scripts/unix/fuzz_long` builds `lumacore-sweep` (profile `sweep`: release + debug
assertions) and runs `lumacore-sweep long`, restarting it after a crash or hang, then the
property tests with a random seed. Phases, all resumable:

1. **Command sweep** — the exhaustive plan (`sweep::lines::exhaustive`), every case as one
   user per rights level (generated from `UserRights::ALL`) from a fresh world (an `Owner` user
   holds a claim on Group 1): every command name, each command × every object × ids × the
   flags its docs mention, each command × its `Args` shape, every object called (ids, `At`,
   `Thru`, flags), every writable property × valid / edge / wrong values (`Set …` read back),
   every read-only field (`Set` refused), every doc example. **Generated from the registries at
   run time** (`sweep::vocab`): a new command, keyword, flag or property is swept the next
   night; never add a hand-written command list.
2. **Random** — sequences grown word by word from the completion engine, random roles,
   `--minutes` long; replayable from (seed, sequence).
3. **Fuzz** — every cargo-fuzz target in `tests/fuzz/` for `--minutes` each (`cargo +nightly
   fuzz run`, crashes kept going, inputs in `tests/fuzz/artifacts/<target>/`).

Lines with `reach: Show` and `reach: Drive` run (`CommandDef::reach`, `ObjectDef::reach`,
`registry::line_reach`, which also counts a path naming `Root.Drives`); the tree shell
(`ls`, `cd`, `cat`, …) is never swept. The sweep runs in process — no server, no protocol
runtimes, no engine threads, no sockets. The kernel reaches files only through the drive door
(tidy `core_reaches_the_disk_only_through_drive`): a **Show** line finds that door shut (a knock
is a `drive` finding); a **Drive** line (SaveShow, LoadShow, Import, Export, Shutdown, …) runs in
the **sandbox** — a throwaway copy of `library/` in `target/fuzz-report/sandbox/` (no station
identity, secrets or logs) — where the door lets only relative paths without `..` and absolute
paths inside it through; anything else is a `drive-escape` finding (archive §29). Planned verbs
are not swept; a test checks they change nothing.

**Checks after every line:** no panic (`Internal error in …`) and nothing over the per-line
budget (a line far over it is a hang: the watchdog ends the process, the next start skips it);
no new duplicate GUID or broken link; a rights / claim refusal changes nothing; a `ReadOnly` word
changes nothing; every `Error:` reply carries an `E-` code (`uncoded-error`; the tidy rule
`error_replies_carry_a_code` keeps `text::render_error` inside the kernel's `errors`); a tracked
change comes back exactly with `Oops` and again with `Redo`; `Set` reads back or refuses with a
code; allowed lines behave as for Admin. **Periodically** (the sweep's own interval): save → load
→ save, the in-process member against the master, one render.

**Report** — `target/fuzz-report/`:

| File | What |
|---|---|
| `summary.md` | segments, findings by kind with counts, lines that took the process down, words never accepted |
| `findings.jsonl` | the first finding of each kind: check, line, detail, a replay command |
| `replays/NNNN.txt` | the lines (role, line) that led to it: `cargo run --profile sweep -p lumacore-station --bin lumacore-sweep -- replay <file>` |
| `run.json` | status, pid, build, phase, seed, counts (a checkpoint every so often) |
| `logs/` | each night's output; `fuzz-<target>.log` per fuzz target |

**Resume:** a start reads `run.json` — completed → a new run (the old one moves to `runs/`);
stopped → continue; still "running" with its pid gone → crashed: the line in flight is a
finding, skipped, continue. A new build continues and runs the cases it added first;
`--fresh` starts over.

## Fuzz targets (`tests/fuzz/`, nightly, outside the workspace)

The targets are the files in `tests/fuzz/fuzz_targets/` (protocol decoders, the wire's
requests and packets, the stream frame, the beacon, the token, command lines, the Lum parser).
Each checks no panic and no hang on any bytes; codecs that also encode check decode → encode →
decode. Private decoders are reached through `lumacore_sweep::fuzzing` (the fuzz crate builds
the sweep, never the server; archive §34 20.10). One by hand:

```bash
cargo +nightly fuzz run osc --fuzz-dir tests/fuzz -- -max_total_time=60
```

Findings are reported, fixed later; there are no input size caps (isolated networks, §19).
