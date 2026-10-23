---
id: 0185
title: Everything live rides one feeds socket
area: feeds
status: built
date: 2026-10-13
built_in: 12d part 1
supersedes: [archive §30 L3901–3904 "open event stream", archive §30 L3591–3594 "existing SSE WS"]
superseded_by: —
confirmed_by: [tidy frontend_feeds_ride_one_socket, tidy frontend_has_no_event_stream_or_fetch_poll, E-FEED-001…007, the router test (only /ws/feeds and /ws/shell), e2e feeds.spec.ts]
source: [archive §30 L3901–3904, archive §30 L4340–4353, archive §30 L4409–4448, archive §30 L4738–4752, archive §30 L4918–4955]
---
# Everything live rides one feeds socket

## Context
The System Monitor streamed its history one line per message, the Realtime window ticked slowly, and each feed had its own `/ws/*` route beside SSE streams and polls. 12b2 built the socket, 12d part 1 moved everything live onto it.

## Decision
One `/ws/feeds` per page. It opens without a login: `hello` carries `CLIENT_PROTOCOL`, the catalog's feed rows (`lumacore_http`'s `FEEDS`) and the limits; feeds marked `open` work before `auth`, and in-band `auth` keeps every subscription. `sub` / `unsub` are answered `subbed` / `nosub` with an `E-FEED-…` code. A feed is State (latest wins; a slow client gets the next newest) or Samples (each item has a `seq`; the station keeps a ring; a batch carries everything since the client's `seq`; what the ring lost arrives as an explicit `gap`; a restart changes the `epoch`). Every feed sends one batch when a window opens, then changes in batches per tick — never one item per message. State feeds are shared builds (`websocket/hub.rs`): one producer per (feed, user, params), rebuilt when `lumacore_http::Revisions` move, never by a timer; `rev` + `unchanged`, keyed diffs for big lists. JSON is columnar; DMX universes are binary frames while shown. Subscription, queue and stall limits are what the feeds hub declares. The kit's `feeds.js` FeedClient reconnects with `since` and draws in one rAF loop per window. `/ws/shell` is the page's shell; no other `/ws/` route.

## Consequences
No SSE, no fetch on a timer, no after-command fetch — tidy fails; fetch stays for one-shot loads. A new live view adds a feed row and a producer, never an endpoint. `rev` is a fingerprint of the build, not a counter (so `unchanged` works across a restart). Open limits are the roadmap's "Feeds follow-ups".
