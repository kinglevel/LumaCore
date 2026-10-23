# Contributor Manual

How to build, extend and contribute to LumaCore. The rules are one record per file in
[`docs/decisions/`](../decisions/); the frozen log they came from is
[`docs/archive/decisions-2026-10.md`](../archive/decisions-2026-10.md) (a `§N` in these pages
names its section); the plan is [`docs/meta/roadmap.md`](../meta/roadmap.md).

One line per file:

## Architecture
- [architecture/crate-map.md](architecture/crate-map.md) — which crate owns what, the direction, quick routing
- [architecture/code-structure.md](architecture/code-structure.md) — how every `.rs` file and crate is laid out, the shared doors, the ratchets

## Build and tools
- [build/vibecode-workflow.md](build/vibecode-workflow.md) — the per-step build and test habit, the long run, the companion workspaces
- [build/build-environment.md](build/build-environment.md) — `.env.local`, path variables, link rules
- [build/dev-init.md](build/dev-init.md) — what the `dev_init` scripts do (Tracy install, snippets, the launcher)
- [build/tools.md](build/tools.md) — the developer utilities and scripts, by pointer

## Testing
- [testing.md](testing.md) — every suite, what to run per step and at the gate, the property tests, the long run

## Diagnostics and performance
- [build/metrics-and-health.md](build/metrics-and-health.md) — the render budget, `Runtime.Health`, enforcement
- [build/profiling.md](build/profiling.md) — Tracy and Perfetto
- [build/server-diagnostics.md](build/server-diagnostics.md) — the System Monitor: lines, levels, readers, tags

## Engines
- [engines/engines.md](engines/engines.md) — the engine boundaries, the registry engines, what is not an engine

## Networking
- [networking/lumasession.md](networking/lumasession.md) — LumaSession: roles, link, election, merge, clock, services, code map
- [networking/replication.md](networking/replication.md) — the change path, the ops, the journal, the member-side apply

## API and UI
- [api/http-api.md](api/http-api.md) — every HTTP route: REST, the two WebSockets, the feeds, Perfetto
- [ui-kit/README.md](ui-kit/README.md) — the kit's elements and modules
- [ui-kit/mockups/README.md](ui-kit/mockups/README.md) — the approved designs, by dated folder

## Integrations
- [integrations/integrations.md](integrations/integrations.md) — protocols, OSC input, APIs, the integration rule

## Firmware

## Measurement logs
- [benchmarks.md](benchmarks.md) — render and change-path measurements
- [build-times.md](build-times.md) — timed builds per machine (`./scripts/unix/build bench`)

## Old and vision docs
- [`docs/archive/`](../archive/README.md) — superseded docs, kept unchanged (the March conceptual
  docs `dev/engines/runtime.md` and `dev/networking/network-engine.md`, the panel-prototype
  decision log `dev/ui-framework.md`, the old networking docs, `api-web-ui.md`, …)
- [`visions/`](../../visions/README.md) — designs for what is not built (presets, macros,
  timelines, effects) and the notes
