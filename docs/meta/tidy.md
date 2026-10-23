# Tidy — the source rules

Tidy is the crate [`lumacore-tidy`](../../crates/dev/lumacore-tidy/README.md): tests that read the
workspace's source files and fail when they break a rule the decisions set. This page is its
rulebook — one entry per rule. The page and the code are checked against each other
(`the_rulebook_matches_the_rules`, decisions §34 20):

- every `###` entry is a test of `lumacore-tidy`, and every test of it is an entry or a
  **Self-test** of one;
- every entry has its **Checks**, **Why** and **Escape** lines;
- every **Ratchet** and **Copies** number here is the one in the code, and every ratchet in the code
  is here.

```bash
cargo test -p lumacore-tidy                  # every rule
cargo test -p lumacore-tidy --features http  # with the HTTP engine's error catalog
cargo test -p lumacore-tidy structure::      # one area
```

**Adding a rule:** write its entry here first (what it checks, the § that decided it, its escape),
then the test in the area's file. **A ratchet** is a count of old code that breaks a rule: it may
only go down, and when it does the constant (and its number here) is lowered with it — the test
fails both when the count rises and when it falls without the constant following.

**Escapes** are a comment naming why, never a bare marker: `// tidy: <tag> (<reason>)` on the
offending line or the one above it. A rule without an escape says `none`.

**The station's names for files:** the rules that read the station see the state and objects
crates' files as `core/…` and a crate cut out of the station under its old folder name
(`terminal/…`; `files::CUT_OUT`). As the station empties (decisions §34 20) these become the crates'
own paths.

## Words

The registry is the one source of command and object words (`words.rs`).

### no_keyword_lists_outside_the_registry
- **Checks:** a `match` pattern (`"a" | "b"`) naming two or more registered words anywhere in the
  station, state or objects source outside the registry.
- **Why:** §15 — words are published through `CommandDef` / `ObjectDef`; a list elsewhere drifts.
- **Escape:** `// tidy: not-keywords (<reason>)` on the pattern's line or up to eight lines above
  (a grammar or value domain that shares names).
- **Self-test:** `scanner_finds_alternations`

### every_command_states_what_decides_it
- **Checks:** every `CommandDef` in the registry states `verb`, `doc`, `run`, `undo`, `scope`,
  `rights` and `reach` itself; only the rest may come from `..CommandDef::BASE`.
- **Why:** §31 — a default for what decides a command hides a choice nobody made.
- **Escape:** none
- **Self-test:** `stated_field_scanner_finds_a_missing_field`

### every_flag_in_core_is_declared
- **Checks:** every `/Flag` string literal in core names a declared flag (a word's `flags` or
  `flags::GLOBAL_FLAGS`); paths such as `/tmp` are skipped.
- **Why:** §31 — a flag no word declares is E-CMD-018 at run time.
- **Escape:** none

### every_flag_in_the_help_is_declared
- **Checks:** every `/Flag` in a command's or object's help syntax and examples is declared.
- **Why:** §31 — the help never shows a flag that is refused.
- **Escape:** none

