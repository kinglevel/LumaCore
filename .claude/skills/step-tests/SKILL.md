---
name: step-tests
description: What to run while building a step and the gate before the docs, the handoff and the commit. Use when deciding which tests to run after a change, or before committing a step.
---

# Step tests and the gate

**While building a step**, run only what the step touches:

- `cargo build` plus the touched crates' tests (`cargo test -p <crate>`).
- The e2e files that drive the feature: one spec file, or `-g "<title part>"`, one `--project` while iterating. `tests/e2e/README.md` says which file drives what.
- A kit file runs on all four projects (desktop, tablet, phone, webkit) before the step is called done.
- A change to shared ground (a kit element, `tests/e2e/support`, `index.html`, the registry, the `Set` route) widens that to the full e2e suite.

**The gate**, once per step, before the docs, the handoff and the commit:

```bash
cd tests/e2e && npx playwright test        # the full e2e; a load flake is rerun by file (E2E_RETRIES=1 for a whole run)
cargo test -p lumacore-tidy                 # the rulebook, the ratchets and the docs (it runs scripts/unix/docs_check)
cargo test -p lumacore-catalog-tests        # the reference, the golden files, the lum examples, the demo show
```

`scripts/unix/docs_check` on its own prints the docs findings without a build; `--index` rebuilds the decisions index after a record change.

Run the demo show test when the show changed. Regenerate before the gate when a registry or a tree path changed: see the `regen-reference` skill.

**Never per step**: fuzzing and the command sweep are a long run only, started by hand (`./scripts/unix/fuzz_long [minutes] [--fresh] [--no-fuzz]`, report in `target/fuzz-report/summary.md`). Property tests (`cargo test -p lumacore-sweep`) are ordinary fast tests.

Every test runs on the rig (`demo::fresh_state()`); never write a small show of your own.
