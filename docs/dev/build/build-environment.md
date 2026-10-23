---
title: Build Environment
category: docs
---
# Build Environment

Canonical language source: [`lum.md`](../../user/syntax/lum.md)

This file defines how developer environment configuration works in LumaCore.
All external paths, companion repos, and developer-specific settings are
managed through environment files — never hardcoded.

## Principle

No absolute user paths should ever appear in committed files.
Internal repo links use relative markdown paths.
External repo references use named variables defined in `.env.local`.

## Environment Files

| File | Committed | Purpose |
|------|-----------|---------|
| `.env.example` | Yes | Template with documented variables and placeholder values |
| `.env.local` | No (gitignored) | Developer's personal paths — copied from `.env.example` |

### Setup

```bash
cp .env.example .env.local
# Edit .env.local with your local paths
```

The `dev_init` scripts (`scripts/unix/dev_init.sh`, `scripts/win/dev_init.ps1`)
read `.env.local` on startup, validate that referenced paths exist, and warn
about any missing entries. If `.env.local` does not exist, the scripts print
setup instructions and continue — external paths are optional.

## Variable Reference

| Variable | Description | Example |
|----------|-------------|---------|
| `SKILL` | Your own companion workspace — reference repos, research notes | `../my-reference-workspace` |

Variables can be absolute or relative to the repo root. The dev_init scripts
resolve relative paths automatically. The variable is commented out in
`.env.example`; uncomment the ones you have.

Future variables (e.g., `GDTF_LIBRARY`, `BLENDER_PATH`) follow the same
pattern: add to `.env.example` with a comment, developers copy to `.env.local`.

## Link Rules

### Internal repo links (files inside this repo)

Always use **relative markdown paths** from the current file:

```markdown
<!-- Good — works for every developer (from docs/dev/build/) -->
[change.rs](../../../crates/model/lumacore-state/src/change.rs)
[lum.md](../../user/syntax/lum.md)

<!-- Bad — an absolute path under one developer's home breaks for everyone else -->
```

### External repo links (companion workspaces)

Reference by variable name. The reader resolves from their `.env.local`:

```markdown
<!-- In documentation -->
See `$SKILL/notes/session-runtime.md` for session
runtime patterns.

<!-- The reader opens their local copy at whatever path SKILL points to -->
```

### Obsidian wikilinks

Obsidian resolves `[[filename]]` automatically within the vault, but GitHub and
other readers don't. In committed docs prefer relative markdown links; doc file
names are kebab-case (`fixtures-patch-output.md`).

## Dev Init Integration

The dev_init scripts:

1. Check for `.env.local` — if missing, print setup instructions
2. Parse each `NAME=path` line
3. Resolve relative paths against repo root
4. Validate that each path exists on disk
5. Warn for each missing path, finish every step, then exit 1 if any entry was
   missing or malformed

A fresh clone with no `.env.local` passes. The external companion repos enhance
the development workflow but are not required for compilation or testing.

## Lum Alignment

The build environment is developer-local. It is read only by the scripts, does
not travel with the show file, and is not in the object tree (there is no
`StationSettings.BuildEnvironment` node; an earlier draft proposed one).
