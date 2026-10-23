# lumacore-headers

The ANSI Shadow banners every `.rs` file is laid out with: one font table, one renderer, and the
command-line tool `./scripts/unix/headers` runs.

## Purpose

Every source file opens with its file header and carries its Part banners in one order
(decisions §34): TYPES · INIT · REGISTRY · COMMANDS · LOGIC · VIEWS · IO · HELPERS · TESTS.
Nobody draws a banner by hand — this crate renders them from the one font, and tidy re-renders
every tag line with it and fails when the art above differs. `lumacore-headers --file <title>`,
`--part <title>` and `--section <title> [purpose…]` print a file header, a Part banner or a
star-box Section; `./scripts/unix/headers` is the door developers use. It is a developer
tool: no product crate depends on it.

## Owns / Doesn't own

- **Owns:** the ANSI Shadow letters, `render`, `banner`, `section_box`, the Part vocabulary
  `PARTS`, `Kind`, `WIDTH`, and the binary's command line (`src/main.rs`).
- **Doesn't own:** where a header must stand and what a file's `//!` doc must say (the tidy
  rules in `crates/dev/lumacore-tidy`, listed in `docs/meta/tidy.md`), the file layout rules
  themselves (`docs/dev/architecture/code-structure.md`), the shell script.

## Public surface

- `render(text) -> Result<[String; 6], String>` — six rows of letters.
- `banner(kind, title) -> Result<Vec<String>, String>` — the rows plus the `// FILE:` or
  `// PART:` tag line.
- `section_box(title, purpose) -> Vec<String>` — the star-box Section.
- `PARTS`, `Kind`, `WIDTH` — the vocabulary and the line width.
- The binary `lumacore-headers` with `--file`, `--part`, `--section`.

## Rules

- Every glyph is six rows of even width and every Part name fits the width.
  [test every_glyph_is_six_even_rows + test every_part_fits_the_width]
- A banner ends with its tag line; a title too long for the width is refused.
  [test a_banner_ends_with_its_tag_and_long_titles_are_refused]
- Every tag line in the workspace stands under the banner this crate renders, and the Parts
  stand in the one order. [tidy banners_match_their_tag + tidy headers_stand_in_order]
- Every `.rs` file has its header. [tidy files_have_headers]
- No product crate depends on it. [tidy dev_tools_never_ship]
- It depends on nothing. [tidy crate_edges_point_down]

## Edges

- **Depends on:** nothing.
- **Used by:** lumacore-tidy (the banner check), `./scripts/unix/headers`.

## Decisions

- `docs/decisions/0221-file-headers-and-parts.md` — every file opens with a header and Parts in one
  order
- `docs/decisions/0233-tidy-test-only-crate-with-rulebook.md` — tidy checks it; every rule in `tidy.md`
- `docs/decisions/0235-dev-tools-never-ship.md` — why it sits in `crates/dev/`
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §34 (25.11)

## Testing

- `cargo test -p lumacore-headers` — the font, the banner and the width, in `src/lib.rs`.
- Against the whole workspace: `cargo test -p lumacore-tidy` re-renders every banner.
- By hand: `./scripts/unix/headers "Logic"` prints a Part banner to compare with.
