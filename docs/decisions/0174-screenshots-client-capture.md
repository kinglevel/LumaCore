---
id: 0174
title: Screenshots are captured in the client, asked by the server
area: ui
status: built
date: 2026-10-12
built_in: 11d
supersedes: [archive §29 L3407–3412 "/api/events viewer stream", archive §29 L3442–3446 "vendor under apps/lumacore-app/frontend", archive §29 L3452–3454 "F11 cd output"]
superseded_by: —
confirmed_by: [e2e `screenshot.spec.ts`, e2e `screenshots-all.spec.ts`, E-CMD-010]
source: [archive §29 L3164–3179, archive §29 L3407–3479]
---
# Screenshots are captured in the client, asked by the server

## Context
The user wanted screenshots before the UI work, for development and tests, on the one path that later serves view thumbnails and `Store Image … /Screen`. A reference console captures every screen on Print Screen / F11 with a green flash. The first build's SSE viewer stream and vendor folder moved with the feeds and the WebUI crate.

## Decision
The server never renders a page: `Screenshot [Screen <n>] [User <x>]` (bare = every open screen of the issuing user) asks the clients viewing those live screens to capture; a browser views a screen while its `screen` feed subscription on `/ws/feeds` stands (`ScreenStore` capture board). No open screen or no viewing client is E-CMD-010; a client silent past the board's deadline is a warning. The page renders itself as seen with the vendored modern-screenshot, flashes green and uploads to `POST /api/screenshots?id=&screen=`; the station checks on the state thread that it asked and that it is the uploader's own screen, writes the file on the HTTP worker through the drive door (`screenshot::write`) and posts "Screenshot saved: <path>" to sysmon and the asker. Files are station-local, never in the show: `library/screenshots/<station>/YYYYMMDD_hhmmss_<user>_screen<n>.png`. F11 and Print Screen are core keys for it (see keys-uikey-gestures). The frontend loads nothing from the internet: libraries live in `crates/ui/lumacore-webui/web/vendor/` (its README lists them). `./scripts/unix/screenshots` captures every window type, menu and overlay taken from the registries through this real path into `target/screenshots/`; it is how UI work is looked at.

## Consequences
A new window type or menu is picked up by the dev script without a list. An unasked or foreign upload is refused. Open (roadmap): the Image pool from the same capture, view thumbnails, visual regression against reference PNGs.
