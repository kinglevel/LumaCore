# 30 — Build and tooling leftovers

Small items from 2026-10-13, none decided yet.

- **`proc-macro-error2 v2.0.1` future-incompat warning** — pulled in by `tabled 0.17 →
  tabled_derive 0.9`. Upgrading `tabled` (0.22 drops it) removes the warning; check API changes.
- **Unused variable `menu`** at `apps/lumacore-app/src/app.rs:~277` (compiler warning).
- **e2e global setup relinks the debug server without perfetto**: `tests/e2e/support/global-setup.ts`
  builds with only `http`, so a later `perfetto,http` build relinks again (and the other way round).
  Left as is because of a possible clash on the fixed Perfetto port (47657) during tests.
- **Not yet run through the new scripts:** `smoke_test` and the e2e suite (`./scripts/unix/test
  smoke`, `test e2e`).
- **Dev box without Rust for the operator user**: on the Debian box only root has cargo
  (`/root/.cargo`); the scripts reuse the existing binaries. Install rustup for the user, or keep
  building as root and `chown` afterwards.
- **Windows twins of the new scripts** — on the roadmap (Later → Developer scripts).
