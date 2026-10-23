# docs/dev moves (md upgrade, 2026-10-20)

One line each: old path → new path, or "deleted, content now in X". For `docs/archive/README.md`.

- `docs/dev/architecture/architecture.md` → deleted; the crate table and repo layout are in `docs/dev/architecture/crate-map.md`, the server / client model and the ports pointer ("the ports are constants in `crates/foundation/lumacore-wire/src/lib.rs`") in `docs/dev/engines/engines.md` and `docs/dev/networking/lumasession.md`, the log folders in `docs/dev/build/server-diagnostics.md`; the bird's-eye view is the root `ARCHITECTURE.md` (written by another task). Links still to fix outside this task's paths: `README.md` (the root, "Architecture" row), `docs/user/identity/software-init.md`, `docs/meta/roadmap.md` (Done lines naming `ui-framework.md`).
- `docs/dev/networking/network-engine.md` → `docs/archive/dev/networking/network-engine.md` (March conceptual doc; the listeners, discovery and NICs are `lumacore-net` and `lumacore-server`, described in `docs/dev/networking/lumasession.md` and `docs/dev/engines/engines.md`)
- `docs/dev/engines/runtime.md` → `docs/archive/dev/engines/runtime.md` (March conceptual doc; live runtime vs saved truth is in `docs/dev/engines/engines.md` "Authored vs runtime" and `crate-map.md`)
- `docs/dev/ui-framework.md` → `docs/archive/dev/ui-framework.md` (the panel-prototype decision log D1–D8; its "Built so far" kit table is superseded by the new `docs/dev/ui-kit/README.md`, which lists every kit element by its file)
- `docs/dev/networking/replication.md` → kept, trimmed to the change path, the ops and the member-side apply; the status note, data-flow sketch and transport section that repeated `lumasession.md` are gone
- `docs/dev/engines/engines.md` → kept; the "Current Implementation Status" list (the roadmap owns status) and the engine relationship trees are gone
- `docs/dev/testing.md` → kept; the systemtest directory layout and group table now live only in `tests/systemtest/README.md` and the code
- new: `docs/dev/ui-kit/README.md` (the kit's elements and modules)
