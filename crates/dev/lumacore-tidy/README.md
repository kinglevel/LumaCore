# lumacore-tidy

The source rules of the whole workspace: tests that read the code and fail when it breaks a rule
the decisions set.

## Purpose

Every rule that a compiler can't hold — one source of words, layers that point down, the render
path that never allocates, one door per shared thing, the UI kit as the only way to build the
page, the shape of every file and crate — is a test here. They read the files from the workspace
root (`crates/`, `apps/`, `tests/systemtest/`) and the registries; the one thing they run is
`scripts/unix/docs_check`, so the docs are gated with the code. The rules
are written down once, in [docs/meta/tidy.md](../../../docs/meta/tidy.md), one entry per rule, and
that page is checked against the code.

## Owns / Doesn't own

- **Owns:** every tidy rule, one file per area — `words` (registry words, flags, error codes,
  bound lines), `layers` (the station's modules), `render` (the per-frame path), `doors` (tree
  paths, the disk, error lines, the TUI's colours, the old resolvers, COPIES), `frontend` (the UI
  kit), `structure` (headers, size, modules, crate edges and names, READMEs), `docs` (the
  `CLAUDE.md` cap, `docs_check` in the gate), `rulebook` (the page against the code); the shared
  file scanners (`files`); the ratchet constants.
- **Doesn't own:** what the rules protect (each area's own crate), the header font
  (lumacore-headers), the guards that live next to their code (the shim count, the reference, the
  demo show — listed at the end of tidy.md).

## Public surface

- None: the crate is its tests. `cargo test -p lumacore-tidy` runs them.

## Rules

- Every rule has its entry in docs/meta/tidy.md, every entry is a rule, and every ratchet's number
  there is the code's. [tidy the_rulebook_matches_the_rules]
- A ratchet only goes down, and is lowered when its count falls. [tidy files_have_headers]
- The rules read files from the workspace root, never relative to the crate that runs them.
  [unchecked]
- An escape is a `// tidy: <tag> (<reason>)` comment naming why, never a bare marker. [unchecked]

## Edges

- **Depends on:** lumacore-cmd-treeshell, lumacore-cmd-keys, lumacore-cmd-tags, lumacore-cmd-showfile, lumacore-cmd-station, lumacore-cmd-devices, lumacore-cmd-protocols, lumacore-cmd-lumasession, lumacore-objects, lumacore-tree, lumacore-headers, lumacore-registry,
  lumacore-http.
- **Used by:** nothing (a test crate).

## Decisions

- [§15](../../../docs/archive/decisions-2026-10.md) — one namespace, words only from the registry
- [§23 "Step 8"](../../../docs/archive/decisions-2026-10.md) — the layers
- [§24 "Step 9 hardening"](../../../docs/archive/decisions-2026-10.md) — the render rule
- [§30](../../../docs/archive/decisions-2026-10.md) — the UI kit rules
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — the structure rules (1–18), this crate and its rulebook (20)
- [0282](../../../docs/decisions/0282-docs-checked-in-the-tidy-gate.md) — the docs checked in the gate

## Testing

- `cargo test -p lumacore-tidy` — every rule; `--features http` adds the HTTP engine's rows to
  the catalog the rules read (its error codes).
- `cargo test -p lumacore-tidy structure::` — one area.
