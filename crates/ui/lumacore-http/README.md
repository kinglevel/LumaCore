# lumacore-http

The HTTP engine: the REST endpoints, the two WebSockets — `/ws/feeds` for everything live in a
page and `/ws/shell` for its terminal — and the embedded WebUI's files.

## Purpose

What a browser and the desktop app talk to (decisions §30 "12d", §36). It never sees
`AppState`: every read and command line crosses the `ConsoleBridge` trait the server implements
on the state thread's machine lane; what it serves is built by `lumacore-views` behind it. `/ws/feeds` opens without a login, says `hello` with
`CLIENT_PROTOCOL` and the catalog's feed rows, answers every `sub` with `subbed` or `nosub`,
tells State feeds from Samples with `seq` and `gap`, and shares one build of a State feed
between subscribers, rebuilt only when `Revisions` move — never on a timer. A line the busy
station didn't run is a 503 (E-CMD-017). Static files come from `lumacore-webui`, or from
`--web-root` while developing.

## Owns / Doesn't own

- **Owns:** `ConsoleBridge`, `HttpEngine::start`, `HttpHandle`, `HttpState`, `Revisions`,
  `epoch`; the router and the endpoints (`endpoints/*`: station, login, shows, menus, pools,
  fixture types, media, screenshots, sysmon, health, trace); `auth` (the token); `websocket::feeds`
  with `FEEDS`, `hub` (shared builds), `shell` (the bridge to the shell port); `layout`;
  `DmxUniverse`, `DmxPatchSpan`; the `E-FEED` catalog.
- **Doesn't own:** the rows and views it sends (`lumacore-views`, through the bridge), the
  token's signing (`lumacore_state::passwords`), the page and the kit (`lumacore-webui`), the
  shell itself (`lumacore-shell`), the port numbers (`lumacore-wire`), the rights gate.

## Public surface

- `ConsoleBridge` — `execute`, `login` / `verify_token` / `logout`, `station`, `pool`,
  `dmx_universe`, `health`, `revisions`, `feed_demand`, `save_screenshot`, …
- `HttpEngine::start(addr, bridge, …)` → `HttpHandle` (`stop`, `take_thread`).
- `Revisions` — the counters that rebuild shared feeds; `epoch()` — this run's id.
- `FEEDS` — the feed rows added to the catalog; `register(&mut Registry)`.
- `DmxUniverse`, `DmxPatchSpan`, `ScreenLayout` — the feeds' shapes.

## Rules

- Only `/ws/feeds` and `/ws/shell`; no SSE and no poll anywhere in the frontend.
  [test the_router_has_only_the_feeds_and_shell_sockets_and_no_sse + tidy frontend_has_no_event_stream_or_fetch_poll]
- Every feed message is one envelope; a keyed diff sets, deletes and orders rows; the same JSON
  has the same `rev`. [test the_envelope_is_one_shape + test a_keyed_diff_sets_deletes_and_orders_rows + test the_same_json_has_the_same_rev]
- A DMX frame is a header then fixed universe blocks. [test a_dmx_frame_is_a_header_then_fixed_universe_blocks]
- Diagnostics reads are lock-free. [test arcswap_lock_free_read + test arcswap_update_visible_to_readers]
- Keyboard hold and release on the shell bridge become key gestures.
  [test keyboard_hold_and_release_become_key_gestures]
- Its feed codes are in the one catalog. [test catalog_is_valid + tidy crates_on_the_registry_register_and_install]
- It never names `AppState`; everything crosses `ConsoleBridge`. [tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-registry, lumacore-tree, lumacore-trace, lumacore-wire, lumacore-webui.
- **Used by:** lumacore-engines (`HttpUiEngine`), lumacore-server, lumacore-tidy.

## Decisions

- `docs/decisions/0185-client-feeds-one-socket.md` — everything live rides `/ws/feeds`
- `docs/decisions/0037-wire-crate-clients.md` — the files, `--web-root`
- `docs/decisions/0026-one-state-thread.md`, `docs/decisions/0026-one-state-thread.md` — the machine lane, 503
- `docs/decisions/0161-rights-one-gate.md` — tokens, logout revokes
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §23, §26, §30, §34, §36; the API in [docs/dev/api/http-api.md](../../../docs/dev/api/http-api.md)

## Testing

- `cargo test -p lumacore-http` — `src/tests/` (endpoints, engine, live screens, state, static
  files, websockets), plus `websocket/hub.rs` and `websocket/shell.rs`.
- In a browser: `cd tests/e2e && npx playwright test` (`tests/e2e/README.md`).
