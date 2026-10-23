---
title: Tools
category: docs
---
# Tools

Small developer utilities for LumaCore.

## `sacn_monitor.py`

Simple no-dependency sACN multicast monitor for local testing.

Examples:

```bash
python3 tools/sacn_monitor.py
python3 tools/sacn_monitor.py --universe 1 --universe 2
python3 tools/sacn_monitor.py --universe 1 --show-all
```

The `dmx` window in the web UI (the `dmx` feed on `/ws/feeds`) and `DMX In <u>` show the same from inside
LumaCore.

## `sendCmd.py`

Send one command to a running LumaCore over the UDP command port (`REMOTE_CMD_PORT` in
`lumacore-wire`; `--port-base` moves it). The packet
is a `RemoteRequest::Command` (`{"kind": "command", "user": …, "cmd": …}`); the server
runs it as that user (through the normal rights gate) and replies with a small JSON
status.

Examples:

```bash
python3 tools/sendCmd.py --target 127.0.0.1 --user Admin --cmd "List Output"
python3 tools/sendCmd.py --target 127.0.0.1 --user Debug --cmd "Fixture 1 Thru 2 At Full"
```

`cargo run -p lumacore-cli -- --cmd "…"` does the same with the Rust client.

## `obsidian-snippets/`

CSS snippets for reading the docs in Obsidian. `scripts/unix/dev_init.sh` copies them
into the vault's `.obsidian/snippets/`.
