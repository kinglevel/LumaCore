# Faster tests (systemtest and every crate) — handoff 2026-10-18

## Status (2026-10-20)

Built and committed (roadmap Later → "Build times": "Test speed, part 1" and `prune_objects` ticked; the frozen log `docs/archive/decisions-2026-10.md` §34 "Build times hardened" and §40 "40f built" carry the rules): step 1 (Developer Tools for the terminal, the `target/` clean), step 1b (`scripts/unix/prune_objects`, run by `build`, `test quick` / `test crate` and the Claude Code hook), step 2 (systemtest: the demo show read once per thread, the cases on one worker thread per core — the "Systemtest" section below says "not committed"; it is in). Since then every test runs on the rig through `demo::fresh_state()` (§40 40f: built once per process, a copy each), which is the "load once, clone per case" idea below. Of step 4, the e2e already run spec files in parallel, every file on its own server (`tests/e2e/playwright.config.ts` sets the workers, `E2E_WORKERS=n` overrides; `tests/e2e/README.md`).

Remains (roadmap Later → "Test speed, the rest (handoff steps 3–5)"): step 3 — `test = false` / `doctest = false` where nothing is tested, one test binary per crate; step 4's rest — desktop-only by default, the fixed `waitForTimeout` sleeps in `tests/e2e/tests/` replaced by waits on the condition; step 5 — nextest with a `check` profile, optional. `prune_objects` goes once rust#162561 ships (its own roadmap line). The findings below are kept as written on 2026-10-18.

## Finding 1: the macOS first-launch scan was the biggest cost

The first run of a newly linked binary waits 20–80 s with **0 CPU** while macOS checks it (Gatekeeper / XProtect; `XprotectService` was busy at the time). Running it again takes 0.00 s.

| Binary | First run | Second run | Its tests |
|---|---|---|---|
| `lumacore-scripts` | 82.9 s | 0.0 s | 3 tests, 0.00 s |
| `lumacore_timecode` | 56.1 s | 0.0 s | 0 tests |
| `lumacore_render` | 78.4 s | 0.5 s | |
| `lumacore_patch` | 75.1 s | 0.0 s | |
| Fresh copy of `lumacore_tags`, re-signed | 27.3 s (user 0, sys 0) | 0.00 s | |

- `cargo test` relinks up to **62 test binaries** (listed with `cargo test --workspace --no-run --message-format=json`), and each one pays this delay.
- So do build scripts (`build.rs` executables) and the e2e server binary.
- `cargo test --workspace --no-run` took 3:18, with 168 s of system time against 130 s of user time.

**Fix (the user does this; it is a security setting):**
1. Run `sudo spctl developer-mode enable-terminal`.
2. Open System Settings → Privacy & Security → **Developer Tools**, add iTerm and any other app that runs cargo, and switch it on.
3. Restart iTerm.

**Check after the restart:**
- Touch one crate and run `cargo test -p lumacore-tags`. The binary should start at once.
- Repeat the copy test: `cp <bin> x; echo junk >> x; codesign -f -s - x; time ./x -q`. It should show about 0 s of wall time.

**Done 2026-10-18:** Developer Tools is on for iTerm. A re-signed copy now starts in 0.013 s. That alone did not speed up `cargo test` (Finding 1b).

## Finding 1b: `target/` had piled up to 155 GB and over a million files

- `target/debug/deps` held about 739,000 files, of which 730,227 were `.o` files.
- On macOS a dev build keeps each link's `.o` files (`split-debuginfo = unpacked`), because the binary reads its debug info from them. Every relink writes a new set, and cargo never removes the old ones.
- rustc lists `deps/` to find crates on every invocation; a plain `ls` of it took more than two minutes. Spotlight was indexing it too.
- The relink after touching `lumacore-base` took 2:36 wall, with 44 s user and **172 s system**. rustc used about 40 % of one core.
- The machine was also short of memory: 5.9 GB of swap on 18 GB of RAM, and another console application using one full core. That is not fixed and still costs something.
- **Done:** `cargo clean` removed 1,088,998 files (154.6 GiB). `target/.metadata_never_index` keeps Spotlight out.

