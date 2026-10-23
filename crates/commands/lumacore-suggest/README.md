# lumacore-suggest

The suggestion engine: what the command line can take at the cursor, as typed items, ranked on
the station.

## Purpose

One engine for every place that suggests (decisions §30 "12b2"): the web's `<luma-suggest>` above
the command bar, the full-screen palette, the TUI's overlay, the sweep and the fuzz targets. It
ranks in a fixed order — **grammar** first (only what is valid at the cursor, read from the
catalog and the show: verbs, objects, flags, properties, names, numbers; never a hand-written word
list), then the fuzzy score of the typed fragment (nucleo), then the acting user's **Usage**
(saved on `ShowData.Users.{n}`, replicated, never undone), then a fixed tie-break. Picking an item
never runs code in a client: it writes the command line, or its actions are command lines. It
has no commands of its own, so it isn't a `cmd-` crate; it sits on the kernel beside the command
crates (decisions §34 25.9), and its one `Hook::Used` records Usage after a user's show line.

## Owns / Doesn't own

- **Owns:** `suggest`, `ghost`, `mode`, the item ranking and `record_usage` (`lib`); what may
  follow at the cursor — `complete_input`, `hint_input`, `collect_attribute_names` (`grammar`);
  its `Hook::Used`; the Tab completion golden record (`testdata/tab_completion.golden.txt`).
- **Doesn't own:** the words and their shapes (the catalog, `lumacore-objects`), drawing the
  items (the kit, the TUI), the `Suggest` / `LineEdit` packets (`lumacore-wire`).

## Public surface

- `suggest(state, &Query)` — the ranked items at the cursor.
- `complete_input`, `hint_input` — Tab completion and the inline hint.
- `ghost`, `mode` — the TUI's ghost text and the profile's `Suggestions` mode.
- `register` — its hook, after the kernel's rows.

## Rules

- Grammar first: only what the catalog and the show allow; never a hand-written list. [unchecked]
- Picking writes the line; nothing runs in a client. [unchecked]
- A Tab completion change shows in the golden record. [test tab_completion_is_unchanged]

## Edges

- **Depends on:** lumacore-registry, lumacore-base, lumacore-wire, lumacore-tree, lumacore-objects.
- **Used by:** lumacore-terminal, lumacore-sweep.

## Decisions

- [§30 "12b2"](../../../docs/archive/decisions-2026-10.md) — the suggestion engine, Usage
- [§34 25.9](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate

## Testing

- `cargo test -p lumacore-suggest` — ranking, Usage, completion, the golden record
  (`LUMACORE_BLESS=1` rewrites it; read the diff).