### every_error_code_in_the_source_is_catalogued
- **Checks:** every `E-AREA-NNN` in a string literal of the station, state or objects source is in a
  catalog (`lumacore_registry::get().error`, the objects' rows and below; with the `http` feature the
  HTTP engine's too, else its codes are skipped).
- **Why:** §26, §31 — an uncatalogued code was caught only by a debug assertion when the line ran.
- **Escape:** none

### tui_hotkeys_run_registered_commands
- **Checks:** every line the TUI binds to a key (`execute_command(state, "…")`) starts with a
  registered word.
- **Why:** §15, §17 — a key runs a command line, and the line is the registry's.
- **Escape:** none

### core_ui_keys_run_registered_commands
- **Checks:** every built-in UI key's Press, DoublePress and Hold line starts with a registered word.
- **Why:** §17 — command lines live in UiKeys and are checked like any other.
- **Escape:** none

### keyboard_words_are_registry_words
- **Checks:** the built-in keyboard's word row (`KB_WORDS` in `frontend/index.html`) is registry
  words (`+`, `-`, `Thru` are the selection grammar).
- **Why:** §30 "12b" — the keyboard types the station's own words.
- **Escape:** none

## The station

The station only assembles (`station.rs`).

### the_station_only_assembles
- **Checks:** the station's sources are `src/catalog.rs` and `src/bin/<binary>.rs`, nothing else.
- **Why:** §34 24.1 — one assembler (the catalog) and a thin main per binary; code that does work
  lives in a crate, whose edges `crate_edges_point_down` checks (it replaced §23's station layers,
  `layers_only_point_down`, when the station's last module went, 34f11). The rules that read "the
  station's files" also read every crate cut out of it (`CUT_OUT`, under the folder name its files
  had).
- **Escape:** none

## Render

The per-frame path (`render.rs`).

### render_paths_never_lock_log_or_allocate
- **Checks:** inside `// tidy: render-begin … render-end` in the render tick, the engine's frame and
  the lines' send (`RENDER_FILES`), nothing locks, logs, formats, builds a string or a collection or
  allocates (`RENDER_FORBIDDEN`); each file keeps at least one region.
- **Why:** §24 "Step 9 hardening" — the tick is audio-style; the allocator check and the budget test
  hold it too.
- **Escape:** `// tidy: render-ok (<reason>)` on the line.
- **Self-test:** `render_rule_scanner_finds_violations`

## Doors

Shared things go through their one door (`doors.rs`).

### tree_paths_are_typed
- **Checks:** no `"Root.…"` string literal outside `core/tree_path.rs` and tests.
- **Why:** §31 — code names a `tree_path::Node`, never a path string.
- **Escape:** `// tidy: tree-path-ok (<reason>)` on the line or the one above.

### core_reaches_the_disk_only_through_drive
- **Checks:** no `std::fs` in core outside the drive door (`crates/formats/lumacore-luma/src/drive.rs`) and tests.
- **Why:** §25 "Step 10 hardening" — the command sweep closes that one door for Show lines.
- **Escape:** `// tidy: drive-ok (<reason>)` on the line or the one above (a non-command thread).

### error_replies_carry_a_code
- **Checks:** nothing outside `core/errors.rs` and `core/text.rs` calls `text::render_error`.
- **Why:** §29 "Codes everywhere" — every error line carries its code (`errors::render_code`,
  `render`, `reply`).
- **Escape:** none

### terminal_colours_only_in_the_look
- **Checks:** no `Color::` in the terminal crate outside `tui_look.rs` (the theme) and
  `command_line.rs` (the ANSI encoder); `nu_ansi_term` output keeps its slot names.
- **Why:** §30 "12b" — the TUI wears the user's theme; drawing code names colours by role.
- **Escape:** none

### old_resolvers_only_go_down
- **Checks:** the uses of the lookups that predate the one resolver (`OLD_RESOLVERS`) outside
  `core/resolve.rs` and tests.
- **Why:** §33 57 — every target becomes objects only through `core::resolve`; §33 part 5 takes it
  to 0.
- **Escape:** none
- **Ratchet:** `OLD_RESOLVER_USES_AT_MOST` = 91

### shared_doors_have_no_copies
- **Checks:** the copies of what one shared door does, by the pattern that gives them away, across
  the workspace (the object door, the trace crate and the rule's table excluded).
- **Why:** §34 12, 13 — one door per shared thing, named in code-structure.md.
- **Escape:** none
- **Copies:** the next free number (pools::next_free) = 6
- **Copies:** the next free number, after the last (pools::next_free) = 23
- **Copies:** the DataPool scope (AppState::in_datapool) = 6
- **Copies:** a not-found reply (errors::not_found) = 24
- **Copies:** an ObjectDef spelled out field by field (ObjectDef::new and its builders) = 42
- **Copies:** on/off words beyond the four (props::on_off) = 5
- **Copies:** a trace macro outside lumacore-trace (lumacore_trace::trace_zone!) = 0

## Frontend

The UI kit is the only way to build the page (`frontend.rs`; the WebUI's `web/index.html`,
`web/connect.html` and `web/kit/*` in `crates/ui/lumacore-webui`, vendored libraries excluded).

### frontend_colours_only_in_the_theme
- **Checks:** no raw colour (`#rgb…`, `rgb(`, `hsl(`) outside `kit/tokens.css` and `kit/theme.js`.
- **Why:** §30 — colours are theme tokens (`--lc-*`).
- **Escape:** none
- **Self-test:** `colour_scanner_tells_colours_from_names`

### frontend_icons_come_from_the_kit
- **Checks:** no emoji or symbol glyph used as an icon.
- **Why:** §30 — icons are the kit's `<luma-icon>` (Lucide).
- **Escape:** none

### frontend_font_sizes_are_tokens
- **Checks:** no raw px font size outside the token definitions, and no tile grid with a fixed
  column count or a raw px minimum.
- **Why:** §30 "12c2" — one type scale (`--lc-fs*`), tile grids that fit every box.
- **Escape:** `tidy: px-ok (<reason>)` / `tidy: grid-ok (<reason>)` on the line.
- **Self-test:** `font_and_grid_scanners`

### frontend_gestures_only_in_the_kit
- **Checks:** outside `kit/` no HTML5 drag, no window- or document-level `pointermove`, no hand-made
  hold ring.
- **Why:** §30 "12e hardening" — one hold and one drag, the kit's pointer core.
- **Escape:** `tidy: hold-ok (<reason>)` on the line or the one before.
- **Self-test:** `hand_gesture_scanner`

### frontend_asks_only_through_the_kit_dialog
- **Checks:** no `alert(` / `confirm(` / `prompt(` and no hand-built ask overlay.
- **Why:** §30 "12b" — every question is the station's one dialog.
- **Escape:** none

### text_selection_only_in_the_kit_rule
- **Checks:** `user-select` / `touch-callout` only in `kit.css`.
- **Why:** §30 "12b2" — text is selectable only where you read or type (`.lc-selectable`).
- **Escape:** none

### frontend_feeds_ride_one_socket
- **Checks:** a page opens only `/ws/feeds` and `/ws/shell`; only `kit/feeds.js` and the shell open
  a WebSocket.
- **Why:** §30 "12b2 hardening" — every live feed is a subscription on one socket (the router side
  is lumacore-http's `the_router_has_only_the_feeds_and_shell_sockets_and_no_sse`).
- **Escape:** none

### frontend_has_no_event_stream_or_fetch_poll
- **Checks:** no `EventSource` / event-stream URL, and no timer whose body fetches.
- **Why:** §30 "12d hardening" — everything live rides `/ws/feeds`; fetch is for one-shot loads.
- **Escape:** `// tidy: not-a-poll (<reason>)` on the timer's line or the one before.

### frontend_tables_only_in_the_grid
- **Checks:** no table markup (`<table`, `role="grid"`, `role="row"`) outside `kit/grid.js`.
- **Why:** §30 "12d" — every list is a `<luma-grid>`.
- **Escape:** `tidy: table-ok (<reason>)` on the line or the one before.

### frontend_actions_only_in_the_slab
- **Checks:** no hand-made New / Copy / Cut / Paste / Delete / Import / Export button or action-row
  item outside the slab, the setup menu, the one context menu (kit/context.js) and the Showrooms.
- **Why:** §30 "12d", "12e" — the slab is the kit's one action block.
- **Escape:** `tidy: slab-ok (<reason>)` on the line or the one before.
- **Self-test:** `grid_and_slab_scanners`

## Structure

The shape of every `.rs` file and crate (`structure.rs`; decisions §34,
[code-structure.md](../dev/architecture/code-structure.md)).

### banners_match_their_tag
- **Checks:** every `// FILE:` / `// PART:` tag line stands under the banner its title renders to.
- **Why:** §34 6 — banners come only from `./scripts/unix/headers`, never drawn by hand.
- **Escape:** none

### headers_stand_in_order
- **Checks:** a file with headers has its banner first, the module doc's `Owns:` / `Doesn't own:` /
  `Entry points:` / `Decisions:` labels (a crate root pointing at its README needs none), Parts from
  the vocabulary in order, only `use` / `mod` / attributes before the first Part, and Section rules
  exactly 100 columns.
- **Why:** §34 1–5, 18.
- **Escape:** none

### files_have_headers
- **Checks:** the files without a file banner.
- **Why:** §34 — old files get their headers when their area is touched; §34 20's last step takes it
  to 0.
- **Escape:** none
- **Ratchet:** `HEADERLESS_AT_MOST` = 266

### files_stay_short
- **Checks:** no file over 800 lines unless `LONG_FILES` names it with a reason, and no listed file
  that is short now (or gone).
- **Why:** §34 9.
- **Escape:** a `LONG_FILES` row with its reason.

### modules_are_foo_rs_not_mod_rs
- **Checks:** the `mod.rs` files left.
- **Why:** §34 10 — `foo.rs` + `foo/` is the one module style.
- **Escape:** none
- **Ratchet:** `MOD_RS_AT_MOST` = 9

### tests_live_in_foo_tests_rs
- **Checks:** the test files that aren't `foo/tests.rs` yet (`*_tests.rs`, `#[path]` modules).
- **Why:** §34 11.
- **Escape:** none
- **Ratchet:** `OLD_TEST_FILES_AT_MOST` = 43

### crate_edges_point_down
- **Checks:** every workspace crate's `[dependencies]` on workspace crates are in its `EDGES` row,
  and every crate has a row.
- **Why:** §34 7, §23 — one direction, the clients on the wire only.
- **Escape:** none

### crates_sit_in_their_group
- **Checks:** every crate under `crates/` is `crates/<group>[/<family>…]/lumacore-<name>`, its
  folder and package one name, its folder one of `GROUPS` (files.rs, with each folder's purpose),
  and every group in `GROUPS` exists; the station, the one assembler, is an app
  (`apps/lumacore-station`).
- **Why:** §34 17, 23, 24.
- **Escape:** none

### dev_tools_never_ship
- **Checks:** no crate outside `crates/dev/` depends on one in it (`[dependencies]`,
  `[build-dependencies]`, target dependencies) — except the station (the sweep's thin main),
  systemtest and the fuzz crate.
- **Why:** §34 25.8 — the developer tools (tidy, the header tool, the sweep) never ship; the rule,
  not the folder's name, keeps them out (Chromium's `DEPS` keeps `//tools` out of `//content`).
- **Escape:** none

### the_webui_is_only_files
- **Checks:** `crates/ui/lumacore-webui/Cargo.toml` has no entry in any dependencies table.
- **Why:** §36 1, 3 — the app and the HTTP engine take the WebUI's files, never code with them
  (clients compile only the wire and these files).
- **Escape:** none

### os_code_lives_in_platform
- **Checks:** no file of `apps/lumacore-app/src` outside `platform.rs` and `platform/` has
  `target_os`, `cfg(unix…)`, `cfg(windows…)` or their `cfg!` forms in its code.
- **Why:** §36 4 — one shell crate; what differs per OS sits in one place, so a core crate can be
  cut out the day one OS gets its own engine.
- **Escape:** none

### crates_have_readmes
- **Checks:** every workspace crate's README: `# <package>` first, the seven sections in order,
  every rule naming its check (and a named test or tidy rule exists), Edges equal to its Cargo
  dependencies; the crates without one only go down.
- **Why:** §34 18.
- **Escape:** `README_LEGACY` (a README older than the seven sections, until its crate's pass).
- **Ratchet:** `READMELESS_AT_MOST` = 4

### crates_on_the_registry_register_and_install
- **Checks:** every crate whose `[dependencies]` name `lumacore-registry` has `pub fn
  register(r: &mut lumacore_registry::Registry)` in its `lib.rs` (tidy itself aside) and the
  test-only `#[cfg(test)] #[ctor::ctor]` hook that installs its catalog; every crate on state or
  objects is on the registry.
- **Why:** §34 21, 21.8 — every crate adds its rows; a test binary has its catalog before its first
  test.
- **Escape:** none

### no_unsafe_code
- **Checks:** no `unsafe` keyword and no `unsafe_code` lint attribute in any `.rs` file's code.
- **Why:** §34 21.8 — the workspace lint is `deny`, not `forbid`, only so ctor's test hook expands;
  this keeps the source as free of `unsafe` as `forbid` did.
- **Escape:** none

## Docs

The docs are checked in the gate (`docs.rs`;
[0282](../decisions/0282-docs-checked-in-the-tidy-gate.md)).

### claude_md_stays_short
- **Checks:** `CLAUDE.md` has at most `CLAUDE_MD_LINES` lines (the constant in `docs.rs`;
  `scripts/unix/docs_check` holds the same cap as `CLAUDE_LINES`).
- **Why:** 0282 — the file loads into every session; a rule that outgrows it moves to
  `.claude/rules/<area>.md` or a record.
- **Escape:** none

### the_docs_pass_docs_check
- **Checks:** `scripts/unix/docs_check` exits clean from the workspace root: every record has its
  fields and sections, every path and local link resolves, no code-owned number stands in prose,
  the roadmap is within its caps. The script's output is the failure message.
- **Why:** 0282 — the docs rot silently otherwise; the gate is the one place every step passes.
- **Escape:** none (the script's own `EXEMPT` list names the history and generated files it skips).

## Release

The tree is public: what must never be in it is checked in the gate (`release.rs`).

### banned_words_are_absent
- **Checks:** no tracked text file (git's index; binary files skipped) matches a word of
  `LUMACORE_BANNED_WORDS` — comma-separated, case-insensitive, each a regex. Unset or empty, the
  rule prints that it was skipped and passes. The failure lists `file:line` for every hit.
- **Why:** the words live only in the environment, typed by whoever runs the gate, so the rule
  never carries what it guards against.
- **Escape:** none (rewrite the line).
- **Self-test:** `release_scanners_find_what_they_guard`

### no_private_details
- **Checks:** no tracked text file names a home folder (`/home/<name>/`, `/Users/<name>/`,
  `C:\Users\<name>\`) or a mail address other than a no-reply one or one at `example.com`,
  `.org` or `.net`; a fixture file name (`Generic@Dimmer.gdtf`) is no address, and third-party
  code (`vendor/`) keeps its authors' addresses.
- **Why:** a path is relative to the repo or `env!("LUMACORE_ROOT")`; nobody's machine or inbox
  belongs in a public tree.
- **Escape:** none.

## Rulebook

### the_rulebook_matches_the_rules
- **Checks:** this page against the crate, both ways (see the top of the page).
- **Why:** §34 20 — the rules are written down once, and the page can't drift from the code.
- **Escape:** none
- **Self-test:** `rulebook_reader_reads_entries_and_numbers`

## Guards elsewhere

Not tidy rules, but the same kind of check, kept next to the code they guard:

- `the_shim_count_only_goes_down` (`tests/catalog/src/rows/tests.rs`) — handlers still taking `&[&str]` (§33).
- `planned_words_reply_planned_and_change_nothing` (`tests/catalog/src/rows/tests.rs`) — planned words change nothing.
- `the_reference_is_current` (`tests/catalog/src/rows/tests.rs`) — the generated reference matches the registries.
- `the_demo_show_builds_loads_and_matches_its_golden` (`tests/catalog/src/kernel/tests.rs`) — the demo builds from its sources and matches its golden (decisions §40 15).
- `the_router_has_only_the_feeds_and_shell_sockets_and_no_sse` (lumacore-http) — the router's side of the feed
  rule.
