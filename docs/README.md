# LumaCore Manual

**Start here:** [Getting Started](getting-started.md).

## The two files that lead

- [meta/roadmap.md](meta/roadmap.md) — **the plan.** One living list: done (with commits),
  now, next, later, ideas. Every task lives here, nowhere else.
- [decisions/](decisions/) — **the rules**: one record per binding decision (the index is its
  `README.md`). The frozen log they came from is [archive/decisions-2026-10.md](archive/decisions-2026-10.md);
  a `§N` anywhere names its section there. Read the records of an area before changing it.

The language itself is defined in [user/syntax/lum.md](user/syntax/lum.md).
Every command and object is listed in the generated reference:
[user/reference/commands.md](user/reference/commands.md) and
[user/reference/objects.md](user/reference/objects.md)
(`cargo run -p lumacore-station --bin gen-docs`; a test fails when they are stale — never edit them by hand).

## Spaces

| Space | Audience | Content |
|-------|----------|---------|
| **[user/](user/README.md)** | Operators, show designers | How to use LumaCore: syntax, terminal, show files, object tree, fixtures, presets, playback, macros, the generated reference |
| **[dev/](dev/README.md)** | Contributors, AI agents | How to build and extend it: architecture, build, testing, engines, networking, API |
| **[meta/](meta/README.md)** | Project | Roadmap and decisions |
| **[archive/](archive/README.md)** | Anyone digging | Superseded docs, kept unchanged for their data |

Outside `docs/`: [visions/](../visions/README.md) (ideas and notes not yet planned),
[docs/plans/](plans/README.md) (active plans, finished ones in `docs/plans/done/`) and [docs/research/](research/README.md) (research: input, never a rule).

## Placement guide

- Teaches operators how to use a feature → `user/`
- Explains how to build or extend LumaCore → `dev/`
- A task or a plan → `meta/roadmap.md`
- A decision (how it works, why) → a record in `decisions/`; a reversal is a new record
- An idea nobody has planned yet → `visions/`
- A doc that is replaced → move it unchanged to `archive/<area>/` and add a row to
  [archive/README.md](archive/README.md). Never delete docs.

## Structure rules

- **Filenames:** kebab-case, lowercase (e.g. `cli-terminal.md`, not `CLITerminal.md`)
- **Folders:** flat, descriptive names matching the topic area
- **Adding pages:** put the file in its space and section, then add it to that space's README
- Command syntax lives only in the generated reference; hand-written docs explain concepts
  and link to it — no hand-kept syntax or abbreviation tables
- `user/syntax/lum.md` is the language source — other files link to it, not compete with it