| After the clean | Wall | User | Sys |
|---|---|---|---|
| Fresh `cargo test --workspace --no-run` | 1:05 | 548 s | 50 s |
| Relink after touching `lumacore-base` | **10.3 s** (was 2:36) | 39 s | 34 s |
| `cargo test --workspace` run, already built | 1:14 | 202 s | 6 s |

- **The pile-up starts again at once:** one relink took `deps/` from 14,115 to 25,972 files.
- **1b measured** (each option in its own target dir: a fresh `cargo test --workspace --no-run`, then three relinks after touching `lumacore-base`; `file:line` checked with a panicking test and `RUST_BACKTRACE=1`):

| `[profile.dev]` | Relink | `.o` per relink | Backtrace `file:line` | Size after 3 relinks |
|---|---|---|---|---|
| unpacked (today) | 10.3 s | **+11,857** (up to 512 per crate) | yes | 6.4 GB, growing |
| `split-debuginfo = "off"` | **9.8 s** | 0 | no | 6.4 GB |
| `split-debuginfo = "packed"` | 14.5 s (dsymutil on every link) | 0, a `.dSYM` | yes | 10 GB |
| `debug = false` | 11.1 s | 0 | no | 5.6 GB |

  - With `off` or `debug = false`, a panic still names its own location (`panicked at src/x.rs:1:29`, and `panic_report`'s `Internal error in <file>:<line>` uses that). Only the backtrace's frames lose `file:line`; they keep their function names.
- **Research (3 agents, 2026-10-18):**
  - **The cause is a rustc bug:** [rust#161824](https://github.com/rust-lang/rust/issues/161824), open, filed 2026-08-26.
    - Since 1.88, [rust#139453](https://github.com/rust-lang/rust/pull/139453) gives each rustc run random temp names, so the `.o` files kept for unpacked debug info never overwrite the old set.
    - The reporter had 3.29M files (341 GB), with rustc stuck in `getdirentries64`.
    - The fix [rust#162561](https://github.com/rust-lang/rust/pull/162561) is open and waiting on its author. [cargo#16665](https://github.com/rust-lang/cargo/issues/16665) reports 5.7× slower incremental builds from it.
    - The workaround from the issue: `find target -name '*.rcgu.o' -mtime +2 -delete`.
  - **No stable target-dir gc exists.** `cargo clean gc` is unstable, [cargo#17524](https://github.com/rust-lang/cargo/pull/17524) is an open PR. Build-dir layout v2 ([cargo#15010](https://github.com/rust-lang/cargo/issues/15010)) arrives with 1.100; it spreads `deps/` into one directory per unit but doesn't stop the leak. cargo-sweep is unmaintained.
  - **What big projects do:**
    - Nobody uses `off` or `packed` for dev. Zed, Biome, Materialize and rustc's own bootstrap keep `unpacked`.
    - The common change is `debug = "line-tables-only"` (uv, Ruff, Turbopack, Polars, Biome), which we already have.
    - Materialize builds `backtrace`/`addr2line`/`gimli`/`object` at `opt-level = 3`, because "otherwise backtraces can take 20s+ to symbolize".
  - **Tools:** with `off`, lldb, Instruments, samply and Tracy also get only function names. With `unpacked` they read the debug map into the `.o` files.
  - **Checked here:** after deleting every `.rcgu.o`, cargo reports everything fresh (nothing rebuilds). A binary built before the delete loses `file:line` in its backtrace until its next relink, which brings them back.
- **Built 2026-10-18:** kept `unpacked` + `line-tables-only` and added `scripts/unix/prune_objects` (Python, for `flock`).
  - **The rule:** past 40,000 entries in a `deps/` (APFS: dir size / 32, 13 ms), delete every `*.rcgu.o` that no executable in `deps/` names in its debug map. The names come from grepping the binaries for `/….rcgu.o` and taking the file name; this was checked equal to `nm -ap` OSO entries (0 missed, 3.4 s against 48 s for `nm`).
  - **Age can't decide it** (tried and dropped):
    - A reused codegen unit is a hard link into the incremental cache and keeps its old mtime, so `-mmin +60` deleted objects that brand-new binaries use.
    - Link counts don't separate orphans either: each relink adds another name for the same inode.
  - **The lock:** it takes `target/<profile>/.cargo-build-lock` with `flock` without waiting. During a build it skips (checked); a build started meanwhile waits.
  - **Measured:** deleted 49,265 of 59,539 in 5 s. Cargo then compiled nothing, and the five binaries checked (tags, systemtest, objects, server, catalog-tests) had 0 missing debug-map objects.
  - **Run by:** `./scripts/unix/build`, `test quick` / `test crate`, and a Claude Code PostToolUse hook. The user adds the hook; the auto-mode classifier refused my editing of the settings.
  - Roadmap: remove it once rust#162561 ships.
  - Earlier options, for the record: `off` (fastest, smaller backtraces), `packed` (keeps backtraces, +4.5 s per relink), or keep unpacked and prune old `.o` files in `./scripts/unix/build`.

### Where the 1:14 test run goes (68.3 s of tests, run one binary after another)

| Binary | Time |
|---|---|
| `systemtest` | 33.1 s (one test) |
| `lumacore_sweep` | 12.9 s (15 tests) |
| `lumacore_objects` | 5.7 s (395 tests) |
| `lumacore_catalog_tests` | 5.1 s |
| `lumacore_http` | 3.0 s |
| `properties` | 1.5 s |
| every other binary | under 1.2 s each |

- 52 doctest runs as well; almost no crate has real doctests (`doctest = false`, step 3).
- Failures: only `lumacore-syntax --test lum_md`, with 3 failing (`clean_lines_highlight_without_errors`, `every_lum_example_parses_and_prints_back`, `the_cursor_expects_what_comes_next`). This is the known `lum-syntax` failure.

## Finding 2: once the scan is out of the way, the tests themselves are fast

- In the 34 binaries measured so far, every test run takes under 0.5 s, except `properties` (proptest) at 1.6 s.
- Not measured yet: `systemtest`, `lumacore-objects`, `catalog-tests`, `lumacore-server`, `lumacore-http`, `lumacore-sweep`, `tidy`.
- Partial data is in the scratchpad `times2.tsv`, which may be lost. To redo it, run each binary with `--list` first (absorbs the scan), then time `-q`.
- **`lum_md` currently fails on `lum-syntax`: 2 passed, 3 failed.** This is not caused by the speed work.

## Systemtest: built 2026-10-18 (33.1 s → 2.2 s), not committed

- Measured: `fresh_state()` was called 694 times (one per case, one per syntax example, one for the batch) at 34.5 ms each, 24.2 s of 31.5 s. `AppState::new()` takes 0.3 ms; `new()` + `replace_show_data(loaded.clone())` takes 0.6 ms.
- `harness::fresh_state` now loads the demo show once per thread (a thread-local template; `AppState` isn't `Sync`). Each case gets `AppState::new()` + `replace_show_data(template.show_data.clone())` + the template's `saved`, which is what `load_show` does after reading the file. That took it to 11.4 s.
- After that, 9.3 s was the Syntax group: 481 examples, ~18 ms dispatch each (mutating lines ~25 ms in debug, the change path's fingerprints; production code, left alone).
- `run_all_groups` now runs every case on one worker thread per core (8 MB stacks, like the main thread). Each case's log is buffered (`run_test(test, &mut log)`) and printed in order after. 15 runs in a row: all 206 pass, 2.15–2.19 s.
- The one socket test (OSC binds) is a single case. The batch-timing threshold is 2,000 ms against ~170 ms.
- README and `docs/dev/testing.md` updated: a case must not depend on another having run first.

### Before (kept for the record)

- `tests/systemtest/src/main.rs`: one `#[test] systemtest_full_suite` runs every group **serially** in one thread.
- `harness::fresh_state()` calls `AppState::load_show(demo.luma)` **for every case**. That means unzipping the show and re-deriving every fixture type from its `.gdtf` each time.
- README: "a full run takes more than a minute".
- Ideas:
  - Load once into a `OnceLock<AppState>` (or the loaded `ShowData`) and clone it per case. This needs `AppState: Clone`, or a cheap "reset to this ShowData" path.
  - Make each domain group its own `#[test]` so cargo runs the groups in parallel. The cwd is process-global (`enter_work_dir`), so check that first.
- Measure first: time `fresh_state()` against the dispatch time per case.

## What big projects do (3 research agents, 2026-10-18)

### Build and link
- **Merge each crate's `tests/*.rs` into one binary**, using `autotests = false` and `tests/it/main.rs` (matklad's "Delete Cargo Integration Tests").
  - Cargo itself saw about 3× faster test compiles this way.
  - nushell and Pernosco do it too (`[lib] test = false` plus one test crate).
  - Fewer binaries also means fewer first-launch scans.
- **`[lib] doctest = false`** on crates without real doctests (rust-analyzer and uv do this almost everywhere). Each doctest is its own link.
- **`test = false` on bins without tests**, as cargo itself does (`[[bin]] test = false`). The station's bins, `lumacore-cli` and `lumacore-headers` currently build empty test binaries.
- `[profile.dev] debug = "line-tables-only"` is already set, and split-debuginfo is unpacked by default on macOS.
- A different linker is low value: ld-prime is about as fast as lld on macOS.
- Neither sccache (doesn't help incremental builds or linking) nor Cranelift (on macOS a panic aborts instead of unwinding, so one failing test kills the binary) helps here.
- `opt-level = 3` on hot dependencies and proc macros (rust-analyzer, Zed, Deno). We already have `opt-level = 1` for every dependency.

### Running the tests
- **cargo-nextest** (uv, ruff, Zed, rust-analyzer, Materialize, TiKV):
  - Runs the tests of every binary at once instead of one binary at a time; benchmarks show 1.4–3.4×.
  - `.config/nextest.toml` gives slow-timeouts, `test-groups` for serial tests, and slow-first `priority`.
  - Catch: each test gets its own process, so a `OnceLock` fixture is rebuilt in every test. Systemtest, which shares one loaded show, would rather stay one binary with threads, or share a prepared file.
- **Tiers** (doc it in `docs/dev/testing.md`):
  - An env gate like rust-analyzer's `skip_slow_tests()` / `RUN_SLOW_TESTS`.
  - Or a nextest `default-filter` that leaves the heavy crates out of local runs, with a `ci`/`check` profile that runs everything.

### e2e (Playwright, `tests/e2e`)
- Config today:
  - `fullyParallel: false`, `workers: 1`. Each of the 26 spec files starts its own server on a hard-coded port.
  - The tablet, phone and webkit projects repeat the 17 `kit*` files, so `npx playwright test` runs each kit file 4 times.
  - `globalSetup` builds a debug server.
- Ideas:
  - **Port clashes to fix first:**
    - 48740: `feedback.spec.ts:9` and `screenshot.spec.ts:12`.
    - 48760: `feedback.spec.ts:127` and `screenshots-all.spec.ts:21`.
    - 48720: `kit-tags.spec.ts:10` and `ui-keys.spec.ts:129`.
  - Then derive the port from `test.info().parallelIndex` and set `workers: '50%'`. Spec files would run in parallel, but each file stays serial.
  - Run desktop by default and the other devices only under `test all` / `check`.
  - Use `--only-changed main` and `--last-failed` locally.
  - Replace the 44 `waitForTimeout` calls (about 46 s per desktop pass) with waits on the condition:
    - `feedback.spec.ts:85,94,107,116` (3.5–6 s each).
    - `login-sysmon.spec.ts:133,144` and `feeds.spec.ts:173` (3 s each).
    - `kit-12e`, `kit-pool` and `kit-12f`.
  - Consider building the `local` profile in `globalSetup` instead of debug.
- Others do the same: Grafana (`fullyParallel`, `storageState`, retries only on CI) and Immich (stateless projects in parallel, stateful ones with `workers: 1`). Our login is already cheap, so `storageState` gains little.

## Suggested order (waits for the user's go)
1. ~~The user enables Developer Tools for iTerm (above), then re-measures `cargo test` end to end.~~ Done; see Finding 1b.
1b. ~~A lasting fix for the `.o` pile-up~~ built: `scripts/unix/prune_objects` (Finding 1b).
2. ~~Systemtest~~ built: 33.1 s → 2.2 s. Was: measure, then load the demo show once and clone it per case, with groups as separate tests.
3. `test = false` / `doctest = false` where nothing is tested; merge each crate's `tests/*.rs` into one binary.
4. e2e: fix the port clashes, run in parallel, desktop by default, replace fixed sleeps.
5. Optional: try nextest with a `check` profile; keep `cargo test` working.
