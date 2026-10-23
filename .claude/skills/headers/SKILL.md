---
name: headers
description: The file header, Part and Section banners every .rs file carries, and the script that renders them. Use when creating a Rust file, adding a Part, or when tidy's header rules fail.
---

# File headers and Parts

Every `.rs` file opens with its ANSI Shadow file header and a `// FILE:` tag, then a `//!` doc with `Owns:` / `Doesn't own:` / `Entry points:` / `Decisions:`, then Part headers in the one order:

TYPES · INIT · REGISTRY · COMMANDS · LOGIC · VIEWS · IO · HELPERS · TESTS (`// PART:` tags), star-box Sections inside a Part, `// ----- Step -----` lines in long functions.

**Banners come only from the script.** Never type one by hand; tidy re-renders them and fails on a drift:

```bash
./scripts/unix/headers <path/to/file.rs>     # prints the header and part banners for that file
./scripts/unix/headers --help                 # the options (section banners, FILE tags)
```

Paste the output, then fill the `//!` block. The `Decisions:` line names the records the file follows (`docs/decisions/<slug>`), not the archive.

Rules that go with it (tidy checks them, `docs/meta/tidy.md` lists them): one folder per object, one file per verb; `foo.rs` + `foo/`, never `mod.rs`; tests in `foo/tests.rs`; files under the soft limit, the hard limit fails unless `LONG_FILES` names the file with a reason. Old files follow when their area is touched; the ratchets (`HEADERLESS_AT_MOST` and the others) only go down.
