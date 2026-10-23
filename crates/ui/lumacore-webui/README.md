# lumacore-webui

The WebUI: every file the station's page and the app's connect page are made of, embedded.

## Purpose

The station serves one UI to every client — the desktop app's windows, a browser, a phone (decisions §30 "App architecture"). Its files live here (decisions §36 1): `web/index.html` (the station's page), `web/connect.html` (the app's connect page, shown before any station is found), `web/kit/` (the `luma-*` elements, tokens, layouts) and `web/vendor/` (fonts, Lucide, xterm, modern-screenshot; see its README). `build.rs` embeds every file, so a server binary carries its UI and no web root has to be found; `lumacore-http` serves the embedded files unless `--web-root` points at a folder (the scripts pass `web/`, so a JS / CSS edit is live without a rebuild).

## Owns / Doesn't own

- **Owns:** the `web/` files, `FILES`, `get`, `INDEX`, `CONNECT`.
- **Doesn't own:** serving them (`lumacore-http`; the app's `lumacore://` handler), the feeds and endpoints the pages talk to.

## Public surface

- `FILES` — every file: (URL path, bytes, MIME type).
- `get(path)` — one file (`/` = the station's page; a query is ignored).
- `INDEX`, `CONNECT` — the two pages.

## Rules

- Both pages and every file they load are embedded. [test both_pages_and_everything_they_load_are_embedded]
- Every module a kit file imports is embedded. [test every_module_a_kit_file_imports_is_embedded]
- Every file has a known type; dot files stay out. [test types_are_known_and_dot_files_stay_out]
- Files only: no dependencies, no logic beyond handing out the files. [tidy the_webui_is_only_files]

## Edges

- **Depends on:** nothing.
- **Used by:** lumacore-http, lumacore-app.

## Decisions

- [§36 1–3](../../../docs/archive/decisions-2026-10.md#36-webui-the-thin-client-and-3d-rendering-2026-10-18) — the crate, embedded with the `--web-root` override, the connect page here too
- [§30](../../../docs/archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening) — the kit (every window built from it), "12a3" (the connect page)

## Testing

- `cargo test -p lumacore-webui` — the embedded files.
- `cargo test -p lumacore-http static_file` — serving them, embedded and from disk.
- `cargo test -p lumacore-tidy frontend::` — the kit rules over `web/`.
