---
title: Integrations
category: docs
---
# Integrations

Canonical language source: [`lum.md`](../../user/syntax/lum.md)

How external systems connect to LumaCore. The goal is clean runtime boundaries
for external systems, with `Lum`, `DataPool` and the object tree as the
framework.

## Principle

LumaCore integrates with external systems through clear runtime boundaries,
not ad hoc side channels.

## Main Integration Areas

| Area | Status | Where |
|---|---|---|
| Lighting protocols: sACN, Art-Net output and input | built | the protocols' lines (`Sacn 1`, `ArtNet 1`), `lumacore-lines`, `lumacore-protocols`, `lumacore-artnet`, `lumacore-sacn` |
| Remote command and shell access | built | the UDP command port and the TCP shell port (the constants in `lumacore-wire`) |
| REST and WebSocket (no SSE) | built | `lumacore-http` on the HTTP port, see [http-api.md](../api/http-api.md) |
| Web UI and desktop app | built | `crates/ui/lumacore-webui` (built into the server), served by the HTTP engine |
| OSC control input | built (partly) | an `Osc` line, `crates/network/protocols/data/lumacore-osc` (the listener) and `lumacore-cmd-osc` (the object) |
| MVR import | built | `Import Mvr`, `crates/formats/lumacore-mvr` |
| Pool XML groups import | built | `Import PoolXml "file" Groups`, `crates/formats/lumacore-poolxml` |
| MIDI control, MVR-xchange, MVR export | not built | the roadmap |

## OSC input

OSC is **not** an engine and has **no default port**. It is a line of the `Osc` protocol
in the show, like a DMX input line:

```text
Store Osc 2 "Companion" /Network "Rig" /Port 8000 /User "LD"
```

`lumacore_lines::ProtocolRuntimes` starts one UDP listener per enabled OSC line, bound to
its network's NIC. Like every line, it runs only on a standalone station or the
LumaSession master (members, a fenced master and an IdleMaster without
OutputWhenAlone don't listen), and only when this station maps the line's
network to a NIC, so a broadcast OSC message runs once in a session.
Commands run as the line's user, through the same dispatch and rights gate as a
typed command. The line needs a user (E-RIGHTS-003): it never runs as Admin, a
line without one doesn't start (the System Monitor says why), and you may name a
user up to your own rights level (changing it later, `Set Osc 2 "User"`, is
Admin's).

Addresses:

- `/lumacore/cmd <string>` — a command line. Works.
- `/lumacore/go`, `/lumacore/back`, `/lumacore/stop` — run as `Go`, `Go-`,
  `Stop` as the line's user. Works.
- `/lumacore/fixture/<id>/<attr> <float>` — decoded to an attribute event; not
  built (planned, said once in the System Monitor; archive §29).

## Integration Rule

External systems connect to the same core runtime truth used by native
clients.

That means integrations must not:

- bypass authorization (every statement passes the one rights gate in dispatch)
- create hidden parallel state
- fork a second runtime model

## ObjectTree Alignment

Authored protocol setup belongs in show data. Live sockets, listeners and
sessions belong in runtime. The station's network-to-NIC map is station-local.

```text
Root
├── ShowData
│   ├── Protocols              Networks; DMXProtocols.<protocol>.<n> and DataProtocols.<protocol>.<n> lines
│   └── Stations               each station's rig wiring (network → NIC), applied by that station
├── StationSettings
│   └── networks               the last-applied copy of this station's map (never replicated)
└── Runtime
    ├── Network                interfaces, lines' live state, input sources, Art-Net nodes
    └── LumaSessions           stations, members
```

Important:

- authored lines and protocol assignment belong in `ShowData.Protocols`
- runtime sockets, listeners, LumaSessions and discovery belong in `Runtime`
- datapool objects remain under `ShowData.DataPools`

## Protocol Thinking

At a simple level:

- output protocols send show data outward
- command and session protocols bring user intent inward
- API protocols expose controlled runtime access

These all end in the same runtime truth: the state thread.

## DataPool Rule

If an integration references stored operator objects such as:

- `Group`
- `Macro`
- `Layout`
- `Timecode`

it should reference them through their real `DataPool` paths and `Lum` nouns,
not through hidden integration-only aliases.

## Lum Alignment

Integrations respect `Lum` semantics when exposing control surfaces.

Examples:

```text
Info Group 1
Info Timecode 1
Join LumaSession "Festival"
Fixture 1 Thru 4 At 50
```

Rule:

- an API or remote client does not invent a different control model
- it carries the same object and action meanings already defined in `Lum`
  (see the generated [command reference](../../user/reference/commands.md))

## Future Direction

LumaCore stays open to:

- MVR-xchange and MVR export
- more output transports (USB/serial DMX)
- MIDI input
- visualizer and external tool exchange

The point is not to support everything at once. The point is to keep the
boundaries clean enough that these integrations can arrive without redesigning
the whole system.
