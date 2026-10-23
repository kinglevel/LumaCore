---
id: 0035
title: Apps are basic clients that open the station's page
area: station
status: built
date: 2026-10-12
built_in: 12a3, 12b2
supersedes: [archive §30 L4224–4226 "entry alive while mDNS lives"]
superseded_by: —
confirmed_by: [SESSION_PROTOCOL bump for leaving, e2e app-mode]
source: [archive §30 L3752–3761, archive §30 L3962–3972, archive §30 L4018–4022, archive §30 L4224–4229, archive §30 L4291–4296]
---
# Apps are basic clients that open the station's page

## Context
The desktop app had to be portable to iOS and Android with HTTP always available; the answer: apps are basic clients that discover, connect and open the HTTP page, a browser gets it directly. The servers list once kept a dead station as long as its mDNS record lived.

## Decision
The station serves the whole UI — lock / login surface included — over HTTP; desktop and mobile apps only **discover, connect and open that page**. Before a connection the app shows a small connect page built from the same kit (clock, the server list, start local server, Clean start, the version prompts), bundled from the WebUI (`web/connect.html`, see wire-crate-clients) at `lumacore://localhost/`. Picking a server opens the station page without a token; the page logs in and calls `LumaHost.loggedIn`. Platform work goes through one host bridge, `window.LumaHost`; a browser lacks it. The shell is wry / tao; phones run no local server. **One OS window per screen**, any size, any display; each screen remembers its display, size and place per server and user (`client.json`); a missing display opens the window on the main display scaled to fit; Window › Screens Overview shows every screen live. Servers list: a clean shutdown withdraws the mDNS service and sends a `leaving` beacon; **the beacon decides** — a station is listed only while its beacon is fresh, late shows amber "not answering", gone after the beacon timeout even if mDNS still lists it; a wrong version stays grey. The app spawns `lumacore-server` next to it (see shutdown-signals-local-server).

## Consequences
No app-only UI or state beyond connect, windows and the bridge; everything else is the station's page. The tiled mode (several screens in one window or tab) is decided, not confirmed built. Navigation and IPC: see clients-app-navigation-cli-disconnect.
