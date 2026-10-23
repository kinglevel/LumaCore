# lumacore-views

What every UI shows: the menus and their rows, the pool windows, the attribute board, the encoder
bar, a profile's or user's detail, the sequence sheet, the Tags pool window, the Patch dialog's rows
and the windows' live state.

## Purpose

The one source of what a page, the desktop app or the TUI draws (decisions §34 20.7). Each view
reads a published show (or, for the menu rows, the station's state on the state thread) and
returns JSON the kit draws; every gesture on it is a command line. The HTTP feeds and the TUI read
these functions; no client builds its own rows, and nothing here changes the show.

## Owns / Doesn't own

- **Owns:** the menus as the kit reads them (`menus`: `menu_types_json`, `menu_json`,
  `column_json`, `slab_json`), the menus' rows (`menu_rows`: `network`, `lumasessions`), the pool
  windows (`pools`: `view`, `detail`), the attribute board (`attribute_board::view`), the encoder
  bar (`encoder::view`), a profile's Content or a user's Work (`detail::view`), the sequence sheet
  (`sequence::{resolve, view}`), the Tags pool window (`tags::view`), the Patch dialog's rows
  (`patch::view`: the Devices draft for its holder, the live devices for everyone else; the one
  tree order, `patch::tree_order`), the Device Sheet (`devices::view`: every device in scope with
  its folders, the attributes in scope by feature group, each shown value's readout, state and
  layers), the windows' live state (`window_state`: `StationSelf`, `screen_view`), the NAV
  inspector (`inspector::view`: a tree path's identity, facts, declared properties with their
  `Set` / `Reset` lines and links).
- **Doesn't own:** the menu table (`lumacore_objects::menu_defs`, rows in the catalog), the objects
  and their properties (`lumacore-objects`), the window types and their rules
  (`lumacore_tree::window_types`), the feeds and their transport (`lumacore-http`, the server's
  console bridge), when a view is rebuilt (the state thread's counters).

## Public surface

- `menus::menu_types_json` — `GET /api/menu-types`: every menu, tab, column, slab cell.
- `menu_rows::{network, lumasessions}` — the rows of the `network` and `lumasessions` feeds, built
  on the state thread.
- `pools::{pools, view, detail}` — the `pools` and `pool_editor` feeds.
- `attribute_board::view`, `encoder::view`, `detail::view`, `sequence::view`, `tags::view`,
  `patch::view`, `window_state::screen_view` — the `attributes`, `encoder`, `detail`, `sequence`,
  `tags`, `patch` and
  `windows` feeds.
- `devices::view` (with `devices::{Params, Layer}`) — the `devices` feed, sampled per window for the
  rows it shows (decisions device-sheet-window).
- `inspector::view` — the TUI's NAV inspector for one tree path (decisions 0291); it reads the
  settings `EditSetting` reads, with the line's datapool in place only while it reads.

## Rules

- A view only reads: it never changes the show or the station. [unchecked]
- Every gesture a view offers is a command line, never a client-only action. [unchecked]
- A grid column reads its kind, range and level from the object's declared property; a column
  naming a property its object doesn't declare fails the test. [test every_column_prop_resolves_on_its_object]
- It depends on the objects crate and below only — never on the server, the network or the
  terminals. [compiler + tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-objects, lumacore-tree, lumacore-base, lumacore-net, lumacore-render, lumacore-registry, lumacore-sysmon, lumacore-luma, lumacore-protocols.
- **Used by:** lumacore-station (the console bridge, the state thread), lumacore-terminal (NAV's
  inspector).

## Decisions

- [§22](../../../docs/archive/decisions-2026-10.md) — the unified pool window
- [§30 "12c", "12d", "12f", "12f-a"](../../../docs/archive/decisions-2026-10.md) — window state, setup menus,
  encoder bar and sequence sheet, the attribute board
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (20.7), its README (18), the catalog (21)
- [0291](../../../docs/decisions/0291-nav-mode-ls-rows-and-an-inspector.md) — the NAV inspector

## Testing

- `cargo test -p lumacore-views` — each view on the demo show or a small station.
- Its tidy rules: `cargo test -p lumacore-tidy` (docs/meta/tidy.md).
