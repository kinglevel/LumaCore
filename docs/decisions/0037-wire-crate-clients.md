---
id: 0037
title: Clients compile only the wire and the WebUI
area: structure
status: built
date: 2026-10-18
built_in: step 8, roadmap 3b
supersedes: [archive §23 L1973 "lumacore-proto", archive §23 L2003–2006 "lumacore-app --server", archive §23 L1998–2001 "clients only wire" (amended), archive §36 L11113 "web/dist default"]
superseded_by: —
confirmed_by: [E-WIRE-001, the_webui_is_only_files, os_code_lives_in_platform, default-members]
source: [archive §23 L1971–2006, archive §23 L2052–2070, archive §36 L11105–11129, archive §36 L11161–11185]
---
# Clients compile only the wire and the WebUI

## Context
Step 8 cut the clients loose from the station with one contract crate replacing the planned `lumacore-proto`. §36 moved the page files out of the app into their own crate so a server binary carries its UI and the app's connect page comes from the same place.

## Decision
`lumacore-wire` is the client ↔ station contract, compiled by both sides, depending only on serde: the ports and multicast groups as constants, the mDNS service name, the discovery beacon (+ `StationKind`: `Server`, `Node` reserved), `RemoteRequest` / `RemoteReply`, shell packets and key codes, `UiEvent`, a typed `Status`, both protocol numbers and the E-WIRE codes. Station ↔ station frames stay out of it. Two numbers: `SESSION_PROTOCOL` must match to join, `CLIENT_PROTOCOL` to attach (E-WIRE-001; a hello without the field is refused); both ride the beacon, Health and the hellos; **bump on any wire change** (a typed `Status` field counts), forward only, no shims. Clients live under `apps/` and compile only `lumacore-wire` and `lumacore-webui` (+ their own UI crates); nothing under `apps/` depends on `lumacore`; the app spawns `lumacore-server` next to it. `crates/ui/lumacore-webui` holds every page file (`web/index.html`, `web/connect.html`, `web/kit/`, `web/vendor/`), embedded by its `build.rs`, no dependencies, no logic beyond handing files out. `lumacore-http` serves the embedded files; `--web-root <dir>` serves from disk instead (the scripts pass `crates/ui/lumacore-webui/web`; no default). The app stays one crate; OS code lives only in `src/platform/{macos,windows,linux}.rs`. The app is out of the default build (`default-members`); `cargo test --workspace` includes it.

## Consequences
A new client type or status field changes the wire crate and bumps its number. Tidy `the_webui_is_only_files` and `os_code_lives_in_platform` catch logic in the WebUI crate and a `cfg(target_os)` elsewhere in the app.
