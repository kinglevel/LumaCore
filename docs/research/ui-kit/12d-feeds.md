# 12d research: one live socket (`/ws/feeds`) for the whole web UI

Date: 2026-10-13. This is research only. No repo files were changed.
Question: how do large open-source server/thin-client projects run one subscription socket, and what should LumaCore's `/ws/feeds` add before it takes over `/api/events`, `/api/station/events` and the 1 s polls?

---

## 0. Where LumaCore is today (read from the code)

- **`/ws/feeds`** (`crates/ui/lumacore-http/src/websocket/feeds.rs`, `kit/feeds.js`):
  - The client sends `{"op":"sub","id","feed",…params,"since"?,"epoch"?,"period_ms"?}` and `{"op":"unsub","id"}`. The client picks the id, and a `sub` that reuses an id replaces that subscription.
  - The envelope is `{feed, kind, id, epoch, from_seq, to_seq, gap, recovered?}`, plus `data` (State) or `cols`/`items` (Samples). DMX goes as binary frames that carry the sub id in their first u32.
  - Errors come back as `{"op":"error","id","message"}`. There is no success ack, and no error code (this conflicts with the project's `E-AREA-NNN` rule).
  - **Auth comes before the socket**: `AuthUser` runs on the upgrade, so nothing can be read before login.
  - **Each connection builds each subscription itself on a timer.** Every period, every sub calls `state.console.health()` / `pool(...)` / `windows_data(...)` and compares the result as `serde_json::Value` with the last batch. The cost is per connection × per sub × per period, even when nothing changed. Nothing is shared between two pages that watch the same pool.
  - A State feed's `from_seq/to_seq` is a per-subscription batch counter (`sent`). It is not a revision of the data, so a resubscribe always gets a full snapshot.
  - Backpressure: one send that stalls for more than 2 s closes the socket. A Samples broadcast that lags is refilled from the history, or a `gap` is reported.
- **`/api/events` (SSE)**:
  - It is a `broadcast` of layout events: `views_changed`, `screen_changed`, `reload_ui`, `capture`.
  - **A lagged receiver drops events silently** (`Err(_) => None` in `endpoints/events.rs`), and there is no id or resume.
  - It also stands for *presence*: the station counts a browser as a viewer of a live screen while its `?screen=n` stream is open (the Screenshot capture board).
- **`/api/station/events` (SSE)**:
  - It is open (no login). Each connection rebuilds `console.station()` every 500 ms and compares the strings.
  - The lock/login surface needs it before login.
- **Polling**: the pool editor (`editorRefresh` 1 s) and the network / DMX-protocol / LumaSession menus (`npRefresh` 1 s).

---

## 1. Projects studied (8)

### 1.1 Home Assistant `websocket_api`

The closest match: one Python process owns the state, and thin frontends subscribe to it.

- **(1) Sockets**: one socket per frontend. Commands, subscriptions and events share it.
- **(2) Auth**:
  - The socket opens unauthenticated. The server sends `auth_required` with `ha_version`. The client sends `auth` with a token, and the server answers `auth_ok` or `auth_invalid`, then disconnects.
  - "Did not receive auth message within 10 seconds" (`AUTH_MESSAGE_TIMEOUT`).
  - Nothing else can be read before auth.
  - Sources: https://developers.home-assistant.io/docs/api/websocket/ and `http.py` (below).
- **(3) Subscription shape**:
  - Every client message has an integer `id`, "an integer that the caller can use to correlate messages to responses".
  - `subscribe_events` / `subscribe_entities` / `subscribe_trigger` get a `result` (`success`, or `error {code, message}`). After that, `{"type":"event","id":<sub id>,"event":…}`.
  - `unsubscribe_events {subscription: <id>}`.
  - A reused id is an error: `ERR_ID_REUSE = "id_reuse"`. Other codes: `not_found`, `not_allowed`, `unauthorized`, `invalid_format`, `unknown_command`, `timeout` (https://raw.githubusercontent.com/home-assistant/core/dev/homeassistant/components/websocket_api/const.py).
- **(4) Snapshot + deltas**:
  - `subscribe_entities` first sends `{"a": {entity_id: compressed_state}}`, then diffs. `"a"` is added, `"c": {id: {"+": changed fields, "-": {attributes: [...]}}}` is changed, `"r": [ids]` is removed.
  - Keys are short (`s`, `a`, `lc`, `lu`), and `last_updated` is sent only when `last_changed` did not move (`messages.py`, https://raw.githubusercontent.com/home-assistant/core/dev/homeassistant/components/websocket_api/messages.py).
  - There are no revision numbers. Ordering on the socket is the guarantee.
- **(5) Reconnect**:
  - There is no resume. The JS client re-subscribes everything after reconnect: "re-established a subscription after a reconnect. Defaults to true" (home-assistant-js-websocket `connection.ts`).
  - The id counter resets to 1 on close.
  - Each resubscribe brings a fresh full snapshot.
- **(6) Backpressure** (`http.py`):
  - An outgoing deque per connection.
  - `MAX_PENDING_MSG = 4096` closes at once: "Client unable to keep up with pending messages. Reached %s pending messages."
  - `PENDING_MSG_PEAK = 1024` held for `PENDING_MSG_PEAK_TIME = 10` s also closes.
  - **Coalescing**: the writer waits while the queue is still growing, up to `PENDING_MSG_MAX_FORCE_READY = 256`, then sends the queued messages as **one JSON array frame**. A client opts in with `supported_features {coalesce_messages: 1}` as message id 1.
- **(7) Server cost**:
  - `_partial_cached_state_diff_message` is `@lru_cache(maxsize=128)` keyed by the event, so the JSON is serialized **once per event for all subscribers**, and each connection only appends `,"id":N}`.
  - The docstring: "the slice happens once per event instead of once per subscriber".
- **(8) RPC**: on the same socket (`call_service`, `get_states`, …).
- **(9) Versioning**: `ha_version` in `auth_required`, plus opt-in features (`supported_features`). There is no protocol number.
- **(10) Pain points**:
  - "Client unable to keep up" disconnects on slow tablets and dashboards.
  - `subscribe_entities` with compressed diffs was added because `subscribe_events state_changed` sent full old and new states for every change, which was too heavy for mobile.
  - The iOS app later also filtered by domain to cut traffic.

### 1.2 Grafana Live (Centrifuge library) and Centrifugo

- **(1) Sockets**: "Each logged-in user opens a WebSocket connection – one per browser tab", and "All subscriptions on a page are multiplexed inside a single WebSocket connection" (https://grafana.com/docs/grafana/latest/setup-grafana/set-up-grafana-live/).
- **(2) Auth**:
  - The connect command carries the token, and channel permissions are checked at subscribe.
  - The Origin is checked against `root_url` and `allowed_origins`.
- **(3) Channels**:
  - Named `scope/namespace/path` (for example `grafana/dashboard/xyz`), with ASCII only and at most 160 characters.
  - Parameters live **in the channel name**, so two clients with the same params share one channel.
- **(4)(5) History and recovery** (https://centrifugal.dev/docs/server/history_and_recovery):
  - Every publication has an `offset`, and every stream has an `epoch`. On resubscribe, the client sends its last `{offset, epoch}`.
  - "Centrifugo never returns a partial recovery". It answers `recovered: true` or `recovered: false`, and on false the app reloads the full state.
  - The **cache recovery mode** (`force_recovery_mode: "cache"`, usually `history_size: 1`) sends only the newest publication, which suits state snapshots.
- **(6) Backpressure and batching**:
  - Per-connection write queues with a size cap; a slow client is disconnected.
  - `client_write_delay` (PRO) "is a time Centrifugo will try to collect messages inside each connection message write loop before sending" (https://centrifugal.dev/docs/pro/client_message_batching).
- **(7) Server cost**:
  - A channel is a hub entry, so a publish is fanned out once per channel.
  - **Delta compression** (fossil) sends each publication as a diff against the previous one in the channel. It needs positioning or recovery, the client opts in with `delta: "fossil"`, and clients without delta support share the channel and get full payloads (https://centrifugal.dev/docs/server/delta_compression). The test gave about 10× less traffic, though this "heavily depends on the nature of data you publish".
- **(8) RPC**: Centrifuge has an `rpc` command on the same socket. Grafana uses it little, and its commands stay REST.
- **(9) Versioning**: a protocol version on connect, and a protobuf or JSON transport chosen per connection.
- **(10) Pain points** (Grafana docs):
  - About 50 KB of memory per connection.
  - The default `max_connections = 100` per instance.
  - In-memory state limits broadcasts to one server process, so HA needs Redis.

### 1.3 Phoenix Channels (and Supabase Realtime on top of it)

- **(1) Sockets**: one socket and many topics. "One channel server lightweight process is created per client, per topic" (https://phoenix.hexdocs.pm/channels.html).
- **(2) Auth in two levels**:
  - `connect/3` verifies a token passed in the socket params, so an open socket can still be anonymous if the app allows it.
  - `join/3` authorizes each topic: `{:error, %{reason: "unauthorized"}}`.
  - Supabase refreshes the JWT **in band**: an `access_token` event "without rejoining" (https://supabase.com/docs/guides/realtime/protocol).
- **(3) Message shape**:
  - `{topic, event, payload, ref, join_ref}`. Supabase v2 uses an array `[join_ref, ref, topic, event, payload]` plus binary frames.
  - The join gets `phx_reply {status: ok|error, response}`, and a channel error is followed by `phx_close`.
- **(4) Deltas**:
  - Plain channels push events.
  - LiveView sends rendered diffs (static parts sent once, then only the dynamic slots).
  - Presence sends a `presence_state` snapshot, then `presence_diff {joins, leaves}`.
- **(5) Reconnect**:
  - Clients use exponential backoff and "attempt to rejoin the topics they had previously subscribed to".
  - **`join_ref`** drops stale messages from an earlier join of the same topic: `isMember` logs "dropping outdated message" (`assets/js/phoenix/channel.js`).
  - Pushes made while disconnected wait in a `pushBuffer` (default timeout 10 s).
  - Phoenix "uses an at-most-once strategy". Apps that need more must track their own `last_seen_id`.
  - Supabase adds an optional `replay {since, limit}` for broadcasts.
- **(6) Backpressure and limits** (Supabase, https://supabase.com/docs/guides/realtime/limits):
  - **100 channels per connection** (`too_many_channels`), joins per second (`too_many_joins`), and messages per second (disconnect with `tenant_events`; the client reconnects).
  - The heartbeat must come "at least every 25 seconds".
- **(7) Server cost**:
  - PubSub broadcast fans out once per topic.
  - `intercept` / `handle_out` runs per recipient, and the docs warn "the handle_out/3 callback will be called for every recipient of a message", the costly path.
  - Phoenix has a known *fastlane* optimization: an encoded broadcast is serialized once and written straight to every subscriber's transport.
- **(8) RPC**: same socket (`push` with a reply).
- **(9) Versioning**: `vsn=1.0.0 | 2.0.0` in the URL.

### 1.4 Meteor DDP

- **(1) Sockets**: one. Subscriptions, methods and data share it.
- **(3) Subscriptions** (https://github.com/meteor/meteor/blob/devel/packages/ddp/DDP.md):
  - `sub {id, name, params}`, `unsub {id}`.
  - The server sends **`ready {subs: [ids]}`** when the initial data is complete, and **`nosub {id, error?}`** when a subscription ends or is refused.
- **(4) Data messages**:
  - `added / changed (fields + cleared) / removed`, keyed by `collection` + `id`.
  - "overlapping subscriptions cause the server to send the union of facts about the one collection's data" (the *mergebox*).
- **(5) Reconnect**:
  - `connect {version, support, session}` is negotiated. A failure returns `failed {version}`.
  - In practice a reconnect is a new session, so the client re-subs and gets everything again.
  - The client keeps its collections and reconciles them when `ready` arrives.
- **(8) RPC**:
  - `method {id}` → `result {id}`, then **`updated {methods: [id]}`** once that method's writes have reached the client.
  - This makes "my command's effect is now visible" explicit.
- **(9) Versioning**: version negotiation in `connect`.
- **(10) Pain points**:
  - The mergebox "stores a copy of the collection data for each user who subscribes". The memory cost per client was large enough that Meteor 2.4 added publication strategies (`SERVER_MERGE`, `NO_MERGE`, `NO_MERGE_NO_HISTORY`) that trade bandwidth for memory (forums.meteor.com, Meteor 2.4 RC thread).
  - Lesson: per-client copies of the state on the server do not scale. Keep only a revision per subscription.

### 1.5 Kubernetes watch

- **(4) Revisions**:
  - Every object and list has a `resourceVersion`.
  - "If a client watch is disconnected then that client can start a new watch from the last returned resourceVersion; the client could also perform a fresh get / list request and begin again" (https://kubernetes.io/docs/reference/using-api/api-concepts/).
- **(5) 410 Gone**: when the version is too old, "clients must … [recognize] the status code 410 Gone, clearing their local cache, performing a new get or list operation, and starting the watch from the resourceVersion that was returned."
- **Bookmarks**:
  - A bookmark is "a special kind of event to mark that all changes up to a given resourceVersion … have already been sent". It carries a revision only, so a quiet watch can still resume cheaply.
  - Clients cannot rely on any interval.
- **Streaming lists** (`sendInitialEvents=true`):
  - The snapshot arrives as synthetic `ADDED` events, then a `BOOKMARK` that marks "initial state done" (KEP-3157, watch-list).
  - This is the same idea as DDP's `ready`.
- **(6)(7) Watch cache (the cacher)**:
  - One shared cache per resource fans out to every watcher, and reads are "Always served from watch cache".
  - A watcher whose buffer fills is **closed** ("Forcing … watcher close due to unresponsiveness", `cache_watcher.go`; metric `terminated_watchers_total`).
  - Overload gives `429` with `Retry-After`.
- **(10) Pain points**:
  - *Relist storms*: many clients getting 410 at once each do a full LIST, causing memory spikes and OOMs (https://www.netdata.cloud/guides/kubernetes/kubernetes-api-server-watch-storm/).
  - Streaming lists and bookmarks were added for exactly this.
  - Red Hat API-1648 shows slow-watcher closes at high rate under load.

### 1.6 OBS WebSocket v5

- **(1) Sockets**: one. Requests, responses and events share it.
- **(2)(9) Handshake** (https://github.com/obsproject/obs-websocket/blob/master/docs/generated/protocol.md):
  - The server sends `Hello (op 0)` "immediately on client connection" with `rpcVersion` and an optional auth challenge.
  - The client sends `Identify (op 1)` with `rpcVersion`, the auth string and an `eventSubscriptions` bitmask. The server answers `Identified` with `negotiatedRpcVersion`.
  - `Reidentify (op 3)` changes subscriptions without reconnecting.
- **(3) Subscriptions**: a bitmask of event categories. "High volume events must be explicitly subscribed to" (for example `InputVolumeMeters` every 50 ms, bit 1<<16).
- **(8) RPC**: `Request {requestType, requestId}` → `RequestResponse`, plus `RequestBatch` with `haltOnFailure`.
- **Errors**: close codes for protocol errors: 4002 MessageDecodeError, 4003–4005 missing or invalid fields, 4006 UnknownOpCode, 4007 NotIdentified, 4009 AuthenticationFailed, 4010 UnsupportedRpcVersion, 4011 SessionInvalidated.
- **(10) Lessons from v4 → v5**:
  - v4 sent every event to everyone.
  - v5 made subscriptions explicit and put the high-rate events behind an opt-in, because meters flooded clients.

### 1.7 graphql-ws (`graphql-transport-ws`, used by Hasura, Apollo, …)

- **(2) Auth** (https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md):
  - The client sends `ConnectionInit {payload}` (auth goes in the payload), and the server answers `ConnectionAck`.
  - A timeout gives `4408`, a subscribe before the ack gives `4401 Unauthorized`, and a second init gives `4429`.
  - `4403 Forbidden` is recommended for an auth rejection.
- **(3) Subscriptions**:
  - `Subscribe {id, payload}`, then `Next {id}`*, then `Error {id}` (which ends the operation) or `Complete {id}` (either direction).
  - A duplicate id **closes the socket**: "4409: Subscriber for <id> already exists". An id can be reused once its operation completes.
  - "Either side may still receive in-flight messages for a completed ID and should ignore them".
- **(8)**: queries and mutations can also ride the socket.
- **(10) Pain points**:
  - The old `subscriptions-transport-ws` was replaced because ack and auth ordering was ambiguous and errors were not scoped.
  - The new protocol made every error either *per id* or a *close code*.

### 1.8 Syncthing events (long-poll, the non-socket contrast)

- `GET /rest/events?since=<id>&timeout=60&events=A,B`. IDs are sequential, and the buffer is bounded.
- "some events might be missed … noting a discontinuity in the event IDs". After a disconnect, `since=0&limit=1` jumps to the newest event, and the client then reloads its state over REST (https://docs.syncthing.net/rest/events-get.html).
- `/rest/events/disk` is a pre-filtered stream, so a named feed replaces a client-side filter.
- Lesson: an **explicit gap** that the client can detect is enough when the state can be reloaded. LumaCore's `gap` already does this for Samples.

---

## 2. Cross-cutting comparison

| Topic | HA | Centrifuge/Grafana | Phoenix/Supabase | DDP | K8s | OBS | graphql-ws | **LumaCore today** |
|---|---|---|---|---|---|---|---|---|
| Sockets | 1 | 1 per tab | 1 | 1 | 1 per watch (HTTP) | 1 | 1 | feeds + shell + 2 SSE |
| Auth | in-band first msg | connect token | socket params + per-topic join | app-level | per request | Hello/Identify | ConnectionInit | before upgrade |
| Pre-auth reads | none | none | allowed if `connect` allows | yes | n/a | none | none | separate open SSE |
| Sub id | client int, reuse = error | channel name | topic + join_ref | client id | n/a | bitmask | client id, reuse = close | client int, reuse = replace |
| Ack / ready | `result` | subscribe reply | `phx_reply` | `ready` / `nosub` | initial-events bookmark | `Identified` | none (first `Next`) | none (first batch) |
| Snapshot then | diffs a/c/r | publications | events / LiveView diffs | added/changed/removed | ADDED then events | events | Next | full replace each change |
| Revision | none | offset + epoch | none (app) | none | resourceVersion | none | none | Samples: seq + epoch; State: none |
| Resume | resubscribe | recover or `recovered:false` | rejoin, at-most-once | resub | rv or 410 → relist | none | none | Samples: since; State: resnapshot |
| Slow client | queue cap 4096 / peak 1024 for 10 s → close | queue cap → close | rate limits → close | none | buffer full → close | none | none | one send > 2 s → close |
| Coalescing | array frame, opt-in | `client_write_delay` | none | none | none | none | none | period per sub (≥ 16 ms) |
| Shared build | serialize once per event (LRU) | per channel | broadcast once per topic | mergebox per client (costly) | watch cache | n/a | per sub | **per connection per sub per tick** |
| RPC on same socket | yes | rpc (optional) | yes | yes + `updated` | no | yes | yes | no (`/ws/shell`) |
| Version | ha_version + features | protocol | `vsn` | connect version | API group version | rpcVersion | subprotocol | `CLIENT_PROTOCOL` (shell hello only) |

---

## 3. Recommendations for `/ws/feeds`

### Keep
1. **One socket per page with client-chosen integer sub ids.** All eight projects multiplex the same way.
2. **State vs Samples, `seq` + `epoch` + explicit `gap` + `recovered`.** This is Centrifugo's model almost exactly, and its "never a partial recovery" rule is the one LumaCore already has.
3. **One batch per period, never one item per message.** HA, Centrifugo PRO and DDP clients all ended up batching.
4. **Binary frames for DMX only.** Supabase v2 does the same (JSON control, binary payload).
5. **High-rate feeds only while a window shows them.** This is OBS v5's lesson.
6. **Commands stay on `/ws/shell`.** Most projects put RPC on the same socket, but LumaCore's shell is a full REPL protocol (`lumacore-wire`) shared with the TUI and CLI. Keeping it separate keeps the rule "every gesture is a command line". Meteor's `updated` is the one idea to borrow: let a command reply carry the station revision it produced (see Add 3).

### Add
1. **A revision per State feed (`rev`)**, taken from the station rather than a per-sub batch counter. Examples are `show_generation`, `StationView` publish counter and the patch generation. `sub` can then carry `rev` so that an unchanged state answers `{unchanged:true}` instead of resending (like K8s `resourceVersion` and Centrifugo cache mode). It also lets the client drop out-of-order replies.
2. **A rule for delta vs replace**:
   - Small states (< about 16 KB: station, network menu, lumasessions, health tables) are **replaced whole**.
   - Keyed collections that can be large (pools, shows list, users, views/screens) send **`{rev, base_rev, set:{key:…}, del:[keys]}`**, HA's `a/c/r` shape keyed by GUID or number.
   - When the client's `base_rev` doesn't match, or the diff is bigger than half the full state, send `replace`.
   - Never fossil or byte deltas. They need history and add complexity for no gain on a LAN.
3. **An explicit ack and refusal**:
   - `{"op":"subbed","id","rev"}` after the first batch (DDP `ready`, the K8s initial-events bookmark), so a window knows its initial state is complete.
   - `{"op":"nosub","id","code":"E-FEED-00n","message"}` when a sub is refused or ended: unknown feed, no rights, object deleted, or the user logged out of a user-scoped feed.
   - Errors carry the project's codes (`E-FEED-…` in the HTTP crate's catalog).
4. **A rule for a reused id**: keep "same id = replace parameters" (it is useful for `sub.set()`), but add a client-side sub **generation** (`gen`) echoed in every envelope, like Phoenix's `join_ref`. The client then drops batches from the replaced parameters that were still in flight. Today a DMX batch for the old universe can arrive after `set({universes})`.
5. **One shared build per (feed, params, user-scope), fanned out**:
   - The state thread (or one producer task per key) builds the JSON **once** when its input revision moves and holds it as `Arc<str>` with its `rev`. Each connection only frames it with its id (HA's partial-cached-message trick, the K8s watch cache, the Centrifugo channel).
   - Build only while at least one subscriber exists, with a refcount per key, and drop it when the last subscriber leaves.
   - Rebuild on change (a revision bump from `StationView` / `ShowData` publish), not by polling and comparing every period. This removes the 500 ms `console.station()` loop per SSE client and the per-tick `pool()`/`windows_data()` rebuilds.
6. **Coalescing**: keep the per-sub `period_ms` as the *maximum* send rate. A change sends at the next due tick, and an idle feed sends nothing. Defaults: State feeds 100 ms (UI), 250 ms (health), 16 ms (DMX, Realtime). Optionally send all due batches of one tick in **one frame as a JSON array**, as HA's coalesce does. This is cheap and lowers the frame count when a page has 20 windows.
7. **Backpressure in two limits** (HA's rule, about K8s's): a queued-bytes or queued-messages cap per connection, plus "over the peak for N s" → close with a code (`4008 slow`).
   - State feeds coalesce by replacing the queued item (latest wins), so they never grow the queue.
   - Samples can grow it, which is what triggers the close.
   - Keep the 2 s send stall.
8. **Resume after reconnect**:
   - Samples: as today (`since` + `epoch`).
   - State: `rev` + `epoch`. A match gives `unchanged`, a mismatch gives the full snapshot with `recovered:false`, which is the 410 → relist pattern.
   - **Jitter the reconnect backoff** so 30 phones or tablets don't resubscribe at the same instant after a station restart (the K8s relist storm). Snapshots are shared builds (Add 5), so a storm costs one build per key.
9. **Pre-login open feeds on the same socket**:
   - The socket upgrades **without** login.
   - A small allow-list of feeds, marked `open: true` in the feed registry (`station` = users, presence, show status, lock-surface data), is subscribable before login.
   - Everything else gets `nosub E-FEED-002 login required`.
   - Logging in happens in band: `{"op":"auth","token"}`, which gives `{"op":"authed","user"}` and keeps the open subs. This is Phoenix's two-level model, and Supabase's `access_token` refresh in band.
   - Logout or a token revoke closes user-scoped subs with `nosub` and keeps the open ones. FeedClient's `reconnect()` on login can then go.
   - Respect the "keep security light" decision: open feeds are read-only and the same data `/api/station` already gives.
10. **Presence as a subscription**:
    - A `screen` (or `views`) sub with `{screen:n}` *is* the viewer registration for Screenshot capture (it replaces `/api/events?screen=n`).
    - Capture requests and `reload_ui` arrive as **Samples** (with `seq`, never silently dropped like today's lagged SSE).
11. **A limit on subscriptions**: for example 64 subs per connection, with `nosub E-FEED-003 too many` (Supabase: 100 channels per connection). Also cap the number of connections per device or IP.
12. **A protocol version on the feed socket**:
    - The first server message is `{"op":"hello","client_protocol":N,"epoch","feeds":[{name,kind,open,params}]}`. This combines HA `auth_required`, OBS `Hello` and DDP `connect`.
    - The client refuses a mismatch (same as the shell's `CLIENT_PROTOCOL`, bumped with no shims).
    - The feed list is published from the registry, matching the project's "publish yourself" rule.
13. **Heartbeat**: WebSocket ping every 10–15 s from the server, and close after 2 missed pongs. This catches half-open phone connections that the 2 s send stall never sees on an idle socket.
14. **New feeds as State, keyed, revisioned**:
    - `station` (open), `network`, `lumasessions`, `shows`, `views` (layout events and the screen list), `pool.editor` (one pool object's detail).
    - Each declares `kind`, `open`, `scope` (station / user / screen), `rev source`, `period` and the delta mode in one registry table, so no hand-made lists.

### Avoid
- **Per-client server copies of the state for diffing** (Meteor's mergebox memory problem). Diff against the revision's shared previous build, not a per-socket copy. One exception: DMX `last` per sub is bounded at 3 KB per universe and fine.
- **Polling and comparing per connection** (today's SSE station loop and the per-tick `Value` compares). Drive builds from revision bumps.
- **Silent loss** (today's `/api/events` drops lagged events). Every non-state stream has a `seq` and a `gap`.
- **Partial recovery**: either resume exactly or resnapshot (Centrifugo).
- **A protocol error that closes the whole socket for one bad sub** (graphql-ws 4409). Answer with a per-id `nosub`, and close only for malformed frames or auth.
- **Byte deltas, protobuf or compression** on a LAN at this scale. permessage-deflate could be considered later for phones over Wi-Fi.
- **Per-feed endpoints, per-transport auth, or a second event path.** `/ws/feeds` and `/ws/shell` stay the only `/ws/` routes (tidy already checks).
- **Too much in the JSON envelope**: keep the envelope keys short and stable. Consider dropping `kind` and `epoch` from every batch and sending them once in `subbed` (HA's short keys).

---

## 4. Sketch of the protocol after 12d (illustrative)

```
S→C {"op":"hello","client_protocol":9,"epoch":"…","user":null,"feeds":[{"name":"station","kind":"state","open":true},…]}
C→S {"op":"sub","id":1,"gen":1,"feed":"station"}                       // open feed, before login
S→C {"op":"subbed","id":1,"gen":1,"rev":412,"kind":"state"}
S→C {"id":1,"gen":1,"rev":412,"replace":{…}}
C→S {"op":"auth","token":"v2.…"}
S→C {"op":"authed","user":"Bob"}
C→S {"op":"sub","id":2,"gen":1,"feed":"pools","pool":"Groups","datapool":"selected","rev":97,"epoch":"…"}
S→C {"op":"subbed","id":2,"gen":1,"rev":97,"unchanged":true}
S→C {"id":2,"gen":1,"rev":98,"base":97,"set":{"3":{…}},"del":["7"]}
C→S {"op":"sub","id":3,"feed":"network"}  // logged-out user
S→C {"op":"nosub","id":3,"code":"E-FEED-002","message":"login required"}
S→C [ {…batch id 4…}, {…batch id 5…} ]                               // one frame, coalesced
```

---

## 5. Sources
- Home Assistant WS API: https://developers.home-assistant.io/docs/api/websocket/
- HA const.py / messages.py / http.py: https://github.com/home-assistant/core/tree/dev/homeassistant/components/websocket_api
- HA JS client: https://github.com/home-assistant/home-assistant-js-websocket/blob/master/lib/connection.ts
- Grafana Live: https://grafana.com/docs/grafana/latest/setup-grafana/set-up-grafana-live/
- Centrifugo history and recovery: https://centrifugal.dev/docs/server/history_and_recovery ; delta: https://centrifugal.dev/docs/server/delta_compression ; batching: https://centrifugal.dev/docs/pro/client_message_batching
- Phoenix Channels: https://phoenix.hexdocs.pm/channels.html ; channel.js: https://github.com/phoenixframework/phoenix/blob/main/assets/js/phoenix/channel.js
- Supabase Realtime protocol / limits: https://supabase.com/docs/guides/realtime/protocol , https://supabase.com/docs/guides/realtime/limits
- Meteor DDP: https://github.com/meteor/meteor/blob/devel/packages/ddp/DDP.md ; publication strategies: https://forums.meteor.com/t/meteor-2-4-release-candidate/56505
- Kubernetes API concepts: https://kubernetes.io/docs/reference/using-api/api-concepts/ ; cacher: k8s.io/apiserver/pkg/storage/cacher/cache_watcher.go ; watch storms: https://www.netdata.cloud/guides/kubernetes/kubernetes-api-server-watch-storm/ ; https://issues.redhat.com/browse/API-1648
- OBS WebSocket v5: https://github.com/obsproject/obs-websocket/blob/master/docs/generated/protocol.md
- graphql-ws: https://github.com/enisdenjo/graphql-ws/blob/master/PROTOCOL.md
- Syncthing events: https://docs.syncthing.net/rest/events-get.html
